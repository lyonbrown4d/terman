//go:build !windows && !linux

package service

import "net"

func validatePlatform() error         { return serviceError("service-unsupported-platform", nil) }
func validateUnixPeer(net.Conn) error { return nil }
