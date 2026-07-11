use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};
use sysinfo::{SUPPORTED_SIGNALS, Signal};

pub(crate) struct SignalMenuState {
    pids: Vec<String>,
    label: String,
    cursor: usize,
}

impl SignalMenuState {
    pub(crate) fn new(mut pids: Vec<String>) -> Self {
        pids.sort_unstable();
        pids.dedup();
        let label = match pids.as_slice() {
            [pid] => pid.clone(),
            _ => terman_common::builtin_htop_tagged_count_hint(pids.len()),
        };
        Self {
            pids,
            label,
            cursor: 0,
        }
    }

    pub(crate) fn pid(&self) -> &str {
        &self.label
    }

    pub(crate) fn pids(&self) -> &[String] {
        &self.pids
    }

    pub(crate) fn cursor(&self) -> usize {
        self.cursor
    }

    pub(crate) fn set_cursor(&mut self, cursor: usize) {
        if !SUPPORTED_SIGNALS.is_empty() {
            self.cursor = cursor.min(SUPPORTED_SIGNALS.len() - 1);
        }
    }

    pub(crate) fn move_cursor(&mut self, forward: bool) {
        let count = SUPPORTED_SIGNALS.len();
        if count == 0 {
            return;
        }
        self.cursor = if forward {
            (self.cursor + 1) % count
        } else {
            self.cursor.checked_sub(1).unwrap_or(count - 1)
        };
    }

    pub(crate) fn selected_signal(&self) -> Option<Signal> {
        SUPPORTED_SIGNALS.get(self.cursor).copied()
    }
}

pub(crate) fn draw(frame: &mut Frame<'_>, state: &SignalMenuState) {
    let area = popup_area(frame.area(), SUPPORTED_SIGNALS.len());
    frame.render_widget(Clear, area);
    let block = Block::default()
        .title(terman_common::builtin_htop_signal_menu_title_hint(state.pid()))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(inner);
    frame.render_widget(Paragraph::new(signal_lines(state, chunks[0].height)), chunks[0]);
    frame.render_widget(
        Paragraph::new(terman_common::builtin_htop_signal_menu_help_hint())
            .style(Style::default().fg(Color::DarkGray)),
        chunks[1],
    );
}

pub(crate) fn index_at(
    terminal: Rect,
    column: u16,
    row: u16,
    cursor: usize,
) -> Option<usize> {
    let popup = popup_area(terminal, SUPPORTED_SIGNALS.len());
    let list = Rect {
        x: popup.x.saturating_add(1),
        y: popup.y.saturating_add(1),
        width: popup.width.saturating_sub(2),
        height: popup.height.saturating_sub(3),
    };
    if column < list.x
        || column >= list.x.saturating_add(list.width)
        || row < list.y
        || row >= list.y.saturating_add(list.height)
    {
        return None;
    }
    let offset = visible_offset(cursor, SUPPORTED_SIGNALS.len(), list.height as usize);
    let index = offset + usize::from(row - list.y);
    (index < SUPPORTED_SIGNALS.len()).then_some(index)
}

fn signal_lines(state: &SignalMenuState, height: u16) -> Vec<Line<'static>> {
    if SUPPORTED_SIGNALS.is_empty() {
        return vec![Line::from(terman_common::builtin_htop_signal_unsupported_hint())];
    }
    let visible = usize::from(height);
    let offset = visible_offset(state.cursor, SUPPORTED_SIGNALS.len(), visible);
    SUPPORTED_SIGNALS
        .iter()
        .enumerate()
        .skip(offset)
        .take(visible)
        .map(|(index, signal)| {
            let selected = index == state.cursor;
            let prefix = if selected { "> " } else { "  " };
            let style = if selected {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Gray)
            };
            Line::from(Span::styled(
                format!("{prefix}{:<10}", signal_label(*signal)),
                style,
            ))
        })
        .collect()
}

fn popup_area(area: Rect, signal_count: usize) -> Rect {
    let width = area.width.min(44);
    let desired_height = u16::try_from(signal_count.saturating_add(4)).unwrap_or(u16::MAX);
    let height = desired_height.min(area.height);
    Rect {
        x: area.x + area.width.saturating_sub(width) / 2,
        y: area.y + area.height.saturating_sub(height) / 2,
        width,
        height,
    }
}

fn visible_offset(cursor: usize, count: usize, visible: usize) -> usize {
    if visible == 0 || count <= visible {
        return 0;
    }
    cursor
        .saturating_sub(visible / 2)
        .min(count.saturating_sub(visible))
}

fn default_cursor() -> usize {
    SUPPORTED_SIGNALS
        .iter()
        .position(|signal| *signal == Signal::Term)
        .or_else(|| SUPPORTED_SIGNALS.iter().position(|signal| *signal == Signal::Kill))
        .unwrap_or(0)
}

fn signal_label(signal: Signal) -> String {
    let debug = format!("{signal:?}");
    let name = match debug.as_str() {
        "Hangup" => "SIGHUP",
        "Interrupt" => "SIGINT",
        "Quit" => "SIGQUIT",
        "Illegal" => "SIGILL",
        "Trap" => "SIGTRAP",
        "Abort" => "SIGABRT",
        "IOT" => "SIGIOT",
        "Bus" => "SIGBUS",
        "FloatingPointException" => "SIGFPE",
        "Kill" => "SIGKILL",
        "User1" => "SIGUSR1",
        "Segv" => "SIGSEGV",
        "User2" => "SIGUSR2",
        "Pipe" => "SIGPIPE",
        "Alarm" => "SIGALRM",
        "Term" => "SIGTERM",
        "Child" => "SIGCHLD",
        "Continue" => "SIGCONT",
        "Stop" => "SIGSTOP",
        "TSTP" => "SIGTSTP",
        "TTIN" => "SIGTTIN",
        "TTOU" => "SIGTTOU",
        "Urgent" => "SIGURG",
        "XCPU" => "SIGXCPU",
        "XFSZ" => "SIGXFSZ",
        "VirtualAlarm" => "SIGVTALRM",
        "Profiling" => "SIGPROF",
        "Winch" => "SIGWINCH",
        "IO" => "SIGIO",
        "Poll" => "SIGPOLL",
        "Power" => "SIGPWR",
        "Sys" => "SIGSYS",
        _ => return format!("SIG{}", debug.to_ascii_uppercase()),
    };
    name.to_string()
}