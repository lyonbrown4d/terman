package ptytest

import (
	"bytes"
	"context"
	"fmt"
	"io"
	"sync"
	"time"

	"github.com/aymanbagabas/go-pty"
)

type Client struct {
	command  *pty.Cmd
	terminal pty.Pty
	buffer   bytes.Buffer
	mu       sync.Mutex
	done     chan error
}

func Start(
	ctx context.Context,
	directory string,
	executable string,
	env []string,
	args ...string,
) (*Client, error) {
	terminal, err := pty.New()
	if err != nil {
		return nil, fmt.Errorf("create PTY: %w", err)
	}
	command := terminal.CommandContext(ctx, executable, args...)
	command.Dir = directory
	command.Env = env
	if err := command.Start(); err != nil {
		_ = terminal.Close()
		return nil, fmt.Errorf("start PTY command: %w", err)
	}
	client := &Client{
		command:  command,
		terminal: terminal,
		done:     make(chan error, 1),
	}
	go client.capture()
	go client.wait()
	return client, nil
}

func (client *Client) capture() {
	_, _ = io.Copy(writer{client: client}, client.terminal)
}

func (client *Client) wait() {
	client.done <- client.command.Wait()
	close(client.done)
}

func (client *Client) Write(data []byte) error {
	_, err := client.terminal.Write(data)
	return err
}

func (client *Client) Resize(columns, rows int) error {
	return client.terminal.Resize(columns, rows)
}

func (client *Client) Output() string {
	client.mu.Lock()
	defer client.mu.Unlock()
	return client.buffer.String()
}

func (client *Client) WaitFor(
	ctx context.Context,
	interval time.Duration,
	text string,
) bool {
	ticker := time.NewTicker(interval)
	defer ticker.Stop()
	for {
		if bytes.Contains([]byte(client.Output()), []byte(text)) {
			return true
		}
		select {
		case <-ctx.Done():
			return false
		case <-ticker.C:
		}
	}
}

func (client *Client) Wait(ctx context.Context) error {
	select {
	case <-ctx.Done():
		return ctx.Err()
	case err := <-client.done:
		_ = client.terminal.Close()
		return err
	}
}

func (client *Client) Kill() error {
	if client.command.Process == nil {
		_ = client.terminal.Close()
		return nil
	}
	err := client.command.Process.Kill()
	_ = client.terminal.Close()
	return err
}

type writer struct {
	client *Client
}

func (output writer) Write(data []byte) (int, error) {
	output.client.mu.Lock()
	defer output.client.mu.Unlock()
	return output.client.buffer.Write(data)
}

var _ io.Writer = writer{}
