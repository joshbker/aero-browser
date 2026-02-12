# Testing — Aero Browser

## Automated Tests

### Rust unit tests
```bash
cd src-tauri && cargo test
```
Tests cover: TabManager CRUD, active tab, adjacent tab selection, navigation state logic, label generation.

### JavaScript unit tests
```bash
npm test          # single run
npm run test:watch  # watch mode
```
Tests cover: URL validation (`isValidUrl`), input resolution (`resolveInput`), display formatting (`displayUrl`).

---

## Manual Test Checklist — Phase 1

Run through this after any significant changes. Launch with `npm run tauri dev`.

### Window & Chrome
- [ ] App launches with custom title bar (no native decorations)
- [ ] Window drag works from the title bar / tab bar area
- [ ] Double-click title bar toggles maximise/restore
- [ ] Minimise, maximise, close buttons all work
- [ ] Window resize correctly repositions UI and content webviews (no overlap, no gaps)

### Tabs
- [ ] New tab button (+) creates a tab
- [ ] Tab shows "New Tab" title initially
- [ ] Clicking a tab switches to it (content webview swaps)
- [ ] Tab shows page title after navigation
- [ ] Tab shows favicon after page loads
- [ ] Long titles truncate with ellipsis
- [ ] Close button (x) appears on hover and active tab
- [ ] Closing a tab switches to the adjacent tab
- [ ] Closing the last tab closes the window (or expected behaviour)
- [ ] Loading spinner shows while page is loading
- [ ] Drag-and-drop reordering works between tabs

### Tab Context Menu (right-click on tab)
- [ ] Right-click on tab opens popup context menu
- [ ] Menu appears near the click position, on top of everything
- [ ] "Duplicate Tab" creates a copy of the tab with same URL
- [ ] "Close Tab" closes the tab
- [ ] "Close Other Tabs" closes all except the right-clicked tab
- [ ] "Close Tabs to the Right" closes tabs to the right only
- [ ] Clicking outside the menu dismisses it
- [ ] Moving or resizing the main window dismisses the menu
- [ ] Menu has no visual artefacts (extra borders, wrong background)

### Navigation
- [ ] Typing a URL in address bar and pressing Enter navigates
- [ ] Typing a search query opens Google search
- [ ] Bare domain (e.g. `github.com`) gets `https://` prepended
- [ ] Back button works after navigating to multiple pages
- [ ] Forward button works after going back
- [ ] Back/Forward buttons disable correctly when at start/end of history
- [ ] Refresh button reloads the current page
- [ ] Stop button appears during loading and stops navigation
- [ ] Home button navigates to Google

### Address Bar
- [ ] Shows current URL of the active tab
- [ ] Updates when switching tabs
- [ ] Updates when navigating within a page (link clicks)
- [ ] Ctrl+L focuses the address bar and selects all text
- [ ] Padlock icon shows for HTTPS pages
- [ ] No padlock for HTTP pages

### Keyboard Shortcuts
- [ ] `Ctrl+T` — opens new tab
- [ ] `Ctrl+W` — closes current tab
- [ ] `Ctrl+Tab` — switches to next tab
- [ ] `Ctrl+Shift+Tab` — switches to previous tab
- [ ] `Ctrl+L` — focuses address bar
- [ ] `Ctrl+R` — refreshes page
- [ ] `F5` — refreshes page
- [ ] `Alt+Left` — navigates back
- [ ] `Alt+Right` — navigates forward
- [ ] `Ctrl+1` through `Ctrl+9` — switches to tab by index
- [ ] `Ctrl+F` — opens find in page
- [ ] `Ctrl+Shift+T` — opens new tab (reopen last closed — currently same behaviour)
- [ ] `Escape` — closes find bar (does NOT hijack Escape system-wide)
- [ ] Shortcuts work even when content webview has focus

### Status Bar
- [ ] Shows link URL when hovering over links in content webview
- [ ] Shows "Loading..." while page is loading
- [ ] Clears after page finishes loading

### Content Webview
- [ ] Web pages render correctly (try google.com, github.com, wikipedia.org)
- [ ] Links with `target="_blank"` open in a new tab (not system browser)
- [ ] Scrolling works in content area
- [ ] Form inputs work (typing, clicking buttons)
- [ ] JavaScript-heavy sites work (e.g. YouTube, Twitter)

### Error States
- [ ] Navigating to an invalid domain shows WebView2's built-in error page
- [ ] Navigating to a non-existent page (404) shows the site's error page

### Find in Page
- [ ] `Ctrl+F` opens find bar
- [ ] Typing highlights matches on the page
- [ ] Escape closes find bar
- [ ] Find bar doesn't interfere with other UI elements

---

## Manual Test Checklist — Phase 2

### Internal Pages (aero://)
- [ ] Typing `aero://settings` in address bar opens settings page
- [ ] Typing `aero://history` opens history page
- [ ] Typing `aero://bookmarks` opens bookmarks manager
- [ ] Address bar displays `aero://settings` (not `tauri://localhost/settings`)
- [ ] Internal pages have proper styling (dark theme, consistent with browser chrome)
- [ ] Back/forward navigation works between internal pages and regular sites

### Settings (aero://settings)
- [ ] Settings page loads and displays all sections (General, Search, Appearance, Privacy)
- [ ] Changing search engine persists after restart
- [ ] Changing homepage persists after restart
- [ ] Toggle "Show bookmarks bar" hides/shows the bookmark bar
- [ ] Toggle "Show status bar" hides/shows the status bar
- [ ] Settings saved in one tab reflect in other tabs / new tabs

### History
- [ ] Visiting a page records it in history
- [ ] `Ctrl+H` opens history page
- [ ] History page shows visited sites grouped by date
- [ ] Search bar filters history entries by URL and title
- [ ] Clicking a history entry navigates to that URL
- [ ] Delete button removes individual history entries
- [ ] "Clear browsing data" clears history for selected timeframe (all, last hour, last day, last week)
- [ ] `aero://` and `about:blank` pages are NOT recorded in history
- [ ] History persists after app restart
- [ ] Visiting the same URL updates visit count and last_visited timestamp

### Address Bar Autocomplete
- [ ] Typing in address bar shows dropdown with matching history entries
- [ ] Suggestions match by URL and title
- [ ] Clicking a suggestion navigates to that URL
- [ ] Pressing Enter with a suggestion selected navigates to it
- [ ] Dropdown disappears when address bar loses focus
- [ ] Dropdown disappears when pressing Escape

### Bookmarks
- [ ] `Ctrl+D` bookmarks the current page
- [ ] Star icon in address bar fills when page is bookmarked
- [ ] Clicking filled star removes the bookmark
- [ ] Bookmarks persist after app restart
- [ ] `aero://bookmarks` shows the full bookmark manager
- [ ] Can create folders in bookmark manager
- [ ] Can rename bookmarks and folders
- [ ] Can delete bookmarks and folders
- [ ] Can move bookmarks between folders
- [ ] Search bar filters bookmarks by title and URL
- [ ] Import bookmarks from HTML file works
- [ ] Export bookmarks to HTML file works

### Bookmarks Bar
- [ ] `Ctrl+Shift+B` toggles bookmarks bar visibility
- [ ] Bookmarks bar appears below the toolbar when visible
- [ ] Content webview repositions correctly when bar shows/hides (no overlap or gap)
- [ ] Clicking a bookmark in the bar navigates to that URL
- [ ] Folders in the bookmarks bar show a dropdown on click
- [ ] Right-click on bookmark bar items shows context menu (edit, delete)
- [ ] Adding a bookmark to "Bookmarks Bar" folder shows it in the bar

### Keyboard Shortcuts (Phase 2 additions)
- [ ] `Ctrl+H` — opens history
- [ ] `Ctrl+D` — bookmarks current page
- [ ] `Ctrl+Shift+B` — toggles bookmarks bar
