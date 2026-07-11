use std::time::{Duration, Instant};

use crossterm::event::KeyEvent;

use crate::attach_keys::{TmuxPrefixCommand, tmux_prefix_command};

const REPEAT_TIME: Duration = Duration::from_millis(500);

#[derive(Default)]
pub(crate) struct PaneResizeRepeat {
    deadline: Option<Instant>,
}

impl PaneResizeRepeat {
    pub(crate) fn arm(&mut self) {
        self.deadline = Some(Instant::now() + REPEAT_TIME);
    }

    pub(crate) fn take_command(&mut self, key: &KeyEvent) -> Option<TmuxPrefixCommand> {
        let active = self.deadline
            .is_some_and(|deadline| Instant::now() <= deadline);
        self.deadline = None;
        if !active {
            return None;
        }
        match tmux_prefix_command(key) {
            Some(command @ TmuxPrefixCommand::ResizePane(_)) => {
                self.arm();
                Some(command)
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    use super::PaneResizeRepeat;
    use crate::{attach_keys::TmuxPrefixCommand, pane_layout::PaneDirection};

    #[test]
    fn accepts_control_arrow_only_while_armed() {
        let key = KeyEvent::new(KeyCode::Left, KeyModifiers::CONTROL);
        let mut repeat = PaneResizeRepeat::default();
        assert!(repeat.take_command(&key).is_none());
        repeat.arm();
        assert_eq!(
            repeat.take_command(&key),
            Some(TmuxPrefixCommand::ResizePane(PaneDirection::Left))
        );
    }
}