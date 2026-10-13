//go:build !windows

package service

func grantServiceControl(Config) error { return nil }
