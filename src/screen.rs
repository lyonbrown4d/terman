use std::{
    env,
    error::Error,
    io::{self, Read, Write},
    process::{Command, ExitStatus, Stdio},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc,
        Arc,
    },
    thread,
    time::Duration,
};

use clap::Args;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    terminal::{self, size as terminal_size},
};
use portable_pty::{native_pty_system, CommandBuilder, PtySize};

#[derive(Args, Debug, Clone)]
pub struct ScreenArgs {
    /// If set, run this command string through the platform shell in built-in mode.
    #[arg(short, long, value_name = "CMD")]
    pub command: Option<String>,

    /// Initial terminal columns.
    #[arg(long)]
    pub cols: Option<u16>,

    /// Initial terminal rows.
    #[arg(long)]
    pub rows: Option<u16>,

    /// Prefer using system `screen` if available.
    #[arg(long)]
    pub system: bool,

    /// Extra args passed to system screen when `--system` is enabled.
    #[arg(trailing_var_arg = true)]
    pub args: Vec<String>,
}

impl Default for ScreenArgs {
    fn default() -> Self {
        Self {
            command: None,
            cols: None,
            rows: None,
            system: false,
            args: Vec::new(),
        }
    }
}

struct RawMode;

impl RawMode {
    fn enter() -> io::Result<Self> {
        terminal::enable_raw_mode()?;
        Ok(Self)
    }
}

impl Drop for RawMode {
    fn drop(&mut self) {
        let _ = terminal::disable_raw_mode();
    }
}

enum ScreenKind {
    Native,
    Wsl,
}

struct ScreenLaunch {
    cmd: String,
    kind: ScreenKind,
    extra_args: Vec<String>,
}

pub fn run(args: ScreenArgs) -> Result<(), Box<dyn Error>> {
    if args.system {
        run_system_screen(args)?;
        return Ok(());
    }

    run_builtin_screen(args)
}

fn run_system_screen(args: ScreenArgs) -> Result<(), Box<dyn Error>> {
    let launch = resolve_screen_launch()?;

    if args.command.is_some() {
        eprintln!("提示：system screen 模式下，请使用 `--system -- <screen_args>`，如 `-S dev`；`--command` 会被忽略。\n");
    }

    let mut cmd = Command::new(&launch.cmd);
    if let ScreenKind::Wsl = launch.kind {
        cmd.args(&launch.extra_args);
        eprintln!("当前使用 WSL screen 回退路径；建议在 WSL 发行版中常驻使用 screen。")
    }

    let status: ExitStatus = cmd
        .args(&args.args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .envs(passthrough_env())
        .env("TERM", env::var("TERM").unwrap_or_else(|_| String::from("xterm-256color")))
        .status()?;

    if status.success() {
        Ok(())
    } else {
        Err(Box::new(io::Error::new(
            io::ErrorKind::Other,
            format!("system screen 退出码: {status}"),
        )))
    }
}

fn run_builtin_screen(args: ScreenArgs) -> Result<(), Box<dyn Error>> {
    let _raw = RawMode::enter()?;
    let (cols, rows) = resolve_size(args.cols, args.rows);

    let pty_system = native_pty_system();
    let pty_size = PtySize {
        cols,
        rows,
        pixel_width: 0,
        pixel_height: 0,
    };

    let pair = pty_system.openpty(pty_size)?;
    let command = build_command(args.command)?;
    let mut child = pair.slave.spawn_command(command)?;

    let mut master = pair.master;
    let mut reader = master.try_clone_reader()?;
    let mut writer = master.take_writer()?;

    let should_run = Arc::new(AtomicBool::new(true));
    let mut stdout = io::stdout();

    let output_running = Arc::clone(&should_run);
    let output_thread = thread::spawn(move || {
        let mut buf = [0u8; 8192];
        while output_running.load(Ordering::Acquire) {
            match reader.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    if stdout.write_all(&buf[..n]).is_err() {
                        break;
                    }
                    if stdout.flush().is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    });

    let (exit_tx, exit_rx) = mpsc::channel::<i32>();
    let child_wait_handle = thread::spawn(move || {
        let status = child.wait().map(|status| status.code().unwrap_or(-1)).unwrap_or(-1);
        let _ = exit_tx.send(status);
    });

    loop {
        match exit_rx.try_recv() {
            Ok(_) => break,
            Err(mpsc::TryRecvError::Empty) => {}
            Err(mpsc::TryRecvError::Disconnected) => break,
        }

        match event::poll(Duration::from_millis(16)) {
            Ok(true) => match event::read() {
                Ok(Event::Key(key)) => {
                    if let Some(bytes) = key_to_bytes(key) {
                        if writer.write_all(&bytes).is_err() {
                            break;
                        }
                        if writer.flush().is_err() {
                            break;
                        }
                    }
                }
                Ok(Event::Resize(cols, rows)) => {
                    let size = PtySize {
                        cols,
                        rows,
                        pixel_width: 0,
                        pixel_height: 0,
                    };
                    let _ = master.resize(size);
                }
                Ok(_) => {}
                Err(_) => break,
            },
            Ok(false) => {}
            Err(_) => break,
        }
    }

    should_run.store(false, Ordering::Release);
    let _ = output_thread.join();
    should_run.store(false, Ordering::Release);
    let _ = child_wait_handle.join();

    Ok(())
}

fn resolve_screen_launch() -> Result<ScreenLaunch, Box<dyn Error>> {
    if let Some(path) = which_binary("screen") {
        return Ok(ScreenLaunch {
            cmd: path,
            kind: ScreenKind::Native,
            extra_args: Vec::new(),
        });
    }

    if cfg!(windows) {
        if which_binary("wsl").is_some() || which_binary("wsl.exe").is_some() {
            return Ok(ScreenLaunch {
                cmd: String::from("wsl"),
                kind: ScreenKind::Wsl,
                extra_args: vec![String::from("-e"), String::from("screen")],
            });
        }

        return Err(Box::new(io::Error::new(
            io::ErrorKind::NotFound,
            "未检测到 screen。Windows 上请先安装 WSL 并在 Linux 子系统内安装 screen，或先使用 terman 内置 screen。",
        )));
    }

    Err(Box::new(io::Error::new(
        io::ErrorKind::NotFound,
        "未检测到 screen。请先安装 screen（apt/yum/brew）。",
    )))
}

fn which_binary(name: &str) -> Option<String> {
    if let Ok(path_env) = env::var("PATH") {
        let exts = if cfg!(windows) {
            vec![".exe", ".bat", ".cmd", ""]
        } else {
            vec![""]
        };

        let paths = env::split_paths(&path_env);
        for path in paths {
            for ext in &exts {
                let candidate = path.join(format!("{name}{ext}"));
                if candidate.is_file() {
                    return Some(candidate.to_string_lossy().to_string());
                }
            }
        }
    }

    None
}

fn passthrough_env() -> impl Iterator<Item = (String, String)> {
    [
        "TERM",
        "COLORTERM",
        "LC_ALL",
        "LANG",
        "LC_CTYPE",
        "TERM_PROGRAM",
        "TERM_PROGRAM_VERSION",
    ]
    .iter()
    .filter_map(|k| env::var(k).ok().map(|v| (k.to_string(), v)))
    .collect::<Vec<_>>()
    .into_iter()
}

fn build_command(command: Option<String>) -> Result<CommandBuilder, io::Error> {
    let shell = default_shell();

    match command {
        Some(cmd) => {
            let mut builder = CommandBuilder::new(&shell);
            if cfg!(windows) {
                builder.arg("/C");
            } else {
                builder.arg("-c");
            }
            builder.arg(cmd);
            Ok(builder)
        }
        None => Ok(CommandBuilder::new(shell)),
    }
}

fn default_shell() -> String {
    if cfg!(windows) {
        env::var("COMSPEC")
            .ok()
            .filter(|s| !s.is_empty())
            .or_else(|| env::var("ComSpec").ok())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "cmd.exe".to_string())
    } else {
        env::var("SHELL")
            .ok()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "/bin/sh".to_string())
    }
}

fn resolve_size(cols_override: Option<u16>, rows_override: Option<u16>) -> (u16, u16) {
    let (cols, rows) = terminal_size().unwrap_or((120, 32));
    (cols_override.unwrap_or(cols), rows_override.unwrap_or(rows))
}

fn key_to_bytes(key: KeyEvent) -> Option<Vec<u8>> {
    if key.kind != KeyEventKind::Press {
        return None;
    }

    if key.modifiers.contains(KeyModifiers::CONTROL) {
        if let KeyCode::Char(c) = key.code {
            return ctrl_char_bytes(c);
        }
    }

    match key.code {
        KeyCode::Char(c) => {
            if key.modifiers.contains(KeyModifiers::ALT) {
                let mut bytes = vec![0x1b];
                bytes.extend_from_slice(c.to_string().as_bytes());
                Some(bytes)
            } else {
                Some(c.to_string().into_bytes())
            }
        }
        KeyCode::Enter => Some(b"\r".to_vec()),
        KeyCode::Tab => Some(b"\t".to_vec()),
        KeyCode::Backspace => Some(vec![0x08]),
        KeyCode::Esc => Some(vec![0x1b]),
        KeyCode::Up => Some(vec![0x1b, b'[', b'A']),
        KeyCode::Down => Some(vec![0x1b, b'[', b'B']),
        KeyCode::Right => Some(vec![0x1b, b'[', b'C']),
        KeyCode::Left => Some(vec![0x1b, b'[', b'D']),
        KeyCode::Home => Some(vec![0x1b, b'[', b'H']),
        KeyCode::End => Some(vec![0x1b, b'[', b'F']),
        KeyCode::PageUp => Some(vec![0x1b, b'[', b'5', b'~']),
        KeyCode::PageDown => Some(vec![0x1b, b'[', b'6', b'~']),
        KeyCode::Insert => Some(vec![0x1b, b'[', b'2', b'~']),
        KeyCode::Delete => Some(vec![0x1b, b'[', b'3', b'~']),
        _ => None,
    }
}

fn ctrl_char_bytes(c: char) -> Option<Vec<u8>> {
    let lower = c.to_ascii_lowercase();
    let b = match lower {
        'a' => 0x01,
        'b' => 0x02,
        'c' => 0x03,
        'd' => 0x04,
        'e' => 0x05,
        'f' => 0x06,
        'g' => 0x07,
        'h' => 0x08,
        'i' => 0x09,
        'j' => 0x0a,
        'k' => 0x0b,
        'l' => 0x0c,
        'm' => 0x0d,
        'n' => 0x0e,
        'o' => 0x0f,
        'p' => 0x10,
        'q' => 0x11,
        'r' => 0x12,
        's' => 0x13,
        't' => 0x14,
        'u' => 0x15,
        'v' => 0x16,
        'w' => 0x17,
        'x' => 0x18,
        'y' => 0x19,
        'z' => 0x1a,
        '[' => 0x1b,
        '\\' => 0x1c,
        ']' => 0x1d,
        '^' => 0x1e,
        '_' => 0x1f,
        _ => return None,
    };
    Some(vec![b])
}
