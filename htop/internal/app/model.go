package app

import (
	"fmt"
	"sort"
	"strings"
	"time"
)

type Config struct {
	Refresh time.Duration
	Once    bool
	Sort    SortKey
	Reverse bool
	Filter  string
}

type Tab int

const (
	TabOverview Tab = iota
	TabProcesses
	TabIO
	TabNetwork
	tabCount
)

var tabNames = [...]string{"Overview", "Processes", "IO", "Network"}

type SortKey int

const (
	SortCPU SortKey = iota
	SortMemory
	SortIO
	SortPID
	SortName
)

func ParseSortKey(value string) (SortKey, error) {
	switch strings.ToLower(value) {
	case "cpu":
		return SortCPU, nil
	case "memory", "mem":
		return SortMemory, nil
	case "io":
		return SortIO, nil
	case "pid":
		return SortPID, nil
	case "name":
		return SortName, nil
	default:
		return SortCPU, fmt.Errorf("invalid --sort %q", value)
	}
}

func (s SortKey) String() string {
	return [...]string{"CPU", "MEM", "IO", "PID", "NAME"}[s]
}

type Snapshot struct {
	At          time.Time
	Hostname    string
	Uptime      uint64
	CPU         []float64
	MemoryUsed  uint64
	MemoryTotal uint64
	Processes   []Process
	IO          []ProcessIO
	Interfaces  []Interface
	Connections []Connection
	Warning     string
}

type Process struct {
	PID        int32
	PPID       int32
	Name       string
	User       string
	Command    string
	Status     string
	CPU        float64
	Memory     uint64
	ReadRate   uint64
	WriteRate  uint64
	ReadTotal  uint64
	WriteTotal uint64
	Started    int64
	Nice       int32
}

type ProcessIO struct {
	PID        int32
	Name       string
	ReadRate   uint64
	WriteRate  uint64
	ReadTotal  uint64
	WriteTotal uint64
}

type Interface struct {
	Name      string
	BytesSent uint64
	BytesRecv uint64
	SendRate  uint64
	RecvRate  uint64
}

type Connection struct {
	PID      int32
	Process  string
	Protocol string
	Local    string
	Remote   string
	Status   string
}

func processRows(snapshot Snapshot, key SortKey, reverse bool, filter string) []Process {
	needle := strings.ToLower(strings.TrimSpace(filter))
	rows := make([]Process, 0, len(snapshot.Processes))
	for _, row := range snapshot.Processes {
		haystack := strings.ToLower(fmt.Sprintf("%d %s %s %s", row.PID, row.User, row.Name, row.Command))
		if needle == "" || strings.Contains(haystack, needle) {
			rows = append(rows, row)
		}
	}
	sort.SliceStable(rows, func(i, j int) bool {
		a, b := rows[i], rows[j]
		var less bool
		switch key {
		case SortMemory:
			less = a.Memory > b.Memory
		case SortIO:
			less = a.ReadRate+a.WriteRate > b.ReadRate+b.WriteRate
		case SortPID:
			less = a.PID < b.PID
		case SortName:
			less = strings.ToLower(a.Name) < strings.ToLower(b.Name)
		default:
			less = a.CPU > b.CPU
		}
		if reverse {
			return !less && !processEqual(a, b, key)
		}
		return less
	})
	return rows
}

func processEqual(a, b Process, key SortKey) bool {
	switch key {
	case SortMemory:
		return a.Memory == b.Memory
	case SortIO:
		return a.ReadRate+a.WriteRate == b.ReadRate+b.WriteRate
	case SortPID:
		return a.PID == b.PID
	case SortName:
		return strings.EqualFold(a.Name, b.Name)
	default:
		return a.CPU == b.CPU
	}
}

func ioRows(snapshot Snapshot, reverse bool) []ProcessIO {
	rows := append([]ProcessIO(nil), snapshot.IO...)
	sort.SliceStable(rows, func(i, j int) bool {
		a := rows[i].ReadRate + rows[i].WriteRate
		b := rows[j].ReadRate + rows[j].WriteRate
		if reverse {
			return a < b
		}
		return a > b
	})
	return rows
}
