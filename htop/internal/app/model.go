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

type ProcessRow struct {
	Process
	Depth       int
	HasChildren bool
	Collapsed   bool
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
	return filteredProcesses(snapshot.Processes, key, reverse, filter, "")
}

func filteredProcesses(processes []Process, key SortKey, reverse bool, filter, user string) []Process {
	needle := strings.ToLower(strings.TrimSpace(filter))
	rows := make([]Process, 0, len(processes))
	for _, row := range processes {
		if user != "" && row.User != user {
			continue
		}
		haystack := strings.ToLower(fmt.Sprintf("%d %s %s %s", row.PID, row.User, row.Name, row.Command))
		if needle == "" || strings.Contains(haystack, needle) {
			rows = append(rows, row)
		}
	}
	sortProcesses(rows, key, reverse)
	return rows
}

func sortProcesses(rows []Process, key SortKey, reverse bool) {
	sort.SliceStable(rows, func(i, j int) bool {
		comparison := compareProcess(rows[i], rows[j], key)
		if reverse {
			return comparison > 0
		}
		return comparison < 0
	})
}

func compareProcess(a, b Process, key SortKey) int {
	switch key {
	case SortMemory:
		return compareDescending(a.Memory, b.Memory)
	case SortIO:
		return compareDescending(a.ReadRate+a.WriteRate, b.ReadRate+b.WriteRate)
	case SortPID:
		return compareOrdered(a.PID, b.PID)
	case SortName:
		return strings.Compare(strings.ToLower(a.Name), strings.ToLower(b.Name))
	default:
		if a.CPU > b.CPU {
			return -1
		}
		if a.CPU < b.CPU {
			return 1
		}
		return compareOrdered(a.PID, b.PID)
	}
}

func compareDescending[T ~uint64](a, b T) int {
	if a > b {
		return -1
	}
	if a < b {
		return 1
	}
	return 0
}

func compareOrdered[T ~int32](a, b T) int {
	if a < b {
		return -1
	}
	if a > b {
		return 1
	}
	return 0
}

func treeProcessRows(processes []Process, collapsed map[int32]bool) []ProcessRow {
	byPID := make(map[int32]Process, len(processes))
	children := make(map[int32][]Process, len(processes))
	roots := make([]Process, 0)
	for _, process := range processes {
		byPID[process.PID] = process
	}
	for _, process := range processes {
		if _, ok := byPID[process.PPID]; ok && process.PPID != process.PID {
			children[process.PPID] = append(children[process.PPID], process)
			continue
		}
		roots = append(roots, process)
	}
	order := make(map[int32]int, len(processes))
	for index, process := range processes {
		order[process.PID] = index
	}
	sortByOrder := func(values []Process) {
		sort.SliceStable(values, func(i, j int) bool { return order[values[i].PID] < order[values[j].PID] })
	}
	sortByOrder(roots)
	for pid := range children {
		sortByOrder(children[pid])
	}
	rows := make([]ProcessRow, 0, len(processes))
	visited := make(map[int32]bool, len(processes))
	var walk func(Process, int)
	walk = func(process Process, depth int) {
		if visited[process.PID] {
			return
		}
		visited[process.PID] = true
		childRows := children[process.PID]
		isCollapsed := collapsed[process.PID] && len(childRows) != 0
		rows = append(rows, ProcessRow{
			Process: process, Depth: depth, HasChildren: len(childRows) != 0, Collapsed: isCollapsed,
		})
		if isCollapsed {
			return
		}
		for _, child := range childRows {
			walk(child, depth+1)
		}
	}
	for _, root := range roots {
		walk(root, 0)
	}
	for _, process := range processes {
		walk(process, 0)
	}
	return rows
}

func ioRows(snapshot Snapshot, reverse bool) []ProcessIO {
	rows := append([]ProcessIO{}, snapshot.IO...)
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
