use std::{
    collections::hash_map::DefaultHasher,
    env, fs,
    error::Error,
    hash::{Hash, Hasher},
    io::{self, Read, Write},
    path::PathBuf,
    process::{Command, ExitStatus, Stdio},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread,
    time::Duration,
};

use clap::Args;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    terminal::{self, size as terminal_size},
};
use portable_pty::{CommandBuilder, PtySize, native_pty_system};
use serde::{Deserialize, Serialize};
use terman_common;

#[derive(Args, Debug, Clone)]
#[command(
    about = "screen 桥接入口（先尝试系统 screen，失败自动回退内置）",
    after_help = "常见用法示例：\n  - terman-screen\n  - terman-screen -S dev\n  - terman-screen --list\n  - terman-screen -r dev\n  - terman-screen -x dev\n  - terman-screen --system\n  - terman-screen --system --list\n  - terman-screen --system -r dev\n  - terman-screen --system -S dev\n  - terman-screen --system --detach\n  - terman-screen --system --no-fallback"
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

    /// Name the screen session; maps to `screen -S <NAME>` in system mode.
    #[arg(short = 'S', long = "session", value_name = "NAME")]
    pub session_name: Option<String>,

    /// List known screen sessions. In system mode this maps to `screen -ls`.
    #[arg(long, alias = "ls", conflicts_with_all = ["command", "detach"])]
    pub list: bool,

    /// Resume a detached screen session; maps to `screen -r [NAME]` in system mode.
    #[arg(
        short = 'r',
        long = "resume",
        value_name = "NAME",
        num_args = 0..=1,
        conflicts_with_all = ["command", "detach", "list", "session_name", "multi_attach"]
    )]
    pub resume: Option<Option<String>>,

    /// Attach to an existing session without detaching other displays; maps to `screen -x [NAME]`.
    #[arg(
        short = 'x',
        long = "multi-attach",
        value_name = "NAME",
        num_args = 0..=1,
        conflicts_with_all = ["command", "detach", "list", "session_name", "resume"]
    )]
    pub multi_attach: Option<Option<String>>,

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
            session_name: None,
            list: false,
            resume: None,
            multi_attach: None,
            system: false,
            detach: false,
            login_shell: false,
            no_fallback: false,
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

struct ScreenLaunch {
    cmd: String,
}

pub fn run(args: ScreenArgs) -> Result<(), Box<dyn Error>> {
    if args.detach && !args.system {
        return Err(Box::new(io::Error::new(
            io::ErrorKind::InvalidInput,
            "--detach 仅可在 --system 模式下使用。",
        )));
    }

    if let Some(session_name) = &args.session_name {
        validate_screen_session_name(session_name)?;
    }
    if let Some(Some(session_name)) = &args.resume {
        validate_screen_session_name(session_name)?;
    }
    if let Some(Some(session_name)) = &args.multi_attach {
        validate_screen_session_name(session_name)?;
    }

    if is_builtin_screen_attach_requested(&args) {
        return Err(Box::new(builtin_screen_attach_unsupported_error(&args)));
    }

    if args.list && !args.system {
        list_builtin_screen_sessions()?;
        return Ok(());
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
    let launch = resolve_screen_launch()?;
    let mut cmd = Command::new(&launch.cmd);
    let system_args = build_system_screen_args(&args);

    let status: ExitStatus = cmd
        .args(&system_args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .envs(terman_common::terminal_env())
        .status()?;

    if status.success() {
        Ok(())
    } else {
        let exit_code = status.code().unwrap_or(-1);
        Err(Box::new(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "{}",
                screen_failure_message(
                    "system screen",
                    exit_code,
                    &screen_system_runtime_hints(&system_args, exit_code)
                )
            ),
        )))
    }
}

fn build_system_screen_args(args: &ScreenArgs) -> Vec<String> {
    let mut system_args = Vec::new();

    if args.list {
        system_args.push(String::from("-ls"));
    }

    if args.detach {
        system_args.push(String::from("-d"));
        system_args.push(String::from("-m"));
    }

    if let Some(session_name) = &args.session_name {
        system_args.push(String::from("-S"));
        system_args.push(session_name.clone());
    }

    if let Some(target) = &args.resume {
        system_args.push(String::from("-r"));
        if let Some(target) = target {
            system_args.push(target.clone());
        }
    }

    if let Some(target) = &args.multi_attach {
        system_args.push(String::from("-x"));
        if let Some(target) = target {
            system_args.push(target.clone());
        }
    }

    system_args.extend(args.args.clone());
    system_args
}

fn is_builtin_screen_attach_requested(args: &ScreenArgs) -> bool {
    !args.system && (args.resume.is_some() || args.multi_attach.is_some())
}

fn builtin_screen_attach_unsupported_error(args: &ScreenArgs) -> io::Error {
    let mode = if args.resume.is_some() {
        "恢复 detached 会话"
    } else {
        "多端附加会话"
    };
    io::Error::new(
        io::ErrorKind::Unsupported,
        format!(
            "内置 screen 暂不支持{mode}。请先使用 `terman-screen --system -r <name>` / `terman-screen --system -x <name>` 走系统 screen；跨平台内置 attach 需要后续会话服务支持。",
        ),
    )
}
fn screen_failure_message(scope: &str, exit_code: i32, detail: &str) -> String {
    format!("{scope} 失败（退出码 {exit_code}）：{detail}")
}

fn screen_system_runtime_hints(args: &[String], exit_code: i32) -> String {
    let mut hints = Vec::new();

    if is_screen_attach_attempt(args) {
        hints.push(
            "检测到恢复会话参数 (-r/-R/-x)。若会话不存在，先执行 `screen -ls`（或 `terman-screen --system -ls`）确认会话名后重试。".to_string(),
        );
    }

    if is_screen_session_name_arg(args) && exit_code == 1 {
        hints.push(
            "检测到 `-S <name>` 场景，退出码 1 常见于会话名不存在或已有同名会话。先执行 `terman-screen --system -ls`/`screen -ls` 查看后重试。".to_string(),
        );
    }

    let runtime_hint = match exit_code {
        1 => {
            "参数错误、会话名不存在，或参数与 screen 版本不兼容。建议先用 `terman-screen --system --help` 复现最小命令。"
        }
        2 => {
            "通常与权限、终端环境或可执行文件上下文有关。建议在普通终端重试，或先确认 screen 安装和 shell 环境。"
        }
        126 => "无法执行，请确认 screen 可执行文件有执行权限。",
        127 => "未找到本机 screen 可执行文件，请先确认 screen 安装正常且在 PATH。",
        _ => {
            "返回非预期状态，建议先执行 `terman-screen --system --help` 获取可用参数并用最小参数重试。"
        }
    };
    hints.push(runtime_hint.to_string());
    hints.join("\n")
}

fn is_screen_attach_attempt(args: &[String]) -> bool {
    args.iter()
        .any(|arg| arg == "-r" || arg == "-R" || arg == "-x")
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
    args.iter()
        .any(|arg| arg == "-d" || arg == "-D" || arg == "--detach")
}

struct BuiltinScreenSessionGuard {
    path: PathBuf,
}

impl Drop for BuiltinScreenSessionGuard {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct BuiltinScreenSession {
    name: String,
    pid: String,
    cwd: String,
    command: String,
}

fn validate_screen_session_name(name: &str) -> io::Result<()> {
    if name.trim().is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "screen 会话名不能为空。",
        ));
    }
    Ok(())
}

fn register_builtin_screen_session(
    args: &ScreenArgs,
) -> io::Result<Option<BuiltinScreenSessionGuard>> {
    let Some(session_name) = &args.session_name else {
        return Ok(None);
    };

    let path = builtin_screen_session_record_path(session_name);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let cwd = env::current_dir()
        .map(|path| path.to_string_lossy().to_string())
        .unwrap_or_else(|_| String::from("<unknown>"));
    let command = args.command.clone().unwrap_or_else(default_shell);
    let record = BuiltinScreenSession {
        name: session_name.clone(),
        pid: std::process::id().to_string(),
        cwd,
        command,
    };
    let record = serde_json::to_string_pretty(&record)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;

    fs::write(&path, format!("{record}\n"))?;
    Ok(Some(BuiltinScreenSessionGuard { path }))
}

fn list_builtin_screen_sessions() -> io::Result<()> {
    let dir = builtin_screen_sessions_dir();
    if !dir.exists() {
        println!("{}", terman_common::builtin_screen_no_sessions_hint());
        return Ok(());
    }

    let mut sessions = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            continue;
        }
        let Ok(record) = fs::read_to_string(entry.path()) else {
            continue;
        };
        if let Some(session) = parse_builtin_screen_session_record(&record) {
            sessions.push(session);
        }
    }

    sessions.sort_by(|left, right| left.name.cmp(&right.name));

    if sessions.is_empty() {
        println!("{}", terman_common::builtin_screen_no_sessions_hint());
        return Ok(());
    }

    println!("{}", terman_common::builtin_screen_session_list_header());
    for session in sessions {
        println!(
            "  {}\tpid={}\tcwd={}\tcommand={}",
            session.name, session.pid, session.cwd, session.command
        );
    }

    Ok(())
}

fn parse_builtin_screen_session_record(record: &str) -> Option<BuiltinScreenSession> {
    serde_json::from_str(record).ok()
}

fn builtin_screen_session_record_path(name: &str) -> PathBuf {
    let mut hasher = DefaultHasher::new();
    name.hash(&mut hasher);
    builtin_screen_sessions_dir().join(format!(
        "{}-{:016x}.session",
        sanitize_session_file_name(name),
        hasher.finish()
    ))
}

fn builtin_screen_sessions_dir() -> PathBuf {
    env::temp_dir().join("terman-screen").join("sessions")
}

fn sanitize_session_file_name(name: &str) -> String {
    let sanitized: String = name
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' || ch == '.' {
                ch
            } else {
                '_'
            }
        })
        .collect();

    if sanitized.is_empty() {
        String::from("session")
    } else {
        sanitized
    }
}

fn run_builtin_screen(args: ScreenArgs) -> Result<(), Box<dyn Error>> {
    let _session_record = register_builtin_screen_session(&args)?;
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

    let master = pair.master;
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
        let status = child
            .wait()
            .map(|status| status.exit_code() as i32)
            .unwrap_or(-1);
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
        "提示：默认会在 system 失败后回退到内置 screen；如需严格仅用系统 screen，请加 --no-fallback。\n建议先确认本机 screen 可执行文件，或直接使用内置 screen。"
    } else {
        "提示：默认会在 system 失败后回退到内置 screen；如需严格仅用系统 screen，请加 --no-fallback。\n建议先执行：\n  - screen -V\n  - sudo apt/yum/brew install screen\n  - terman-screen --system --no-fallback"
    }
}
fn resolve_screen_launch() -> Result<ScreenLaunch, Box<dyn Error>> {
    if let Some(path) = terman_common::which_binary("screen") {
        return Ok(ScreenLaunch { cmd: path });
    }

    Err(Box::new(io::Error::new(
        io::ErrorKind::NotFound,
        screen_not_found_hint(),
    )))
}
fn screen_not_found_hint() -> String {
    terman_common::native_tool_not_found_hint("screen")
}
fn build_command(args: &ScreenArgs) -> Result<CommandBuilder, io::Error> {
    let shell = default_shell();

    let mut builder = match args.command.clone() {
        Some(cmd) => {
            let mut builder = CommandBuilder::new(&shell);
            for arg in shell_command_args(&shell, args.login_shell) {
                builder.arg(arg);
            }
            builder.arg(cmd);
            builder
        }
        None => {
            if !cfg!(windows) && args.login_shell {
                let mut builder = CommandBuilder::new(&shell);
                builder.arg("-l");
                builder
            } else {
                CommandBuilder::new(shell)
            }
        }
    };

    if let Some(session_name) = &args.session_name {
        builder.env("STY", session_name.as_str());
        builder.env("TERMAN_SCREEN_SESSION", session_name.as_str());
    }

    Ok(builder)
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

#[cfg(test)]
mod tests {
    use super::{
        ScreenArgs, build_system_screen_args,
        is_builtin_screen_attach_requested, is_screen_attach_attempt, is_screen_detached_arg,
        is_screen_session_name_arg, parse_builtin_screen_session_record, sanitize_session_file_name,
        screen_failure_message, screen_not_found_hint,
    };

    #[test]
    fn recognizes_screen_detach_flags() {
        assert!(is_screen_detached_arg(&["-d".to_string()]));
        assert!(is_screen_detached_arg(&["-D".to_string()]));
        assert!(is_screen_detached_arg(&["--detach".to_string()]));
        assert!(!is_screen_detached_arg(&["-r".to_string()]));
    }

    #[test]
    fn recognizes_screen_attach_args() {
        assert!(is_screen_attach_attempt(&["-r".to_string()]));
        assert!(is_screen_attach_attempt(&["-R".to_string()]));
        assert!(is_screen_attach_attempt(&["-x".to_string()]));
        assert!(!is_screen_attach_attempt(&["attach".to_string()]));
    }

    #[test]
    fn detects_screen_session_name_arg_requires_value() {
        assert!(is_screen_session_name_arg(&[
            "-S".to_string(),
            "dev".to_string()
        ]));
        assert!(!is_screen_session_name_arg(&["-S".to_string()]));
        assert!(!is_screen_session_name_arg(&[]));
    }

    #[test]
    fn screen_failure_message_formats_error() {
        let msg = screen_failure_message("system screen", 127, "未找到 screen");
        assert_eq!(msg, "system screen 失败（退出码 127）：未找到 screen");
    }

    #[test]
    fn screen_not_found_hint_uses_common_i18n_message() {
        let hint = screen_not_found_hint();
        assert!(hint.contains("screen"));
    }

    #[test]
    fn builds_system_args_with_detach_session_and_passthrough_args() {
        let args = ScreenArgs {
            system: true,
            detach: true,
            session_name: Some(String::from("dev")),
            args: vec![String::from("-ls")],
            ..ScreenArgs::default()
        };

        assert_eq!(
            build_system_screen_args(&args),
            vec![
                String::from("-d"),
                String::from("-m"),
                String::from("-S"),
                String::from("dev"),
                String::from("-ls"),
            ]
        );
    }

    #[test]
    fn builds_system_args_for_list() {
        let args = ScreenArgs {
            system: true,
            list: true,
            ..ScreenArgs::default()
        };

        assert_eq!(build_system_screen_args(&args), vec![String::from("-ls")]);
    }

    #[test]
    fn builds_system_args_for_resume_with_optional_target() {
        let args = ScreenArgs {
            system: true,
            resume: Some(Some(String::from("dev"))),
            ..ScreenArgs::default()
        };

        assert_eq!(
            build_system_screen_args(&args),
            vec![String::from("-r"), String::from("dev")]
        );
    }

    #[test]
    fn builds_system_args_for_multi_attach_without_target() {
        let args = ScreenArgs {
            system: true,
            multi_attach: Some(None),
            ..ScreenArgs::default()
        };

        assert_eq!(build_system_screen_args(&args), vec![String::from("-x")]);
    }

    #[test]
    fn detects_builtin_attach_modes() {
        let builtin_resume = ScreenArgs {
            resume: Some(Some(String::from("dev"))),
            ..ScreenArgs::default()
        };
        let system_resume = ScreenArgs {
            system: true,
            resume: Some(Some(String::from("dev"))),
            ..ScreenArgs::default()
        };

        assert!(is_builtin_screen_attach_requested(&builtin_resume));
        assert!(!is_builtin_screen_attach_requested(&system_resume));
    }

    #[test]
    fn sanitizes_builtin_session_record_name() {
        assert_eq!(sanitize_session_file_name("dev/session:1"), "dev_session_1");
    }

    #[test]
    fn parses_builtin_session_record() {
        let record = r#"{"name":"dev","pid":"42","cwd":"C:/repo","command":"pwsh"}"#;
        let parsed = parse_builtin_screen_session_record(record).expect("record should parse");

        assert_eq!(parsed.name, "dev");
        assert_eq!(parsed.pid, "42");
        assert_eq!(parsed.cwd, "C:/repo");
        assert_eq!(parsed.command, "pwsh");
    }

}
