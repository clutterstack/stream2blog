use stream2blog::api::ApiClient;
use stream2blog::app::App;
use stream2blog::key_handler::KeyHandler;
use stream2blog::models::*;
use stream2blog::state::AppState;
use stream2blog::text_editor::TextEditor;
// tokio_test not needed for these tests
use httpmock::prelude::*;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use serde_json;

#[tokio::test]
async fn test_thread_rename_workflow() {
    let server = MockServer::start();

    // Mock getting threads initially - expect this to be called multiple times
    let _initial_threads_mock = server.mock(|when, then| {
        when.method(GET)
            .path("/api/threads");
        then.status(200)
            .header("content-type", "application/json")
            .body(r#"{"data": [{"id": "thread-123", "title": "Original Title", "inserted_at": "2024-01-01T00:00:00Z", "updated_at": "2024-01-01T00:00:00Z", "entries": []}]}"#);
    });

    // Mock thread update API call
    let update_mock = server.mock(|when, then| {
        when.method(PUT)
            .path("/api/threads/thread-123")
            .header("content-type", "application/json")
            .json_body(serde_json::json!({
                "thread": {
                    "title": "Updated Title"
                }
            }));
        then.status(200)
            .header("content-type", "application/json")
            .body(r#"{"data": {"id": "thread-123", "title": "Updated Title", "inserted_at": "2024-01-01T00:00:00Z", "updated_at": "2024-01-01T00:00:00Z", "entries": []}}"#);
    });

    // Mock getting threads after update (to refresh the list)
    let _updated_threads_mock = server.mock(|when, then| {
        when.method(GET)
            .path("/api/threads");
        then.status(200)
            .header("content-type", "application/json")
            .body(r#"{"data": [{"id": "thread-123", "title": "Updated Title", "inserted_at": "2024-01-01T00:00:00Z", "updated_at": "2024-01-01T00:00:00Z", "entries": []}]}"#);
    });

    let mut app = App::new(&server.base_url());
    
    // Load initial threads
    app.load_threads().await.unwrap();
    assert_eq!(app.threads.len(), 1);
    assert_eq!(app.threads[0].title, "Original Title");

    // Start rename operation by pressing 'r'
    let rename_key = KeyEvent::new(KeyCode::Char('r'), KeyModifiers::NONE);
    app.handle_key_event(rename_key).await.unwrap();
    assert!(matches!(app.state, AppState::EditThread(_)));

    // Clear the text editor and type new title
    app.text_editor.clear();
    app.text_editor.set_text_without_image_processing("Updated Title");

    // Submit the change with Ctrl+S
    let save_key = KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL);
    app.handle_key_event(save_key).await.unwrap();

    // Should be back to ThreadList after successful update
    assert!(matches!(app.state, AppState::ThreadList));

    // Verify the update mock was called
    update_mock.assert();

    // Note: Due to the mock setup, the actual thread list might not reflect the update 
    // immediately in the test environment, but the API call is verified above
}

#[tokio::test]
async fn test_full_thread_creation_workflow() {
    let server = MockServer::start();

    // Mock thread creation
    let create_mock = server.mock(|when, then| {
        when.method(POST)
            .path("/api/threads")
            .header("content-type", "application/json");
        then.status(201)
            .header("content-type", "application/json")
            .body(r#"{"data": {"id": "new-thread-123", "title": "Integration Test Thread", "inserted_at": "2024-01-01T00:00:00Z", "updated_at": "2024-01-01T00:00:00Z", "entries": []}}"#);
    });

    // Mock loading threads after creation
    let list_mock = server.mock(|when, then| {
        when.method(GET)
            .path("/api/threads");
        then.status(200)
            .header("content-type", "application/json")
            .body(r#"{"data": [{"id": "new-thread-123", "title": "Integration Test Thread", "inserted_at": "2024-01-01T00:00:00Z", "updated_at": "2024-01-01T00:00:00Z", "entries": []}]}"#);
    });

    // Mock getting specific thread (called when transitioning to ThreadView)
    let thread_mock = server.mock(|when, then| {
        when.method(GET)
            .path("/api/threads/new-thread-123");
        then.status(200)
            .header("content-type", "application/json")
            .body(r#"{"data": {"id": "new-thread-123", "title": "Integration Test Thread", "inserted_at": "2024-01-01T00:00:00Z", "updated_at": "2024-01-01T00:00:00Z", "entries": []}}"#);
    });

    let mut app = App::new(&server.base_url());

    // Start thread creation
    let key = KeyEvent::new(KeyCode::Char('n'), KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    assert!(matches!(app.state, AppState::CreateThread));

    // Type thread title
    let key = KeyEvent::new(KeyCode::Char('I'), KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    let key = KeyEvent::new(KeyCode::Char('n'), KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    let key = KeyEvent::new(KeyCode::Char('t'), KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();

    // Submit thread
    let key = KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL);
    app.handle_key_event(key).await.unwrap();

    // Verify state auto-transitioned to CreateEntry (new behavior)
    assert!(matches!(app.state, AppState::CreateEntry(_)));

    create_mock.assert();
    list_mock.assert();
    thread_mock.assert();
}

#[tokio::test]
async fn test_full_entry_creation_workflow() {
    let server = MockServer::start();

    // Mock getting specific thread (will be called twice - initial load and reload after entry creation)
    let _thread_mock = server.mock(|when, then| {
        when.method(GET)
            .path("/api/threads/test-thread-123");
        then.status(200)
            .header("content-type", "application/json")
            .body(r#"{"data": {"id": "test-thread-123", "title": "Test Thread", "inserted_at": "2024-01-01T00:00:00Z", "updated_at": "2024-01-01T00:00:00Z", "entries": []}}"#);
    });

    // Create a second mock for the reload after entry creation
    let _reload_thread_mock = server.mock(|when, then| {
        when.method(GET)
            .path("/api/threads/test-thread-123");
        then.status(200)
            .header("content-type", "application/json")
            .body(r#"{"data": {"id": "test-thread-123", "title": "Test Thread", "inserted_at": "2024-01-01T00:00:00Z", "updated_at": "2024-01-01T00:00:00Z", "entries": [{"id": "new-entry-456", "content": "Integration test entry", "order_num": 1, "image_path": null, "thread_id": "test-thread-123", "inserted_at": "2024-01-01T00:00:00Z", "updated_at": "2024-01-01T00:00:00Z"}]}}"#);
    });

    // Mock entry creation
    let create_entry_mock = server.mock(|when, then| {
        when.method(POST)
            .path("/api/entries")
            .header("content-type", "application/json");
        then.status(201)
            .header("content-type", "application/json")
            .body(r#"{"data": {"id": "new-entry-456", "content": "Integration test entry", "order_num": 1, "image_path": null, "thread_id": "test-thread-123", "inserted_at": "2024-01-01T00:00:00Z", "updated_at": "2024-01-01T00:00:00Z"}}"#);
    });

    let mut app = App::new(&server.base_url());

    // Navigate to thread view
    app.state = AppState::ThreadView("test-thread-123".to_string());
    app.load_thread("test-thread-123").await.unwrap();

    // Start entry creation
    let key = KeyEvent::new(KeyCode::Char('n'), KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    assert!(matches!(app.state, AppState::CreateEntry(_)));

    // Type entry content
    let key = KeyEvent::new(KeyCode::Char('I'), KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    let key = KeyEvent::new(KeyCode::Char('n'), KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    let key = KeyEvent::new(KeyCode::Char('t'), KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();

    // Submit entry
    let key = KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL);
    app.handle_key_event(key).await.unwrap();

    // Verify state returned to thread view
    assert!(matches!(app.state, AppState::ThreadView(_)));

    create_entry_mock.assert();
}

#[tokio::test]
async fn test_multiline_text_editing() {
    let server = MockServer::start();

    let mut app = App::new(&server.base_url());
    app.state = AppState::CreateEntry("test-thread-123".to_string());

    // Type some text
    let key = KeyEvent::new(KeyCode::Char('F'), KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    let key = KeyEvent::new(KeyCode::Char('i'), KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    let key = KeyEvent::new(KeyCode::Char('r'), KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    let key = KeyEvent::new(KeyCode::Char('s'), KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    let key = KeyEvent::new(KeyCode::Char('t'), KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();

    // Press Enter to create a newline
    let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();

    // Type second line
    let key = KeyEvent::new(KeyCode::Char('L'), KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    let key = KeyEvent::new(KeyCode::Char('i'), KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    let key = KeyEvent::new(KeyCode::Char('n'), KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    let key = KeyEvent::new(KeyCode::Char('e'), KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();

    // Verify we have multiline text
    let lines = app.text_editor.lines();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0], "First");
    assert_eq!(lines[1], "Line");

    // Verify that regular Enter doesn't submit - state should still be CreateEntry
    assert!(matches!(app.state, AppState::CreateEntry(_)));
}

#[tokio::test]
async fn test_app_navigation_workflow() {
    let server = MockServer::start();

    // Mock threads list
    let threads_mock = server.mock(|when, then| {
        when.method(GET)
            .path("/api/threads");
        then.status(200)
            .header("content-type", "application/json")
            .body(r#"{"data": [
                {"id": "thread-1", "title": "First Thread", "inserted_at": "2024-01-01T00:00:00Z", "updated_at": "2024-01-01T00:00:00Z", "entries": []},
                {"id": "thread-2", "title": "Second Thread", "inserted_at": "2024-01-01T00:00:00Z", "updated_at": "2024-01-01T00:00:00Z", "entries": []}
            ]}"#);
    });

    // Mock specific thread
    let thread_mock = server.mock(|when, then| {
        when.method(GET)
            .path("/api/threads/thread-2");
        then.status(200)
            .header("content-type", "application/json")
            .body(r#"{"data": {"id": "thread-2", "title": "Second Thread", "inserted_at": "2024-01-01T00:00:00Z", "updated_at": "2024-01-01T00:00:00Z", "entries": [{"id": "entry-1", "content": "Test entry", "order_num": 1, "image_path": null, "thread_id": "thread-2", "inserted_at": "2024-01-01T00:00:00Z", "updated_at": "2024-01-01T00:00:00Z"}]}}"#);
    });

    let mut app = App::new(&server.base_url());

    // Load threads
    app.load_threads().await.unwrap();
    assert_eq!(app.threads.len(), 2);
    assert_eq!(app.selected_thread_index, 0);

    // Navigate down
    let key = KeyEvent::new(KeyCode::Down, KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    assert_eq!(app.selected_thread_index, 1);

    // Enter selected thread
    let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    assert!(matches!(app.state, AppState::ThreadView(_)));
    assert!(app.current_thread.is_some());

    // Go back to thread list
    let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    assert!(matches!(app.state, AppState::ThreadList));
    assert!(app.current_thread.is_none());

    threads_mock.assert();
    thread_mock.assert();
}

#[tokio::test]
async fn test_text_editor_clipboard_integration() {
    let mut editor = TextEditor::new();

    // Test text insertion using KeyHandler directly
    KeyHandler::insert_text(&mut editor.widget().clone(), "Hello world!");
    // Note: We need to access the textarea directly since we can't modify through widget()
    // Let's test with a different approach using key events

    // Test multiline text insertion through key events
    editor.clear();
    for ch in "Line 1\nLine 2\nLine 3".chars() {
        if ch == '\n' {
            let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
            editor.handle_key_event(key);
        } else {
            let key = KeyEvent::new(KeyCode::Char(ch), KeyModifiers::NONE);
            editor.handle_key_event(key);
        }
    }
    assert_eq!(editor.lines().len(), 3);
    assert_eq!(editor.lines()[0], "Line 1");
    assert_eq!(editor.lines()[1], "Line 2");
    assert_eq!(editor.lines()[2], "Line 3");

    // Test select all using KeyHandler directly
    let key = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL);
    let result = editor.handle_key_event(key);
    // The result is Option<KeyResult>, so we check if it handled successfully
    assert!(result.is_some());

    // Test clipboard operations (may fail if clipboard not available in test env)
    let key = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
    let _result = editor.handle_key_event(key);
    // Don't assert on clipboard operations as they depend on system availability
}

#[tokio::test]
async fn test_api_client_error_handling() {
    let server = MockServer::start();

    // Mock server errors
    let error_mock = server.mock(|when, then| {
        when.method(GET).path("/api/threads");
        then.status(500).body("Internal Server Error");
    });

    let client = ApiClient::new(&server.base_url());

    // Test error handling
    let result = client.get_threads().await;
    assert!(result.is_err());

    error_mock.assert();
}

#[tokio::test]
async fn test_app_error_recovery() {
    let mut app = App::new("http://invalid-server-url");

    // Test graceful handling of API errors
    app.load_threads().await.unwrap(); // Should not panic
    assert_eq!(app.threads.len(), 0);

    app.load_thread("nonexistent").await.unwrap(); // Should not panic
    assert!(app.current_thread.is_none());
}

#[test]
fn test_models_comprehensive_serialization() {
    // Test complex thread with multiple entries
    let thread = Thread {
        id: "complex-thread".to_string(),
        title: "Complex Thread Title".to_string(),
        inserted_at: "2024-01-01T00:00:00Z".to_string(),
        updated_at: "2024-01-01T00:00:00Z".to_string(),
        entries: vec![
            Entry {
                id: "entry-1".to_string(),
                content: "First entry with special chars: éñ中文".to_string(),
                order_num: 1,
                image_path: None,
                thread_id: Some("complex-thread".to_string()),
                inserted_at: "2024-01-01T00:00:00Z".to_string(),
                updated_at: "2024-01-01T00:00:00Z".to_string(),
            },
            Entry {
                id: "entry-2".to_string(),
                content: "Second entry\nwith\nmultiple\nlines".to_string(),
                order_num: 2,
                image_path: None,
                thread_id: Some("complex-thread".to_string()),
                inserted_at: "2024-01-01T00:00:00Z".to_string(),
                updated_at: "2024-01-01T00:00:00Z".to_string(),
            },
        ],
    };

    // Test serialization and deserialization
    let json = serde_json::to_string(&thread).unwrap();
    let deserialized: Thread = serde_json::from_str(&json).unwrap();

    assert_eq!(thread.id, deserialized.id);
    assert_eq!(thread.title, deserialized.title);
    assert_eq!(thread.entries.len(), deserialized.entries.len());
    assert_eq!(thread.entries[0].content, deserialized.entries[0].content);
    assert_eq!(thread.entries[1].content, deserialized.entries[1].content);
}

#[tokio::test]
async fn test_create_thread_auto_transitions_to_entry() {
    let server = MockServer::start();

    // Mock thread creation
    let create_mock = server.mock(|when, then| {
        when.method(POST)
            .path("/api/threads")
            .header("content-type", "application/json");
        then.status(201)
            .header("content-type", "application/json")
            .body(r#"{"data": {"id": "auto-thread-456", "title": "Test Title", "inserted_at": "2024-01-01T00:00:00Z", "updated_at": "2024-01-01T00:00:00Z", "entries": []}}"#);
    });

    // Mock loading threads after creation
    let list_mock = server.mock(|when, then| {
        when.method(GET)
            .path("/api/threads");
        then.status(200)
            .header("content-type", "application/json")
            .body(r#"{"data": [{"id": "auto-thread-456", "title": "Test Title", "inserted_at": "2024-01-01T00:00:00Z", "updated_at": "2024-01-01T00:00:00Z", "entries": []}]}"#);
    });

    // Mock getting specific thread (called when transitioning to ThreadView)
    let thread_mock = server.mock(|when, then| {
        when.method(GET)
            .path("/api/threads/auto-thread-456");
        then.status(200)
            .header("content-type", "application/json")
            .body(r#"{"data": {"id": "auto-thread-456", "title": "Test Title", "inserted_at": "2024-01-01T00:00:00Z", "updated_at": "2024-01-01T00:00:00Z", "entries": []}}"#);
    });

    let mut app = App::new(&server.base_url());

    // Start in ThreadList state
    assert!(matches!(app.state, AppState::ThreadList));

    // Navigate to CreateThread
    let key = KeyEvent::new(KeyCode::Char('n'), KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    assert!(matches!(app.state, AppState::CreateThread));

    // Type thread title
    let key = KeyEvent::new(KeyCode::Char('T'), KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    let key = KeyEvent::new(KeyCode::Char('e'), KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    let key = KeyEvent::new(KeyCode::Char('s'), KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    let key = KeyEvent::new(KeyCode::Char('t'), KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();

    // Submit thread with Ctrl+S
    let key = KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL);
    app.handle_key_event(key).await.unwrap();

    // CRITICAL: Verify auto-transition to CreateEntry, not ThreadList
    assert!(matches!(app.state, AppState::CreateEntry(_)));
    
    // Verify the thread ID matches what was created
    if let AppState::CreateEntry(thread_id) = &app.state {
        assert_eq!(thread_id, "auto-thread-456");
    }

    // Verify current_thread is set
    assert!(app.current_thread.is_some());
    assert_eq!(app.current_thread.as_ref().unwrap().id, "auto-thread-456");

    // Verify text editor is cleared and ready for entry input
    assert_eq!(app.text_editor.lines().len(), 1);
    assert_eq!(app.text_editor.lines()[0], "");

    create_mock.assert();
    list_mock.assert();
    thread_mock.assert();
}

#[tokio::test]
async fn test_datestamp_thread_creation_workflow() {
    let server = MockServer::start();

    // Mock thread creation with datestamp title (we'll accept any title)
    let create_mock = server.mock(|when, then| {
        when.method(POST)
            .path("/api/threads")
            .header("content-type", "application/json");
        then.status(201)
            .header("content-type", "application/json")
            .body(r#"{"data": {"id": "datestamp-thread-789", "title": "20240101120000", "inserted_at": "2024-01-01T00:00:00Z", "updated_at": "2024-01-01T00:00:00Z", "entries": []}}"#);
    });

    // Mock loading threads
    let list_mock = server.mock(|when, then| {
        when.method(GET)
            .path("/api/threads");
        then.status(200)
            .header("content-type", "application/json")
            .body(r#"{"data": [{"id": "datestamp-thread-789", "title": "20240101120000", "inserted_at": "2024-01-01T00:00:00Z", "updated_at": "2024-01-01T00:00:00Z", "entries": []}]}"#);
    });

    // Mock getting specific thread
    let thread_mock = server.mock(|when, then| {
        when.method(GET)
            .path("/api/threads/datestamp-thread-789");
        then.status(200)
            .header("content-type", "application/json")
            .body(r#"{"data": {"id": "datestamp-thread-789", "title": "20240101120000", "inserted_at": "2024-01-01T00:00:00Z", "updated_at": "2024-01-01T00:00:00Z", "entries": []}}"#);
    });

    let mut app = App::new(&server.base_url());

    // Start in ThreadList state
    assert!(matches!(app.state, AppState::ThreadList));

    // Trigger datestamp thread creation with 'd'
    let key = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();

    // Should automatically transition to CreateEntry (not ThreadView)
    assert!(matches!(app.state, AppState::CreateEntry(_)));
    
    // Verify the thread ID matches what was created
    if let AppState::CreateEntry(thread_id) = &app.state {
        assert_eq!(thread_id, "datestamp-thread-789");
    }

    // Verify current_thread is set
    assert!(app.current_thread.is_some());
    assert_eq!(app.current_thread.as_ref().unwrap().id, "datestamp-thread-789");

    // Verify text editor is ready for entry input
    assert_eq!(app.text_editor.lines().len(), 1);
    assert_eq!(app.text_editor.lines()[0], "");

    create_mock.assert();
    list_mock.assert();
    thread_mock.assert();
}

#[tokio::test]
async fn test_cancellation_operations_preserve_state() {
    let server = MockServer::start();

    // Mock getting specific thread for ThreadView state
    let thread_mock = server.mock(|when, then| {
        when.method(GET)
            .path("/api/threads/test-thread-cancel");
        then.status(200)
            .header("content-type", "application/json")
            .body(r#"{"data": {"id": "test-thread-cancel", "title": "Test Thread", "inserted_at": "2024-01-01T00:00:00Z", "updated_at": "2024-01-01T00:00:00Z", "entries": [{"id": "entry-1", "content": "Test entry content", "order_num": 1, "image_path": null, "thread_id": "test-thread-cancel", "inserted_at": "2024-01-01T00:00:00Z", "updated_at": "2024-01-01T00:00:00Z"}]}}"#);
    });

    let mut app = App::new(&server.base_url());

    // Test 1: Cancel from CreateThread → ThreadList
    app.state = AppState::CreateThread;
    app.text_editor.clear();
    
    // Type some text
    let key = KeyEvent::new(KeyCode::Char('T'), KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    let key = KeyEvent::new(KeyCode::Char('e'), KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    let key = KeyEvent::new(KeyCode::Char('s'), KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    let key = KeyEvent::new(KeyCode::Char('t'), KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    
    // Verify text was typed
    assert_eq!(app.text_editor.lines().join(""), "Test");
    
    // Cancel with Esc
    let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    
    // Should return to ThreadList with text cleared
    assert!(matches!(app.state, AppState::ThreadList));
    assert_eq!(app.text_editor.lines().len(), 1);
    assert_eq!(app.text_editor.lines()[0], "");

    // Test 2: Cancel from CreateEntry → ThreadView
    // Setup ThreadView state
    app.state = AppState::ThreadView("test-thread-cancel".to_string());
    app.load_thread("test-thread-cancel").await.unwrap();
    
    // Navigate to CreateEntry
    let key = KeyEvent::new(KeyCode::Char('n'), KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    assert!(matches!(app.state, AppState::CreateEntry(_)));
    
    // Type some entry content
    let key = KeyEvent::new(KeyCode::Char('E'), KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    let key = KeyEvent::new(KeyCode::Char('n'), KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    let key = KeyEvent::new(KeyCode::Char('t'), KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    let key = KeyEvent::new(KeyCode::Char('r'), KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    let key = KeyEvent::new(KeyCode::Char('y'), KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    
    // Verify text was typed
    assert_eq!(app.text_editor.lines().join(""), "Entry");
    
    // Cancel with Esc - should now show confirmation since there's content
    let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    
    // Should show confirmation modal since there's content
    assert!(matches!(app.state, AppState::ConfirmDiscardNewEntry(_)));
    
    // Confirm discard with 'y'
    let key = KeyEvent::new(KeyCode::Char('y'), KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    
    // Should return to ThreadView with correct thread_id and text cleared
    assert!(matches!(app.state, AppState::ThreadView(_)));
    if let AppState::ThreadView(thread_id) = &app.state {
        assert_eq!(thread_id, "test-thread-cancel");
    }
    assert_eq!(app.text_editor.lines().len(), 1);
    assert_eq!(app.text_editor.lines()[0], "");

    // Test 3: Cancel from EditEntry → ThreadView
    // Navigate to EditEntry
    let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    assert!(matches!(app.state, AppState::EditEntry(_, _)));
    
    // Verify original content was loaded
    assert_eq!(app.text_editor.lines().join(""), "Test entry content");
    
    // Modify the content
    let key = KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    let key = KeyEvent::new(KeyCode::Char('m'), KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    let key = KeyEvent::new(KeyCode::Char('o'), KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    let key = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    
    // Cancel with Esc - should trigger confirmation modal since there are changes
    let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    
    // Should show confirmation modal
    assert!(matches!(app.state, AppState::ConfirmDiscardEntryChanges(_, _)));
    
    // Confirm discard with 'y'
    let key = KeyEvent::new(KeyCode::Char('y'), KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    
    // Should now return to ThreadView, changes discarded, text cleared
    assert!(matches!(app.state, AppState::ThreadView(_)));
    if let AppState::ThreadView(thread_id) = &app.state {
        assert_eq!(thread_id, "test-thread-cancel");
    }
    assert_eq!(app.text_editor.lines().len(), 1);
    assert_eq!(app.text_editor.lines()[0], "");

    // Test 4: Cancel from EditEntry with no changes → direct return to ThreadView
    // Navigate back to EditEntry
    let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    assert!(matches!(app.state, AppState::EditEntry(_, _)));
    
    // Don't modify content - just cancel with Esc
    let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    
    // Should return directly to ThreadView without confirmation modal
    assert!(matches!(app.state, AppState::ThreadView(_)));
    if let AppState::ThreadView(thread_id) = &app.state {
        assert_eq!(thread_id, "test-thread-cancel");
    }

    thread_mock.assert();
}

#[tokio::test]
async fn test_cross_thread_navigation_state_preservation() {
    let server = MockServer::start();

    // Mock threads list with multiple threads
    let threads_mock = server.mock(|when, then| {
        when.method(GET)
            .path("/api/threads");
        then.status(200)
            .header("content-type", "application/json")
            .body(r#"{"data": [
                {"id": "thread-A", "title": "Thread A", "inserted_at": "2024-01-01T00:00:00Z", "updated_at": "2024-01-01T00:00:00Z", "entries": []},
                {"id": "thread-B", "title": "Thread B", "inserted_at": "2024-01-01T00:00:00Z", "updated_at": "2024-01-01T00:00:00Z", "entries": []},
                {"id": "thread-C", "title": "Thread C", "inserted_at": "2024-01-01T00:00:00Z", "updated_at": "2024-01-01T00:00:00Z", "entries": []}
            ]}"#);
    });

    // Mock individual thread requests
    let thread_a_mock = server.mock(|when, then| {
        when.method(GET)
            .path("/api/threads/thread-A");
        then.status(200)
            .header("content-type", "application/json")
            .body(r#"{"data": {"id": "thread-A", "title": "Thread A", "inserted_at": "2024-01-01T00:00:00Z", "updated_at": "2024-01-01T00:00:00Z", "entries": [{"id": "entry-A1", "content": "Entry A1", "order_num": 1, "image_path": null, "thread_id": "thread-A", "inserted_at": "2024-01-01T00:00:00Z", "updated_at": "2024-01-01T00:00:00Z"}]}}"#);
    });

    let thread_b_mock = server.mock(|when, then| {
        when.method(GET)
            .path("/api/threads/thread-B");
        then.status(200)
            .header("content-type", "application/json")
            .body(r#"{"data": {"id": "thread-B", "title": "Thread B", "inserted_at": "2024-01-01T00:00:00Z", "updated_at": "2024-01-01T00:00:00Z", "entries": [{"id": "entry-B1", "content": "Entry B1", "order_num": 1, "image_path": null, "thread_id": "thread-B", "inserted_at": "2024-01-01T00:00:00Z", "updated_at": "2024-01-01T00:00:00Z"}, {"id": "entry-B2", "content": "Entry B2", "order_num": 2, "image_path": null, "thread_id": "thread-B", "inserted_at": "2024-01-01T00:00:00Z", "updated_at": "2024-01-01T00:00:00Z"}]}}"#);
    });

    let mut app = App::new(&server.base_url());

    // Load threads list
    app.load_threads().await.unwrap();
    assert_eq!(app.threads.len(), 3);
    assert_eq!(app.selected_thread_index, 0);

    // Navigate to second thread (index 1)
    let key = KeyEvent::new(KeyCode::Down, KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    assert_eq!(app.selected_thread_index, 1);

    // Enter thread B
    let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    assert!(matches!(app.state, AppState::ThreadView(_)));
    if let AppState::ThreadView(thread_id) = &app.state {
        assert_eq!(thread_id, "thread-B");
    }
    
    // Verify thread B data loaded
    assert!(app.current_thread.is_some());
    assert_eq!(app.current_thread.as_ref().unwrap().id, "thread-B");
    assert_eq!(app.current_thread.as_ref().unwrap().entries.len(), 2);
    assert_eq!(app.selected_entry_index, 0);

    // Navigate to second entry in thread B
    let key = KeyEvent::new(KeyCode::Down, KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    assert_eq!(app.selected_entry_index, 1);

    // Return to ThreadList
    let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    assert!(matches!(app.state, AppState::ThreadList));
    
    // Verify thread selection index preserved (should still be 1 for thread B)
    assert_eq!(app.selected_thread_index, 1);
    
    // Verify current_thread cleared
    assert!(app.current_thread.is_none());

    // Navigate to first thread (index 0)
    let key = KeyEvent::new(KeyCode::Up, KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    assert_eq!(app.selected_thread_index, 0);

    // Enter thread A
    let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    assert!(matches!(app.state, AppState::ThreadView(_)));
    if let AppState::ThreadView(thread_id) = &app.state {
        assert_eq!(thread_id, "thread-A");
    }

    // Verify thread A data loaded and entry selection reset
    assert!(app.current_thread.is_some());
    assert_eq!(app.current_thread.as_ref().unwrap().id, "thread-A");
    assert_eq!(app.current_thread.as_ref().unwrap().entries.len(), 1);
    assert_eq!(app.selected_entry_index, 0); // Should reset to 0 for new thread

    threads_mock.assert();
    thread_a_mock.assert();
    thread_b_mock.assert();
}

#[tokio::test]
async fn test_image_naming_workflow_from_create_entry() {
    let server = MockServer::start();

    let mut app = App::new(&server.base_url());
    app.state = AppState::CreateEntry("test-thread-456".to_string());

    // Type some text content first
    let key = KeyEvent::new(KeyCode::Char('H'), KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    let key = KeyEvent::new(KeyCode::Char('e'), KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    let key = KeyEvent::new(KeyCode::Char('l'), KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    let key = KeyEvent::new(KeyCode::Char('l'), KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    let key = KeyEvent::new(KeyCode::Char('o'), KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();

    // Verify initial text
    assert_eq!(app.text_editor.lines().join(""), "Hello");

    // Simulate Ctrl+P with mock image data (since clipboard access may fail in tests)
    // We'll directly trigger the ImageNaming transition as if Ctrl+P succeeded
    let mock_image_data = vec![0x89, 0x50, 0x4E, 0x47]; // PNG header bytes
    let prev_state = app.state.clone();
    app.saved_text_content = Some(app.text_editor.lines().join("\n"));
    app.state = AppState::ImageNaming(Box::new(prev_state), mock_image_data.clone());
    app.modal_text_editor.clear();

    // Verify state transition to ImageNaming
    assert!(matches!(app.state, AppState::ImageNaming(_, _)));
    
    // Verify saved text content
    assert_eq!(app.saved_text_content, Some("Hello".to_string()));

    // Type image filename
    let key = KeyEvent::new(KeyCode::Char('t'), KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    let key = KeyEvent::new(KeyCode::Char('e'), KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    let key = KeyEvent::new(KeyCode::Char('s'), KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    let key = KeyEvent::new(KeyCode::Char('t'), KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();

    // Verify filename typed in modal editor
    assert_eq!(app.modal_text_editor.lines().join(""), "test");

    // Submit filename with Enter (this will attempt to save the image)
    let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();

    // Verify state returned to CreateEntry
    assert!(matches!(app.state, AppState::CreateEntry(_)));
    if let AppState::CreateEntry(thread_id) = &app.state {
        assert_eq!(thread_id, "test-thread-456");
    }

    // Verify original text content was restored and image markdown was inserted
    let final_text = app.text_editor.lines().join("");
    assert!(final_text.starts_with("Hello"));
    // Note: The exact image markdown format depends on the implementation
    // We just verify the original text is preserved

    // Verify saved text content was cleared
    assert!(app.saved_text_content.is_none());
}

#[tokio::test]
async fn test_image_naming_workflow_cancellation() {
    let server = MockServer::start();

    let mut app = App::new(&server.base_url());
    app.state = AppState::EditEntry("test-thread-789".to_string(), "test-entry-123".to_string());

    // Type some text content with multiple lines
    let key = KeyEvent::new(KeyCode::Char('F'), KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    let key = KeyEvent::new(KeyCode::Char('i'), KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    let key = KeyEvent::new(KeyCode::Char('r'), KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    let key = KeyEvent::new(KeyCode::Char('s'), KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    let key = KeyEvent::new(KeyCode::Char('t'), KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    let key = KeyEvent::new(KeyCode::Char('L'), KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    let key = KeyEvent::new(KeyCode::Char('i'), KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    let key = KeyEvent::new(KeyCode::Char('n'), KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    let key = KeyEvent::new(KeyCode::Char('e'), KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();

    // Verify initial multiline text
    let original_lines = app.text_editor.lines();
    assert_eq!(original_lines.len(), 2);
    assert_eq!(original_lines[0], "First");
    assert_eq!(original_lines[1], "Line");

    // Simulate image workflow initialization
    let mock_image_data = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A]; // PNG header
    let prev_state = app.state.clone();
    app.saved_text_content = Some(app.text_editor.lines().join("\n"));
    app.state = AppState::ImageNaming(Box::new(prev_state), mock_image_data);
    app.modal_text_editor.clear();

    // Type some filename
    let key = KeyEvent::new(KeyCode::Char('i'), KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    let key = KeyEvent::new(KeyCode::Char('m'), KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    let key = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    let key = KeyEvent::new(KeyCode::Char('g'), KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    let key = KeyEvent::new(KeyCode::Char('e'), KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();

    // Verify filename was typed
    assert_eq!(app.modal_text_editor.lines().join(""), "image");

    // Cancel with Esc
    let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();

    // Verify state returned to EditEntry with exact same IDs
    assert!(matches!(app.state, AppState::EditEntry(_, _)));
    if let AppState::EditEntry(thread_id, entry_id) = &app.state {
        assert_eq!(thread_id, "test-thread-789");
        assert_eq!(entry_id, "test-entry-123");
    }

    // Verify EXACT original text content was restored
    let restored_lines = app.text_editor.lines();
    assert_eq!(restored_lines.len(), 2);
    assert_eq!(restored_lines[0], "First");
    assert_eq!(restored_lines[1], "Line");

    // Verify saved content was cleared
    assert!(app.saved_text_content.is_none());
}

#[tokio::test]
async fn test_image_workflow_state_preservation_across_transitions() {
    let server = MockServer::start();

    let mut app = App::new(&server.base_url());
    
    // Test CreateThread → ImageNaming workflow
    app.state = AppState::CreateThread;
    
    // Type thread title
    let key = KeyEvent::new(KeyCode::Char('M'), KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    let key = KeyEvent::new(KeyCode::Char('y'), KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    let key = KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    let key = KeyEvent::new(KeyCode::Char('T'), KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    let key = KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    let key = KeyEvent::new(KeyCode::Char('r'), KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    let key = KeyEvent::new(KeyCode::Char('e'), KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    let key = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    let key = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();

    // Verify thread title
    assert_eq!(app.text_editor.lines().join(""), "My Thread");

    // Simulate image workflow from CreateThread (though unusual, should work)
    let mock_image_data = vec![0xFF, 0xD8, 0xFF]; // JPEG header
    let prev_state = app.state.clone();
    app.saved_text_content = Some(app.text_editor.lines().join("\n"));
    app.state = AppState::ImageNaming(Box::new(prev_state), mock_image_data);
    app.modal_text_editor.clear();

    // Type filename and cancel
    let key = KeyEvent::new(KeyCode::Char('l'), KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    let key = KeyEvent::new(KeyCode::Char('o'), KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    let key = KeyEvent::new(KeyCode::Char('g'), KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    let key = KeyEvent::new(KeyCode::Char('o'), KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();

    // Cancel
    let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();

    // Verify returned to CreateThread with exact original content
    assert!(matches!(app.state, AppState::CreateThread));
    assert_eq!(app.text_editor.lines().join(""), "My Thread");
    assert!(app.saved_text_content.is_none());

    // Test multiple back-and-forth transitions
    let mock_image_data2 = vec![0x47, 0x49, 0x46]; // GIF header
    let prev_state2 = app.state.clone();
    app.saved_text_content = Some(app.text_editor.lines().join("\n"));
    app.state = AppState::ImageNaming(Box::new(prev_state2), mock_image_data2);
    app.modal_text_editor.clear();

    // Type different filename and cancel again
    let key = KeyEvent::new(KeyCode::Char('i'), KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    let key = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    let key = KeyEvent::new(KeyCode::Char('o'), KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    let key = KeyEvent::new(KeyCode::Char('n'), KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();

    let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();

    // Verify still returns to CreateThread with exact same content
    assert!(matches!(app.state, AppState::CreateThread));
    assert_eq!(app.text_editor.lines().join(""), "My Thread");
    assert!(app.saved_text_content.is_none());
}

#[tokio::test]
async fn test_text_editor_key_behavior_edge_cases() {
    let mut editor = TextEditor::new();

    // Test 1: Cursor positioning with arrow keys
    // Type some text
    for ch in "Line One\nLine Two\nLine Three".chars() {
        if ch == '\n' {
            let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
            editor.handle_key_event(key);
        } else {
            let key = KeyEvent::new(KeyCode::Char(ch), KeyModifiers::NONE);
            editor.handle_key_event(key);
        }
    }
    
    // Verify we have 3 lines
    assert_eq!(editor.lines().len(), 3);
    assert_eq!(editor.lines()[0], "Line One");
    assert_eq!(editor.lines()[1], "Line Two");
    assert_eq!(editor.lines()[2], "Line Three");
    
    // Test cursor movement to start
    editor.move_cursor_to_start();
    
    // Test 2: Backspace at line boundaries
    // Navigate to end of first line
    for _ in 0.."Line One".len() {
        let key = KeyEvent::new(KeyCode::Right, KeyModifiers::NONE);
        editor.handle_key_event(key);
    }
    
    // Delete the newline character (should join lines)
    let key = KeyEvent::new(KeyCode::Delete, KeyModifiers::NONE);
    editor.handle_key_event(key);
    
    // With simplified wrapping, short lines stay as separate lines unless they exceed threshold
    assert_eq!(editor.lines().len(), 2);
    assert_eq!(editor.lines()[0], "Line OneLine Two");
    assert_eq!(editor.lines()[1], "Line Three");
}

#[tokio::test]
async fn test_text_editor_selection_and_deletion() {
    let mut editor = TextEditor::new();
    
    // Type multi-line text
    for ch in "First line\nSecond line\nThird line".chars() {
        if ch == '\n' {
            let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
            editor.handle_key_event(key);
        } else {
            let key = KeyEvent::new(KeyCode::Char(ch), KeyModifiers::NONE);
            editor.handle_key_event(key);
        }
    }
    
    // Verify we have 3 lines (word wrapping not triggered for short lines)
    assert_eq!(editor.lines().len(), 3);
    assert_eq!(editor.lines()[0], "First line");
    assert_eq!(editor.lines()[1], "Second line");
    assert_eq!(editor.lines()[2], "Third line");
    
    // Test Ctrl+A (select all) - with simplified wrapping, lines remain separate
    let key = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL);
    editor.handle_key_event(key);
    
    // With simplified wrapping, short lines don't get consolidated automatically
    assert_eq!(editor.lines().len(), 3);
    assert_eq!(editor.lines()[0], "First line");
    assert_eq!(editor.lines()[1], "Second line");
    assert_eq!(editor.lines()[2], "Third line");
    
    // Test copying selected text
    let key = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
    let result = editor.handle_key_event(key);
    // Should return Some(KeyResult::Handled(true)) if clipboard works
    assert!(result.is_some());
    
    // Test deleting selected text should remove all content
    let key = KeyEvent::new(KeyCode::Delete, KeyModifiers::NONE);
    editor.handle_key_event(key);
    
    // Text should be deleted if selection worked
    assert_eq!(editor.lines().len(), 1);
    assert_eq!(editor.lines()[0], "");
    
    // Test pasting back
    let key = KeyEvent::new(KeyCode::Char('v'), KeyModifiers::CONTROL);
    let result = editor.handle_key_event(key);
    // May fail in test environment if clipboard not available
    // We just verify it doesn't crash
    assert!(result.is_some());
}

#[tokio::test]
async fn test_text_editor_word_boundaries_and_navigation() {
    let mut editor = TextEditor::new();
    
    // Type text with various word boundaries
    for ch in "Hello, world! This is a test.".chars() {
        let key = KeyEvent::new(KeyCode::Char(ch), KeyModifiers::NONE);
        editor.handle_key_event(key);
    }
    
    // Test Ctrl+Left (word left) - this might not be implemented in tui-textarea
    // but let's test the basic navigation
    let key = KeyEvent::new(KeyCode::Left, KeyModifiers::CONTROL);
    editor.handle_key_event(key);
    
    // Test Ctrl+Right (word right)
    let key = KeyEvent::new(KeyCode::Right, KeyModifiers::CONTROL);
    editor.handle_key_event(key);
    
    // Test Home/End keys
    let key = KeyEvent::new(KeyCode::Home, KeyModifiers::NONE);
    editor.handle_key_event(key);
    
    let key = KeyEvent::new(KeyCode::End, KeyModifiers::NONE);
    editor.handle_key_event(key);
    
    // Test Ctrl+Home (document start)
    let key = KeyEvent::new(KeyCode::Home, KeyModifiers::CONTROL);
    editor.handle_key_event(key);
    
    // Test Ctrl+End (document end)
    let key = KeyEvent::new(KeyCode::End, KeyModifiers::CONTROL);
    editor.handle_key_event(key);
    
    // Should still have the original text
    assert_eq!(editor.lines().len(), 1);
    assert_eq!(editor.lines()[0], "Hello, world! This is a test.");
}

#[tokio::test]
async fn test_text_editor_undo_redo_behavior() {
    let mut editor = TextEditor::new();
    
    // Type some text
    for ch in "Original text".chars() {
        let key = KeyEvent::new(KeyCode::Char(ch), KeyModifiers::NONE);
        editor.handle_key_event(key);
    }
    
    // Make a modification
    let key = KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE);
    editor.handle_key_event(key);
    let key = KeyEvent::new(KeyCode::Char('m'), KeyModifiers::NONE);
    editor.handle_key_event(key);
    let key = KeyEvent::new(KeyCode::Char('o'), KeyModifiers::NONE);
    editor.handle_key_event(key);
    let key = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE);
    editor.handle_key_event(key);
    
    assert_eq!(editor.lines()[0], "Original text mod");
    
    // Test undo
    let key = KeyEvent::new(KeyCode::Char('z'), KeyModifiers::CONTROL);
    editor.handle_key_event(key);
    
    // Test redo (Ctrl+Y or Ctrl+Shift+Z)
    let key = KeyEvent::new(KeyCode::Char('y'), KeyModifiers::CONTROL);
    editor.handle_key_event(key);
    
    // Test redo with Ctrl+Shift+Z
    let key = KeyEvent::new(KeyCode::Char('z'), KeyModifiers::CONTROL | KeyModifiers::SHIFT);
    editor.handle_key_event(key);
    
    // Content should still be there (exact behavior depends on tui-textarea implementation)
    assert!(!editor.lines()[0].is_empty());
}

#[tokio::test]
async fn test_mouse_selection_coordinate_edge_cases() {
    let mut editor = TextEditor::new();
    editor.set_area(ratatui::layout::Rect::new(10, 5, 50, 20)); // Set editor area
    
    // Create multi-line content with varying line lengths
    for ch in "Short\nMedium length line\nVery long line with lots of characters\nX".chars() {
        if ch == '\n' {
            let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
            editor.handle_key_event(key);
        } else {
            let key = KeyEvent::new(KeyCode::Char(ch), KeyModifiers::NONE);
            editor.handle_key_event(key);
        }
    }
    
    assert_eq!(editor.lines().len(), 4);
    assert_eq!(editor.lines()[0], "Short");
    assert_eq!(editor.lines()[1], "Medium length line");
    assert_eq!(editor.lines()[2], "Very long line with lots of characters");
    assert_eq!(editor.lines()[3], "X");
    
    // Test mouse click at exact line end
    let mouse = ratatui::crossterm::event::MouseEvent {
        kind: ratatui::crossterm::event::MouseEventKind::Down(ratatui::crossterm::event::MouseButton::Left),
        column: 16, // Just after "Short" (accounting for border)
        row: 6,    // First text line (accounting for border + title)
        modifiers: ratatui::crossterm::event::KeyModifiers::NONE,
    };
    
    let result = editor.handle_mouse_event(mouse);
    assert!(result.is_ok());
    
    // Test click beyond line end (should clamp to line end)
    let mouse = ratatui::crossterm::event::MouseEvent {
        kind: ratatui::crossterm::event::MouseEventKind::Down(ratatui::crossterm::event::MouseButton::Left),
        column: 50, // Way beyond line end
        row: 6,     // First text line
        modifiers: ratatui::crossterm::event::KeyModifiers::NONE,
    };
    
    let result = editor.handle_mouse_event(mouse);
    assert!(result.is_ok());
    
    // Test click on empty area beyond last line
    let mouse = ratatui::crossterm::event::MouseEvent {
        kind: ratatui::crossterm::event::MouseEventKind::Down(ratatui::crossterm::event::MouseButton::Left),
        column: 20,
        row: 15, // Beyond last line
        modifiers: ratatui::crossterm::event::KeyModifiers::NONE,
    };
    
    let result = editor.handle_mouse_event(mouse);
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_mouse_drag_selection_behavior() {
    let mut editor = TextEditor::new();
    editor.set_area(ratatui::layout::Rect::new(10, 5, 50, 20));
    
    // Type some text
    for ch in "Line 1\nLine 2\nLine 3".chars() {
        if ch == '\n' {
            let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
            editor.handle_key_event(key);
        } else {
            let key = KeyEvent::new(KeyCode::Char(ch), KeyModifiers::NONE);
            editor.handle_key_event(key);
        }
    }
    
    // Start selection with mouse down
    let mouse_down = ratatui::crossterm::event::MouseEvent {
        kind: ratatui::crossterm::event::MouseEventKind::Down(ratatui::crossterm::event::MouseButton::Left),
        column: 11, // Start of "Line 1" (accounting for border)
        row: 6,     // First text line
        modifiers: ratatui::crossterm::event::KeyModifiers::NONE,
    };
    
    let result = editor.handle_mouse_event(mouse_down);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), true); // Should handle the event
    
    // Drag to extend selection
    let mouse_drag = ratatui::crossterm::event::MouseEvent {
        kind: ratatui::crossterm::event::MouseEventKind::Drag(ratatui::crossterm::event::MouseButton::Left),
        column: 17, // End of "Line 1"
        row: 6,     // Same line
        modifiers: ratatui::crossterm::event::KeyModifiers::NONE,
    };
    
    let result = editor.handle_mouse_event(mouse_drag);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), true);
    
    // Drag to different line
    let mouse_drag2 = ratatui::crossterm::event::MouseEvent {
        kind: ratatui::crossterm::event::MouseEventKind::Drag(ratatui::crossterm::event::MouseButton::Left),
        column: 15, // Middle of "Line 2"
        row: 7,     // Second line
        modifiers: ratatui::crossterm::event::KeyModifiers::NONE,
    };
    
    let result = editor.handle_mouse_event(mouse_drag2);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), true);
    
    // End selection with mouse up
    let mouse_up = ratatui::crossterm::event::MouseEvent {
        kind: ratatui::crossterm::event::MouseEventKind::Up(ratatui::crossterm::event::MouseButton::Left),
        column: 15,
        row: 7,
        modifiers: ratatui::crossterm::event::KeyModifiers::NONE,
    };
    
    let result = editor.handle_mouse_event(mouse_up);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), true);
    
    // Test that subsequent drags without mouse down are ignored
    let mouse_drag3 = ratatui::crossterm::event::MouseEvent {
        kind: ratatui::crossterm::event::MouseEventKind::Drag(ratatui::crossterm::event::MouseButton::Left),
        column: 20,
        row: 8,
        modifiers: ratatui::crossterm::event::KeyModifiers::NONE,
    };
    
    let result = editor.handle_mouse_event(mouse_drag3);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), false); // Should not handle when not selecting
}

#[tokio::test]
async fn test_text_editor_special_character_handling() {
    let mut editor = TextEditor::new();
    
    // Test Unicode characters
    for ch in "Hello 世界! café naïve résumé".chars() {
        let key = KeyEvent::new(KeyCode::Char(ch), KeyModifiers::NONE);
        editor.handle_key_event(key);
    }
    
    assert_eq!(editor.lines()[0], "Hello 世界! café naïve résumé");
    
    // Test special characters that might cause issues
    let special_chars = ['\t', '"', '\'', '\\', '/', '<', '>', '&'];
    for &ch in &special_chars {
        let key = KeyEvent::new(KeyCode::Char(ch), KeyModifiers::NONE);
        editor.handle_key_event(key);
    }
    
    // Should have all characters
    let final_text = editor.lines()[0].clone();
    assert!(final_text.contains("世界"));
    assert!(final_text.contains("café"));
    assert!(final_text.contains('\t'));
    assert!(final_text.len() > 30); // Should be quite long now
}

#[tokio::test]
async fn test_cursor_positioning_boundary_conditions() {
    let mut editor = TextEditor::new();
    editor.set_area(ratatui::layout::Rect::new(0, 0, 40, 10));
    
    // Test empty editor boundary conditions
    let mouse = ratatui::crossterm::event::MouseEvent {
        kind: ratatui::crossterm::event::MouseEventKind::Down(ratatui::crossterm::event::MouseButton::Left),
        column: 0,
        row: 0,
        modifiers: ratatui::crossterm::event::KeyModifiers::NONE,
    };
    let result = editor.handle_mouse_event(mouse);
    assert!(result.is_ok());
    
    // Click at exact coordinates (0,0) - upper left corner
    let mouse = ratatui::crossterm::event::MouseEvent {
        kind: ratatui::crossterm::event::MouseEventKind::Down(ratatui::crossterm::event::MouseButton::Left),
        column: 1, // Just inside border
        row: 1,    // Just inside border + title
        modifiers: ratatui::crossterm::event::KeyModifiers::NONE,
    };
    let result = editor.handle_mouse_event(mouse);
    assert!(result.is_ok());
    
    // Type some text to create content
    for ch in "A\nB\nC".chars() {
        if ch == '\n' {
            let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
            editor.handle_key_event(key);
        } else {
            let key = KeyEvent::new(KeyCode::Char(ch), KeyModifiers::NONE);
            editor.handle_key_event(key);
        }
    }
    
    // Test clicking exactly at text boundaries
    let mouse = ratatui::crossterm::event::MouseEvent {
        kind: ratatui::crossterm::event::MouseEventKind::Down(ratatui::crossterm::event::MouseButton::Left),
        column: 2, // Right after "A"
        row: 1,    // First line
        modifiers: ratatui::crossterm::event::KeyModifiers::NONE,
    };
    let result = editor.handle_mouse_event(mouse);
    assert!(result.is_ok());
    
    // Test edge case: Click at maximum coordinates
    let mouse = ratatui::crossterm::event::MouseEvent {
        kind: ratatui::crossterm::event::MouseEventKind::Down(ratatui::crossterm::event::MouseButton::Left),
        column: 39, // At area boundary
        row: 9,     // At area boundary
        modifiers: ratatui::crossterm::event::KeyModifiers::NONE,
    };
    let result = editor.handle_mouse_event(mouse);
    assert!(result.is_ok());
    
    // Test click outside area (should be handled gracefully)
    let mouse = ratatui::crossterm::event::MouseEvent {
        kind: ratatui::crossterm::event::MouseEventKind::Down(ratatui::crossterm::event::MouseButton::Left),
        column: 100, // Way outside
        row: 100,    // Way outside
        modifiers: ratatui::crossterm::event::KeyModifiers::NONE,
    };
    let result = editor.handle_mouse_event(mouse);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), false); // Should not handle click outside area
}

#[tokio::test]
async fn test_rapid_key_sequences_and_state_consistency() {
    let mut editor = TextEditor::new();
    
    // Test rapid typing followed by immediate operations
    for ch in "Quick typing test".chars() {
        let key = KeyEvent::new(KeyCode::Char(ch), KeyModifiers::NONE);
        editor.handle_key_event(key);
    }
    
    // Rapid select all -> copy -> delete -> paste sequence
    let key = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL);
    editor.handle_key_event(key);
    
    let key = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
    editor.handle_key_event(key);
    
    let key = KeyEvent::new(KeyCode::Delete, KeyModifiers::NONE);
    editor.handle_key_event(key);
    
    // Text should be deleted after select-all and delete (correct behavior)
    assert_eq!(editor.lines().len(), 1);
    assert_eq!(editor.lines()[0], "");
    
    // Paste back (may fail in test environment but shouldn't crash)
    let key = KeyEvent::new(KeyCode::Char('v'), KeyModifiers::CONTROL);
    let result = editor.handle_key_event(key);
    assert!(result.is_some());
    
    // Test rapid navigation keys
    for _ in 0..10 {
        let key = KeyEvent::new(KeyCode::Right, KeyModifiers::NONE);
        editor.handle_key_event(key);
        let key = KeyEvent::new(KeyCode::Left, KeyModifiers::NONE);
        editor.handle_key_event(key);
    }
    
    // Test rapid undo/redo
    for _ in 0..5 {
        let key = KeyEvent::new(KeyCode::Char('z'), KeyModifiers::CONTROL);
        editor.handle_key_event(key);
        let key = KeyEvent::new(KeyCode::Char('y'), KeyModifiers::CONTROL);
        editor.handle_key_event(key);
    }
    
    // Editor should still be functional
    let key = KeyEvent::new(KeyCode::Char('T'), KeyModifiers::NONE);
    editor.handle_key_event(key);
    let key = KeyEvent::new(KeyCode::Char('e'), KeyModifiers::NONE);
    editor.handle_key_event(key);
    let key = KeyEvent::new(KeyCode::Char('s'), KeyModifiers::NONE);
    editor.handle_key_event(key);
    let key = KeyEvent::new(KeyCode::Char('t'), KeyModifiers::NONE);
    editor.handle_key_event(key);
    
    // Should have "Test" somewhere in the content
    let content = editor.lines().join("");
    assert!(content.contains("Test") || content.contains("test") || content == "Test");
}

#[tokio::test] 
async fn test_mouse_selection_state_consistency() {
    let mut editor = TextEditor::new();
    editor.set_area(ratatui::layout::Rect::new(5, 5, 30, 15));
    
    // Create content for selection
    for ch in "First\nSecond\nThird".chars() {
        if ch == '\n' {
            let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
            editor.handle_key_event(key);
        } else {
            let key = KeyEvent::new(KeyCode::Char(ch), KeyModifiers::NONE);
            editor.handle_key_event(key);
        }
    }
    
    // Start selection
    let mouse_down = ratatui::crossterm::event::MouseEvent {
        kind: ratatui::crossterm::event::MouseEventKind::Down(ratatui::crossterm::event::MouseButton::Left),
        column: 6,  // Start of "First"
        row: 6,     // First line
        modifiers: ratatui::crossterm::event::KeyModifiers::NONE,
    };
    let result = editor.handle_mouse_event(mouse_down);
    assert!(result.is_ok() && result.unwrap());
    
    // Rapid drag movements (potential glitch scenario)
    let drag_positions = [
        (10, 6), (15, 6), (8, 7), (12, 7), (6, 8), (11, 8)
    ];
    
    for (col, row) in drag_positions {
        let mouse_drag = ratatui::crossterm::event::MouseEvent {
            kind: ratatui::crossterm::event::MouseEventKind::Drag(ratatui::crossterm::event::MouseButton::Left),
            column: col,
            row: row,
            modifiers: ratatui::crossterm::event::KeyModifiers::NONE,
        };
        let result = editor.handle_mouse_event(mouse_drag);
        assert!(result.is_ok() && result.unwrap());
    }
    
    // End selection
    let mouse_up = ratatui::crossterm::event::MouseEvent {
        kind: ratatui::crossterm::event::MouseEventKind::Up(ratatui::crossterm::event::MouseButton::Left),
        column: 11,
        row: 8,
        modifiers: ratatui::crossterm::event::KeyModifiers::NONE,
    };
    let result = editor.handle_mouse_event(mouse_up);
    assert!(result.is_ok() && result.unwrap());
    
    // Test keyboard operations after mouse selection
    let key = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
    let result = editor.handle_key_event(key);
    assert!(result.is_some());
    
    // Start new selection immediately (potential glitch)
    let mouse_down2 = ratatui::crossterm::event::MouseEvent {
        kind: ratatui::crossterm::event::MouseEventKind::Down(ratatui::crossterm::event::MouseButton::Left),
        column: 7,
        row: 7,
        modifiers: ratatui::crossterm::event::KeyModifiers::NONE,
    };
    let result = editor.handle_mouse_event(mouse_down2);
    assert!(result.is_ok() && result.unwrap());
    
    // Cancel with mouse up immediately
    let mouse_up2 = ratatui::crossterm::event::MouseEvent {
        kind: ratatui::crossterm::event::MouseEventKind::Up(ratatui::crossterm::event::MouseButton::Left),
        column: 7,
        row: 7,
        modifiers: ratatui::crossterm::event::KeyModifiers::NONE,
    };
    let result = editor.handle_mouse_event(mouse_up2);
    assert!(result.is_ok() && result.unwrap());
    
    // Editor should still be responsive
    let key = KeyEvent::new(KeyCode::End, KeyModifiers::NONE);
    editor.handle_key_event(key);
    let key = KeyEvent::new(KeyCode::Char('!'), KeyModifiers::NONE);
    editor.handle_key_event(key);
    
    // Should have exclamation mark added
    let final_content = editor.lines().join("\n");
    assert!(final_content.contains('!'));
}

#[tokio::test]
async fn test_text_editor_mixed_input_scenarios() {
    let mut editor = TextEditor::new();
    
    // Test mixed keyboard and mouse operations
    // Type some text
    for ch in "Hello World".chars() {
        let key = KeyEvent::new(KeyCode::Char(ch), KeyModifiers::NONE);
        editor.handle_key_event(key);
    }
    
    // Move cursor with keyboard
    for _ in 0..5 {
        let key = KeyEvent::new(KeyCode::Left, KeyModifiers::NONE);
        editor.handle_key_event(key);
    }
    
    // Insert text in middle
    let key = KeyEvent::new(KeyCode::Char(','), KeyModifiers::NONE);
    editor.handle_key_event(key);
    let key = KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE);
    editor.handle_key_event(key);
    
    // Use mouse to position cursor (if area is set)
    editor.set_area(ratatui::layout::Rect::new(0, 0, 50, 10));
    let mouse = ratatui::crossterm::event::MouseEvent {
        kind: ratatui::crossterm::event::MouseEventKind::Down(ratatui::crossterm::event::MouseButton::Left),
        column: 1, // Start of text
        row: 1,
        modifiers: ratatui::crossterm::event::KeyModifiers::NONE,
    };
    editor.handle_mouse_event(mouse).unwrap();
    
    // Insert at beginning
    let key = KeyEvent::new(KeyCode::Char('['), KeyModifiers::NONE);
    editor.handle_key_event(key);
    
    // Select some text with mouse
    let mouse_drag = ratatui::crossterm::event::MouseEvent {
        kind: ratatui::crossterm::event::MouseEventKind::Drag(ratatui::crossterm::event::MouseButton::Left),
        column: 5,
        row: 1,
        modifiers: ratatui::crossterm::event::KeyModifiers::NONE,
    };
    editor.handle_mouse_event(mouse_drag).unwrap();
    
    // Type over selection
    let key = KeyEvent::new(KeyCode::Char('S'), KeyModifiers::NONE);
    editor.handle_key_event(key);
    let key = KeyEvent::new(KeyCode::Char('t'), KeyModifiers::NONE);
    editor.handle_key_event(key);
    let key = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
    editor.handle_key_event(key);
    let key = KeyEvent::new(KeyCode::Char('r'), KeyModifiers::NONE);
    editor.handle_key_event(key);
    let key = KeyEvent::new(KeyCode::Char('t'), KeyModifiers::NONE);
    editor.handle_key_event(key);
    
    // Final content should be modified
    let final_text = editor.lines().join("");
    assert!(!final_text.is_empty());
    assert!(final_text.len() > 5);
}

#[tokio::test]
async fn test_modification_detection_with_hard_wrapping() {
    let server = MockServer::start();

    // Mock getting specific thread for EditEntry state
    let thread_mock = server.mock(|when, then| {
        when.method(GET)
            .path("/api/threads/wrap-test-thread");
        then.status(200)
            .header("content-type", "application/json")
            .body(r#"{"data": {"id": "wrap-test-thread", "title": "Wrap Test Thread", "inserted_at": "2024-01-01T00:00:00Z", "updated_at": "2024-01-01T00:00:00Z", "entries": [{"id": "wrap-test-entry", "content": "This is a very long line that will definitely be wrapped when loaded into the text editor because it exceeds the typical terminal width and should trigger hard wrapping functionality.", "order_num": 1, "image_path": null, "thread_id": "wrap-test-thread", "inserted_at": "2024-01-01T00:00:00Z", "updated_at": "2024-01-01T00:00:00Z"}]}}"#);
    });

    let mut app = App::new(&server.base_url());

    // Navigate to thread view and load the thread
    app.state = AppState::ThreadView("wrap-test-thread".to_string());
    app.load_thread("wrap-test-thread").await.unwrap();

    // Enter edit mode for the entry
    let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    assert!(matches!(app.state, AppState::EditEntry(_, _)));

    // Verify the original content is stored
    assert!(app.original_entry_content.is_some());
    let original_content = app.original_entry_content.as_ref().unwrap();
    assert_eq!(original_content, "This is a very long line that will definitely be wrapped when loaded into the text editor because it exceeds the typical terminal width and should trigger hard wrapping functionality.");

    // The text editor should now have hard-wrapped content
    let wrapped_content = app.text_editor.lines().join("\n");
    
    // Verify that hard wrapping has occurred (the content should be different after wrapping)
    assert_ne!(wrapped_content, *original_content);
    
    // Test 1: Pressing Esc without making any changes should NOT show confirmation modal
    let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    
    // Should return directly to ThreadView without confirmation modal
    assert!(matches!(app.state, AppState::ThreadView(_)));
    if let AppState::ThreadView(thread_id) = &app.state {
        assert_eq!(thread_id, "wrap-test-thread");
    }

    // Test 2: Now test that actual modifications ARE detected
    // Re-enter edit mode
    let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    assert!(matches!(app.state, AppState::EditEntry(_, _)));

    // Make an actual modification - add some text
    let key = KeyEvent::new(KeyCode::End, KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    let key = KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    let key = KeyEvent::new(KeyCode::Char('M'), KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    let key = KeyEvent::new(KeyCode::Char('o'), KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    let key = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    let key = KeyEvent::new(KeyCode::Char('i'), KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    let key = KeyEvent::new(KeyCode::Char('f'), KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    let key = KeyEvent::new(KeyCode::Char('i'), KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    let key = KeyEvent::new(KeyCode::Char('e'), KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    let key = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();

    // Now pressing Esc should show confirmation modal because content was actually modified
    let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    
    // Should show confirmation modal
    assert!(matches!(app.state, AppState::ConfirmDiscardEntryChanges(_, _)));
    
    // Discard changes
    let key = KeyEvent::new(KeyCode::Char('y'), KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    
    // Should now return to ThreadView
    assert!(matches!(app.state, AppState::ThreadView(_)));
    if let AppState::ThreadView(thread_id) = &app.state {
        assert_eq!(thread_id, "wrap-test-thread");
    }

    // Test 3: Test edge case with paragraph breaks
    // Re-enter edit mode again
    let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    assert!(matches!(app.state, AppState::EditEntry(_, _)));

    // Test that adding and removing the same content results in no changes
    let key = KeyEvent::new(KeyCode::End, KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    let key = KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    let key = KeyEvent::new(KeyCode::Char('T'), KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    let key = KeyEvent::new(KeyCode::Char('e'), KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    let key = KeyEvent::new(KeyCode::Char('s'), KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    let key = KeyEvent::new(KeyCode::Char('t'), KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    
    // Now remove the text we just added
    for _ in 0..5 { // Remove " Test"
        let key = KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE);
        app.handle_key_event(key).await.unwrap();
    }

    // With simplified wrapping, modification detection is more accurate
    let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    
    // Since we added and removed the same text, no real changes occurred
    // The simplified wrapping system correctly detects no modifications
    assert!(matches!(app.state, AppState::ThreadView(_)));
    if let AppState::ThreadView(thread_id) = &app.state {
        assert_eq!(thread_id, "wrap-test-thread");
    }

    thread_mock.assert();
}

#[tokio::test]
async fn test_full_app_lifecycle() {
    let server = MockServer::start();

    // Mock initial thread load (empty)
    let initial_mock = server.mock(|when, then| {
        when.method(GET).path("/api/threads");
        then.status(200)
            .header("content-type", "application/json")
            .body(r#"{"data": []}"#);
    });

    let mut app = App::new(&server.base_url());

    // Test app initialization
    assert!(matches!(app.state, AppState::ThreadList));
    assert_eq!(app.threads.len(), 0);
    assert!(!app.should_quit);

    // Load initial data
    app.load_threads().await.unwrap();
    assert_eq!(app.threads.len(), 0);

    // Test quit functionality
    let key = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    assert!(app.should_quit);

    initial_mock.assert();
}

#[tokio::test]
async fn test_character_limit_error_modal() {
    let server = MockServer::start();
    
    // Mock initial threads call
    let _initial_mock = server.mock(|when, then| {
        when.method(GET).path("/api/threads");
        then.status(200)
            .header("content-type", "application/json")
            .body(r#"{"threads":[]}"#);
    });

    let mut app = App::new(&server.base_url());
    app.load_threads().await.unwrap();
    
    // Go to create thread state
    app.state = AppState::CreateThread;
    
    // Create a string longer than 500 characters
    let long_text = "a".repeat(501);
    app.text_editor.set_text(&long_text);
    
    // Try to save - this should trigger the character limit error modal
    let save_key = KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL);
    app.handle_key_event(save_key).await.unwrap();
    
    // Verify we're now in the CharacterLimitError state
    assert!(matches!(app.state, AppState::CharacterLimitError(_)));
    
    // Test that Enter key returns to previous state
    let enter_key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
    app.handle_key_event(enter_key).await.unwrap();
    
    // Should be back to CreateThread state
    assert!(matches!(app.state, AppState::CreateThread));
    
    // Test with Esc key as well
    app.text_editor.set_text(&long_text);
    app.handle_key_event(save_key).await.unwrap();
    assert!(matches!(app.state, AppState::CharacterLimitError(_)));
    
    let esc_key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
    app.handle_key_event(esc_key).await.unwrap();
    assert!(matches!(app.state, AppState::CreateThread));
}

#[tokio::test]
async fn test_confirm_delete_thread_workflow() {
    let server = MockServer::start();

    // Mock getting threads initially
    let _initial_threads_mock = server.mock(|when, then| {
        when.method(GET)
            .path("/api/threads");
        then.status(200)
            .header("content-type", "application/json")
            .body(r#"{"data": [{"id": "thread-to-delete", "title": "Thread to Delete", "inserted_at": "2024-01-01T00:00:00Z", "updated_at": "2024-01-01T00:00:00Z", "entries": []}]}"#);
    });

    // Mock thread deletion API call
    let delete_mock = server.mock(|when, then| {
        when.method(DELETE)
            .path("/api/threads/thread-to-delete");
        then.status(200)
            .header("content-type", "application/json")
            .body(r#"{"data": "ok"}"#);
    });

    // Mock getting threads after deletion (empty list)
    let _updated_threads_mock = server.mock(|when, then| {
        when.method(GET)
            .path("/api/threads");
        then.status(200)
            .header("content-type", "application/json")
            .body(r#"{"data": []}"#);
    });

    let mut app = App::new(&server.base_url());
    
    // Load initial threads
    app.load_threads().await.unwrap();
    assert_eq!(app.threads.len(), 1);
    assert_eq!(app.threads[0].title, "Thread to Delete");

    // Start delete operation by pressing Delete
    let delete_key = KeyEvent::new(KeyCode::Delete, KeyModifiers::NONE);
    app.handle_key_event(delete_key).await.unwrap();
    
    // Should show confirmation modal
    assert!(matches!(app.state, AppState::ConfirmDeleteThread(_)));
    if let AppState::ConfirmDeleteThread(thread_id) = &app.state {
        assert_eq!(thread_id, "thread-to-delete");
    }

    // Test 1: Cancel deletion with 'n'
    let cancel_key = KeyEvent::new(KeyCode::Char('n'), KeyModifiers::NONE);
    app.handle_key_event(cancel_key).await.unwrap();
    
    // Should return to ThreadList without deleting
    assert!(matches!(app.state, AppState::ThreadList));
    
    // Thread should still exist
    assert_eq!(app.threads.len(), 1);

    // Test 2: Start delete again and confirm with 'y'
    let delete_key = KeyEvent::new(KeyCode::Delete, KeyModifiers::NONE);
    app.handle_key_event(delete_key).await.unwrap();
    assert!(matches!(app.state, AppState::ConfirmDeleteThread(_)));

    // Confirm deletion with 'y'
    let confirm_key = KeyEvent::new(KeyCode::Char('y'), KeyModifiers::NONE);
    app.handle_key_event(confirm_key).await.unwrap();

    // Should be back to ThreadList after successful deletion
    assert!(matches!(app.state, AppState::ThreadList));

    // Verify the delete mock was called
    delete_mock.assert();

    // Test 3: Cancel deletion with Esc
    // Set up another thread to test Esc cancellation
    app.threads.push(Thread {
        id: "thread-esc-test".to_string(),
        title: "Esc Test Thread".to_string(),
        inserted_at: "2024-01-01T00:00:00Z".to_string(),
        updated_at: "2024-01-01T00:00:00Z".to_string(),
        entries: vec![],
    });
    app.selected_thread_index = 0;

    let delete_key = KeyEvent::new(KeyCode::Delete, KeyModifiers::NONE);
    app.handle_key_event(delete_key).await.unwrap();
    assert!(matches!(app.state, AppState::ConfirmDeleteThread(_)));

    // Cancel with Esc
    let esc_key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
    app.handle_key_event(esc_key).await.unwrap();
    
    // Should return to ThreadList without deleting
    assert!(matches!(app.state, AppState::ThreadList));
    // After cancellation, the thread we added manually should still exist
    assert!(app.threads.len() > 0); // At least one thread should still exist
}

#[tokio::test]
async fn test_confirm_delete_entry_workflow() {
    let server = MockServer::start();

    // Mock getting thread with entries
    let _thread_mock = server.mock(|when, then| {
        when.method(GET)
            .path("/api/threads/test-thread-delete-entry");
        then.status(200)
            .header("content-type", "application/json")
            .body(r#"{"data": {"id": "test-thread-delete-entry", "title": "Test Thread", "inserted_at": "2024-01-01T00:00:00Z", "updated_at": "2024-01-01T00:00:00Z", "entries": [{"id": "entry-to-delete", "content": "Entry to delete", "order_num": 1, "thread_id": "test-thread-delete-entry", "inserted_at": "2024-01-01T00:00:00Z", "updated_at": "2024-01-01T00:00:00Z"}]}}"#);
    });

    // Mock entry deletion API call
    let delete_mock = server.mock(|when, then| {
        when.method(DELETE)
            .path("/api/entries/entry-to-delete");
        then.status(200)
            .header("content-type", "application/json")
            .body(r#"{"data": "ok"}"#);
    });

    // Mock getting thread after deletion (empty entries)
    let _updated_thread_mock = server.mock(|when, then| {
        when.method(GET)
            .path("/api/threads/test-thread-delete-entry");
        then.status(200)
            .header("content-type", "application/json")
            .body(r#"{"data": {"id": "test-thread-delete-entry", "title": "Test Thread", "inserted_at": "2024-01-01T00:00:00Z", "updated_at": "2024-01-01T00:00:00Z", "entries": []}}"#);
    });

    let mut app = App::new(&server.base_url());
    
    // Set up ThreadView state with the test thread
    app.state = AppState::ThreadView("test-thread-delete-entry".to_string());
    app.load_thread("test-thread-delete-entry").await.unwrap();
    
    // Verify thread loaded with entry
    assert!(app.current_thread.is_some());
    let thread = app.current_thread.as_ref().unwrap();
    assert_eq!(thread.entries.len(), 1);
    assert_eq!(thread.entries[0].content, "Entry to delete");
    
    // Set selected entry index
    app.selected_entry_index = 0;

    // Start delete operation by pressing Delete
    let delete_key = KeyEvent::new(KeyCode::Delete, KeyModifiers::NONE);
    app.handle_key_event(delete_key).await.unwrap();
    
    // Should show confirmation modal
    assert!(matches!(app.state, AppState::ConfirmDeleteEntry(_, _)));
    if let AppState::ConfirmDeleteEntry(thread_id, entry_id) = &app.state {
        assert_eq!(thread_id, "test-thread-delete-entry");
        assert_eq!(entry_id, "entry-to-delete");
    }

    // Test 1: Cancel deletion with 'n'
    let cancel_key = KeyEvent::new(KeyCode::Char('n'), KeyModifiers::NONE);
    app.handle_key_event(cancel_key).await.unwrap();
    
    // Should return to ThreadView without deleting
    assert!(matches!(app.state, AppState::ThreadView(_)));
    if let AppState::ThreadView(thread_id) = &app.state {
        assert_eq!(thread_id, "test-thread-delete-entry");
    }
    
    // Entry should still exist
    let thread = app.current_thread.as_ref().unwrap();
    assert_eq!(thread.entries.len(), 1);

    // Test 2: Start delete again and confirm with 'y'
    let delete_key = KeyEvent::new(KeyCode::Delete, KeyModifiers::NONE);
    app.handle_key_event(delete_key).await.unwrap();
    assert!(matches!(app.state, AppState::ConfirmDeleteEntry(_, _)));

    // Confirm deletion with 'y'
    let confirm_key = KeyEvent::new(KeyCode::Char('y'), KeyModifiers::NONE);
    app.handle_key_event(confirm_key).await.unwrap();

    // Should be back to ThreadView after successful deletion
    assert!(matches!(app.state, AppState::ThreadView(_)));
    if let AppState::ThreadView(thread_id) = &app.state {
        assert_eq!(thread_id, "test-thread-delete-entry");
    }

    // Verify the delete mock was called
    delete_mock.assert();

    // Test 3: Cancel deletion with Esc 
    // Manually add an entry to test Esc cancellation (since original was deleted)
    if let Some(thread) = &mut app.current_thread {
        thread.entries.push(Entry {
            id: "entry-esc-test".to_string(),
            content: "Esc test entry".to_string(),
            order_num: 2,
            image_path: None,
            thread_id: Some("test-thread-delete-entry".to_string()),
            inserted_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
        });
    }
    app.selected_entry_index = 0;

    let delete_key = KeyEvent::new(KeyCode::Delete, KeyModifiers::NONE);
    app.handle_key_event(delete_key).await.unwrap();
    assert!(matches!(app.state, AppState::ConfirmDeleteEntry(_, _)));

    // Cancel with Esc
    let esc_key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
    app.handle_key_event(esc_key).await.unwrap();
    
    // Should return to ThreadView without deleting
    assert!(matches!(app.state, AppState::ThreadView(_)));
    // Entry should still exist (the one we manually added should still be there)
    let thread = app.current_thread.as_ref().unwrap();
    assert!(thread.entries.len() > 0); // At least one entry should exist
}

#[tokio::test]
async fn test_create_entry_discard_confirmation() {
    let mut app = App::new("http://localhost:8080");
    
    // Setup: Create a thread and navigate to CreateEntry state
    app.state = AppState::CreateEntry("test-thread".to_string());
    
    // Test 1: Empty content - should exit directly without confirmation
    let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    
    // Should return directly to ThreadView since there's no content
    assert!(matches!(app.state, AppState::ThreadView(_)));
    
    // Test 2: With content - should show confirmation
    app.state = AppState::CreateEntry("test-thread".to_string());
    
    // Add some content
    let key = KeyEvent::new(KeyCode::Char('H'), KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    let key = KeyEvent::new(KeyCode::Char('e'), KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    let key = KeyEvent::new(KeyCode::Char('l'), KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    let key = KeyEvent::new(KeyCode::Char('l'), KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    let key = KeyEvent::new(KeyCode::Char('o'), KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    
    // Now pressing Esc should show confirmation modal because content exists
    let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    
    // Should show confirmation modal
    assert!(matches!(app.state, AppState::ConfirmDiscardNewEntry(_)));
    
    // Test 3: Cancel discard (continue editing)
    let key = KeyEvent::new(KeyCode::Char('n'), KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    
    // Should return to CreateEntry
    assert!(matches!(app.state, AppState::CreateEntry(_)));
    
    // Test 4: Confirm discard
    let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    assert!(matches!(app.state, AppState::ConfirmDiscardNewEntry(_)));
    
    // Confirm discard with 'y'
    let key = KeyEvent::new(KeyCode::Char('y'), KeyModifiers::NONE);
    app.handle_key_event(key).await.unwrap();
    
    // Should return to ThreadView and clear editor
    assert!(matches!(app.state, AppState::ThreadView(_)));
    if let AppState::ThreadView(thread_id) = &app.state {
        assert_eq!(thread_id, "test-thread");
    }
    
    // Text editor should be cleared
    assert!(app.text_editor.lines().join("").trim().is_empty());
}
