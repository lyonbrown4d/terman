package ptywin

import (
	"context"
	"fmt"
	"image/color"
	"os"
	"strings"

	"github.com/aymanbagabas/go-pty"
	"github.com/charmbracelet/x/vt"
	"github.com/lyonbrown4d/terman/screen/internal/proto"
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

func New(
	parent context.Context,
	id int64,
	title, shell string,
	args, env []string,
	cwd string,
	cols, rows int,
	events chan<- Event,
) (*Window, error) {
	ctx, cancel := context.WithCancel(parent)
	terminal, err := pty.New()
	if err != nil {
		cancel()
		return nil, fmt.Errorf("create pty: %w", err)
	}
	if err := terminal.Resize(cols, rows); err != nil {
		cancel()
		_ = terminal.Close()
		return nil, fmt.Errorf("resize pty: %w", err)
	}
	cmd := terminal.CommandContext(ctx, shell, args...)
	cmd.Env = env
	cmd.Dir = cwd
	if err := cmd.Start(); err != nil {
		cancel()
		_ = terminal.Close()
		return nil, fmt.Errorf("start pty command: %w", err)
	}
	emulator := vt.NewEmulator(cols, rows)
	emulator.SetScrollbackSize(5000)
	window := &Window{
		ID: id, Title: title, pty: terminal, cmd: cmd,
		term: emulator, cancel: cancel,
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
			return
		}
	}
}

func (w *Window) Apply(data []byte) {
	w.Bytes += int64(len(data))
	_, _ = w.term.Write(data)
}

func (w *Window) Write(data []byte) error {
	if _, err := w.pty.Write(data); err != nil {
		return fmt.Errorf("write pty: %w", err)
	}
	return nil
}

func (w *Window) Resize(cols, rows int) {
	if cols < 1 || rows < 1 {
		return
	}
	_ = w.pty.Resize(cols, rows)
	w.term.Resize(cols, rows)
}

func (w *Window) SetScrollbackSize(lines int) {
	w.term.SetScrollbackSize(max(lines, 0))
}

func (w *Window) Lines() []string {
	lines := make([]string, w.term.Height())
	for y := range w.term.Height() {
		lines[y] = w.lineText(y)
	}
	return lines
}

func (w *Window) HistoryLines() []string {
	scrollback := w.term.Scrollback()
	lines := make([]string, 0, scrollback.Len()+w.term.Height())
	for index := range scrollback.Len() {
		lines = append(lines, strings.TrimRight(scrollback.Line(index).String(), " "))
	}
	return append(lines, w.Lines()...)
}

func (w *Window) Cells() [][]proto.Cell {
	result := make([][]proto.Cell, w.term.Height())
	for y := range w.term.Height() {
		row := make([]proto.Cell, 0, w.term.Width())
		for x := range w.term.Width() {
			cell := w.term.CellAt(x, y)
			if cell == nil || cell.Width == 0 {
				continue
			}
			text := cell.Content
			if text == "" {
				text = " "
			}
			row = append(row, proto.Cell{
				X: x, Text: text, Width: max(cell.Width, 1),
				Foreground:     colorString(cell.Style.Fg),
				Background:     colorString(cell.Style.Bg),
				UnderlineColor: colorString(cell.Style.UnderlineColor),
				Attributes:     cell.Style.Attrs,
				Underline:      cell.Style.Underline,
			})
		}
		result[y] = row
	}
	return result
}

func (w *Window) lineText(y int) string {
	var line strings.Builder
	for x := range w.term.Width() {
		cell := w.term.CellAt(x, y)
		if cell == nil || cell.Width == 0 {
			continue
		}
		if cell.Content == "" {
			line.WriteByte(' ')
			continue
		}
		line.WriteString(cell.Content)
	}
	return strings.TrimRight(line.String(), " ")
}

func colorString(value color.Color) string {
	if value == nil {
		return ""
	}
	red, green, blue, _ := value.RGBA()
	return fmt.Sprintf("#%02x%02x%02x", red>>8, green>>8, blue>>8)
}

func (w *Window) Close() {
	w.cancel()
	_ = w.pty.Close()
	_ = w.term.Close()
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
