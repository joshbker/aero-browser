use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

/// Global tab ID counter — ensures unique labels across the app lifetime
static TAB_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Generate the next unique tab label (e.g. "tab-1", "tab-2")
pub fn next_tab_label() -> String {
    let id = TAB_COUNTER.fetch_add(1, Ordering::SeqCst);
    format!("tab-{}", id)
}

/// Info about a single tab, sent to the frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabInfo {
    pub label: String,
    pub url: String,
    pub title: String,
    pub is_loading: bool,
    pub favicon: Option<String>,
    pub can_go_back: bool,
    pub can_go_forward: bool,
}

/// Manages the list of open tabs and which one is active
pub struct TabManager {
    pub tabs: Mutex<Vec<TabInfo>>,
    pub active_tab: Mutex<Option<String>>,
}

impl TabManager {
    pub fn new() -> Self {
        Self {
            tabs: Mutex::new(Vec::new()),
            active_tab: Mutex::new(None),
        }
    }

    pub fn add_tab(&self, info: TabInfo) {
        let mut tabs = self.tabs.lock().unwrap();
        tabs.push(info);
    }

    pub fn remove_tab(&self, label: &str) -> Option<TabInfo> {
        let mut tabs = self.tabs.lock().unwrap();
        if let Some(pos) = tabs.iter().position(|t| t.label == label) {
            Some(tabs.remove(pos))
        } else {
            None
        }
    }

    pub fn get_all_tabs(&self) -> Vec<TabInfo> {
        let tabs = self.tabs.lock().unwrap();
        tabs.clone()
    }

    pub fn get_tab(&self, label: &str) -> Option<TabInfo> {
        let tabs = self.tabs.lock().unwrap();
        tabs.iter().find(|t| t.label == label).cloned()
    }

    pub fn update_tab<F>(&self, label: &str, updater: F)
    where
        F: FnOnce(&mut TabInfo),
    {
        let mut tabs = self.tabs.lock().unwrap();
        if let Some(tab) = tabs.iter_mut().find(|t| t.label == label) {
            updater(tab);
        }
    }

    pub fn get_active_tab(&self) -> Option<String> {
        let active = self.active_tab.lock().unwrap();
        active.clone()
    }

    pub fn set_active_tab(&self, label: Option<String>) {
        let mut active = self.active_tab.lock().unwrap();
        *active = label;
    }

    pub fn tab_count(&self) -> usize {
        let tabs = self.tabs.lock().unwrap();
        tabs.len()
    }

    pub fn get_tab_labels(&self) -> Vec<String> {
        let tabs = self.tabs.lock().unwrap();
        tabs.iter().map(|t| t.label.clone()).collect()
    }

    /// Get the label of the tab adjacent to the given one (for switching after close)
    pub fn get_adjacent_tab(&self, label: &str) -> Option<String> {
        let tabs = self.tabs.lock().unwrap();
        if let Some(pos) = tabs.iter().position(|t| t.label == label) {
            // Prefer the tab to the right, fall back to the left
            if pos + 1 < tabs.len() {
                Some(tabs[pos + 1].label.clone())
            } else if pos > 0 {
                Some(tabs[pos - 1].label.clone())
            } else {
                None
            }
        } else {
            None
        }
    }
}
