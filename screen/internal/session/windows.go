package session

import (
	"fmt"
	"path/filepath"
	"strings"

	"github.com/lyonbrown4d/terman/common"
	"github.com/lyonbrown4d/terman/screen/internal/proto"
	"github.com/lyonbrown4d/terman/screen/internal/ptywin"
)

func (o *Owner) newWindow(command string) error {
	o.nextID++
	shell := common.DefaultShell()
	args := []string{}
	title := filepath.Base(shell)
	if command != "" {
		args = append(common.ShellCommandArgs(shell, o.config.LoginShell), command)
		title = command
	}
	window, err := ptywin.New(o.ctx, o.nextID, title, shell, args,
		ptywin.Environment(o.term, o.env), o.cwd, o.cols, o.rows-1, o.output)
	if err != nil {
		return err
	}
	o.last = o.active
	o.windows = append(o.windows, window)
	o.active = len(o.windows) - 1
	if len(o.regions) == 0 {
		o.regions = []region{{windowID: window.ID}}
	} else {
		o.regions[o.focused].windowID = window.ID
	}
	o.broadcast()
	return nil
}

func (o *Owner) closeWindow(index int) {
	if index < 0 || index >= len(o.windows) {
		return
	}
	id := o.windows[index].ID
	o.windows[index].Close()
	o.windows = append(o.windows[:index], o.windows[index+1:]...)
	if len(o.windows) == 0 {
		o.cancel()
		return
	}
	fallback := o.windows[min(index, len(o.windows)-1)].ID
	for regionIndex := range o.regions {
		if o.regions[regionIndex].windowID == id {
			o.regions[regionIndex].windowID = fallback
		}
	}
	o.active = min(o.active, len(o.windows)-1)
	o.broadcast()
}

func (o *Owner) selectWindow(selector string) error {
	if selector == "" || selector == "." {
		return nil
	}
	if selector == "-" {
		o.last, o.active = o.active, o.last
	} else if value, err := atoi(selector); err == nil {
		if value < 0 || value >= len(o.windows) {
			return fmt.Errorf("window %s not found", selector)
		}
		o.last, o.active = o.active, value
	} else {
		found := -1
		for index, window := range o.windows {
			if window.Title == selector || strings.Contains(window.Title, selector) {
				found = index
				break
			}
		}
		if found < 0 {
			return fmt.Errorf("window %s not found", selector)
		}
		o.last, o.active = o.active, found
	}
	o.regions[o.focused].windowID = o.windows[o.active].ID
	o.broadcast()
	return nil
}

func atoi(value string) (int, error) {
	result := 0
	if value == "" {
		return 0, fmt.Errorf("empty")
	}
	for _, char := range value {
		if char < '0' || char > '9' {
			return 0, fmt.Errorf("not numeric")
		}
		result = result*10 + int(char-'0')
	}
	return result, nil
}

func (o *Owner) navigate(delta int) {
	if len(o.windows) == 0 {
		return
	}
	o.last = o.active
	o.active = (o.active + delta + len(o.windows)) % len(o.windows)
	o.regions[o.focused].windowID = o.windows[o.active].ID
	o.broadcast()
}

func (o *Owner) split(vertical bool) {
	o.vertical = vertical
	current := o.regions[o.focused]
	o.regions = append(o.regions, region{})
	copy(o.regions[o.focused+2:], o.regions[o.focused+1:])
	o.regions[o.focused+1] = current
	o.focused++
	o.resizeWindows()
	o.broadcast()
}

func (o *Owner) focus(index int) {
	if len(o.regions) == 0 {
		return
	}
	if index < 0 {
		index = (o.focused + 1) % len(o.regions)
	}
	if index >= 0 && index < len(o.regions) {
		o.focused = index
		o.active = o.windowByID(o.regions[index].windowID)
		o.broadcast()
	}
}

func (o *Owner) removeRegion() {
	if len(o.regions) <= 1 {
		return
	}
	o.regions = append(o.regions[:o.focused], o.regions[o.focused+1:]...)
	o.focused = min(o.focused, len(o.regions)-1)
	o.resizeWindows()
	o.broadcast()
}

func (o *Owner) onlyRegion() {
	o.regions = []region{o.regions[o.focused]}
	o.focused = 0
	o.resizeWindows()
	o.broadcast()
}

func (o *Owner) resizeWindows() {
	geometry := layout(o.regions, o.focused, o.cols, o.rows, o.vertical)
	for _, item := range geometry {
		index := o.windowByID(int64(item.Window))
		if index >= 0 {
			o.windows[index].Resize(item.Width, item.Height)
		}
	}
}

func (o *Owner) windowByID(id int64) int {
	for index, window := range o.windows {
		if window.ID == id {
			return index
		}
	}
	return -1
}

func (o *Owner) frame() *proto.Frame {
	frame := &proto.Frame{
		Session: o.config.Name, Cols: o.cols, Rows: o.rows,
		Attached: len(o.clients), Status: o.lastMessage,
	}
	for index, window := range o.windows {
		frame.Windows = append(frame.Windows, proto.Window{
			Index: index, Title: window.Title, Active: index == o.active, Bytes: window.Bytes,
		})
	}
	frame.Regions = layout(o.regions, o.focused, o.cols, o.rows, o.vertical)
	for index := range frame.Regions {
		windowIndex := o.windowByID(int64(frame.Regions[index].Window))
		if windowIndex >= 0 {
			frame.Regions[index].Window = windowIndex
			frame.Regions[index].Lines = o.windows[windowIndex].Lines()
		}
	}
	return frame
}

func (o *Owner) broadcast() {
	response := proto.Response{Type: "frame", Frame: o.frame()}
	for _, queue := range o.clients {
		o.send(queue, response)
	}
}

func (o *Owner) send(queue chan proto.Response, response proto.Response) {
	select {
	case queue <- response:
	default:
		select {
		case <-queue:
		default:
		}
		select {
		case queue <- response:
		default:
		}
	}
}

func (o *Owner) detachClient(id string) {
	queue, ok := o.clients[id]
	if !ok {
		return
	}
	delete(o.clients, id)
	o.send(queue, proto.Response{Type: "detached"})
	close(queue)
}

func (o *Owner) removeClient(id string) {
	queue, ok := o.clients[id]
	if ok {
		delete(o.clients, id)
		close(queue)
	}
}
