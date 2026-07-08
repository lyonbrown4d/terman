use std::{
    env,
    fs::OpenOptions,
    io::{self, Write},
    path::{Path, PathBuf},
};

const ATTACH_HARDCOPY_PREFIX_ENV: &str = "TERMAN_SCREEN_HARDCOPY_PREFIX";
const DEFAULT_ATTACH_HARDCOPY_PREFIX: &str = "hardcopy";

pub(super) struct AttachHardcopySettings {
    pub(super) append: bool,
    pub(super) directory: Option<PathBuf>,
    pub(super) window_index: usize,
}

pub(super) fn write_numbered_hardcopy(
    settings: &AttachHardcopySettings,
    bytes: &[u8],
) -> io::Result<String> {
    let prefix = attach_hardcopy_prefix();
    let path = attach_hardcopy_path(
        settings.directory.as_deref(),
        &prefix,
        settings.window_index,
    );
    write_hardcopy(&path, settings.append, bytes)?;
    Ok(path.display().to_string())
}

fn write_hardcopy(path: &Path, append: bool, bytes: &[u8]) -> io::Result<()> {
    let mut options = OpenOptions::new();
    options.create(true).write(true);
    if append {
        options.append(true);
    } else {
        options.truncate(true);
    }
    let mut file = options.open(path)?;
    file.write_all(bytes)
}

fn attach_hardcopy_prefix() -> String {
    env::var(ATTACH_HARDCOPY_PREFIX_ENV)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| DEFAULT_ATTACH_HARDCOPY_PREFIX.to_string())
}

fn attach_hardcopy_path(hardcopy_dir: Option<&Path>, prefix: &str, index: usize) -> PathBuf {
    let file_name = format!("{prefix}.{index}");
    hardcopy_dir
        .map(|directory| directory.join(&file_name))
        .unwrap_or_else(|| PathBuf::from(file_name))
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::attach_hardcopy_path;

    #[test]
    fn formats_attach_hardcopy_path() {
        assert_eq!(
            attach_hardcopy_path(None, "hardcopy", 0),
            PathBuf::from("hardcopy.0")
        );
        assert_eq!(
            attach_hardcopy_path(Some(Path::new("copies")), "screen-copy", 42),
            PathBuf::from("copies").join("screen-copy.42")
        );
    }
}
