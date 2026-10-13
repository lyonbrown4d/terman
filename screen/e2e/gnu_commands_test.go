//go:build integration

package e2e

import (
	"os"
	"path/filepath"
	"strings"
	"testing"
)

const upstreamGNUCommands = `
acladd aclchg acldel aclgrp aclumask activity addacl allpartial altscreen at
attrcolor autodetach autonuke backtick bce bell_msg bind bindkey blanker
blankerprg break breaktype bufferfile bumpleft bumpright c1 caption chacl
charset chdir cjkwidth clear collapse colon command compacthist console copy
copy_reg crlf defautonuke defbce defbreaktype defc1 defcharset defencoding
defescape defflow defgr defhstatus deflog deflogin defmode defmonitor
defmousetrack defnonblock defobuflimit defscrollback defshell defsilence
defslowpaste defutf8 defwrap defwritelock detach digraph dinfo displays
dumptermcap echo encoding escape eval exec fit flow focus focusminsize gr group
hardcopy hardcopy_append hardcopydir hardstatus height help history hstatus idle
ignorecase info ins_reg kill lastmsg layout license lockscreen log logfile
login logtstamp mapdefault mapnotnext maptimeout markkeys meta monitor
mousetrack msgminwait msgwait multiuser next nonblock number obuflimit only
other parent partial password paste pastefont pow_break pow_detach pow_detach_msg
prev printcmd process quit readbuf readreg redisplay register remove removebuf
rendition reset resize screen scrollback select sessionname setenv setsid shell
shelltitle silence silencewait sleep slowpaste sorendition source split
startup_message stuff su suspend term termcap termcapinfo terminfo title umask
unbindall unsetenv utf8 vbell vbell_msg vbellwait verbose version wall width
windowlist windows wrap writebuf writelock xoff xon zmodem zombie zombie_timeout
`

func TestOfficialGNUCommandInventoryIsClassified(t *testing.T) {
	data, err := os.ReadFile(filepath.Join(moduleRoot, "COMPATIBILITY.md"))
	if err != nil {
		t.Fatalf("read compatibility matrix: %v", err)
	}
	matrix := string(data)
	for _, command := range strings.Fields(upstreamGNUCommands) {
		if !strings.Contains(matrix, "`"+command+"`") {
			t.Errorf("GNU command %q has no compatibility classification", command)
		}
	}
}
