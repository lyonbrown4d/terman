package client

import (
	"context"
	"fmt"
	"os"
	"os/signal"
	"strings"
	"unicode/utf8"

	"github.com/gdamore/tcell/v3"
	"github.com/lyonbrown4d/terman/tmux/internal/protocol"
	"github.com/lyonbrown4d/terman/tmux/internal/store"
)

type attachState struct {
	stream       *Stream
	screen       tcell.Screen
	frame        *protocol.Frame
	prefix       bool
	nextID       uint64
	copy         *copyMode
	copyRequest  uint64
	mouseButton  tcell.ButtonMask
	displayInput string
}

func Attach(parent context.Context, record store.Record) error {
	ctx, cancel := signal.NotifyContext(parent, os.Interrupt)
	defer cancel()
	stream, err := OpenStream(ctx, record)
	if err != nil {
		return err
	}
	defer stream.Close()
	screen, err := tcell.NewScreen()
	if err != nil {
		return fmt.Errorf("create terminal screen: %w", err)
	}
	if err := screen.Init(); err != nil {
		return fmt.Errorf("initialize terminal screen: %w", err)
	}
	defer func() {
		screen.DisableMouse()
		screen.DisablePaste()
		screen.Fini()
	}()
	screen.EnableMouse(tcell.MouseButtonEvents, tcell.MouseDragEvents)
	screen.EnablePaste()
	width, height := screen.Size()
	state := &attachState{
		stream: stream, screen: screen, nextID: 1,
	}
	attach := protocol.Request{
		ID: state.id(), Op: "attach",
		ClientID: fmt.Sprintf("%d", os.Getpid()),
		Width:    width, Height: height,
	}
	if err := stream.Encoder.Encode(attach); err != nil {
		return fmt.Errorf("send attach request: %w", err)
	}
	var first protocol.Response
	if err := stream.Decoder.Decode(&first); err != nil {
		return fmt.Errorf("read attach response: %w", err)
	}
	if !first.OK || first.Frame == nil {
		return fmt.Errorf("attach rejected: %s", first.Error)
	}
	state.frame = first.Frame
	state.draw()

	events := make(chan tcell.Event, 8)
	responses := make(chan protocol.Response, 8)
	go pollEvents(ctx, screen, events)
	go readResponses(ctx, stream, responses)
	for {
		select {
		case <-ctx.Done():
			return nil
		case event := <-events:
			detach, err := state.handleEvent(event)
			if err != nil {
				return err
			}
			if detach {
				return nil
			}
		case response, ok := <-responses:
			if !ok {
				if ctx.Err() != nil {
					return nil
				}
				return fmt.Errorf("attach stream closed")
			}
			if !response.OK {
				state.message(response.Error)
				continue
			}
			if response.Frame != nil {
				state.frame = response.Frame
				if !state.frame.DisplayPanes {
					state.displayInput = ""
				}
				state.draw()
			}
			if state.copyRequest != 0 && response.ID == state.copyRequest {
				state.copy = newCopyMode(response.Data)
				state.copyRequest = 0
				state.mouseButton = 0
				state.draw()
			}
			if response.Event == "detached" {
				return nil
			}
		}
	}
}

func pollEvents(
	ctx context.Context,
	screen tcell.Screen,
	output chan<- tcell.Event,
) {
	for {
		event := <-screen.EventQ()
		select {
		case output <- event:
		case <-ctx.Done():
			return
		}
	}
}

func readResponses(
	ctx context.Context,
	stream *Stream,
	output chan<- protocol.Response,
) {
	defer close(output)
	for {
		var response protocol.Response
		if stream.Decoder.Decode(&response) != nil {
			return
		}
		select {
		case output <- response:
		case <-ctx.Done():
			return
		}
	}
}

func (s *attachState) id() uint64 {
	value := s.nextID
	s.nextID++
	return value
}

func (s *attachState) send(req protocol.Request) error {
	if req.ID == 0 {
		req.ID = s.id()
	}
	return s.stream.Encoder.Encode(req)
}

func (s *attachState) handleEvent(event tcell.Event) (bool, error) {
	switch value := event.(type) {
	case *tcell.EventResize:
		width, height := value.Size()
		s.screen.Sync()
		return false, s.send(protocol.Request{
			Op: "resize-client", Width: width, Height: height,
		})
	case *tcell.EventMouse:
		return false, s.handleMouse(value)
	case *tcell.EventKey:
		if s.copy != nil {
			return false, s.handleCopyKey(value)
		}
		if s.displayPanesActive() {
			return false, s.handleDisplayKey(value)
		}
		return s.handleKey(value)
	case *tcell.EventError:
		return false, value
	}
	return false, nil
}

func (s *attachState) handleKey(key *tcell.EventKey) (bool, error) {
	if s.prefix {
		s.prefix = false
		return s.handlePrefix(key)
	}
	if isCtrl(key, 'b') {
		s.prefix = true
		s.message("prefix")
		return false, nil
	}
	return false, s.send(protocol.Request{
		Op: "input", Data: string(keyBytes(key)),
	})
}

func isCtrl(key *tcell.EventKey, char rune) bool {
	return key.Key() == tcell.Key(char-'a'+1) ||
		(key.Key() == tcell.KeyRune &&
			key.Modifiers()&tcell.ModCtrl != 0 &&
			strings.EqualFold(key.Str(), string(char)))
}

func keyBytes(key *tcell.EventKey) []byte {
	if key.Key() == tcell.KeyRune {
		if key.Modifiers()&tcell.ModCtrl != 0 && len(key.Str()) == 1 {
			r, _ := utf8.DecodeRuneInString(strings.ToLower(key.Str()))
			if r >= 'a' && r <= 'z' {
				return []byte{byte(r - 'a' + 1)}
			}
		}
		return []byte(key.Str())
	}
	switch key.Key() {
	case tcell.KeyEnter:
		return []byte{'\r'}
	case tcell.KeyTab:
		return []byte{'\t'}
	case tcell.KeyBacktab:
		return []byte("\x1b[Z")
	case tcell.KeyBackspace, tcell.KeyBackspace2:
		return []byte{0x7f}
	case tcell.KeyEsc:
		return []byte{0x1b}
	case tcell.KeyUp:
		return []byte("\x1b[A")
	case tcell.KeyDown:
		return []byte("\x1b[B")
	case tcell.KeyRight:
		return []byte("\x1b[C")
	case tcell.KeyLeft:
		return []byte("\x1b[D")
	case tcell.KeyHome:
		return []byte("\x1b[H")
	case tcell.KeyEnd:
		return []byte("\x1b[F")
	case tcell.KeyPgUp:
		return []byte("\x1b[5~")
	case tcell.KeyPgDn:
		return []byte("\x1b[6~")
	case tcell.KeyDelete:
		return []byte("\x1b[3~")
	default:
		return nil
	}
}

func (s *attachState) message(text string) {
	if s.frame == nil {
		return
	}
	copy := *s.frame
	copy.Message = text
	s.frame = &copy
	s.draw()
}
