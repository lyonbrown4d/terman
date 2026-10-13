//go:build windows

package service

import (
	"fmt"
	"os"
	"os/exec"
	"sync"
	"syscall"
	"unsafe"

	"golang.org/x/sys/windows"
)

type processTree struct{ job *jobHandle }
type jobHandle struct {
	mu     sync.Mutex
	handle windows.Handle
}

func prepareProcess(command *exec.Cmd) (processTree, error) {
	job, err := windows.CreateJobObject(nil, nil)
	if err != nil {
		return processTree{}, fmt.Errorf("create session job: %w", err)
	}
	info := windows.JOBOBJECT_EXTENDED_LIMIT_INFORMATION{}
	info.BasicLimitInformation.LimitFlags = windows.JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE
	_, err = windows.SetInformationJobObject(job, windows.JobObjectExtendedLimitInformation, uintptr(unsafe.Pointer(&info)), uint32(unsafe.Sizeof(info)))
	if err != nil {
		_ = windows.CloseHandle(job)
		return processTree{}, err
	}
	// Keep the child suspended until it belongs to the job, so descendants
	// cannot escape containment during process creation.
	command.SysProcAttr = &syscall.SysProcAttr{CreationFlags: windows.CREATE_NEW_PROCESS_GROUP | windows.CREATE_SUSPENDED, HideWindow: true}
	return processTree{job: &jobHandle{handle: job}}, nil
}
func (tree processTree) attach(process *os.Process) error {
	handle, err := windows.OpenProcess(windows.PROCESS_SET_QUOTA|windows.PROCESS_TERMINATE, false, uint32(process.Pid))
	if err != nil {
		return err
	}
	defer windows.CloseHandle(handle)
	if err = windows.AssignProcessToJobObject(tree.job.handle, handle); err != nil {
		return err
	}
	return resumeProcessThread(uint32(process.Pid))
}
func (tree processTree) signal(_ *os.Process) error {
	return fmt.Errorf("Windows sessions require a Shutdown callback")
}
func (tree processTree) kill(_ *os.Process) error {
	tree.job.mu.Lock()
	defer tree.job.mu.Unlock()
	if tree.job.handle == 0 {
		return nil
	}
	return windows.TerminateJobObject(tree.job.handle, 1)
}
func (tree processTree) close() {
	if tree.job == nil {
		return
	}
	tree.job.mu.Lock()
	defer tree.job.mu.Unlock()
	if tree.job.handle != 0 {
		_ = windows.CloseHandle(tree.job.handle)
		tree.job.handle = 0
	}
}
func resumeProcessThread(pid uint32) error {
	snapshot, err := windows.CreateToolhelp32Snapshot(windows.TH32CS_SNAPTHREAD, 0)
	if err != nil {
		return err
	}
	defer windows.CloseHandle(snapshot)
	entry := windows.ThreadEntry32{Size: uint32(unsafe.Sizeof(windows.ThreadEntry32{}))}
	for err = windows.Thread32First(snapshot, &entry); err == nil; err = windows.Thread32Next(snapshot, &entry) {
		if entry.OwnerProcessID != pid {
			continue
		}
		thread, err := windows.OpenThread(windows.THREAD_SUSPEND_RESUME, false, entry.ThreadID)
		if err != nil {
			return err
		}
		_, err = windows.ResumeThread(thread)
		windows.CloseHandle(thread)
		return err
	}
	return fmt.Errorf("find suspended session thread: %w", err)
}
