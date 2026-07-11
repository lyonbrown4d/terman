use crate::pane_layout::PaneDirection;

pub(crate) fn resize_pane_direction_arg(args: &[String]) -> Option<PaneDirection> {
    resize_arguments(args).iter().find_map(|arg| match arg.as_str() {
        "-L" | "--left" => Some(PaneDirection::Left),
        "-R" | "--right" => Some(PaneDirection::Right),
        "-U" | "--up" => Some(PaneDirection::Up),
        "-D" | "--down" => Some(PaneDirection::Down),
        _ => None,
    })
}

pub(crate) fn resize_pane_adjustment_arg(args: &[String]) -> u16 {
    let mut skip_value = false;
    for arg in resize_arguments(args) {
        if skip_value {
            skip_value = false;
            continue;
        }
        match arg.as_str() {
            "-t" | "--target" | "-x" | "--width" | "-y" | "--height" => {
                skip_value = true;
            }
            value if value.starts_with('-') => {}
            value => {
                if let Ok(adjustment) = value.parse::<u16>() {
                    return adjustment.max(1);
                }
            }
        }
    }
    1
}

fn resize_arguments(args: &[String]) -> &[String] {
    let start = args.iter()
        .position(|arg| matches!(arg.as_str(), "resize-pane" | "resizep"))
        .map(|index| index + 1)
        .unwrap_or(0);
    &args[start..]
}

#[cfg(test)]
mod tests {
    use super::{resize_pane_adjustment_arg, resize_pane_direction_arg};
    use crate::pane_layout::PaneDirection;

    #[test]
    fn parses_direction_and_adjustment_without_using_target_numbers() {
        let args = ["resizep".into(), "-t".into(), "dev:1.0".into(), "-L".into(), "5".into()];
        assert_eq!(resize_pane_direction_arg(&args), Some(PaneDirection::Left));
        assert_eq!(resize_pane_adjustment_arg(&args), 5);
    }
}