package client

import (
	"fmt"
	"strings"

	"github.com/gdamore/tcell/v3"
	"github.com/lyonbrown4d/terman/screen/internal/proto"
)

type promptMode struct {
	kind  string
	label string
	text  []rune
}

func (s *attachState) handlePromptKey(key *tcell.EventKey) error {
	switch key.Key() {
	case tcell.KeyEsc:
		s.prompt = nil
	case tcell.KeyBackspace, tcell.KeyBackspace2:
		if len(s.prompt.text) > 0 {
			s.prompt.text = s.prompt.text[:len(s.prompt.text)-1]
		}
	case tcell.KeyEnter:
		prompt := s.prompt
		s.prompt = nil
		switch prompt.kind {
		case "title":
			if len(prompt.text) == 0 {
				return nil
			}
			return s.send(proto.Request{
				Type: "command", Command: "title",
				Args: []string{string(prompt.text)},
			})
		case "command":
			words, err := parseCommand(string(prompt.text))
			if err != nil {
				s.notice = err.Error()
				s.draw()
				return nil
			}
			if len(words) > 0 {
				return s.send(proto.Request{
					Type: "command", Command: words[0], Args: words[1:],
				})
			}
		}
	case tcell.KeyRune:
		s.prompt.text = append(s.prompt.text, eventRune(key))
	}
	s.draw()
	return nil
}

func (s *attachState) drawPrompt() {
	cols, rows := s.screen.Size()
	text := s.prompt.label + string(s.prompt.text)
	drawText(s.screen, 0, rows-1, cols, text, tcell.StyleDefault.Reverse(true))
	column := len([]rune(text))
	s.screen.ShowCursor(min(column, cols-1), rows-1)
}

func parseCommand(value string) ([]string, error) {
	words := []string{}
	var current strings.Builder
	var quote rune
	escaped := false
	flush := func() {
		if current.Len() > 0 {
			words = append(words, current.String())
			current.Reset()
		}
	}
	for _, char := range value {
		switch {
		case escaped:
			current.WriteRune(char)
			escaped = false
		case char == '\\':
			escaped = true
		case quote != 0 && char == quote:
			quote = 0
		case quote != 0:
			current.WriteRune(char)
		case char == '\'' || char == '"':
			quote = char
		case char == ' ' || char == '\t':
			flush()
		default:
			current.WriteRune(char)
		}
	}
	if escaped || quote != 0 {
		return nil, fmt.Errorf("unfinished quote or escape")
	}
	flush()
	return words, nil
}
