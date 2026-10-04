package session

import (
	"context"
	"time"

	"github.com/aymanbagabas/go-pty"
	"github.com/charmbracelet/x/vt"
	"github.com/lyonbrown4d/terman/tmux/internal/protocol"
)

const (
	defaultCols = 80
	defaultRows = 23
	maxHistory  = 1 << 20
)

type Pane struct {
	Index   int
	PTY     pty.Pty
	Term    *vt.Emulator
	Command *pty.Cmd
	History []byte
	Dead    bool
	Width   int
	Height  int
}

type Window struct {
	Index       int
	Name        string
	Panes       map[int]*Pane
	PaneOrder   []int
	ActivePane  int
	LastPane    int
	NextPane    int
	Layout      *Layout
	LayoutName  string
	Synchronize bool
	Zoomed      bool
}

type Buffer struct {
	Name    string
	Data    []byte
	Created time.Time
}

type Client struct {
	ID       string
	Attached time.Time
	Width    int
	Height   int
	Frames   chan protocol.Response
}

type State struct {
	Name              string
	CreatedAt         time.Time
	Windows           map[int]*Window
	WindowOrder       []int
	ActiveWindow      int
	LastWindow        int
	NextWindow        int
	Buffers           map[string]Buffer
	BufferOrder       []string
	NextBuffer        int
	Clients           map[uint64]*Client
	NextClient        uint64
	Cols              int
	Rows              int
	Message           string
	DisplayPanesUntil time.Time
}

type requestEvent struct {
	request protocol.Request
	result  chan protocol.Response
}
type outputEvent struct {
	window int
	pane   int
	data   []byte
}
type exitEvent struct {
	window int
	pane   int
}
type subscribeEvent struct {
	client *Client
	result chan subscription
}
type unsubscribeEvent struct{ id uint64 }
type resizeClientEvent struct {
	id     uint64
	width  int
	height int
	result chan error
}
type subscription struct {
	id    uint64
	frame protocol.Frame
	ch    <-chan protocol.Response
}

type Reactor struct {
	ctx                 context.Context
	cancel              context.CancelFunc
	events              chan any
	done                chan struct{}
	state               State
	command             string
	displayPanesTimeout time.Duration
}

func New(
	ctx context.Context,
	name,
	command string,
	cols,
	rows int,
	options ...Option,
) (*Reactor, error) {
	if cols <= 0 {
		cols = defaultCols
	}
	if rows <= 1 {
		rows = defaultRows + 1
	}
	settings := defaultReactorOptions()
	for _, option := range options {
		if option != nil {
			option(&settings)
		}
	}
	child, cancel := context.WithCancel(ctx)
	r := &Reactor{
		ctx: child, cancel: cancel,
		events: make(chan any, 256), done: make(chan struct{}),
		command:             command,
		displayPanesTimeout: settings.displayPanesTimeout,
		state: State{
			Name: name, CreatedAt: time.Now(),
			Windows: make(map[int]*Window),
			Buffers: make(map[string]Buffer),
			Clients: make(map[uint64]*Client),
			Cols:    cols, Rows: rows,
		},
	}
	if _, err := r.newWindow("shell", command); err != nil {
		cancel()
		return nil, err
	}
	go r.loop()
	return r, nil
}

func (r *Reactor) Done() <-chan struct{} { return r.done }

func (r *Reactor) Stop() {
	select {
	case r.events <- requestEvent{
		request: protocol.Request{Op: "shutdown"},
		result:  make(chan protocol.Response, 1),
	}:
	case <-r.done:
	}
}

func (r *Reactor) Call(
	ctx context.Context,
	req protocol.Request,
) (protocol.Response, error) {
	result := make(chan protocol.Response, 1)
	select {
	case r.events <- requestEvent{request: req, result: result}:
	case <-ctx.Done():
		return protocol.Response{}, ctx.Err()
	case <-r.done:
		return protocol.Response{}, context.Canceled
	}
	select {
	case resp := <-result:
		return resp, nil
	case <-ctx.Done():
		return protocol.Response{}, ctx.Err()
	case <-r.done:
		return protocol.Response{}, context.Canceled
	}
}

func (r *Reactor) Subscribe(
	ctx context.Context,
	client *Client,
) (uint64, protocol.Frame, <-chan protocol.Response, error) {
	result := make(chan subscription, 1)
	select {
	case r.events <- subscribeEvent{client: client, result: result}:
	case <-ctx.Done():
		return 0, protocol.Frame{}, nil, ctx.Err()
	case <-r.done:
		return 0, protocol.Frame{}, nil, context.Canceled
	}
	select {
	case sub := <-result:
		return sub.id, sub.frame, sub.ch, nil
	case <-ctx.Done():
		return 0, protocol.Frame{}, nil, ctx.Err()
	case <-r.done:
		return 0, protocol.Frame{}, nil, context.Canceled
	}
}

func (r *Reactor) ResizeClient(
	ctx context.Context,
	id uint64,
	width,
	height int,
) error {
	result := make(chan error, 1)
	select {
	case r.events <- resizeClientEvent{
		id: id, width: width, height: height, result: result,
	}:
	case <-ctx.Done():
		return ctx.Err()
	case <-r.done:
		return context.Canceled
	}
	select {
	case err := <-result:
		return err
	case <-ctx.Done():
		return ctx.Err()
	case <-r.done:
		return context.Canceled
	}
}

func (r *Reactor) Unsubscribe(id uint64) {
	select {
	case r.events <- unsubscribeEvent{id: id}:
	case <-r.done:
	}
}
