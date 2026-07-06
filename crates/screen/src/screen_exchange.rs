use std::{env, path::PathBuf};

const DEFAULT_EXCHANGE_FILE_NAME: &str = "screen-exchange";

pub(crate) fn default_screen_exchange_file() -> PathBuf {
    env::temp_dir().join(DEFAULT_EXCHANGE_FILE_NAME)
}