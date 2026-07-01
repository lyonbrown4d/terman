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
use which::which;

#[derive(Args, Debug, Clone)]
pub struct ScreenArgs {
    /// If set, run this command string through the platform shell in built-in mode.
    #[arg(short, long, value_name = "CMD", conflicts_with = "system")]
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

    /// Start screen in detached mode (system mode only).
    #[arg(long)]
    pub detach: bool,

    /// Start a login shell in built-in mode; ignored in system mode.
    #[arg(long, conflicts_with = "system")]
    pub login_shell: bool,

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
            detach: false,
            login_shell: false,
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
    if args.detach && !args.system {
        return Err(Box::new(io::Error::new(
            io::ErrorKind::InvalidInput,
            "--detach 仅可在 --system 模式下使用。",
        )));
    }

    if args.system {
        run_system_screen(args)?;
        return Ok(());
    }

    run_builtin_screen(args)
}


fn run_system_screen(args: ScreenArgs) -> Result<(), Box<dyn Error>> {
    let launch = resolve_screen_launch()?;

    let mut cmd = Command::new(&launch.cmd);
    let mut system_args = args.args;

    if args.detach {
        system_args.insert(0, String::from("-m"));
        system_args.insert(0, String::from("-d"));
    }

    if let ScreenKind::Wsl = launch.kind {
        cmd.args(&launch.extra_args);
        eprintln!("当前使用 WSL screen 回退路径；建议在 WSL 发行版中常驻使用 screen。");
    }

    let status: ExitStatus = cmd
        .args(&system_args)
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
    let command = build_command(&args)?;
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

    let mut exit_code: Option<i32> = None;

    loop {
        match exit_rx.try_recv() {
            Ok(code) => {
                exit_code = Some(code);
                break;
            }
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

    let exit_code = exit_code.unwrap_or(-1);
    if exit_code == 0 {
        Ok(())
    } else {
        Err(Box::new(io::Error::new(
            io::ErrorKind::Other,
            format!("内置 screen 退出码: {exit_code}"),
        )))
    }
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
        if let Some(path) = which_binary("wsl").or_else(|| which_binary("wsl.exe")) {
            return Ok(ScreenLaunch {
                cmd: path,
                kind: ScreenKind::Wsl,
                extra_args: vec![String::from("-e"), String::from("screen")],
            });
        }

        return Err(Box::new(io::Error::new(io::ErrorKind::NotFound, screen_not_found_hint())));
    }

    Err(Box::new(io::Error::new(
        io::ErrorKind::NotFound,
        screen_not_found_hint(),
    )))
}

fn screen_not_found_hint() -> &'static str {
    if cfg!(windows) {
        "未检测到 screen。可选方案：1) 使用 WSL 安装 screen（推荐）：wsl -e sudo apt install screen；2) 安装 Windows 版本 screen（如 scoop install screen）；3) 先使用 terman 内置 screen。"
    } else {
        "未检测到 screen。请先安装 screen（apt/yum/brew）。"
    }
}
fn which_binary(name: &str) -> Option<String> {
    which(name).ok().map(|path| path.to_string_lossy().to_string())
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

fn build_command(args: &ScreenArgs) -> Result<CommandBuilder, io::Error> {
    let shell = default_shell();

    match args.command.clone() {
        Some(cmd) => {
            let mut builder = CommandBuilder::new(&shell);
            for arg in shell_command_args(&shell, args.login_shell) {
                builder.arg(arg);
            }
            builder.arg(cmd);
            Ok(builder)
        }
        None => {
            if !cfg!(windows) && args.login_shell {
                let mut builder = CommandBuilder::new(&shell);
                builder.arg("-l");
                Ok(builder)
            } else {
                Ok(CommandBuilder::new(shell))
            }
        }
    }
}

fn shell_command_args(shell: &str, login_shell: bool) -> Vec<String> {
    let file_name = std::path::Path::new(shell)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(shell)
        .to_ascii_lowercase();

    if file_name.contains("cmd.exe") {
        return vec![String::from("/C")];
    }

    if file_name.contains("powershell") || file_name.contains("pwsh") {
        return vec![String::from("-Command")];
    }

    if file_name.ends_with("bash")
        || file_name.ends_with("bash.exe")
        || file_name.ends_with("sh")
        || file_name.ends_with("sh.exe")
    {
        if login_shell {
            return vec![String::from("-lc")];
        }
        return vec![String::from("-c")];
    }

    vec![String::from("-c")]
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
use clap::Parser;

#[derive(Parser)]
struct Cli {
    #[command(flatten)]
    args: ScreenArgs,
}

pub fn run_with_binary_parse() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    run(cli.args)
}
