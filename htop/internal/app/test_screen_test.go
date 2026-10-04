package app

import (
	"sync"

	"github.com/gdamore/tcell/v3"
)

type testCell struct {
	text  string
	style tcell.Style
}

type testScreen struct {
	tcell.Screen
	mu            sync.RWMutex
	width         int
	height        int
	cells         []testCell
	events        chan tcell.Event
	defaultStyle  tcell.Style
	fini          chan struct{}
	mouseDisabled chan struct{}
	finiOnce      sync.Once
	mouseOnce     sync.Once
}

func newTestScreen(width, height int) *testScreen {
	return &testScreen{
		width: width, height: height, events: make(chan tcell.Event, 32),
		fini: make(chan struct{}), mouseDisabled: make(chan struct{}),
	}
}

func (s *testScreen) Init() error {
	s.mu.Lock()
	defer s.mu.Unlock()
	s.cells = make([]testCell, s.width*s.height)
	return nil
}

func (s *testScreen) Fini() {
	s.finiOnce.Do(func() {
		close(s.events)
		close(s.fini)
	})
}

func (s *testScreen) Clear() {
	s.Fill(' ', s.defaultStyle)
}

func (s *testScreen) Fill(value rune, style tcell.Style) {
	s.FillArea(0, 0, s.width, s.height, value, style)
}

func (s *testScreen) FillArea(x, y, width, height int, value rune, style tcell.Style) {
	s.mu.Lock()
	defer s.mu.Unlock()
	for row := max(0, y); row < min(s.height, y+height); row++ {
		for column := max(0, x); column < min(s.width, x+width); column++ {
			s.cells[row*s.width+column] = testCell{text: string(value), style: style}
		}
	}
}

func (s *testScreen) Put(x, y int, value string, style tcell.Style) (string, int) {
	runes := []rune(value)
	if len(runes) == 0 {
		return "", 0
	}
	s.mu.Lock()
	if x >= 0 && x < s.width && y >= 0 && y < s.height {
		s.cells[y*s.width+x] = testCell{text: string(runes[0]), style: style}
	}
	s.mu.Unlock()
	return string(runes[1:]), 1
}

func (s *testScreen) PutStr(x, y int, value string) {
	s.PutStrStyled(x, y, value, s.defaultStyle)
}

func (s *testScreen) PutStrStyled(x, y int, value string, style tcell.Style) {
	for _, character := range value {
		_, width := s.Put(x, y, string(character), style)
		x += width
	}
}

func (s *testScreen) Get(x, y int) (string, tcell.Style, int) {
	s.mu.RLock()
	defer s.mu.RUnlock()
	if x < 0 || x >= s.width || y < 0 || y >= s.height {
		return "", tcell.StyleDefault, 0
	}
	index := y*s.width + x
	if index >= len(s.cells) {
		return "", tcell.StyleDefault, 0
	}
	cell := s.cells[index]
	return cell.text, cell.style, 1
}

func (s *testScreen) SetStyle(style tcell.Style) {
	s.defaultStyle = style
}

func (s *testScreen) Size() (int, int) {
	return s.width, s.height
}

func (s *testScreen) EventQ() chan tcell.Event {
	return s.events
}

func (s *testScreen) EnableMouse(...tcell.MouseFlags) {}

func (s *testScreen) DisableMouse() {
	s.mouseOnce.Do(func() { close(s.mouseDisabled) })
}

func (s *testScreen) Show() {}

func (s *testScreen) Sync() {}

func (s *testScreen) SetSize(width, height int) {
	s.mu.Lock()
	defer s.mu.Unlock()
	s.width, s.height = width, height
	s.cells = make([]testCell, width*height)
}

func testFrame(screen *testScreen) string {
	width, height := screen.Size()
	frame := make([]rune, 0, (width+1)*height)
	for y := range height {
		for x := range width {
			value, _, _ := screen.Get(x, y)
			if value == "" {
				value = " "
			}
			frame = append(frame, []rune(value)[0])
		}
		frame = append(frame, '\n')
	}
	return string(frame)
}
