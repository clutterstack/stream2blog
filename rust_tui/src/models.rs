use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Thread {
    pub id: String,
    pub title: String,
    pub inserted_at: String,
    pub updated_at: String,
    pub entries: Vec<Entry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub id: String,
    pub content: String,
    pub order_num: i32,
    pub image_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread_id: Option<String>,
    pub inserted_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
pub struct CreateThreadRequest {
    pub thread: CreateThread,
}

#[derive(Debug, Serialize)]
pub struct CreateThread {
    pub title: String,
}

#[derive(Debug, Serialize)]
pub struct UpdateThreadRequest {
    pub thread: UpdateThread,
}

#[derive(Debug, Serialize)]
pub struct UpdateThread {
    pub title: String,
}

#[derive(Debug, Serialize)]
pub struct CreateEntryRequest {
    pub entry: CreateEntry,
}

#[derive(Debug, Serialize)]
pub struct CreateEntry {
    pub content: String,
    pub order_num: i32,
    pub image_path: Option<String>,
    pub thread_id: String,
}

#[derive(Debug, Serialize)]
pub struct UpdateEntryRequest {
    pub entry: UpdateEntry,
}

#[derive(Debug, Serialize)]
pub struct UpdateEntry {
    pub content: String,
    pub order_num: i32,
    pub image_path: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ReorderEntriesRequest {
    pub entries: Vec<ReorderEntry>,
}

#[derive(Debug, Serialize)]
pub struct ReorderEntry {
    pub id: String,
    pub order_num: i32,
}

#[derive(Debug, Deserialize)]
pub struct ApiResponse<T> {
    pub data: T,
}

#[derive(Debug, Deserialize)]
pub struct ApiListResponse<T> {
    pub data: Vec<T>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_thread_serialization() {
        let thread = Thread {
            id: "test-id".to_string(),
            title: "Test Thread".to_string(),
            inserted_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
            entries: vec![],
        };

        let json = serde_json::to_string(&thread).unwrap();
        let deserialized: Thread = serde_json::from_str(&json).unwrap();

        assert_eq!(thread.id, deserialized.id);
        assert_eq!(thread.title, deserialized.title);
        assert_eq!(thread.inserted_at, deserialized.inserted_at);
        assert_eq!(thread.updated_at, deserialized.updated_at);
        assert_eq!(thread.entries.len(), deserialized.entries.len());
    }

    #[test]
    fn test_entry_serialization() {
        let entry = Entry {
            id: "entry-id".to_string(),
            content: "Test content".to_string(),
            order_num: 1,
            image_path: Some("/path/to/image.png".to_string()),
            thread_id: Some("thread-id".to_string()),
            inserted_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
        };

        let json = serde_json::to_string(&entry).unwrap();
        let deserialized: Entry = serde_json::from_str(&json).unwrap();

        assert_eq!(entry.id, deserialized.id);
        assert_eq!(entry.content, deserialized.content);
        assert_eq!(entry.order_num, deserialized.order_num);
        assert_eq!(entry.image_path, deserialized.image_path);
        assert_eq!(entry.thread_id, deserialized.thread_id);
    }

    #[test]
    fn test_entry_without_thread_id() {
        let entry = Entry {
            id: "entry-id".to_string(),
            content: "Test content".to_string(),
            order_num: 1,
            image_path: None,
            thread_id: None,
            inserted_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
        };

        let json = serde_json::to_string(&entry).unwrap();
        assert!(!json.contains("thread_id"));
        assert!(!json.contains("image_path"));

        let deserialized: Entry = serde_json::from_str(&json).unwrap();
        assert_eq!(entry.image_path, deserialized.image_path);
        assert_eq!(entry.thread_id, deserialized.thread_id);
    }

    #[test]
    fn test_create_thread_request() {
        let request = CreateThreadRequest {
            thread: CreateThread {
                title: "New Thread".to_string(),
            },
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("New Thread"));
    }

    #[test]
    fn test_create_entry_request() {
        let request = CreateEntryRequest {
            entry: CreateEntry {
                content: "New entry content".to_string(),
                order_num: 5,
                image_path: Some("/path/to/test.png".to_string()),
                thread_id: "thread-123".to_string(),
            },
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("New entry content"));
        assert!(json.contains("thread-123"));
        assert!(json.contains("5"));
        assert!(json.contains("/path/to/test.png"));
    }

    #[test]
    fn test_api_response() {
        let response_json = r#"{"data": {"id": "test", "title": "Test", "inserted_at": "2024-01-01T00:00:00Z", "updated_at": "2024-01-01T00:00:00Z", "entries": []}}"#;
        let response: ApiResponse<Thread> = serde_json::from_str(response_json).unwrap();

        assert_eq!(response.data.id, "test");
        assert_eq!(response.data.title, "Test");
    }

    #[test]
    fn test_api_list_response() {
        let response_json = r#"{"data": [{"id": "test1", "title": "Test 1", "inserted_at": "2024-01-01T00:00:00Z", "updated_at": "2024-01-01T00:00:00Z", "entries": []}, {"id": "test2", "title": "Test 2", "inserted_at": "2024-01-01T00:00:00Z", "updated_at": "2024-01-01T00:00:00Z", "entries": []}]}"#;
        let response: ApiListResponse<Thread> = serde_json::from_str(response_json).unwrap();

        assert_eq!(response.data.len(), 2);
        assert_eq!(response.data[0].id, "test1");
        assert_eq!(response.data[1].id, "test2");
    }

    #[test]
    fn test_thread_with_entries() {
        let thread = Thread {
            id: "thread-id".to_string(),
            title: "Thread with entries".to_string(),
            inserted_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
            entries: vec![
                Entry {
                    id: "entry-1".to_string(),
                    content: "First entry".to_string(),
                    order_num: 1,
                    image_path: Some("/path/to/first.png".to_string()),
                    thread_id: Some("thread-id".to_string()),
                    inserted_at: "2024-01-01T00:00:00Z".to_string(),
                    updated_at: "2024-01-01T00:00:00Z".to_string(),
                },
                Entry {
                    id: "entry-2".to_string(),
                    content: "Second entry".to_string(),
                    order_num: 2,
                    image_path: None,
                    thread_id: Some("thread-id".to_string()),
                    inserted_at: "2024-01-01T00:00:00Z".to_string(),
                    updated_at: "2024-01-01T00:00:00Z".to_string(),
                },
            ],
        };

        let json = serde_json::to_string(&thread).unwrap();
        let deserialized: Thread = serde_json::from_str(&json).unwrap();

        assert_eq!(thread.entries.len(), 2);
        assert_eq!(deserialized.entries.len(), 2);
        assert_eq!(deserialized.entries[0].content, "First entry");
        assert_eq!(deserialized.entries[1].content, "Second entry");
    }
}
