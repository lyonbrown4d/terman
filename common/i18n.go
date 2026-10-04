package common

import (
	"bufio"
	"embed"
	"fmt"
	"os"
	"regexp"
	"strings"
	"sync"

	systemlocale "github.com/Xuanwo/go-locale"
)

// Language is a supported message language.
type Language string

const (
	LanguageZhCN Language = "zh-CN"
	LanguageEnUS Language = "en-US"
)

var (
	//go:embed i18n/*.ftl
	messageFiles embed.FS

	messageOnce sync.Once
	messages    map[Language]map[string]string
	variableRE  = regexp.MustCompile(`\{\s*\$([A-Za-z0-9_-]+)\s*\}`)
	selectRE    = regexp.MustCompile(`^\{\s*\$([A-Za-z0-9_-]+)\s*->\s*$`)
	variantRE   = regexp.MustCompile(`^\s*(\*)?\[([^]]+)\]\s*(.*)$`)
)

// LanguageFromTag maps every Chinese locale to zh-CN and all others to en-US.
func LanguageFromTag(tag string) Language {
	normalized := strings.ToLower(strings.ReplaceAll(tag, "_", "-"))
	if strings.HasPrefix(normalized, "zh") {
		return LanguageZhCN
	}
	return LanguageEnUS
}

// CurrentLanguage selects TERMAN_LANG first, then the operating-system locale.
func CurrentLanguage() Language {
	if configured := os.Getenv("TERMAN_LANG"); configured != "" {
		return LanguageFromTag(configured)
	}
	tag, err := systemlocale.Detect()
	if err != nil {
		return LanguageEnUS
	}
	return LanguageFromTag(tag.String())
}

// LocalizedMessage renders key in the currently selected language.
func LocalizedMessage(key string, variables map[string]string) string {
	return LocalizedMessageForLanguage(CurrentLanguage(), key, variables)
}

// LocalizedMessageForLanguage renders key using an explicit supported language.
func LocalizedMessageForLanguage(language Language, key string, variables map[string]string) string {
	messageOnce.Do(loadMessages)
	if language != LanguageZhCN {
		language = LanguageEnUS
	}
	message, ok := messages[language][key]
	if !ok {
		return fallbackMessage(key, variables)
	}
	message = resolveSelect(message, variables)
	return variableRE.ReplaceAllStringFunc(message, func(match string) string {
		parts := variableRE.FindStringSubmatch(match)
		if value, ok := variables[parts[1]]; ok {
			return value
		}
		return match
	})
}

// NativeToolNotFoundHint renders the shared missing-native-tool message.
func NativeToolNotFoundHint(tool string) string {
	return LocalizedMessage("native-tool-not-found", map[string]string{"tool": tool})
}

func loadMessages() {
	messages = map[Language]map[string]string{
		LanguageZhCN: {},
		LanguageEnUS: {},
	}
	for _, language := range []Language{LanguageZhCN, LanguageEnUS} {
		prefix := string(language)
		for _, suffix := range []string{"", ".htop", ".screen", ".tmux"} {
			data, err := messageFiles.ReadFile("i18n/" + prefix + suffix + ".ftl")
			if err != nil {
				continue
			}
			parseFTL(messages[language], string(data))
		}
	}
}

func parseFTL(destination map[string]string, source string) {
	scanner := bufio.NewScanner(strings.NewReader(source))
	key := ""
	var value strings.Builder
	flush := func() {
		if key != "" {
			destination[key] = strings.TrimSpace(value.String())
		}
		key = ""
		value.Reset()
	}

	for scanner.Scan() {
		line := strings.TrimSuffix(scanner.Text(), "\r")
		if line == "" || strings.HasPrefix(strings.TrimSpace(line), "#") {
			continue
		}
		if line[0] != ' ' && line[0] != '\t' {
			name, initial, ok := strings.Cut(line, "=")
			if !ok {
				continue
			}
			flush()
			key = strings.TrimSpace(name)
			value.WriteString(strings.TrimSpace(initial))
			continue
		}
		if key != "" {
			if value.Len() > 0 {
				value.WriteByte('\n')
			}
			value.WriteString(strings.TrimSpace(line))
		}
	}
	flush()
}

func resolveSelect(message string, variables map[string]string) string {
	lines := strings.Split(message, "\n")
	if len(lines) < 2 {
		return message
	}
	selectMatch := selectRE.FindStringSubmatch(strings.TrimSpace(lines[0]))
	if selectMatch == nil {
		return message
	}

	selected := ""
	fallback := ""
	for _, line := range lines[1:] {
		variant := variantRE.FindStringSubmatch(line)
		if variant == nil {
			continue
		}
		if variant[1] == "*" {
			fallback = variant[3]
		}
		if variant[2] == variables[selectMatch[1]] {
			selected = variant[3]
		}
	}
	if selected != "" {
		return selected
	}
	return fallback
}

func fallbackMessage(key string, variables map[string]string) string {
	message := key
	for name, value := range variables {
		message += fmt.Sprintf(" %s=%s", name, value)
	}
	return message
}
