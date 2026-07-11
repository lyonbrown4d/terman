use crate::{
    app_input::TreeBranchAction,
    model::ProcessRow,
    process_tree::ProcessTreeState,
    render::Tab,
};

pub(crate) fn process_tree_active(tab: Tab, tree: bool) -> bool {
    tree && matches!(tab, Tab::Overview | Tab::Processes)
}

pub(crate) fn apply_tree_branch(
    tree_state: &mut ProcessTreeState,
    processes: &[ProcessRow],
    selected: usize,
    action: TreeBranchAction,
) {
    let Some(row) = processes.get(selected).filter(|row| row.has_children) else {
        return;
    };
    match action {
        TreeBranchAction::Expand => tree_state.expand(row.pid.as_str()),
        TreeBranchAction::Collapse => tree_state.collapse(row.pid.as_str()),
    }
}
