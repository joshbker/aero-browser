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

## Manual Test Checklist — Phase 2 (2.0–2.3)

### Known Issues
- **aero:// URL display**: Address bar may show `tauri://localhost/settings` instead of `aero://settings` — the `to_aero_url()` converter exists in Rust but may not apply to all tab URL updates
- **Chrome height on startup**: If bookmarks bar was hidden, first frame may briefly show wrong chrome height before settings load
- **Bookmark folder dropdowns**: Not yet implemented — folders show in bar but aren't clickable (TASKS.md 2.3 item unchecked)
- **Bookmark import/export UI**: Storage functions exist but no UI buttons wired up in manager page
- **Bookmark drag-and-drop**: `bookmark_move()` exists in storage but no drag UI in manager
- **Bookmark folder creation**: No "New Folder" button in manager UI — folders can only be created via storage API
- **Settings privacy section**: Placeholder only ("Coming in future update")

### Internal Pages (aero://)
- [ ] Typing `aero://settings` in address bar navigates to settings page
- [ ] Typing `aero://history` navigates to history page
- [ ] Typing `aero://bookmarks` navigates to bookmarks manager
- [ ] Internal pages render with dark theme styling
- [ ] Back/forward navigation works between internal pages and regular sites
- [ ] Internal pages are NOT recorded in history

### Settings (aero://settings)
- [ ] Settings page loads with sidebar (General, Search, Appearance, Privacy sections)
- [ ] General section: homepage, new tab page, restore on startup, download path, ask download location toggle
- [ ] Search section: search engine dropdown (Google, DuckDuckGo, Bing, Brave Search)
- [ ] Appearance section: theme selector, zoom slider (50–200%), show bookmarks bar toggle, show status bar toggle
- [ ] Privacy section: shows placeholder text
- [ ] Changing a setting saves immediately (no save button needed)
- [ ] Settings persist after app restart
- [ ] Toggle "Show bookmarks bar" hides/shows the bookmark bar and resizes content webview

### History
- [ ] Visiting an external page records it in history
- [ ] `Ctrl+H` opens history page in current tab
- [ ] History page shows entries with time, title, URL
- [ ] Entries grouped by date (Today, Yesterday, or full date)
- [ ] Search bar filters entries by URL and title in real-time
- [ ] Clicking a history entry navigates to that URL
- [ ] Delete button (trash icon) removes individual entries
- [ ] "Clear browsing data" button opens dialog with: Last hour, Last 24h, Last 7 days, All time
- [ ] Clearing data removes entries and updates the list
- [ ] `aero://`, `about:blank`, and `tauri://` pages are NOT recorded
- [ ] Pages with empty or "New Tab" titles are skipped
- [ ] Revisiting a URL increments visit_count and updates last_visited
- [ ] History persists after app restart

### Address Bar Autocomplete
- [ ] Typing in address bar shows dropdown of up to 6 matching history entries
- [ ] Suggestions appear after 150ms debounce
- [ ] Suggestions match by URL and title substring
- [ ] Arrow Up/Down navigates between suggestions
- [ ] Enter with suggestion selected navigates to that URL
- [ ] Clicking a suggestion navigates to that URL
- [ ] Dropdown disappears when address bar loses focus

### Bookmarks
- [ ] `Ctrl+D` toggles bookmark for current page (adds if not bookmarked, removes if bookmarked)
- [ ] Star icon appears in address bar only for http/https pages (not internal pages)
- [ ] Star is filled yellow when current page is bookmarked, outline grey when not
- [ ] New bookmarks default to "Bookmarks Bar" folder
- [ ] Bookmarks persist after app restart
- [ ] `aero://bookmarks` shows bookmark manager with tree view
- [ ] Root folders visible: "Bookmarks Bar" and "Other Bookmarks" with child counts
- [ ] Folders expand/collapse with chevron icons
- [ ] Nested subfolders expand/collapse (2 levels deep in UI)
- [ ] Edit button (pencil icon) on hover reveals inline title/URL editing
- [ ] Save/Cancel buttons appear during inline edit
- [ ] Delete button (trash icon) on hover removes bookmark
- [ ] Deleting a folder cascades to delete all children
- [ ] Search bar filters bookmarks by title and URL (flat results, not tree)
- [ ] Click on a bookmark in manager navigates to its URL

### Bookmarks Bar
- [ ] `Ctrl+Shift+B` toggles bookmarks bar visibility
- [ ] Bar appears below toolbar (28px height) when visible
- [ ] Content webview repositions correctly when bar shows/hides (dynamic CHROME_HEIGHT)
- [ ] Bar shows items from "Bookmarks Bar" folder with favicons
- [ ] Folders show folder icon, bookmarks show Google S2 favicon
- [ ] Clicking a bookmark navigates to its URL
- [ ] Long titles truncate with ellipsis
- [ ] Empty state shows "No bookmarks yet — press Ctrl+D to add one"
- [ ] Adding a bookmark (Ctrl+D) on a page updates the bar

### Keyboard Shortcuts (Phase 2 additions)
- [ ] `Ctrl+H` — opens history in current tab
- [ ] `Ctrl+D` — toggles bookmark for current page
- [ ] `Ctrl+Shift+B` — toggles bookmarks bar visibility
