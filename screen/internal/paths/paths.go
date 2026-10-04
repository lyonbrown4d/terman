package paths

import (
	"crypto/sha256"
	"fmt"
	"os"
	"path/filepath"
)

func Root() (string, error) {
	base, err := os.UserCacheDir()
	if err != nil {
		return "", err
	}
	root := filepath.Join(base, "terman", "screen")
	if err := os.MkdirAll(root, 0o700); err != nil {
		return "", err
	}
	return root, nil
}

func Key(name string) string {
	sum := sha256.Sum256([]byte(name))
	return fmt.Sprintf("%x", sum[:10])
}

func Record(name string) (string, error) {
	root, err := Root()
	if err != nil {
		return "", err
	}
	return filepath.Join(root, Key(name)+".json"), nil
}

func Buffer() (string, error) {
	root, err := Root()
	if err != nil {
		return "", err
	}
	return filepath.Join(root, "exchange"), nil
}
