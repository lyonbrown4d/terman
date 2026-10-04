package ptywin

import (
	"context"
	"errors"
	"io"
	"os"
	"strings"

	"github.com/aymanbagabas/go-pty"
	"github.com/charmbracelet/x/vt"
)

type Event struct {
	ID   int64
	Data []byte
	Exit bool
}

type Window struct {
	ID     int64
	Title  string
	Bytes  int64
	pty    pty.Pty
	cmd    *pty.Cmd
	term   *vt.Emulator
	cancel context.CancelFunc
}

func New(parent context.Context, id int64, title, shell string, args, env []string, cwd string, cols, rows int, events chan<- Event) (*Window, error) {
	ctx, cancel := context.WithCancel(parent)
	terminal, err := pty.New()
	if err != nil {
		cancel()
		return nil, err
	}
	if err := terminal.Resize(cols, rows); err != nil {
		cancel()
		_ = terminal.Close()
		return nil, err
	}
	cmd := terminal.CommandContext(ctx, shell, args...)
	cmd.Env = env
	cmd.Dir = cwd
	if err := cmd.Start(); err != nil {
		cancel()
		_ = terminal.Close()
		return nil, err
	}
	window := &Window{
		ID: id, Title: title, pty: terminal, cmd: cmd,
		term: vt.NewEmulator(cols, rows), cancel: cancel,
	}
	go window.read(ctx, events)
	go func() {
		_ = cmd.Wait()
		select {
		case events <- Event{ID: id, Exit: true}:
		case <-ctx.Done():
		}
	}()
	return window, nil
}

func (w *Window) read(ctx context.Context, events chan<- Event) {
	buffer := make([]byte, 32*1024)
	for {
		n, err := w.pty.Read(buffer)
		if n > 0 {
			data := append([]byte(nil), buffer[:n]...)
			select {
			case events <- Event{ID: w.ID, Data: data}:
			case <-ctx.Done():
				return
			}
		}
		if err != nil {
			if !errors.Is(err, io.EOF) {
				return
			}
			return
		}
	}
}

func (w *Window) Apply(data []byte) {
	w.Bytes += int64(len(data))
	_, _ = w.term.Write(data)
}

func (w *Window) Write(data []byte) error {
	_, err := w.pty.Write(data)
	return err
}

func (w *Window) Resize(cols, rows int) {
	if cols < 1 || rows < 1 {
		return
	}
	_ = w.pty.Resize(cols, rows)
	w.term.Resize(cols, rows)
}

func (w *Window) Lines() []string {
	text := strings.ReplaceAll(w.term.String(), "\r", "")
	return strings.Split(text, "\n")
}

func (w *Window) Close() {
	w.cancel()
	_ = w.pty.Close()
}

func Environment(term string, overrides map[string]string) []string {
	values := map[string]string{}
	for _, item := range os.Environ() {
		if name, value, ok := strings.Cut(item, "="); ok {
			values[name] = value
		}
	}
	values["TERM"] = term
	for name, value := range overrides {
		values[name] = value
	}
	result := make([]string, 0, len(values))
	for name, value := range values {
		result = append(result, name+"="+value)
	}
	return result
}
