use std::sync::Mutex;

/// Default chrome height: 36px tab bar + 40px toolbar = 76px
pub const DEFAULT_CHROME_HEIGHT: f64 = 76.0;

/// Height added by the bookmarks bar
pub const BOOKMARKS_BAR_HEIGHT: f64 = 28.0;

/// Thread-safe mutable chrome height — changes when bookmarks bar toggles
pub struct ChromeHeight {
	height: Mutex<f64>,
}

impl ChromeHeight {
	pub fn new() -> Self {
		Self {
			height: Mutex::new(DEFAULT_CHROME_HEIGHT),
		}
	}

	pub fn get(&self) -> f64 {
		*self.height.lock().unwrap()
	}

	pub fn set(&self, h: f64) {
		*self.height.lock().unwrap() = h;
	}

	pub fn set_bookmarks_bar(&self, visible: bool) {
		let h = if visible {
			DEFAULT_CHROME_HEIGHT + BOOKMARKS_BAR_HEIGHT
		} else {
			DEFAULT_CHROME_HEIGHT
		};
		self.set(h);
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn default_height() {
		let ch = ChromeHeight::new();
		assert_eq!(ch.get(), DEFAULT_CHROME_HEIGHT);
	}

	#[test]
	fn bookmarks_bar_toggle() {
		let ch = ChromeHeight::new();
		ch.set_bookmarks_bar(true);
		assert_eq!(ch.get(), DEFAULT_CHROME_HEIGHT + BOOKMARKS_BAR_HEIGHT);
		ch.set_bookmarks_bar(false);
		assert_eq!(ch.get(), DEFAULT_CHROME_HEIGHT);
	}
}
