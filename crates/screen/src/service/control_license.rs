use std::io;

pub(super) fn request_license_command() -> io::Result<()> {
    println!(
        "{}",
        terman_common::builtin_screen_control_license_hint(env!("CARGO_PKG_VERSION"))
    );
    Ok(())
}