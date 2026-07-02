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
use terman_common;


#[derive(Args, Debug, Clone)]
#[command(
    about = "screen 桥接入口（先尝试系统 screen，失败自动回退内置）",
    after_help = "常见用法示例：\n  - terman screen\n  - terman screen --system\n  - terman screen --system -S dev\n  - terman screen --system --detach\n  - terman screen --system --wsl\n  - terman screen --system --no-fallback",
)]
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

    /// 启动系统 screen 后台模式（等价于 `screen -d -m`）。
    #[arg(long, requires = "system")]
    pub detach: bool,

    /// Start a login shell in built-in mode; ignored in system mode.
    #[arg(long, conflicts_with = "system")]
    pub login_shell: bool,

    /// 回退到内置 screen（`--system` 启动失败时）。
    #[arg(long)]
    pub no_fallback: bool,

    /// 强制在 Windows 下使用 WSL 作为系统 screen 后端。仅 `--system` 可用。
    #[arg(long, requires = "system")]
    pub wsl: bool,

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
            no_fallback: false,
            wsl: false,
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
        match run_system_screen(args.clone()) {
            Ok(()) => return Ok(()),
            Err(err) => {
                if args.no_fallback {
                    return Err(err);
                }
                eprintln!("系统 screen 执行失败，回退到内置 screen: {err}");
                eprintln!("{}", system_screen_fallback_hint());
            }
        }

        let mut fallback_args = args;
        fallback_args.system = false;
        fallback_args.detach = false;
        return run_builtin_screen(fallback_args);
    }

    run_builtin_screen(args)
}


fn run_system_screen(args: ScreenArgs) -> Result<(), Box<dyn Error>> {
    let launch = resolve_screen_launch(args.wsl)?;

    if let ScreenKind::Wsl = launch.kind {
        validate_screen_wsl_launch(&launch)?;
    }

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
        .envs(terman_common::passthrough_env())
        .env("TERM", env::var("TERM").unwrap_or_else(|_| String::from("xterm-256color")))
        .status()?;

    if status.success() {
        Ok(())
    } else {
        let exit_code = status.code().unwrap_or(-1);
        Err(Box::new(io::Error::new(
            io::ErrorKind::Other,
            format!("{}", screen_failure_message("system screen", exit_code, &screen_system_runtime_hints(&system_args, exit_code, &launch.kind)))
        )))
    }
}

fn validate_screen_wsl_launch(launch: &ScreenLaunch) -> Result<(), Box<dyn Error>> {
    let status = Command::new(&launch.cmd)
        .arg("-e")
        .arg("which")
        .arg("screen")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?;

    if status.success() {
        return Ok(());
    }

    let code = status.code().unwrap_or(-1);
    Err(Box::new(io::Error::new(
        io::ErrorKind::Other,
        format!(
            "{}",
            screen_failure_message(
                "system screen WSL 预检",
                code,
                &format!(
                    "当前已进入 WSL 回退路径，但未检测到 WSL 内 screen。建议先执行 `wsl -l -v` 检查发行版状态，再执行 `wsl --status` 检查子系统可用性，最后运行 `wsl -e screen -V` 进行预检。{}",
                    screen_wsl_runtime_hint(),
                ),
            ),
        ),
    )))
}

fn screen_failure_message(scope: &str, exit_code: i32, detail: &str) -> String {
    format!("{scope} 失败（退出码 {exit_code}）：{detail}")
}

fn screen_wsl_runtime_hint() -> &'static str {
    "建议先在 WSL 内执行 `wsl -e which screen` / `wsl -e screen --version` 确认安装与环境。"
}
fn screen_system_runtime_hints(args: &[String], exit_code: i32, kind: &ScreenKind) -> String {
    let mut hints = Vec::new();

    if let ScreenKind::Wsl = kind {
        hints.push(
            "WSL 回退路径失败时，建议先执行 `wsl -l -v`（检查发行版）、`wsl --status`（检查子系统）与 `wsl -e screen -V`（确认 screen 可用）。".to_string(),
        );
    }


    if let ScreenKind::Wsl = kind {
        if is_screen_detached_arg(args) {
            hints.push(
                "WSL 回退路径执行 detached 场景失败时，建议先在 WSL 终端直接复现：wsl -e screen <同样参数>，确认会话名、路径与环境变量无差异。".to_string(),
            );
        }
    }

    if is_screen_attach_attempt(args) {
        hints.push(
            "检测到恢复会话参数 (-r/-R/-x)。若会话不存在，先执行 `screen -list`（或 `terman screen --system -ls`）确认会话名后重试。".to_string(),
        );
    }

    if is_screen_session_name_arg(args) && exit_code == 1 {
        hints.push(
            "检测到 `-S <name>` 场景，退出码 1 常见于会话名不存在或已有同名会话。先执行 `terman screen --system -list`/`screen -list` 查看后重试。".to_string(),
        );
    }

    let runtime_hint = match exit_code {
        1 => {
            "参数错误、会话名不存在，或参数与 screen 版本不兼容。建议先用 `terman screen --system --help` 复现最小命令。"
        }
        2 => {
            "通常与权限、终端环境或可执行文件上下文有关。建议在普通终端重试，或先确认 screen 安装和 shell 环境。"
        }
        126 => {
            "无法执行，请确认 screen 可执行文件有执行权限。"
        }
        127 => {
            "未找到可执行文件，请先确认 screen 安装正常且在 PATH。"
        }
        _ => {
            "返回非预期状态，建议先执行 `terman screen --system --help` 获取可用参数并用最小参数重试。"
        }
    };
    hints.push(runtime_hint.to_string());
    hints.join("\n")
}

fn is_screen_attach_attempt(args: &[String]) -> bool {
    args.iter().any(|arg| arg == "-r" || arg == "-R" || arg == "-x")
}

fn is_screen_session_name_arg(args: &[String]) -> bool {
    let mut iter = args.iter().peekable();
    while let Some(arg) = iter.next() {
        if arg == "-S" {
            return iter.peek().is_some();
        }
    }
    false
}

fn is_screen_detached_arg(args: &[String]) -> bool {
    args.iter().any(|arg| arg == "-d" || arg == "-D" || arg == "--detach")
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
            screen_failure_message("内置 screen", exit_code, "进程退出码非零"),
        )))
    }
}
fn system_screen_fallback_hint() -> &'static str {
    if cfg!(windows) {
        "提示：默认会在 system 失败后回退到内置 screen；如需严格仅用系统 screen，请加 --no-fallback。\n建议先执行：\n  - wsl -e screen --version\n  - wsl -e sudo apt install screen\n  - terman screen --system --no-fallback"
    } else {
        "提示：默认会在 system 失败后回退到内置 screen；如需严格仅用系统 screen，请加 --no-fallback。\n建议先执行：\n  - screen --version\n  - sudo apt/yum/brew install screen\n  - terman screen --system --no-fallback"
    }
}
fn resolve_screen_launch(use_wsl: bool) -> Result<ScreenLaunch, Box<dyn Error>> {
    if use_wsl {
        if !cfg!(windows) {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::InvalidInput,
                "--wsl 仅在 Windows 下可用。",
            )));
        }

        if let Some(path) = terman_common::which_binary("wsl").or_else(|| terman_common::which_binary("wsl.exe")) {
            return Ok(ScreenLaunch {
                cmd: path,
                kind: ScreenKind::Wsl,
                extra_args: vec![String::from("-e"), String::from("screen")],
            });
        }

        return Err(Box::new(io::Error::new(
            io::ErrorKind::NotFound,
            "未检测到 WSL。请先启用 Windows 的 WSL（如安装并配置一个发行版），再执行 `wsl -e sudo apt install screen`。",
        )));
    }

    if let Some(path) = terman_common::which_binary("screen") {
        return Ok(ScreenLaunch {
            cmd: path,
            kind: ScreenKind::Native,
            extra_args: Vec::new(),
        });
    }

    if cfg!(windows) {
        if let Some(path) = terman_common::which_binary("wsl").or_else(|| terman_common::which_binary("wsl.exe")) {
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



