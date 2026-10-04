package common

import (
	"embed"
	"encoding/json"
	"fmt"
	"io/fs"
	"os"
	"strings"

	systemlocale "github.com/Xuanwo/go-locale"
	goi18n "github.com/nicksnyder/go-i18n/v2/i18n"
	"golang.org/x/text/language"
)

// Language is a supported message language.
type Language string

const (
	LanguageZhCN Language = "zh-CN"
	LanguageEnUS Language = "en-US"
)

var (
	//go:embed i18n/*.json
	messageFiles  embed.FS
	messageBundle = newMessageBundle()
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
	if language != LanguageZhCN {
		language = LanguageEnUS
	}

	localizer := goi18n.NewLocalizer(messageBundle, string(language), string(LanguageEnUS))
	message, err := localizer.Localize(&goi18n.LocalizeConfig{
		MessageID:    key,
		TemplateData: variables,
	})
	if err == nil || message != "" {
		return message
	}
	return fallbackMessage(key, variables)
}

// NativeToolNotFoundHint renders the shared missing-native-tool message.
func NativeToolNotFoundHint(tool string) string {
	return LocalizedMessage("native-tool-not-found", map[string]string{"tool": tool})
}

func newMessageBundle() *goi18n.Bundle {
	bundle := goi18n.NewBundle(language.AmericanEnglish)
	bundle.RegisterUnmarshalFunc("json", json.Unmarshal)

	paths, err := fs.Glob(messageFiles, "i18n/*.json")
	if err != nil {
		panic(fmt.Errorf("find embedded message files: %w", err))
	}
	for _, path := range paths {
		if _, err := bundle.LoadMessageFileFS(messageFiles, path); err != nil {
			panic(fmt.Errorf("load embedded message file %q: %w", path, err))
		}
	}
	return bundle
}

func fallbackMessage(key string, variables map[string]string) string {
	message := key
	for name, value := range variables {
		message += fmt.Sprintf(" %s=%s", name, value)
	}
	return message
}
