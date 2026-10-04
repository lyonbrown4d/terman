package app

import (
	"context"
	"fmt"
	"strings"
	"time"

	"github.com/shirou/gopsutil/v4/cpu"
	"github.com/shirou/gopsutil/v4/host"
	gnet "github.com/shirou/gopsutil/v4/net"
	gprocess "github.com/shirou/gopsutil/v4/process"
)

type procSample struct {
	cpu   float64
	read  uint64
	write uint64
}

type netSample struct {
	sent uint64
	recv uint64
}

type collector struct {
	at      time.Time
	procs   map[int32]procSample
	network map[string]netSample
}

// Collector supplies process and system snapshots to a Runner.
type Collector interface {
	Collect(context.Context) Snapshot
}

// NewCollector creates the default gopsutil-backed collector.
func NewCollector() Collector {
	return newCollector()
}

func newCollector() *collector {
	return &collector{procs: make(map[int32]procSample), network: make(map[string]netSample)}
}

func (c *collector) Collect(ctx context.Context) Snapshot {
	return c.collect(ctx)
}

func (c *collector) collect(ctx context.Context) Snapshot {
	now := time.Now()
	elapsed := now.Sub(c.at).Seconds()
	snapshot := Snapshot{At: now}
	var warnings []string
	if info, err := host.InfoWithContext(ctx); err == nil {
		snapshot.Hostname, snapshot.Uptime = info.Hostname, info.Uptime
	} else {
		warnings = append(warnings, "host: "+err.Error())
	}
	if values, err := cpu.PercentWithContext(ctx, 0, true); err == nil {
		snapshot.CPU = values
	} else {
		warnings = append(warnings, "cpu: "+err.Error())
	}
	if memory, err := memoryProvider(ctx); err == nil {
		snapshot.MemoryUsed, snapshot.MemoryTotal = memory.Used, memory.Total
	} else {
		warnings = append(warnings, "memory: "+err.Error())
	}
	processes, ioRows, next, err := c.collectProcesses(ctx, elapsed)
	if err != nil {
		warnings = append(warnings, "processes: "+err.Error())
	}
	snapshot.Processes, snapshot.IO, c.procs = processes, ioRows, next
	interfaces, nextNetwork, err := c.collectNetwork(ctx, elapsed)
	if err != nil {
		warnings = append(warnings, "network: "+err.Error())
	}
	snapshot.Interfaces, c.network = interfaces, nextNetwork
	snapshot.Connections, err = collectConnections(ctx, processes)
	if err != nil {
		warnings = append(warnings, "connections: "+err.Error())
	}
	c.at = now
	snapshot.Warning = strings.Join(warnings, " | ")
	return snapshot
}

func (c *collector) collectProcesses(ctx context.Context, elapsed float64) ([]Process, []ProcessIO, map[int32]procSample, error) {
	systemProcesses, err := gprocess.ProcessesWithContext(ctx)
	if err != nil {
		return nil, nil, c.procs, err
	}
	rows := make([]Process, 0, len(systemProcesses))
	ioRows := make([]ProcessIO, 0, len(systemProcesses))
	next := make(map[int32]procSample, len(systemProcesses))
	for index, process := range systemProcesses {
		if index%32 == 0 {
			select {
			case <-ctx.Done():
				return rows, ioRows, next, ctx.Err()
			default:
			}
		}
		row, sample := c.processRow(ctx, process, elapsed)
		next[row.PID] = sample
		rows = append(rows, row)
		if row.ReadTotal != 0 || row.WriteTotal != 0 {
			ioRows = append(ioRows, ProcessIO{
				PID: row.PID, Name: row.Name, ReadRate: row.ReadRate,
				WriteRate: row.WriteRate, ReadTotal: row.ReadTotal, WriteTotal: row.WriteTotal,
			})
		}
	}
	return rows, ioRows, next, nil
}

func (c *collector) processRow(ctx context.Context, process *gprocess.Process, elapsed float64) (Process, procSample) {
	row := Process{PID: process.Pid, User: "-", Status: "-"}
	row.PPID, _ = process.PpidWithContext(ctx)
	row.Name, _ = process.NameWithContext(ctx)
	row.User, _ = process.UsernameWithContext(ctx)
	row.Command, _ = process.CmdlineWithContext(ctx)
	row.Started, _ = process.CreateTimeWithContext(ctx)
	row.Nice, _ = process.NiceWithContext(ctx)
	if states, err := process.StatusWithContext(ctx); err == nil && len(states) != 0 {
		row.Status = states[0]
	}
	if memory, err := process.MemoryInfoWithContext(ctx); err == nil && memory != nil {
		row.Memory = memory.RSS
	}
	sample := procSample{}
	if times, err := process.TimesWithContext(ctx); err == nil && times != nil {
		sample.cpu = times.Total()
	}
	if counters, err := process.IOCountersWithContext(ctx); err == nil && counters != nil {
		sample.read, sample.write = counters.ReadBytes, counters.WriteBytes
		row.ReadTotal, row.WriteTotal = counters.ReadBytes, counters.WriteBytes
	}
	if previous, ok := c.procs[row.PID]; ok && elapsed > 0 {
		row.CPU = positiveFloat(sample.cpu-previous.cpu) / elapsed * 100
		row.ReadRate = rate(sample.read, previous.read, elapsed)
		row.WriteRate = rate(sample.write, previous.write, elapsed)
	}
	return row, sample
}

func (c *collector) collectNetwork(ctx context.Context, elapsed float64) ([]Interface, map[string]netSample, error) {
	counters, err := gnet.IOCountersWithContext(ctx, true)
	if err != nil {
		return nil, c.network, err
	}
	rows := make([]Interface, 0, len(counters))
	next := make(map[string]netSample, len(counters))
	for _, counter := range counters {
		sample := netSample{sent: counter.BytesSent, recv: counter.BytesRecv}
		row := Interface{Name: counter.Name, BytesSent: sample.sent, BytesRecv: sample.recv}
		if previous, ok := c.network[counter.Name]; ok && elapsed > 0 {
			row.SendRate = rate(sample.sent, previous.sent, elapsed)
			row.RecvRate = rate(sample.recv, previous.recv, elapsed)
		}
		rows = append(rows, row)
		next[counter.Name] = sample
	}
	return rows, next, nil
}

func collectConnections(ctx context.Context, processes []Process) ([]Connection, error) {
	connections, err := gnet.ConnectionsWithContext(ctx, "inet")
	if err != nil {
		return nil, err
	}
	names := make(map[int32]string, len(processes))
	for _, process := range processes {
		names[process.PID] = process.Name
	}
	rows := make([]Connection, 0, len(connections))
	for _, connection := range connections {
		protocol := "TCP"
		if connection.Type == 2 {
			protocol = "UDP"
		}
		rows = append(rows, Connection{
			PID: connection.Pid, Process: names[connection.Pid], Protocol: protocol,
			Local:  endpoint(connection.Laddr.IP, connection.Laddr.Port),
			Remote: endpoint(connection.Raddr.IP, connection.Raddr.Port), Status: connection.Status,
		})
	}
	return rows, nil
}

func endpoint(address string, port uint32) string {
	if address == "" {
		return "-"
	}
	if strings.Contains(address, ":") {
		return fmt.Sprintf("[%s]:%d", address, port)
	}
	return fmt.Sprintf("%s:%d", address, port)
}

func positiveFloat(value float64) float64 {
	if value < 0 {
		return 0
	}
	return value
}

func rate(current, previous uint64, seconds float64) uint64 {
	if current < previous || seconds <= 0 {
		return 0
	}
	return uint64(float64(current-previous) / seconds)
}
