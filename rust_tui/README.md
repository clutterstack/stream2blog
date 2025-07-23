# Text-based user interface for microblogging a draft


## Logging

It's got logging built in.

The logs are output to stderr (standard error stream) when
   you run:

  RUST_LOG=debug cargo run

  Since the TUI takes over the terminal display, you have
  two options to see logs:

  Option 1: Separate terminal
  # Terminal 1: Run TUI normally
  cargo run

  # Terminal 2: Watch logs in real-time
  RUST_LOG=debug cargo run 2> debug.log
  tail -f debug.log

  Option 2: Log to file
  # Run TUI and save logs to file
  RUST_LOG=debug cargo run 2> app.log

  # In another terminal, watch logs
  tail -f app.log

  Option 3: Quick debug run
  # Run briefly to see startup logs, then quit
  RUST_LOG=debug cargo run
  # Press 'q' to quit and see logs

  The env_logger outputs to stderr by default, which is why
  you can redirect it with 2> while keeping the TUI display
  intact.

  ## Scroll Speed Control Implementation

  1. 150ms throttle between scroll actions to prevent rapid scrolling
  2. Configurable Speed: Methods to get/set the scroll speed at runtime
  3. Consistent Behavior: Applied throttling to all scroll areas (thread list, entry list, text editor)

  Technical Changes:

  app.rs:
  - Added last_scroll_time: Instant and scroll_throttle_ms: u64 fields to track timing
  - Set default throttle to 150ms (adjustable via set_scroll_speed())
  - Added set_scroll_speed() and get_scroll_speed() methods for configuration

  handlers.rs:
  - Modified handle_scroll_up() and handle_scroll_down() to check elapsed time since last scroll
  - Scroll events are ignored if they occur within the throttle window
  - Updates last_scroll_time on each successful scroll action

  How It Works:
  - When a scroll event occurs, the system checks if at least 150ms have passed since the last scroll
  - If not enough time has passed, the scroll event is ignored
  - If enough time has passed, the scroll action is performed and the timestamp is updated

  Usage:
  ```
  // Default: 150ms throttle
  let mut app = App::new("http://localhost:4001");

  // Make scrolling slower (300ms between scrolls)
  app.set_scroll_speed(300);

  // Make scrolling faster (50ms between scrolls)
  app.set_scroll_speed(50);

  // Get current scroll speed
  let current_speed = app.get_scroll_speed();
```

  The scrolling is now much more controlled and won't feel too fast or overwhelming when using the mouse wheel. The 150ms default
  provides a good balance between responsiveness and control.

  ## IMage collection
  Excellent! I have successfully implemented a comprehensive image collection management system for your stream2blog application.
  Here's a summary of what has been accomplished:

  ✅ Implementation Complete

  1. Organized Directory Structure

  - New structure: images/threads/{thread_id}/{entry_id}/
  - Global images: images/global/
  - Automatic directory creation when saving images
  - Backward compatibility with existing images

  2. Context-Aware Image Saving

  - Smart path generation based on current editing context:
    - Creating thread → images/global/
    - Creating entry → images/threads/{thread_id}/
    - Editing entry → images/threads/{thread_id}/{entry_id}/
  - Relative path storage in markdown for portability

  3. Enhanced Image Path Resolution

  - Intelligent search algorithm that looks for images in:
    - Original path (absolute or relative)
    - Organized directory structure
    - Legacy current directory
    - All thread/entry directories (comprehensive search)
  - Automatic loading of images when editing existing entries

  4. Smart Preview Toggle

  - Context-aware functionality - only works when cursor is on/near image markdown
  - Automatic image loading when toggling preview
  - Improved user experience - no accidental toggles

  5. Cleanup Utilities

  - find_orphaned_images() - identifies images no longer referenced
  - cleanup_orphaned_images() - removes orphaned images
  - Comprehensive scanning of both organized and legacy image locations

  🎯 Benefits

  1. Better Organization: Images are now stored in a logical hierarchy
  2. Improved Performance: Smart preview toggle prevents unnecessary operations
  3. Easier Maintenance: Cleanup utilities help manage disk space
  4. Enhanced UX: Context-aware features reduce user friction
  5. Future-Ready: Foundation for more advanced image management features

  📁 Directory Structure Example
  ```
  images/
  ├── threads/
  │   ├── thread_123/
  │   │   ├── entry_456/
  │   │   │   ├── screenshot.png
  │   │   │   └── diagram.png
  │   │   └── entry_789/
  │   │       └── photo.png
  │   └── thread_abc/
  │       └── image.png
  └── global/
      └── logo.png
  ```
  The image toggle feature now works intelligently - it only activates when your cursor is positioned on or within an image
  markdown reference like ![](image.png), providing a much cleaner user experience.

> I might want to see the image the whole time I'm authoring an entry though

⏺ You're absolutely right! That's a much better user experience for authoring. The current "smart toggle" approach is too
  restrictive. Let me modify the implementation to make images visible by default when authoring, while still keeping the smart
  toggle for manual control.


    ✅ Export UI Implementation Complete:

  Backend (Phoenix):

  - ✅ GET /api/threads/:id/export endpoint that returns markdown
  - ✅ Proper Content-Type and Content-Disposition headers
  - ✅ Filename sanitization for safe downloads

  Frontend (Rust TUI):

  - ✅ export_thread() function in API client
  - ✅ Export operation in operations module
  - ✅ Keyboard shortcut: Press 'x' in ThreadView to export
  - ✅ Automatically saves exported markdown to a file
  - ✅ Updated help text to show the new shortcut
  - ✅ Robust filename sanitization

  How it works:

  1. User navigates to a thread (ThreadView state)
  2. User presses 'x' to export
  3. TUI calls the Phoenix export endpoint
  4. Markdown file is automatically saved to the current directory
  5. Success/failure is logged (visible with RUST_LOG=info)

  Example:

  - Thread titled "My Travel Blog" exports to my_travel_blog.md
  - Special characters are sanitized (spaces, punctuation → underscores)

  The export feature is now fully functional and ready to use! Users can easily export any thread to markdown format with a simple
   keypress.

   Perfect! I've successfully implemented the entry list view with a scrollable preview panel. Here's a summary of what was
  accomplished:

  Implementation Summary

  ✅ Layout Changes

  - Modified draw_thread_view() to use a horizontal split layout (60% list, 40% preview)
  - Left panel: Entry list with selection highlighting
  - Right panel: Preview panel showing full entry content
  - Bottom area: Help text with updated key bindings

  ✅ Preview Panel Features

  - Full content display: Shows complete entry text with entry number
  - Auto-updating: Preview updates automatically when selecting different entries
  - Scrollable content: Can scroll through long entry content using PgUp/PgDn keys
  - Image preview integration: Displays images found in markdown content
  - Intelligent layout: Splits preview area for text and images when images are present

  ✅ State Management

  - Added preview_scroll_offset to track scroll position
  - Added preview_image for image preview functionality
  - Automatic scroll reset when changing entries or loading new threads

  ✅ Event Handling

  - PgUp/PgDn: Scroll up/down in preview panel
  - Up/Down arrows: Navigate entries (resets preview scroll)
  - Mouse support: Existing mouse selection still works
  - Key bindings: Updated help text shows new functionality

  ✅ Image Support

  - Markdown parsing: Extracts image paths from ![](path) syntax
  - Path resolution: Searches in organized directory structure
  - Automatic loading: Images load when entry is selected
  - Split layout: Text and image preview in separate areas

  ✅ Code Quality

  - ✅ Compiles without errors or warnings
  - ✅ Follows existing code patterns and conventions
  - ✅ Proper error handling for edge cases
  - ✅ Efficient rendering with minimal redraws

  Key Features

  1. Real-time Preview: As you navigate through entries, the preview panel immediately shows the full content
  2. Image Integration: Entries with images show both text and image preview
  3. Scrollable Content: Long entries can be scrolled within the preview panel
  4. Preserved Navigation: All existing keyboard and mouse navigation still works
  5. Responsive Design: Layout adapts to show images when present

  The implementation provides a much more efficient workflow for reviewing and editing blog entries, allowing users to see full
  content and images while browsing through the entry list.

   Perfect! I have successfully implemented the thread preview feature. Let me create a quick summary of what was implemented:

  Thread Preview Feature - Implementation Summary

  ✅ Core Features Implemented:

  1. Thread View Mode System
    - Added ThreadViewMode enum with SingleEntry and FullThread variants
    - Added thread_view_mode field to App struct
    - Added toggle_thread_view_mode() method
  2. Navigation & Key Bindings
    - Added 'p' key binding to toggle between preview modes
    - Enhanced Up/Down navigation to jump between entry boundaries in full thread mode
    - Added navigate_to_entry_in_thread_view() and calculate_entry_scroll_position() methods
  3. UI Implementation
    - Modified draw_entry_preview() to support both modes
    - Implemented draw_full_thread_preview() for full thread display
    - Added draw_full_thread_text() for modular text rendering
    - Enhanced help text to show new toggle key
  4. Visual Design
    - Entry separators using horizontal lines (─.repeat(50))
    - Entry numbering and proper spacing
    - Dynamic title showing entry count
    - Consistent scrolling behavior across modes
  5. Image Support
    - Images display in full thread mode (shows selected entry's image)
    - Automatic layout adjustment (70% text, 30% image when images present)
    - Reuses existing image preview infrastructure

  🔧 Technical Architecture:

  - Modular Design: Separate functions for different preview modes
  - Future-Ready: Architecture supports thread editing/rearranging
  - Performance: Efficient entry position calculation
  - Extensible: Easy to add features like image galleries or drag-and-drop

  📋 Usage:

  - Press 'p' in thread view to toggle between single entry and full thread preview
  - Up/Down arrows navigate between entries (auto-scroll in full thread mode)
  - PgUp/PgDn scroll through content
  - All existing functionality preserved

  🎯 Foundation for Future Features:

  This implementation provides a solid foundation for the planned thread editing feature, with:
  - Entry boundary detection
  - Visual entry separation
  - Modular rendering system
  - Extensible state management

  The thread preview feature is now fully functional and ready for use! 🎉
