package proto

type Request struct {
	Type           string
	ClientID       string
	Mode           string
	DetachExisting bool
	Command        string
	Args           []string
	Target         string
	Data           []byte
	Cols           int
	Rows           int
}

type Response struct {
	Type    string
	Error   string
	Message string
	Frame   *Frame
	Exit    bool
}

type Frame struct {
	Session  string
	Cols     int
	Rows     int
	Attached int
	Windows  []Window
	Regions  []Region
	Status   string
}

type Window struct {
	Index  int
	Title  string
	Active bool
	Bytes  int64
}

type Region struct {
	Index   int
	X       int
	Y       int
	Width   int
	Height  int
	Focused bool
	Window  int
	Lines   []string
}
