package session

import (
	"context"
	"fmt"
	"time"

	"github.com/lyonbrown4d/terman/screen/internal/proto"
	"github.com/lyonbrown4d/terman/screen/internal/ptywin"
)

type requestEvent struct {
	request proto.Request
	reply   chan proto.Response
}

type attachEvent struct {
	id, mode string
	detach   bool
	reply    chan attachResult
}

type attachResult struct {
	queue <-chan proto.Response
	err   error
}

type removeEvent struct{ id string }

type logConfig struct {
	enabled bool
	last    time.Time
}

type Owner struct {
	config     Config
	ctx        context.Context
	cancel     context.CancelFunc
	events     chan any
	output     chan ptywin.Event
	done       chan struct{}
	windows    []*ptywin.Window
	regions    []region
	clients    map[string]chan proto.Response
	registers  map[string][]byte
	logs       map[int64]logConfig
	focused    int
	active     int
	last       int
	cols, rows int
	vertical   bool
	nextID     int64
	term       string
	cwd        string
	env        map[string]string

	pasteBuffer    []byte
	bufferFile     string
	hardcopyDir    string
	hardcopyAppend bool
	scrollback     int
	logfile        string
	deflog         bool
	logTimestamp   bool
	logAfter       time.Duration
	logStamp       string
	lastMessage    string
	started        time.Time
}

func New(parent context.Context, config Config) (*Owner, error) {
	config = normalizeConfig(config)
	ctx, cancel := context.WithCancel(parent)
	owner := &Owner{
		config: config, ctx: ctx, cancel: cancel,
		events: make(chan any, 64), output: make(chan ptywin.Event, 64),
		done: make(chan struct{}), clients: map[string]chan proto.Response{},
		registers: map[string][]byte{}, logs: map[int64]logConfig{},
		cols: config.Cols, rows: config.Rows,
		term: config.Term, cwd: config.Cwd, env: map[string]string{},
		hardcopyDir: config.HardcopyDir, hardcopyAppend: config.HardcopyAppend,
		scrollback: config.Scrollback, logfile: config.Logfile,
		deflog: config.Deflog, logTimestamp: config.LogTimestamp,
		logAfter: config.LogAfter, logStamp: config.LogStamp,
		started: time.Now(),
	}
	if err := owner.newWindow(config.Command); err != nil {
		cancel()
		return nil, err
	}
	go owner.run()
	return owner, nil
}

func (o *Owner) Done() <-chan struct{} { return o.done }

func (o *Owner) Submit(ctx context.Context, request proto.Request) (proto.Response, error) {
	reply := make(chan proto.Response, 1)
	select {
	case o.events <- requestEvent{request: request, reply: reply}:
	case <-ctx.Done():
		return proto.Response{}, ctx.Err()
	case <-o.done:
		return proto.Response{}, context.Canceled
	}
	select {
	case response := <-reply:
		return response, nil
	case <-ctx.Done():
		return proto.Response{}, ctx.Err()
	case <-o.done:
		return proto.Response{}, context.Canceled
	}
}

func (o *Owner) Attach(ctx context.Context, id, mode string, detach bool) (<-chan proto.Response, error) {
	reply := make(chan attachResult, 1)
	select {
	case o.events <- attachEvent{id: id, mode: mode, detach: detach, reply: reply}:
	case <-ctx.Done():
		return nil, ctx.Err()
	case <-o.done:
		return nil, context.Canceled
	}
	select {
	case result := <-reply:
		return result.queue, result.err
	case <-ctx.Done():
		return nil, ctx.Err()
	}
}

func (o *Owner) Remove(id string) {
	select {
	case o.events <- removeEvent{id: id}:
	case <-o.done:
	}
}

func (o *Owner) run() {
	defer close(o.done)
	for {
		select {
		case <-o.ctx.Done():
			o.shutdown()
			return
		case raw := <-o.events:
			if o.handleEvent(raw) {
				o.shutdown()
				return
			}
		case event := <-o.output:
			o.handleOutput(event)
		}
	}
}

func (o *Owner) handleEvent(raw any) bool {
	switch event := raw.(type) {
	case attachEvent:
		o.attach(event)
	case removeEvent:
		o.removeClient(event.id)
	case requestEvent:
		response := o.control(event.request)
		event.reply <- response
		if response.Exit {
			return true
		}
	}
	return false
}

func (o *Owner) attach(event attachEvent) {
	if event.mode == "resume" && len(o.clients) > 0 && !event.detach {
		event.reply <- attachResult{err: fmt.Errorf("session is already attached; use -d -r or -x")}
		return
	}
	if event.detach {
		for id := range o.clients {
			o.detachClient(id)
		}
	}
	queue := make(chan proto.Response, 8)
	o.clients[event.id] = queue
	o.send(queue, proto.Response{Type: "frame", Frame: o.frame()})
	event.reply <- attachResult{queue: queue}
}

func (o *Owner) handleOutput(event ptywin.Event) {
	index := o.windowByID(event.ID)
	if index < 0 {
		return
	}
	if event.Exit {
		o.closeWindow(index)
		return
	}
	o.writeLog(index, event.Data)
	o.windows[index].Apply(event.Data)
	o.broadcast()
}

func (o *Owner) shutdown() {
	for _, window := range o.windows {
		window.Close()
	}
	for id := range o.clients {
		o.detachClient(id)
	}
	o.cancel()
}
