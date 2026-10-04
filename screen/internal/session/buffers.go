package session

import (
	"errors"
	"fmt"
	"io"
	"io/fs"
	"os"
	"path/filepath"
	"strconv"
	"strings"
	"unicode/utf8"

	"github.com/lyonbrown4d/terman/screen/internal/paths"
	"github.com/lyonbrown4d/terman/screen/internal/proto"
)

const maxBufferBytes = 16 << 20

func (o *Owner) bufferCommand(command string, args []string, target string) proto.Response {
	var message string
	var err error
	switch command {
	case "register":
		err = o.setRegister(args)
	case "readreg":
		message, err = o.readRegister(args)
	case "readbuf":
		message, err = o.readBuffer(args)
	case "writebuf":
		message, err = o.writeBuffer(args)
	case "removebuf":
		message, err = o.removeBuffer()
	case "paste":
		err = o.paste(args, target)
	case "bufferfile", "pastefile":
		if len(args) == 0 {
			path, pathErr := o.bufferPath("")
			if pathErr != nil {
				err = pathErr
			} else {
				message = path
			}
		} else if len(args) == 1 {
			o.bufferFile = o.resolvePath(args[0])
			message = o.bufferFile
		} else {
			err = fmt.Errorf("%s accepts one path", command)
		}
	}
	if err != nil {
		return rejected(err)
	}
	return accepted(message)
}

func (o *Owner) setRegister(args []string) error {
	encoding, values, err := encodingArgs(args)
	if err != nil {
		return err
	}
	if len(values) < 2 {
		return fmt.Errorf("register requires a register and text")
	}
	name, err := registerName(values[0])
	if err != nil {
		return err
	}
	data := decodeEscapes([]byte(strings.Join(values[1:], " ")))
	if err := validateEncoded(data, encoding); err != nil {
		return err
	}
	if len(data) > maxBufferBytes {
		return fmt.Errorf("register data exceeds %d bytes", maxBufferBytes)
	}
	o.storeRegister(name, data)
	return nil
}

func (o *Owner) readRegister(args []string) (string, error) {
	encoding, values, err := encodingArgs(args)
	if err != nil {
		return "", err
	}
	if len(values) != 2 {
		return "", fmt.Errorf("readreg requires a register and path")
	}
	name, err := registerName(values[0])
	if err != nil {
		return "", err
	}
	data, path, err := o.readBufferFile(values[1], encoding)
	if err != nil {
		return "", err
	}
	o.storeRegister(name, data)
	return fmt.Sprintf("read %d bytes into register %s from %s", len(data), name, path), nil
}

func (o *Owner) readBuffer(args []string) (string, error) {
	encoding, values, err := encodingArgs(args)
	if err != nil {
		return "", err
	}
	if len(values) > 1 {
		return "", fmt.Errorf("readbuf accepts at most one path")
	}
	data, resolved, err := o.readBufferFile(first(values), encoding)
	if err != nil {
		return "", err
	}
	o.storeRegister(".", data)
	return fmt.Sprintf("read %d bytes from %s", len(data), resolved), nil
}

func (o *Owner) writeBuffer(args []string) (string, error) {
	encoding, values, err := encodingArgs(args)
	if err != nil {
		return "", err
	}
	if len(values) > 1 {
		return "", fmt.Errorf("writebuf accepts at most one path")
	}
	data := o.registers["."]
	if err := validateEncoded(data, encoding); err != nil {
		return "", err
	}
	path, err := o.bufferPath(first(values))
	if err != nil {
		return "", err
	}
	if err := writePrivate(path, data, false); err != nil {
		return "", err
	}
	return fmt.Sprintf("wrote %d bytes to %s", len(data), path), nil
}

func (o *Owner) removeBuffer() (string, error) {
	path, err := o.bufferPath("")
	if err != nil {
		return "", err
	}
	if err := os.Remove(path); err != nil && !errors.Is(err, fs.ErrNotExist) {
		return "", fmt.Errorf("remove buffer file %q: %w", path, err)
	}
	o.storeRegister(".", nil)
	return "removed buffer file " + path, nil
}

func (o *Owner) paste(args []string, target string) error {
	if len(args) == 0 {
		args = []string{"."}
	}
	data := make([]byte, 0)
	for _, name := range args {
		value, ok := o.registers[name]
		if !ok {
			return fmt.Errorf("register %q is empty", name)
		}
		data = append(data, value...)
	}
	return o.writeWindow(target, data)
}

func (o *Owner) storeRegister(name string, data []byte) {
	value := append([]byte(nil), data...)
	o.registers[name] = value
	if name == "." {
		o.pasteBuffer = value
	}
}

func (o *Owner) readBufferFile(path, encoding string) ([]byte, string, error) {
	resolved, err := o.bufferPath(path)
	if err != nil {
		return nil, "", err
	}
	file, err := os.Open(resolved)
	if err != nil {
		return nil, "", fmt.Errorf("open buffer file %q: %w", resolved, err)
	}
	defer file.Close()
	data, err := io.ReadAll(io.LimitReader(file, maxBufferBytes+1))
	if err != nil {
		return nil, "", fmt.Errorf("read buffer file %q: %w", resolved, err)
	}
	if len(data) > maxBufferBytes {
		return nil, "", fmt.Errorf("buffer file exceeds %d bytes", maxBufferBytes)
	}
	if err := validateEncoded(data, encoding); err != nil {
		return nil, "", err
	}
	return data, resolved, nil
}

func (o *Owner) bufferPath(path string) (string, error) {
	if path == "" && o.bufferFile != "" {
		return o.bufferFile, nil
	}
	if path == "" {
		return paths.Buffer()
	}
	if filepath.IsAbs(path) {
		return filepath.Clean(path), nil
	}
	return filepath.Join(o.cwd, filepath.Clean(path)), nil
}

func encodingArgs(args []string) (string, []string, error) {
	encoding := "utf-8"
	if len(args) >= 2 && args[0] == "-e" {
		encoding = strings.ToLower(args[1])
		args = args[2:]
	}
	switch encoding {
	case "utf-8", "utf8", "ascii", "us-ascii", "latin1", "iso-8859-1", "binary", "raw":
		return encoding, args, nil
	default:
		return "", nil, fmt.Errorf("unsupported buffer encoding %q", encoding)
	}
}

func validateEncoded(data []byte, encoding string) error {
	switch encoding {
	case "utf-8", "utf8":
		if !utf8.Valid(data) {
			return fmt.Errorf("buffer is not valid UTF-8")
		}
	case "ascii", "us-ascii":
		for _, value := range data {
			if value > 0x7f {
				return fmt.Errorf("buffer contains non-ASCII data")
			}
		}
	}
	return nil
}

func registerName(value string) (string, error) {
	if utf8.RuneCountInString(value) != 1 {
		return "", fmt.Errorf("register name must be one character")
	}
	return value, nil
}

func decodeEscapes(data []byte) []byte {
	result := make([]byte, 0, len(data))
	for index := 0; index < len(data); index++ {
		if data[index] != '\\' || index+1 >= len(data) {
			result = append(result, data[index])
			continue
		}
		end := index + 1
		for end < len(data) && end <= index+3 && data[end] >= '0' && data[end] <= '7' {
			end++
		}
		if end == index+1 {
			result = append(result, data[index])
			continue
		}
		value, _ := strconv.ParseUint(string(data[index+1:end]), 8, 8)
		result = append(result, byte(value))
		index = end - 1
	}
	return result
}

func writePrivate(path string, data []byte, appendMode bool) error {
	flags := os.O_CREATE | os.O_WRONLY
	if appendMode {
		flags |= os.O_APPEND
	} else {
		flags |= os.O_TRUNC
	}
	file, err := os.OpenFile(path, flags, 0o600)
	if err != nil {
		return fmt.Errorf("open output file %q: %w", path, err)
	}
	if err := file.Chmod(0o600); err != nil {
		_ = file.Close()
		return err
	}
	if _, err := file.Write(data); err != nil {
		_ = file.Close()
		return fmt.Errorf("write output file %q: %w", path, err)
	}
	if err := file.Close(); err != nil {
		return fmt.Errorf("close output file %q: %w", path, err)
	}
	return nil
}
