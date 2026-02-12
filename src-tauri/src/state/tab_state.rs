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
    /// Internal: navigation history stack
    #[serde(skip)]
    pub nav_stack: Vec<String>,
    /// Internal: current position in nav_stack
    #[serde(skip)]
    pub nav_pos: i32,
    /// Internal: true when a back/forward navigation is in progress
    #[serde(skip)]
    pub nav_traversing: bool,
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: create a TabInfo with sensible defaults
    fn make_tab(label: &str, url: &str) -> TabInfo {
        TabInfo {
            label: label.to_string(),
            url: url.to_string(),
            title: "Test".to_string(),
            is_loading: false,
            favicon: None,
            can_go_back: false,
            can_go_forward: false,
            nav_stack: Vec::new(),
            nav_pos: -1,
            nav_traversing: false,
        }
    }

    // ── TabManager basics ──────────────────────────────────

    #[test]
    fn new_manager_is_empty() {
        let tm = TabManager::new();
        assert_eq!(tm.tab_count(), 0);
        assert!(tm.get_active_tab().is_none());
        assert!(tm.get_all_tabs().is_empty());
    }

    #[test]
    fn add_and_count() {
        let tm = TabManager::new();
        tm.add_tab(make_tab("t1", "https://a.com"));
        assert_eq!(tm.tab_count(), 1);
        tm.add_tab(make_tab("t2", "https://b.com"));
        assert_eq!(tm.tab_count(), 2);
    }

    #[test]
    fn get_tab_hit_and_miss() {
        let tm = TabManager::new();
        tm.add_tab(make_tab("t1", "https://a.com"));

        assert_eq!(tm.get_tab("t1").unwrap().url, "https://a.com");
        assert!(tm.get_tab("missing").is_none());
    }

    #[test]
    fn remove_tab_returns_removed_and_shrinks() {
        let tm = TabManager::new();
        tm.add_tab(make_tab("t1", "https://a.com"));
        tm.add_tab(make_tab("t2", "https://b.com"));

        let removed = tm.remove_tab("t1").unwrap();
        assert_eq!(removed.label, "t1");
        assert_eq!(tm.tab_count(), 1);
        assert!(tm.get_tab("t1").is_none());
    }

    #[test]
    fn remove_missing_tab_returns_none() {
        let tm = TabManager::new();
        assert!(tm.remove_tab("nope").is_none());
    }

    #[test]
    fn get_all_tabs_preserves_order() {
        let tm = TabManager::new();
        tm.add_tab(make_tab("t1", "https://a.com"));
        tm.add_tab(make_tab("t2", "https://b.com"));
        tm.add_tab(make_tab("t3", "https://c.com"));

        let labels: Vec<String> = tm.get_all_tabs().iter().map(|t| t.label.clone()).collect();
        assert_eq!(labels, vec!["t1", "t2", "t3"]);
    }

    #[test]
    fn get_tab_labels() {
        let tm = TabManager::new();
        tm.add_tab(make_tab("t1", "https://a.com"));
        tm.add_tab(make_tab("t2", "https://b.com"));
        assert_eq!(tm.get_tab_labels(), vec!["t1", "t2"]);
    }

    // ── Active tab ─────────────────────────────────────────

    #[test]
    fn set_and_get_active() {
        let tm = TabManager::new();
        tm.set_active_tab(Some("t1".to_string()));
        assert_eq!(tm.get_active_tab(), Some("t1".to_string()));
    }

    #[test]
    fn clear_active() {
        let tm = TabManager::new();
        tm.set_active_tab(Some("t1".to_string()));
        tm.set_active_tab(None);
        assert!(tm.get_active_tab().is_none());
    }

    // ── update_tab ─────────────────────────────────────────

    #[test]
    fn update_tab_modifies_fields() {
        let tm = TabManager::new();
        tm.add_tab(make_tab("t1", "https://a.com"));

        tm.update_tab("t1", |tab| {
            tab.url = "https://b.com".to_string();
            tab.title = "Updated".to_string();
            tab.is_loading = true;
        });

        let tab = tm.get_tab("t1").unwrap();
        assert_eq!(tab.url, "https://b.com");
        assert_eq!(tab.title, "Updated");
        assert!(tab.is_loading);
    }

    #[test]
    fn update_missing_tab_is_noop() {
        let tm = TabManager::new();
        // Should not panic
        tm.update_tab("missing", |tab| tab.url = "x".to_string());
    }

    // ── get_adjacent_tab ───────────────────────────────────

    #[test]
    fn adjacent_prefers_right_neighbor() {
        let tm = TabManager::new();
        tm.add_tab(make_tab("t1", "https://a.com"));
        tm.add_tab(make_tab("t2", "https://b.com"));
        tm.add_tab(make_tab("t3", "https://c.com"));

        assert_eq!(tm.get_adjacent_tab("t1"), Some("t2".to_string()));
        assert_eq!(tm.get_adjacent_tab("t2"), Some("t3".to_string()));
    }

    #[test]
    fn adjacent_falls_back_to_left() {
        let tm = TabManager::new();
        tm.add_tab(make_tab("t1", "https://a.com"));
        tm.add_tab(make_tab("t2", "https://b.com"));

        // t2 is rightmost → falls back to t1
        assert_eq!(tm.get_adjacent_tab("t2"), Some("t1".to_string()));
    }

    #[test]
    fn adjacent_returns_none_when_alone() {
        let tm = TabManager::new();
        tm.add_tab(make_tab("t1", "https://a.com"));
        assert!(tm.get_adjacent_tab("t1").is_none());
    }

    #[test]
    fn adjacent_returns_none_for_missing() {
        let tm = TabManager::new();
        assert!(tm.get_adjacent_tab("nope").is_none());
    }

    // ── Navigation state logic ─────────────────────────────

    #[test]
    fn initial_nav_state() {
        let tab = make_tab("t1", "https://google.com");
        assert!(tab.nav_stack.is_empty());
        assert_eq!(tab.nav_pos, -1);
        assert!(!tab.can_go_back);
        assert!(!tab.can_go_forward);
        assert!(!tab.nav_traversing);
    }

    #[test]
    fn nav_push_builds_stack() {
        let mut tab = make_tab("t1", "https://a.com");

        // Simulate first page load
        tab.nav_stack.push("https://a.com".to_string());
        tab.nav_pos = 0;

        // Navigate to b.com
        let new_pos = tab.nav_pos + 1;
        tab.nav_stack.truncate(new_pos as usize);
        tab.nav_stack.push("https://b.com".to_string());
        tab.nav_pos = new_pos;

        assert_eq!(tab.nav_stack, vec!["https://a.com", "https://b.com"]);
        assert_eq!(tab.nav_pos, 1);
    }

    #[test]
    fn nav_push_truncates_forward_history() {
        let mut tab = make_tab("t1", "https://a.com");
        tab.nav_stack = vec![
            "https://a.com".to_string(),
            "https://b.com".to_string(),
            "https://c.com".to_string(),
        ];
        tab.nav_pos = 0; // Went back to a.com

        // Navigate to d.com — should drop b.com and c.com
        let new_pos = tab.nav_pos + 1;
        tab.nav_stack.truncate(new_pos as usize);
        tab.nav_stack.push("https://d.com".to_string());
        tab.nav_pos = new_pos;

        assert_eq!(tab.nav_stack, vec!["https://a.com", "https://d.com"]);
        assert_eq!(tab.nav_pos, 1);
    }

    #[test]
    fn nav_back_and_forward_flags() {
        let tab = TabInfo {
            nav_stack: vec![
                "https://a.com".to_string(),
                "https://b.com".to_string(),
                "https://c.com".to_string(),
            ],
            nav_pos: 1, // At b.com (middle)
            ..make_tab("t1", "https://b.com")
        };

        assert!(tab.nav_pos > 0, "should be able to go back");
        assert!(
            tab.nav_pos < (tab.nav_stack.len() as i32 - 1),
            "should be able to go forward"
        );
    }

    #[test]
    fn nav_at_start_cannot_go_back() {
        let tab = TabInfo {
            nav_stack: vec!["https://a.com".to_string()],
            nav_pos: 0,
            ..make_tab("t1", "https://a.com")
        };
        assert!(!(tab.nav_pos > 0));
    }

    #[test]
    fn nav_at_end_cannot_go_forward() {
        let tab = TabInfo {
            nav_stack: vec!["https://a.com".to_string(), "https://b.com".to_string()],
            nav_pos: 1,
            ..make_tab("t1", "https://b.com")
        };
        assert!(!(tab.nav_pos < (tab.nav_stack.len() as i32 - 1)));
    }

    // ── next_tab_label ─────────────────────────────────────

    #[test]
    fn tab_label_has_correct_format() {
        let label = next_tab_label();
        assert!(label.starts_with("tab-"));
    }

    #[test]
    fn tab_labels_are_unique() {
        let a = next_tab_label();
        let b = next_tab_label();
        assert_ne!(a, b);
    }
}
