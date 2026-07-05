pub(super) fn request_version_command() {
    println!(
        "{}",
        terman_common::builtin_screen_control_version_hint(env!("CARGO_PKG_VERSION"))
    );
}