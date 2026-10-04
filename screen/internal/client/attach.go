package client

import (
	"context"
	"encoding/json"
	"fmt"
	"net"
	"os"
	"time"

	"github.com/gdamore/tcell/v3"
	"github.com/lyonbrown4d/terman/common"
	"github.com/lyonbrown4d/terman/screen/internal/proto"
	"github.com/lyonbrown4d/terman/screen/internal/transport"
)

type attachState struct {
	screen  tcell.Screen
	encoder *json.Encoder
	frame   *proto.Frame
	prefix  bool
	copy    *copyMode
	prompt  *promptMode
	help    bool
	notice  string
}

func Attach(ctx context.Context, endpoint, mode string, detachExisting bool) error {
	conn, err := transport.Dial(ctx, endpoint)
	if err != nil {
		return err
	}
	defer conn.Close()
	id := fmt.Sprintf("%d-%d", os.Getpid(), time.Now().UnixNano())
	encoder := json.NewEncoder(conn)
	if err := encoder.Encode(proto.Request{
		Type: "attach", ClientID: id, Mode: mode, DetachExisting: detachExisting,
	}); err != nil {
		return err
	}
	screen, err := tcell.NewScreen()
	if err != nil {
		return err
	}
	if err := screen.Init(); err != nil {
		return err
	}
	defer screen.Fini()
	screen.EnableMouse()

	state := &attachState{screen: screen, encoder: encoder}
	frames := make(chan *proto.Frame, 2)
	notices := make(chan string, 8)
	failures := make(chan error, 1)
	go readFrames(conn, frames, notices, failures)

	for {
		select {
		case <-ctx.Done():
			_ = state.send(proto.Request{Type: "detach", ClientID: id})
			return nil
		case err := <-failures:
			return err
		case frame := <-frames:
			state.frame = frame
			state.draw()
		case notice := <-notices:
			state.notice = notice
			state.draw()
		case event := <-screen.EventQ():
			detach, eventErr := state.handleEvent(event, id)
			if eventErr != nil {
				return eventErr
			}
			if detach {
				return nil
			}
		}
	}
}

func (s *attachState) handleEvent(event tcell.Event, id string) (bool, error) {
	switch value := event.(type) {
	case *tcell.EventResize:
		cols, rows := value.Size()
		s.screen.Sync()
		return false, s.send(proto.Request{Type: "resize", Cols: cols, Rows: rows})
	case *tcell.EventMouse:
		handleMouse(s.encoder, s.frame, value)
	case *tcell.EventKey:
		return s.handleKey(value, id)
	case *tcell.EventError:
		return false, value
	}
	return false, nil
}

func (s *attachState) handleKey(event *tcell.EventKey, id string) (bool, error) {
	if s.help {
		s.help = false
		s.draw()
		return false, nil
	}
	if s.copy != nil {
		return false, s.handleCopyKey(event)
	}
	if s.prompt != nil {
		return false, s.handlePromptKey(event)
	}
	action := decodeKey(event, &s.prefix)
	if action.detach {
		_ = s.send(proto.Request{Type: "detach", ClientID: id})
		return true, nil
	}
	if action.local != "" {
		s.openLocal(action.local)
		return false, nil
	}
	if action.command != "" {
		return false, s.send(proto.Request{
			Type: "command", Command: action.command, Args: action.args,
		})
	}
	if len(action.data) > 0 {
		return false, s.send(proto.Request{Type: "input", Data: action.data})
	}
	if s.prefix {
		s.notice = "Ctrl-A"
		s.draw()
	}
	return false, nil
}

func (s *attachState) openLocal(mode string) {
	s.notice = ""
	switch mode {
	case "help":
		s.help = true
	case "title":
		initial := ""
		if s.frame != nil {
			for _, window := range s.frame.Windows {
				if window.Active {
					initial = window.Title
				}
			}
		}
		s.prompt = &promptMode{kind: mode, label: "title: ", text: []rune(initial)}
	case "command":
		s.prompt = &promptMode{kind: mode, label: ": "}
	case "copy":
		if history := focusedHistory(s.frame); len(history) > 0 {
			s.copy = newCopyMode(history)
		} else {
			s.notice = "no scrollback available"
		}
	}
	s.draw()
}

func (s *attachState) send(request proto.Request) error {
	if err := s.encoder.Encode(request); err != nil {
		return fmt.Errorf("send screen request: %w", err)
	}
	return nil
}

func readFrames(
	conn net.Conn,
	frames chan *proto.Frame,
	notices chan<- string,
	failures chan<- error,
) {
	decoder := json.NewDecoder(conn)
	seenFrame := false
	for {
		var response proto.Response
		if err := decoder.Decode(&response); err != nil {
			failures <- err
			return
		}
		if response.Error != "" {
			if !seenFrame {
				failures <- fmt.Errorf("%s", response.Error)
				return
			}
			select {
			case notices <- response.Error:
			default:
			}
			continue
		}
		if response.Message != "" {
			select {
			case notices <- response.Message:
			default:
			}
		}
		if response.Type == "detached" || response.Exit {
			failures <- nil
			return
		}
		if response.Frame != nil {
			seenFrame = true
			select {
			case frames <- response.Frame:
			default:
				select {
				case <-frames:
				default:
				}
				frames <- response.Frame
			}
		}
	}
}

func focusedHistory(frame *proto.Frame) []string {
	if frame == nil {
		return nil
	}
	for _, region := range frame.Regions {
		if region.Focused {
			return append([]string(nil), region.History...)
		}
	}
	return nil
}

func handleMouse(encoder *json.Encoder, frame *proto.Frame, event *tcell.EventMouse) {
	if frame == nil {
		return
	}
	x, y := event.Position()
	buttons := event.Buttons()
	if buttons&tcell.WheelUp != 0 {
		_ = encoder.Encode(proto.Request{Type: "command", Command: "prev"})
		return
	}
	if buttons&tcell.WheelDown != 0 {
		_ = encoder.Encode(proto.Request{Type: "command", Command: "next"})
		return
	}
	for _, region := range frame.Regions {
		inside := x >= region.X && x < region.X+region.Width &&
			y >= region.Y && y < region.Y+region.Height
		if !inside {
			continue
		}
		if !region.Focused && buttons&tcell.Button1 != 0 {
			_ = encoder.Encode(proto.Request{
				Type: "command", Command: "focus",
				Args: []string{fmt.Sprint(region.Index)},
			})
		}
		kind := common.MouseDown
		if buttons == tcell.ButtonNone {
			kind = common.MouseUp
		}
		mouse := common.MouseEvent{
			Kind: kind, Button: common.MouseButtonLeft,
			Column: uint16(max(x-region.X, 0)),
			Row:    uint16(max(y-region.Y, 0)),
		}
		if data, ok := common.MouseEventBytes(mouse); ok {
			_ = encoder.Encode(proto.Request{
				Type: "input", Data: data, Target: fmt.Sprint(region.Window),
			})
		}
		return
	}
}
