package protocol

import "time"

const SchemaVersion = 1

type Request struct {
	ID         uint64      `json:"id,omitempty"`
	Op         string      `json:"op"`
	Target     string      `json:"target,omitempty"`
	Name       string      `json:"name,omitempty"`
	ClientID   string      `json:"client_id,omitempty"`
	Window     *int        `json:"window,omitempty"`
	Pane       *int        `json:"pane,omitempty"`
	SourcePane *int        `json:"source_pane,omitempty"`
	Data       string      `json:"data,omitempty"`
	Args       []string    `json:"args,omitempty"`
	Width      int         `json:"width,omitempty"`
	Height     int         `json:"height,omitempty"`
	Horizontal bool        `json:"horizontal,omitempty"`
	Direction  string      `json:"direction,omitempty"`
	Delta      int         `json:"delta,omitempty"`
	Enabled    *bool       `json:"enabled,omitempty"`
	Mouse      *MouseEvent `json:"mouse,omitempty"`
}

type Response struct {
	ID       uint64        `json:"id,omitempty"`
	OK       bool          `json:"ok"`
	Error    string        `json:"error,omitempty"`
	Event    string        `json:"event,omitempty"`
	Data     string        `json:"data,omitempty"`
	Session  *SessionInfo  `json:"session,omitempty"`
	Sessions []SessionInfo `json:"sessions,omitempty"`
	Windows  []WindowInfo  `json:"windows,omitempty"`
	Panes    []PaneInfo    `json:"panes,omitempty"`
	Buffers  []BufferInfo  `json:"buffers,omitempty"`
	Clients  []ClientInfo  `json:"clients,omitempty"`
	Frame    *Frame        `json:"frame,omitempty"`
}

type SessionInfo struct {
	Name          string    `json:"name"`
	CreatedAt     time.Time `json:"created_at"`
	Windows       int       `json:"windows"`
	ActiveWindow  int       `json:"active_window"`
	Attached      int       `json:"attached"`
	NextWindow    int       `json:"next_window_index"`
	Synchronize   bool      `json:"synchronize_panes"`
	Endpoint      string    `json:"endpoint,omitempty"`
	ServerPID     int       `json:"server_pid,omitempty"`
	SchemaVersion int       `json:"schema_version"`
}

type WindowInfo struct {
	Index       int    `json:"window_index"`
	Name        string `json:"name"`
	Active      bool   `json:"active"`
	PaneCount   int    `json:"pane_count"`
	ActivePane  int    `json:"active_pane"`
	NextPane    int    `json:"next_pane_index"`
	Layout      string `json:"layout"`
	Synchronize bool   `json:"synchronize_panes"`
	Zoomed      bool   `json:"zoomed"`
}

type PaneInfo struct {
	Index  int  `json:"pane_index"`
	Window int  `json:"window_index"`
	Active bool `json:"active"`
	Dead   bool `json:"dead"`
	Width  int  `json:"width"`
	Height int  `json:"height"`
}

type BufferInfo struct {
	Name    string    `json:"name"`
	Bytes   int       `json:"bytes"`
	Preview string    `json:"preview"`
	Created time.Time `json:"created_at"`
}

type ClientInfo struct {
	ID       string    `json:"id"`
	Attached time.Time `json:"attached_at"`
	Width    int       `json:"width"`
	Height   int       `json:"height"`
}

type Rect struct {
	X int `json:"x"`
	Y int `json:"y"`
	W int `json:"width"`
	H int `json:"height"`
}

type PaneFrame struct {
	Index  int      `json:"pane_index"`
	Rect   Rect     `json:"rect"`
	Lines  []string `json:"lines"`
	Active bool     `json:"active"`
	Dead   bool     `json:"dead"`
}

type WindowHit struct {
	Index int `json:"window_index"`
	Start int `json:"start"`
	End   int `json:"end"`
}

type Frame struct {
	Session     string      `json:"session"`
	Window      int         `json:"window"`
	ActivePane  int         `json:"active_pane"`
	Panes       []PaneFrame `json:"panes"`
	Status      string      `json:"status"`
	WindowHits  []WindowHit `json:"window_hits"`
	Message     string      `json:"message,omitempty"`
	GeneratedAt time.Time   `json:"generated_at"`
}

type MouseEvent struct {
	X      int    `json:"x"`
	Y      int    `json:"y"`
	Button string `json:"button"`
	Action string `json:"action"`
	Mod    int    `json:"mod,omitempty"`
}
