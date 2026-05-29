use std::collections::HashMap;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(in crate::agent) struct RepairScanCursors {
    start_after_by_prefix: HashMap<String, String>,
}

impl RepairScanCursors {
    pub(in crate::agent::repair) fn start_after(&self, prefix: &str) -> Option<&str> {
        self.start_after_by_prefix.get(prefix).map(String::as_str)
    }

    pub(in crate::agent::repair) fn update_after_page(
        &mut self,
        prefix: &str,
        next_start_after: Option<String>,
    ) -> bool {
        match next_start_after {
            Some(value) => {
                self.start_after_by_prefix.insert(prefix.to_owned(), value);
                true
            }
            None => {
                self.start_after_by_prefix.remove(prefix);
                false
            }
        }
    }
}
