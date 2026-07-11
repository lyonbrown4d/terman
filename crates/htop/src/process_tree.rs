use std::collections::HashSet;

#[derive(Default)]
pub(crate) struct ProcessTreeState {
    all_collapsed: bool,
    exceptions: HashSet<String>,
}

impl ProcessTreeState {
    pub(crate) fn is_collapsed(&self, pid: &str) -> bool {
        self.all_collapsed != self.exceptions.contains(pid)
    }

    pub(crate) fn collapse(&mut self, pid: &str) {
        if self.all_collapsed {
            self.exceptions.remove(pid);
        } else {
            self.exceptions.insert(pid.to_string());
        }
    }

    pub(crate) fn expand(&mut self, pid: &str) {
        if self.all_collapsed {
            self.exceptions.insert(pid.to_string());
        } else {
            self.exceptions.remove(pid);
        }
    }

    pub(crate) fn toggle_all(&mut self) {
        self.all_collapsed = !self.all_collapsed;
        self.exceptions.clear();
    }
}