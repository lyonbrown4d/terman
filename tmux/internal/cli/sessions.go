package cli

import (
	"context"
	"fmt"
	"sort"
	"strings"
	"time"

	"github.com/lyonbrown4d/terman/tmux/internal/client"
	"github.com/lyonbrown4d/terman/tmux/internal/protocol"
	"github.com/lyonbrown4d/terman/tmux/internal/store"
)

type sessionList struct {
	SchemaVersion int                    `json:"schema_version"`
	Sessions      []protocol.SessionInfo `json:"sessions"`
}

func listSessions(args []string) error {
	records, err := client.LiveRecords(true)
	if err != nil {
		return err
	}
	sessions := make([]protocol.SessionInfo, 0, len(records))
	for _, record := range records {
		resp, callErr := client.Call(context.Background(), record, protocol.Request{Op: "info"})
		if callErr != nil || resp.Session == nil {
			continue
		}
		info := *resp.Session
		info.Endpoint, info.ServerPID = record.Endpoint, record.PID
		sessions = append(sessions, info)
	}
	sort.Slice(sessions, func(i, j int) bool { return sessions[i].Name < sessions[j].Name })
	if has(args, "--json") {
		return printJSON(sessionList{SchemaVersion: protocol.SchemaVersion, Sessions: sessions})
	}
	for _, info := range sessions {
		fmt.Printf("%s: %d windows (created %s) [%d attached]\n", info.Name, info.Windows, info.CreatedAt.Format(time.RFC3339), info.Attached)
	}
	return nil
}

func killServer() error {
	records, err := client.LiveRecords(true)
	if err != nil {
		return err
	}
	var failures []string
	for _, record := range records {
		if _, callErr := client.Call(context.Background(), record, protocol.Request{Op: "shutdown"}); callErr != nil {
			failures = append(failures, callErr.Error())
		}
	}
	if len(failures) > 0 {
		return fmt.Errorf("kill server: %s", strings.Join(failures, "; "))
	}
	return nil
}

func renameSession(args []string) error {
	record, err := targetRecord(args)
	if err != nil {
		return err
	}
	name := firstPositional(args, firstCommand(args))
	if err := store.ValidateName(name); err != nil {
		return err
	}
	if _, err := client.Call(context.Background(), record, protocol.Request{Op: "rename-session", Name: name}); err != nil {
		return err
	}
	return store.Rename(record.Name, name)
}

func displayMessage(args []string) error {
	message := firstPositional(args, firstCommand(args))
	return printData(args, protocol.Request{Op: "display-message", Data: message})
}

func listClients(args []string) error {
	record, err := targetRecord(args)
	if err != nil {
		return err
	}
	resp, err := client.Call(context.Background(), record, protocol.Request{Op: "list-clients"})
	if err != nil {
		return err
	}
	if has(args, "--json") {
		return printJSON(struct {
			SchemaVersion int                   `json:"schema_version"`
			Session       string                `json:"session"`
			Clients       []protocol.ClientInfo `json:"clients"`
		}{protocol.SchemaVersion, record.Name, resp.Clients})
	}
	for _, value := range resp.Clients {
		fmt.Printf("%s: %dx%d attached %s\n", value.ID, value.Width, value.Height, value.Attached.Format(time.RFC3339))
	}
	return nil
}
