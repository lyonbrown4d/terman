package tuitest

import (
	"strings"
	"sync"
	"time"

	"github.com/gdamore/tcell/v3"
	"github.com/mattn/go-runewidth"
)

type cell struct {
	text  string
	style tcell.Style
	width int
}

type Screen struct {
	tcell.Screen

	mu           sync.RWMutex
	width        int
	height       int
	cells        []cell
	events       chan tcell.Event
	defaultStyle tcell.Style

	finalized     chan struct{}
	mouseDisabled chan struct{}
	pasteDisabled chan struct{}
	finiOnce      sync.Once
	mouseOnce     sync.Once
	pasteOnce     sync.Once
}

func NewScreen(width, height int) *Screen {
	return &Screen{
		width: width, height: height,
		cells:         make([]cell, width*height),
		events:        make(chan tcell.Event, 64),
		finalized:     make(chan struct{}),
		mouseDisabled: make(chan struct{}),
		pasteDisabled: make(chan struct{}),
	}
}

func (s *Screen) Init() error {
	s.mu.Lock()
	defer s.mu.Unlock()
	s.cells = make([]cell, s.width*s.height)
	return nil
}

func (s *Screen) Fini() {
	s.finiOnce.Do(func() { close(s.finalized) })
}

func (s *Screen) Clear() {
	s.Fill(' ', s.defaultStyle)
}

func (s *Screen) Fill(value rune, style tcell.Style) {
	s.FillArea(0, 0, s.width, s.height, value, style)
}

func (s *Screen) FillArea(x, y, width, height int, value rune, style tcell.Style) {
	s.mu.Lock()
	defer s.mu.Unlock()
	for row := max(0, y); row < min(s.height, y+height); row++ {
		for column := max(0, x); column < min(s.width, x+width); column++ {
			s.cells[row*s.width+column] = cell{text: string(value), style: style, width: 1}
		}
	}
}

func (s *Screen) SetContent(x, y int, main rune, combining []rune, style tcell.Style) {
	text := string(append([]rune{main}, combining...))
	width := max(runewidth.RuneWidth(main), 1)
	s.setCell(x, y, text, style, width)
}

func (s *Screen) Put(x, y int, value string, style tcell.Style) (string, int) {
	runes := []rune(value)
	if len(runes) == 0 {
		return "", 0
	}
	width := max(runewidth.RuneWidth(runes[0]), 1)
	s.setCell(x, y, string(runes[0]), style, width)
	return string(runes[1:]), width
}

func (s *Screen) setCell(x, y int, text string, style tcell.Style, width int) {
	s.mu.Lock()
	defer s.mu.Unlock()
	if x < 0 || x >= s.width || y < 0 || y >= s.height {
		return
	}
	s.cells[y*s.width+x] = cell{text: text, style: style, width: width}
	for offset := 1; offset < width && x+offset < s.width; offset++ {
		s.cells[y*s.width+x+offset] = cell{text: " ", style: style, width: 0}
	}
}

func (s *Screen) PutStr(x, y int, value string) {
	s.PutStrStyled(x, y, value, s.defaultStyle)
}

func (s *Screen) PutStrStyled(x, y int, value string, style tcell.Style) {
	for len(value) != 0 {
		rest, width := s.Put(x, y, value, style)
		if width == 0 || rest == value {
			return
		}
		x += width
		value = rest
	}
}

func (s *Screen) Get(x, y int) (string, tcell.Style, int) {
	s.mu.RLock()
	defer s.mu.RUnlock()
	if x < 0 || x >= s.width || y < 0 || y >= s.height || len(s.cells) == 0 {
		return "", tcell.StyleDefault, 0
	}
	value := s.cells[y*s.width+x]
	return value.text, value.style, value.width
}

func (s *Screen) SetStyle(style tcell.Style) {
	s.defaultStyle = style
}

func (s *Screen) Size() (int, int) {
	s.mu.RLock()
	defer s.mu.RUnlock()
	return s.width, s.height
}

func (s *Screen) SetSize(width, height int) {
	s.mu.Lock()
	defer s.mu.Unlock()
	s.width, s.height = width, height
	s.cells = make([]cell, width*height)
}

func (s *Screen) EventQ() chan tcell.Event {
	return s.events
}

func (s *Screen) Send(event tcell.Event) bool {
	select {
	case <-s.finalized:
		return false
	case s.events <- event:
		return true
	}
}

func (s *Screen) EnableMouse(...tcell.MouseFlags) {}

func (s *Screen) DisableMouse() {
	s.mouseOnce.Do(func() { close(s.mouseDisabled) })
}

func (s *Screen) EnablePaste() {}

func (s *Screen) DisablePaste() {
	s.pasteOnce.Do(func() { close(s.pasteDisabled) })
}

func (s *Screen) HideCursor() {}

func (s *Screen) Show() {}

func (s *Screen) Sync() {}

func (s *Screen) Frame() string {
	s.mu.RLock()
	defer s.mu.RUnlock()
	var output strings.Builder
	output.Grow((s.width + 1) * s.height)
	for y := range s.height {
		for x := range s.width {
			text := s.cells[y*s.width+x].text
			if text == "" {
				text = " "
			}
			output.WriteString(text)
		}
		output.WriteByte('\n')
	}
	return output.String()
}

func (s *Screen) WaitFrame(timeout time.Duration, match func(string) bool) (string, bool) {
	deadline := time.Now().Add(timeout)
	for time.Now().Before(deadline) {
		frame := s.Frame()
		if match(frame) {
			return frame, true
		}
		time.Sleep(time.Millisecond)
	}
	return s.Frame(), false
}

func (s *Screen) Finalized() <-chan struct{} {
	return s.finalized
}

func (s *Screen) MouseDisabled() <-chan struct{} {
	return s.mouseDisabled
}

func (s *Screen) PasteDisabled() <-chan struct{} {
	return s.pasteDisabled
}
