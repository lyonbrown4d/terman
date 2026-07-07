use std::io::{self, Write};

use crossterm::{
    cursor::MoveTo,
    queue,
    style::{Attribute, Print, SetAttribute},
    terminal::{self, Clear, ClearType},
};

use crate::metrics::Snapshot;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Tab {
    Processes,
    Io,
    Network,
}

impl Tab {
    pub(crate) fn next(self) -> Self {
        match self {
            Self::Processes => Self::Io,
            Self::Io => Self::Network,
            Self::Network => Self::Processes,
        }
    }

    pub(crate) fn previous(self) -> Self {
        match self {
            Self::Processes => Self::Network,
            Self::Io => Self::Processes,
            Self::Network => Self::Io,
        }
    }
}

pub(crate) fn draw(stdout: &mut impl Write, snapshot: &Snapshot, tab: Tab) -> io::Result<()> {
    let (cols, rows) = terminal::size()?;
    queue!(stdout, Clear(ClearType::All), MoveTo(0, 0))?;
    draw_header(stdout, cols, snapshot, tab)?;
    match tab {
        Tab::Processes => draw_processes(stdout, rows, snapshot)?,
        Tab::Io => draw_io(stdout, rows, snapshot)?,
        Tab::Network => draw_network(stdout, rows, snapshot)?,
    }
    stdout.flush()
}

fn draw_header(stdout: &mut impl Write, cols: u16, snapshot: &Snapshot, tab: Tab) -> io::Result<()> {
    let memory = format_bytes(snapshot.used_memory);
    let total = format_bytes(snapshot.total_memory);
    line(stdout, 0, cols, &format!("terman-htop  mem {memory}/{total}"))?;
    queue!(stdout, MoveTo(0, 1))?;
    draw_tab(stdout, tab, Tab::Processes, &terman_common::builtin_htop_tab_processes_hint())?;
    draw_tab(stdout, tab, Tab::Io, &terman_common::builtin_htop_tab_io_hint())?;
    draw_tab(stdout, tab, Tab::Network, &terman_common::builtin_htop_tab_network_hint())?;
    line(stdout, 2, cols, &terman_common::builtin_htop_help_hint())
}

fn draw_tab(stdout: &mut impl Write, active: Tab, tab: Tab, label: &str) -> io::Result<()> {
    if active == tab {
        queue!(stdout, SetAttribute(Attribute::Reverse))?;
    }
    queue!(stdout, Print(format!(" {label} ")))?;
    if active == tab {
        queue!(stdout, SetAttribute(Attribute::Reset))?;
    }
    queue!(stdout, Print(" "))
}

fn draw_processes(stdout: &mut impl Write, rows: u16, snapshot: &Snapshot) -> io::Result<()> {
    line(stdout, 4, u16::MAX, "PID        CPU%    MEM        NAME")?;
    for (offset, row) in snapshot.processes.iter().take(body_rows(rows)).enumerate() {
        line(stdout, 5 + offset as u16, u16::MAX, &format!(
            "{:<10} {:>5.1}   {:>8}   {}",
            row.pid,
            row.cpu,
            format_bytes(row.memory),
            row.name
        ))?;
    }
    Ok(())
}

fn draw_io(stdout: &mut impl Write, rows: u16, snapshot: &Snapshot) -> io::Result<()> {
    line(stdout, 4, u16::MAX, "PID        READ       WRITE      NAME")?;
    for (offset, row) in snapshot.io.iter().take(body_rows(rows)).enumerate() {
        line(stdout, 5 + offset as u16, u16::MAX, &format!(
            "{:<10} {:>9}  {:>9}  {}",
            row.pid,
            format_bytes(row.read),
            format_bytes(row.written),
            row.name
        ))?;
    }
    Ok(())
}

fn draw_network(stdout: &mut impl Write, rows: u16, snapshot: &Snapshot) -> io::Result<()> {
    line(stdout, 4, u16::MAX, "IFACE                 RX        TX        TOTAL RX   TOTAL TX")?;
    for (offset, row) in snapshot.networks.iter().take(body_rows(rows)).enumerate() {
        line(stdout, 5 + offset as u16, u16::MAX, &format!(
            "{:<20} {:>8} {:>8} {:>10} {:>10}",
            row.name,
            format_bytes(row.received),
            format_bytes(row.transmitted),
            format_bytes(row.total_received),
            format_bytes(row.total_transmitted)
        ))?;
    }
    Ok(())
}

fn line(stdout: &mut impl Write, y: u16, cols: u16, text: &str) -> io::Result<()> {
    let width = cols as usize;
    let mut text = text.to_string();
    if width != usize::from(u16::MAX) && text.len() > width {
        text.truncate(width);
    }
    queue!(stdout, MoveTo(0, y), Print(text))
}

fn body_rows(rows: u16) -> usize {
    rows.saturating_sub(5) as usize
}

fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit + 1 < UNITS.len() {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{}{}", bytes, UNITS[unit])
    } else {
        format!("{value:.1}{}", UNITS[unit])
    }
}