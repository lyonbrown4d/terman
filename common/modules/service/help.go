package service

import (
	"fmt"
	"os"
)

func printHelp(cfg Config, action string) error {
	if _, err := fmt.Fprintln(os.Stdout, localized("service-about", nil)); err != nil {
		return err
	}
	if _, err := fmt.Fprintln(os.Stdout, localized("service-usage", map[string]string{"tool": cfg.Tool})); err != nil {
		return err
	}
	if _, err := fmt.Fprintf(os.Stdout, "  --name NAME  %s\n", localized("service-flag-name", nil)); err != nil {
		return err
	}
	if action == "" || action == "status" {
		if _, err := fmt.Fprintf(os.Stdout, "  --json  %s\n", localized("service-flag-json", nil)); err != nil {
			return err
		}
	}
	if action == "" || action == "install" {
		_, err := fmt.Fprintf(os.Stdout, "  --username USER  %s\n  --password-env ENV  %s\n",
			localized("service-flag-username", nil), localized("service-flag-password-env", nil))
		return err
	}
	return nil
}
