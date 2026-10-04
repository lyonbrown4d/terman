package store

import (
	"encoding/json"
	"errors"
	"fmt"
	"io/fs"
	"os"
	"path/filepath"
	"sort"
	"time"

	"github.com/lyonbrown4d/terman/screen/internal/paths"
)

type Record struct {
	Name     string
	PID      int
	Endpoint string
	Cwd      string
	Command  string
	Started  time.Time
}

func Save(record Record) error {
	path, err := paths.Record(record.Name)
	if err != nil {
		return err
	}
	data, err := json.MarshalIndent(record, "", "  ")
	if err != nil {
		return fmt.Errorf("encode session record: %w", err)
	}
	return writeAtomic(path, data)
}

func Delete(name string) error {
	path, err := paths.Record(name)
	if err != nil {
		return err
	}
	err = os.Remove(path)
	if errors.Is(err, fs.ErrNotExist) {
		return nil
	}
	return err
}

func Rename(oldName string, record Record) error {
	oldPath, err := paths.Record(oldName)
	if err != nil {
		return err
	}
	newPath, err := paths.Record(record.Name)
	if err != nil {
		return err
	}
	if oldPath == newPath {
		return Save(record)
	}
	if _, err := os.Stat(newPath); err == nil {
		return fmt.Errorf("session %q already exists", record.Name)
	} else if !errors.Is(err, fs.ErrNotExist) {
		return err
	}
	original, err := os.ReadFile(oldPath)
	if err != nil {
		return fmt.Errorf("read old session record: %w", err)
	}
	updated, err := json.MarshalIndent(record, "", "  ")
	if err != nil {
		return fmt.Errorf("encode renamed session record: %w", err)
	}
	if err := writeAtomic(oldPath, updated); err != nil {
		return err
	}
	if err := os.Rename(oldPath, newPath); err != nil {
		_ = writeAtomic(oldPath, original)
		return fmt.Errorf("rename session record: %w", err)
	}
	return nil
}

func writeAtomic(path string, data []byte) error {
	file, err := os.CreateTemp(filepath.Dir(path), ".record-*.tmp")
	if err != nil {
		return fmt.Errorf("create temporary record: %w", err)
	}
	temp := file.Name()
	defer os.Remove(temp)
	if err := file.Chmod(0o600); err != nil {
		_ = file.Close()
		return err
	}
	if _, err := file.Write(data); err != nil {
		_ = file.Close()
		return fmt.Errorf("write temporary record: %w", err)
	}
	if err := file.Sync(); err != nil {
		_ = file.Close()
		return fmt.Errorf("sync temporary record: %w", err)
	}
	if err := file.Close(); err != nil {
		return err
	}
	if err := os.Rename(temp, path); err != nil {
		return fmt.Errorf("install session record: %w", err)
	}
	return nil
}

func List() ([]Record, error) {
	root, err := paths.Root()
	if err != nil {
		return nil, err
	}
	entries, err := filepath.Glob(filepath.Join(root, "*.json"))
	if err != nil {
		return nil, err
	}
	records := make([]Record, 0, len(entries))
	for _, path := range entries {
		data, readErr := os.ReadFile(path)
		if readErr != nil {
			continue
		}
		var record Record
		if json.Unmarshal(data, &record) == nil && record.Name != "" && record.Endpoint != "" {
			records = append(records, record)
		}
	}
	sort.Slice(records, func(i, j int) bool { return records[i].Name < records[j].Name })
	return records, nil
}

func Find(name string) (Record, error) {
	records, err := List()
	if err != nil {
		return Record{}, err
	}
	if name == "" {
		if len(records) == 1 {
			return records[0], nil
		}
		return Record{}, fs.ErrNotExist
	}
	for _, record := range records {
		if record.Name == name {
			return record, nil
		}
	}
	return Record{}, fs.ErrNotExist
}
