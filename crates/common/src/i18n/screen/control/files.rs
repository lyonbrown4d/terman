use crate::i18n::{MessageKey, localized_message};

pub fn builtin_screen_control_hardcopy_path_required_hint() -> String {
    localized_message(MessageKey::BuiltinScreenControlHardcopyPathRequired, &[])
}

pub fn builtin_screen_control_hardcopydir_required_hint() -> String {
    localized_message(MessageKey::BuiltinScreenControlHardcopydirRequired, &[])
}

pub fn builtin_screen_control_hardcopy_append_required_hint() -> String {
    localized_message(MessageKey::BuiltinScreenControlHardcopyAppendRequired, &[])
}

pub fn builtin_screen_control_pastefile_path_required_hint() -> String {
    localized_message(MessageKey::BuiltinScreenControlPastefilePathRequired, &[])
}

pub fn builtin_screen_control_readbuf_path_required_hint() -> String {
    localized_message(MessageKey::BuiltinScreenControlReadbufPathRequired, &[])
}

pub fn builtin_screen_control_readreg_required_hint() -> String {
    localized_message(MessageKey::BuiltinScreenControlReadregRequired, &[])
}

pub fn builtin_screen_control_writebuf_path_required_hint() -> String {
    localized_message(MessageKey::BuiltinScreenControlWritebufPathRequired, &[])
}

pub fn builtin_screen_control_buffer_encoding_required_hint() -> String {
    localized_message(MessageKey::BuiltinScreenControlBufferEncodingRequired, &[])
}

pub fn builtin_screen_control_source_path_required_hint() -> String {
    localized_message(MessageKey::BuiltinScreenControlSourcePathRequired, &[])
}

pub fn builtin_screen_control_hardcopy_complete_hint(path: &str, bytes: usize) -> String {
    let bytes = bytes.to_string();
    localized_message(
        MessageKey::BuiltinScreenControlHardcopyComplete,
        &[("path", path), ("bytes", &bytes)],
    )
}

pub fn builtin_screen_control_writebuf_complete_hint(path: &str, bytes: usize) -> String {
    let bytes = bytes.to_string();
    localized_message(
        MessageKey::BuiltinScreenControlWritebufComplete,
        &[("path", path), ("bytes", &bytes)],
    )
}