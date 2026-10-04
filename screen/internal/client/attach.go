package client

import (
	"context"
	"encoding/json"
	"fmt"
	"net"
	"os"
	"strconv"
	"sync/atomic"
	"time"

	"github.com/gdamore/tcell/v3"
	"github.com/lyonbrown4d/terman/common"
	"github.com/lyonbrown4d/terman/screen/internal/proto"
	"github.com/lyonbrown4d/terman/screen/internal/transport"
)

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

	frames := make(chan *proto.Frame, 2)
	failures := make(chan error, 1)
	go readFrames(conn, frames, failures)
	events := screen.EventQ()

	var latest atomic.Pointer[proto.Frame]
	prefix := false
	for {
		select {
		case <-ctx.Done():
			_ = encoder.Encode(proto.Request{Type: "detach", ClientID: id})
			return nil
		case err := <-failures:
			return err
		case frame := <-frames:
			latest.Store(frame)
			draw(screen, frame)
		case event := <-events:
			switch value := event.(type) {
			case *tcell.EventResize:
				cols, rows := screen.Size()
				_ = encoder.Encode(proto.Request{Type: "resize", Cols: cols, Rows: rows})
				screen.Sync()
			case *tcell.EventMouse:
				handleMouse(encoder, latest.Load(), value)
			case *tcell.EventKey:
				action := decodeKey(value, &prefix)
				if action.detach {
					_ = encoder.Encode(proto.Request{Type: "detach", ClientID: id})
					return nil
				}
				if action.command != "" {
					_ = encoder.Encode(proto.Request{
						Type: "command", Command: action.command, Args: action.args,
					})
				} else if len(action.data) > 0 {
					_ = encoder.Encode(proto.Request{Type: "input", Data: action.data})
				}
			}
		}
	}
}

func readFrames(conn net.Conn, frames chan *proto.Frame, failures chan<- error) {
	decoder := json.NewDecoder(conn)
	for {
		var response proto.Response
		if err := decoder.Decode(&response); err != nil {
			failures <- err
			return
		}
		if response.Error != "" {
			failures <- fmt.Errorf("%s", response.Error)
			return
		}
		if response.Type == "detached" || response.Exit {
			failures <- nil
			return
		}
		if response.Frame != nil {
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

func draw(screen tcell.Screen, frame *proto.Frame) {
	screen.Clear()
	for _, region := range frame.Regions {
		style := tcell.StyleDefault
		if region.Focused {
			style = style.Bold(true)
		}
		for y := 0; y < region.Height && y < len(region.Lines); y++ {
			drawText(screen, region.X, region.Y+y, region.Width, region.Lines[y], style)
		}
		if len(frame.Regions) > 1 {
			for x := region.X; x < region.X+region.Width; x++ {
				screen.SetContent(x, region.Y+region.Height-1, '-', nil, style)
			}
		}
	}
	status := fmt.Sprintf("screen %s | %s | Ctrl-A ? help", frame.Session, windowStatus(frame))
	drawText(screen, 0, frame.Rows-1, frame.Cols, status, tcell.StyleDefault.Reverse(true))
	screen.Show()
}

func drawText(screen tcell.Screen, x, y, width int, text string, style tcell.Style) {
	column := 0
	for _, char := range text {
		if column >= width {
			break
		}
		screen.SetContent(x+column, y, char, nil, style)
		column++
	}
}

func windowStatus(frame *proto.Frame) string {
	parts := make([]string, 0, len(frame.Windows))
	for _, window := range frame.Windows {
		marker := "-"
		if window.Active {
			marker = "*"
		}
		parts = append(parts, strconv.Itoa(window.Index)+marker+window.Title)
	}
	return join(parts, " ")
}

func join(values []string, separator string) string {
	result := ""
	for index, value := range values {
		if index > 0 {
			result += separator
		}
		result += value
	}
	return result
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
		if x >= region.X && x < region.X+region.Width && y >= region.Y && y < region.Y+region.Height {
			if !region.Focused && buttons&tcell.Button1 != 0 {
				_ = encoder.Encode(proto.Request{Type: "command", Command: "focus", Args: []string{strconv.Itoa(region.Index)}})
			}
			kind := common.MouseDown
			if buttons == tcell.ButtonNone {
				kind = common.MouseUp
			}
			mouse := common.MouseEvent{
				Kind: kind, Button: common.MouseButtonLeft,
				Column: uint16(max(x-region.X, 0)), Row: uint16(max(y-region.Y, 0)),
			}
			if data, ok := common.MouseEventBytes(mouse); ok {
				_ = encoder.Encode(proto.Request{Type: "input", Data: data, Target: strconv.Itoa(region.Window)})
			}
			return
		}
	}
}
