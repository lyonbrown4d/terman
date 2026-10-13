//go:build windows

package transport

import (
	"os/user"
	"strings"
	"testing"
	"time"

	"golang.org/x/sys/windows"
)

func TestSessionPipeIsUserScoped(t *testing.T) {
	account, err := user.Current()
	if err != nil {
		t.Fatal(err)
	}
	endpoint, err := Endpoint("security-test")
	if err != nil {
		t.Fatal(err)
	}
	if !strings.Contains(endpoint, account.Uid) {
		t.Fatalf("pipe %q does not contain user SID %q", endpoint, account.Uid)
	}
}

func TestSessionPipeHasRestrictedDACL(t *testing.T) {
	endpoint, err := Endpoint(t.Name())
	if err != nil {
		t.Fatal(err)
	}
	listener, err := Listen(endpoint)
	if err != nil {
		t.Fatal(err)
	}
	defer listener.Close()
	release := make(chan struct{})
	defer close(release)
	go func() {
		conn, err := listener.Accept()
		if err == nil {
			<-release
			_ = conn.Close()
		}
	}()
	name, err := windows.UTF16PtrFromString(endpoint)
	if err != nil {
		t.Fatal(err)
	}
	var handle windows.Handle
	for deadline := time.Now().Add(time.Second); ; {
		handle, err = windows.CreateFile(name, windows.GENERIC_READ|windows.GENERIC_WRITE|windows.READ_CONTROL, 0, nil, windows.OPEN_EXISTING, 0, 0)
		if err == nil || time.Now().After(deadline) {
			break
		}
		time.Sleep(time.Millisecond)
	}
	if err != nil {
		t.Fatal(err)
	}
	defer windows.CloseHandle(handle)
	descriptor, err := windows.GetSecurityInfo(handle, windows.SE_KERNEL_OBJECT, windows.DACL_SECURITY_INFORMATION)
	if err != nil {
		t.Fatal(err)
	}
	permissions := descriptor.String()
	if strings.Contains(permissions, ";;;WD)") || strings.Contains(permissions, ";;;AU)") || !strings.Contains(permissions, "D:P") {
		t.Fatalf("session pipe has unrestricted ACL: %s", permissions)
	}
}
