//go:build windows

package service

import (
	"context"
	"errors"
	"fmt"
	"net"
	"os"
	"syscall"

	"github.com/Microsoft/go-winio"
	"golang.org/x/sys/windows"
)

// UserIdentity returns the current process token's SID.
func UserIdentity() (string, error) {
	account, err := windows.GetCurrentProcessToken().GetTokenUser()
	if err != nil {
		return "", fmt.Errorf("resolve service user SID: %w", err)
	}
	return account.User.Sid.String(), nil
}

func privilegedIdentity(sid string) bool {
	return sid == "S-1-5-18" || sid == "S-1-5-19" || sid == "S-1-5-20" ||
		len(sid) >= 9 && (sid[:9] == "S-1-5-80-" || sid[:9] == "S-1-5-82-")
}

// PipeSecurityDescriptor grants access only to the current user's SID.
// go-winio rejects remote named-pipe clients independently of this ACL.
func PipeSecurityDescriptor() (string, error) {
	sid, err := UserIdentity()
	if err != nil {
		return "", err
	}
	return "O:" + sid + "D:P(A;;GA;;;" + sid + ")", nil
}

func endpoint(tool, identity string) (string, error) {
	return `\\.\pipe\terman-service-` + tool + "-" + identityHash(identity), nil
}

func listenEndpoint(address string) (net.Listener, error) {
	descriptor, err := PipeSecurityDescriptor()
	if err != nil {
		return nil, err
	}
	return winio.ListenPipe(address, &winio.PipeConfig{SecurityDescriptor: descriptor,
		InputBufferSize: maxRequestBytes + 1, OutputBufferSize: maxRequestBytes})
}
func dialEndpoint(ctx context.Context, address string) (net.Conn, error) {
	conn, err := winio.DialPipeContext(ctx, address)
	if err != nil {
		return nil, err
	}
	if err = validatePipePeer(conn); err != nil {
		conn.Close()
		return nil, err
	}
	return conn, nil
}
func endpointUnavailable(err error) bool {
	return errors.Is(err, os.ErrNotExist) || errors.Is(err, syscall.ERROR_FILE_NOT_FOUND) || errors.Is(err, windows.ERROR_PIPE_NOT_CONNECTED)
}

func restoreUserProfile() error {
	if _, err := UserIdentity(); err != nil {
		return err
	}
	folders := []struct {
		name string
		id   *windows.KNOWNFOLDERID
	}{
		{"USERPROFILE", windows.FOLDERID_Profile}, {"HOME", windows.FOLDERID_Profile},
		{"LOCALAPPDATA", windows.FOLDERID_LocalAppData}, {"APPDATA", windows.FOLDERID_RoamingAppData},
	}
	for _, folder := range folders {
		if os.Getenv(folder.name) != "" {
			continue
		}
		value, err := windows.KnownFolderPath(folder.id, windows.KF_FLAG_DEFAULT)
		if err != nil {
			return fmt.Errorf("resolve service profile %s: %w", folder.name, err)
		}
		if err = os.Setenv(folder.name, value); err != nil {
			return err
		}
	}
	return nil
}

func configurePlatform(cfg *nativeConfig, username, passwordEnv string) error {
	if err := requireRegularUser(); err != nil {
		return err
	}
	if username == "" {
		return serviceError("service-user-required", nil)
	}
	if passwordEnv == "" {
		return serviceError("service-windows-account-required", nil)
	}
	sid, _, _, err := windows.LookupSID("", username)
	if err != nil {
		return fmt.Errorf("resolve service account: %w", err)
	}
	current, err := UserIdentity()
	if err != nil {
		return err
	}
	if privilegedIdentity(sid.String()) || sid.String() != current {
		return serviceError("service-user-mismatch", nil)
	}
	password, ok := os.LookupEnv(passwordEnv)
	if !ok || password == "" {
		return serviceError("service-password-required", map[string]string{"environment": passwordEnv})
	}
	cfg.UserName = username
	cfg.Option["Password"] = password
	return nil
}

func validatePlatform() error { return nil }

func requireRegularUser() error {
	sid, err := UserIdentity()
	if err != nil {
		return err
	}
	if privilegedIdentity(sid) {
		return serviceError("service-system-account-rejected", nil)
	}
	return nil
}
