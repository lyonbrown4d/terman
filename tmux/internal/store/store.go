package store

import (
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"errors"
	"fmt"
	"os"
	"path/filepath"
	"strings"
	"time"
)

const SchemaVersion = 1

type Record struct {
	SchemaVersion int       `json:"schema_version"`
	Name          string    `json:"name"`
	Endpoint      string    `json:"endpoint"`
	PID           int       `json:"pid"`
	CreatedAt     time.Time `json:"created_at"`
	UpdatedAt     time.Time `json:"updated_at"`
}

func ValidateName(name string) error {
	name = strings.TrimSpace(name)
	if name == "" {
		return errors.New("session name is required")
	}
	if len(name) > 128 {
		return errors.New("session name exceeds 128 bytes")
	}
	for _, r := range name {
		if r < 0x20 || r == 0x7f {
			return errors.New("session name contains control characters")
		}
	}
	return nil
}

func Dir() (string, error) {
	base, err := os.UserCacheDir()
	if err != nil {
		return "", fmt.Errorf("locate user cache: %w", err)
	}
	dir := filepath.Join(base, "terman", "tmux", "sessions")
	if err := os.MkdirAll(dir, 0o700); err != nil {
		return "", fmt.Errorf("create session store: %w", err)
	}
	return dir, nil
}

func Reserve(name, endpoint string) (Record, error) {
	if err := ValidateName(name); err != nil {
		return Record{}, err
	}
	now := time.Now()
	rec := Record{SchemaVersion: SchemaVersion, Name: name, Endpoint: endpoint, CreatedAt: now, UpdatedAt: now}
	data, err := json.Marshal(rec)
	if err != nil {
		return Record{}, fmt.Errorf("encode session record: %w", err)
	}
	path, err := pathFor(name)
	if err != nil {
		return Record{}, err
	}
	f, err := os.OpenFile(path, os.O_WRONLY|os.O_CREATE|os.O_EXCL, 0o600)
	if err != nil {
		return Record{}, fmt.Errorf("reserve session %q: %w", name, err)
	}
	if _, err := f.Write(data); err != nil {
		_ = f.Close()
		_ = os.Remove(path)
		return Record{}, fmt.Errorf("write session reservation: %w", err)
	}
	if err := f.Close(); err != nil {
		return Record{}, fmt.Errorf("close session reservation: %w", err)
	}
	return rec, nil
}

func Save(rec Record) error {
	if err := ValidateName(rec.Name); err != nil {
		return err
	}
	rec.SchemaVersion = SchemaVersion
	rec.UpdatedAt = time.Now()
	data, err := json.MarshalIndent(rec, "", "  ")
	if err != nil {
		return fmt.Errorf("encode session record: %w", err)
	}
	path, err := pathFor(rec.Name)
	if err != nil {
		return err
	}
	tmp := path + ".tmp"
	if err := os.WriteFile(tmp, data, 0o600); err != nil {
		return fmt.Errorf("write session record: %w", err)
	}
	if err := os.Rename(tmp, path); err != nil {
		_ = os.Remove(tmp)
		return fmt.Errorf("replace session record: %w", err)
	}
	return nil
}

func Load() ([]Record, error) {
	dir, err := Dir()
	if err != nil {
		return nil, err
	}
	entries, err := os.ReadDir(dir)
	if err != nil {
		return nil, fmt.Errorf("read session store: %w", err)
	}
	records := make([]Record, 0, len(entries))
	for _, entry := range entries {
		if entry.IsDir() || filepath.Ext(entry.Name()) != ".json" {
			continue
		}
		data, readErr := os.ReadFile(filepath.Join(dir, entry.Name()))
		if readErr != nil {
			continue
		}
		var rec Record
		if json.Unmarshal(data, &rec) == nil && rec.SchemaVersion == SchemaVersion && ValidateName(rec.Name) == nil {
			records = append(records, rec)
		}
	}
	return records, nil
}

func Remove(name string) error {
	path, err := pathFor(name)
	if err != nil {
		return err
	}
	err = os.Remove(path)
	if errors.Is(err, os.ErrNotExist) {
		return nil
	}
	return err
}

func Rename(oldName, newName string) error {
	if err := ValidateName(newName); err != nil {
		return err
	}
	records, err := Load()
	if err != nil {
		return err
	}
	for _, rec := range records {
		if rec.Name != oldName {
			continue
		}
		rec.Name = newName
		if err := Save(rec); err != nil {
			return err
		}
		return Remove(oldName)
	}
	return os.ErrNotExist
}

func pathFor(name string) (string, error) {
	dir, err := Dir()
	if err != nil {
		return "", err
	}
	sum := sha256.Sum256([]byte(name))
	return filepath.Join(dir, hex.EncodeToString(sum[:16])+".json"), nil
}
