builtin-tmux-find-prompt = tmux find-window | {$query}_
builtin-tmux-find-no-match = tmux find-window | no window matches "{$query}"
builtin-tmux-status-line = tmux { $session } | { $windows } | Ctrl-B n/p switch  mouse click/wheel  right list  middle help
builtin-tmux-kill-pane-confirm = tmux confirm | kill current pane? y yes  n/Esc no
builtin-tmux-kill-window-confirm = tmux confirm | kill current window? y yes  n/Esc no
