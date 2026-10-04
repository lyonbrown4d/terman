package store

import (
	"encoding/json"
	"errors"
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
		return err
	}
	tmp := path + ".tmp"
	if err := os.WriteFile(tmp, data, 0o600); err != nil {
		return err
	}
	return os.Rename(tmp, path)
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
	if err := Save(record); err != nil {
		return err
	}
	return Delete(oldName)
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
