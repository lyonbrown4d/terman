//go:build windows

package service

import (
	"os"
	"os/user"
	"strings"
	"testing"

	"golang.org/x/sys/windows"
)

func TestPipeACLUsesTokenIdentity(t *testing.T) {
	sid, err := UserIdentity()
	if err != nil {
		t.Fatal(err)
	}
	descriptor, err := PipeSecurityDescriptor()
	if err != nil {
		t.Fatal(err)
	}
	if !strings.Contains(descriptor, sid) || strings.Contains(descriptor, ";;;WD") || strings.Contains(descriptor, ";;;AU") {
		t.Fatalf("unsafe pipe descriptor: %s", descriptor)
	}
	if _, err = windows.SecurityDescriptorFromString(descriptor); err != nil {
		t.Fatal(err)
	}
	for _, sid := range []string{"S-1-5-18", "S-1-5-19", "S-1-5-20", "S-1-5-80-123", "S-1-5-82-123"} {
		if !privilegedIdentity(sid) {
			t.Fatalf("privileged identity accepted: %s", sid)
		}
	}
}

func TestWindowsInstallRequiresExplicitCurrentAccount(t *testing.T) {
	config := &nativeConfig{Option: map[string]interface{}{}}
	for _, args := range [][2]string{{"", ""}, {"user", ""}, {"", "PASSWORD"}} {
		if err := configurePlatform(config, args[0], args[1]); err == nil {
			t.Fatal("implicit service account accepted")
		}
	}
	account, err := user.Current()
	if err != nil {
		t.Fatal(err)
	}
	t.Setenv("TERMAN_TEST_PASSWORD", "")
	if err := configurePlatform(config, account.Username, "TERMAN_TEST_PASSWORD"); err == nil {
		t.Fatal("empty password accepted")
	}
	t.Setenv("TERMAN_TEST_PASSWORD", "test-only-password")
	if err := configurePlatform(config, account.Username, "TERMAN_TEST_PASSWORD"); err != nil {
		t.Fatal(err)
	}
	if config.UserName != account.Username || config.Option["Password"] != "test-only-password" {
		t.Fatal("account configuration lost")
	}
	if err := configurePlatform(config, "NT AUTHORITY\\SYSTEM", "TERMAN_TEST_PASSWORD"); err == nil {
		t.Fatal("system account accepted")
	}
}

func TestMissingWindowsProfileEnvironmentIsRestored(t *testing.T) {
	for _, key := range []string{"HOME", "USERPROFILE", "LOCALAPPDATA", "APPDATA"} {
		t.Setenv(key, "")
	}
	if err := PrepareUserProfile(); err != nil {
		t.Fatal(err)
	}
	for _, key := range []string{"HOME", "USERPROFILE", "LOCALAPPDATA", "APPDATA"} {
		if os.Getenv(key) == "" {
			t.Fatalf("profile variable %s was not restored", key)
		}
	}
}
