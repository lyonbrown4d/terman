//go:build windows

package service

import (
	"fmt"
	"runtime"

	"golang.org/x/sys/windows"
)

// Grant only lifecycle/query rights to the account that owns this service.
// Installation still requires the native manager's administrative privileges.
func grantServiceControl(cfg Config) error {
	name, err := effectiveName(cfg)
	if err != nil {
		return err
	}
	identity, err := UserIdentity()
	if err != nil {
		return err
	}
	sid, err := windows.StringToSid(identity)
	if err != nil {
		return err
	}
	descriptor, err := windows.GetNamedSecurityInfo(name, windows.SE_SERVICE, windows.DACL_SECURITY_INFORMATION)
	if err != nil {
		return fmt.Errorf("read service control permissions: %w", err)
	}
	existing, _, err := descriptor.DACL()
	if err != nil {
		return err
	}
	var pinned runtime.Pinner
	pinned.Pin(sid)
	defer pinned.Unpin()
	entries := []windows.EXPLICIT_ACCESS{{
		AccessPermissions: windows.ACCESS_MASK(windows.SERVICE_QUERY_STATUS | windows.SERVICE_QUERY_CONFIG | windows.SERVICE_START | windows.SERVICE_STOP | windows.READ_CONTROL),
		AccessMode:        windows.GRANT_ACCESS,
		Trustee:           windows.TRUSTEE{TrusteeForm: windows.TRUSTEE_IS_SID, TrusteeType: windows.TRUSTEE_IS_USER, TrusteeValue: windows.TrusteeValueFromSID(sid)},
	}}
	acl, err := windows.ACLFromEntries(entries, existing)
	if err != nil {
		return err
	}
	if err = windows.SetNamedSecurityInfo(name, windows.SE_SERVICE, windows.DACL_SECURITY_INFORMATION, nil, nil, acl, nil); err != nil {
		return fmt.Errorf("grant owner service control permissions: %w", err)
	}
	return nil
}
