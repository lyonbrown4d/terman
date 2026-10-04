package session

import (
	"fmt"
	"io"
	"os"
	"strings"

	"github.com/aymanbagabas/go-pty"
	"github.com/charmbracelet/x/vt"
	"github.com/lyonbrown4d/terman/common"
)

func (r *Reactor) startPane(window, index, cols, rows int, command string) (*Pane, error) {
	pt, err := pty.New()
	if err != nil {
		return nil, fmt.Errorf("create pty: %w", err)
	}
	if err := pt.Resize(max(cols, 1), max(rows, 1)); err != nil {
		_ = pt.Close()
		return nil, fmt.Errorf("resize pty: %w", err)
	}
	shell := common.DefaultShell()
	var args []string
	if strings.TrimSpace(command) != "" {
		args = append(common.ShellCommandArgs(shell, false), command)
	}
	cmd := pt.CommandContext(r.ctx, shell, args...)
	cmd.Env = append(os.Environ(), "TERM=xterm-256color")
	if err := cmd.Start(); err != nil {
		_ = pt.Close()
		return nil, fmt.Errorf("start pane command: %w", err)
	}
	term := vt.NewEmulator(max(cols, 1), max(rows, 1))
	term.SetScrollbackSize(5000)
	pane := &Pane{Index: index, PTY: pt, Term: term, Command: cmd, Width: cols, Height: rows}
	go r.readPane(window, pane)
	go r.waitPane(window, pane)
	return pane, nil
}

func (r *Reactor) readPane(window int, pane *Pane) {
	buffer := make([]byte, 32*1024)
	for {
		n, err := pane.PTY.Read(buffer)
		if n > 0 {
			data := append([]byte(nil), buffer[:n]...)
			select {
			case r.events <- outputEvent{window: window, pane: pane.Index, data: data}:
			case <-r.ctx.Done():
				return
			}
		}
		if err != nil {
			if err != io.EOF {
				select {
				case r.events <- outputEvent{window: window, pane: pane.Index, data: []byte("\r\n[pty read failed]\r\n")}:
				case <-r.ctx.Done():
				}
			}
			return
		}
	}
}

func (r *Reactor) waitPane(window int, pane *Pane) {
	_ = pane.Command.Wait()
	select {
	case r.events <- exitEvent{window: window, pane: pane.Index}:
	case <-r.ctx.Done():
	}
}

func closePane(pane *Pane) {
	if pane == nil {
		return
	}
	if pane.Command != nil && pane.Command.Process != nil {
		_ = pane.Command.Process.Kill()
	}
	if pane.PTY != nil {
		_ = pane.PTY.Close()
	}
	if pane.Term != nil {
		_ = pane.Term.Close()
	}
}

func (r *Reactor) newWindow(name, command string) (*Window, error) {
	index := r.state.NextWindow
	r.state.NextWindow++
	if name == "" {
		name = fmt.Sprintf("window-%d", index)
	}
	window := &Window{
		Index: index, Name: name, Panes: make(map[int]*Pane),
		ActivePane: 0, LastPane: 0, NextPane: 1, Layout: leaf(0), LayoutName: "manual",
	}
	pane, err := r.startPane(index, 0, r.state.Cols, max(r.state.Rows-1, 1), command)
	if err != nil {
		return nil, err
	}
	window.Panes[0] = pane
	window.PaneOrder = []int{0}
	r.state.Windows[index] = window
	r.state.WindowOrder = append(r.state.WindowOrder, index)
	if len(r.state.WindowOrder) == 1 {
		r.state.ActiveWindow = index
	}
	return window, nil
}
