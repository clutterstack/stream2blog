use crate::models::*;
use anyhow::Result;
use reqwest::Client;

pub struct ApiClient {
    client: Client,
    base_url: String,
}

impl ApiClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.to_string(),
        }
    }

    pub async fn get_threads(&self) -> Result<Vec<Thread>> {
        let url = format!("{}/api/threads", self.base_url);
        let response = self.client.get(&url).send().await?;
        let api_response: ApiListResponse<Thread> = response.json().await?;
        Ok(api_response.data)
    }

    pub async fn get_thread(&self, id: &str) -> Result<Thread> {
        let url = format!("{}/api/threads/{}", self.base_url, id);
        let response = self.client.get(&url).send().await?;
        let api_response: ApiResponse<Thread> = response.json().await?;
        Ok(api_response.data)
    }

    pub async fn create_thread(&self, title: &str) -> Result<Thread> {
        let url = format!("{}/api/threads", self.base_url);
        let request = CreateThreadRequest {
            thread: CreateThread {
                title: title.to_string(),
            },
        };

        let response = self.client.post(&url).json(&request).send().await?;

        let api_response: ApiResponse<Thread> = response.json().await?;
        Ok(api_response.data)
    }

    pub async fn update_thread(&self, id: &str, title: &str) -> Result<Thread> {
        let url = format!("{}/api/threads/{}", self.base_url, id);
        let request = UpdateThreadRequest {
            thread: UpdateThread {
                title: title.to_string(),
            },
        };

        let response = self.client.put(&url).json(&request).send().await?;

        let api_response: ApiResponse<Thread> = response.json().await?;
        Ok(api_response.data)
    }

    pub async fn create_entry(
        &self,
        thread_id: &str,
        content: &str,
        order_num: i32,
        image_path: Option<String>,
    ) -> Result<Entry> {
        let url = format!("{}/api/entries", self.base_url);
        let request = CreateEntryRequest {
            entry: CreateEntry {
                content: content.to_string(),
                order_num,
                image_path,
                thread_id: thread_id.to_string(),
            },
        };

        let response = self.client.post(&url).json(&request).send().await?;

        let api_response: ApiResponse<Entry> = response.json().await?;
        Ok(api_response.data)
    }

    pub async fn delete_thread(&self, id: &str) -> Result<()> {
        let url = format!("{}/api/threads/{}", self.base_url, id);
        let response = self.client.delete(&url).send().await?;
        if response.status().is_success() {
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Delete failed with status: {}",
                response.status()
            ))
        }
    }

    pub async fn update_entry(&self, id: &str, content: &str, order_num: i32, image_path: Option<String>) -> Result<Entry> {
        let url = format!("{}/api/entries/{}", self.base_url, id);
        let request = UpdateEntryRequest {
            entry: UpdateEntry {
                content: content.to_string(),
                order_num,
                image_path,
            },
        };

        let response = self.client.put(&url).json(&request).send().await?;

        let api_response: ApiResponse<Entry> = response.json().await?;
        Ok(api_response.data)
    }

    pub async fn delete_entry(&self, id: &str) -> Result<()> {
        let url = format!("{}/api/entries/{}", self.base_url, id);
        let response = self.client.delete(&url).send().await?;
        if response.status().is_success() {
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Delete failed with status: {}",
                response.status()
            ))
        }
    }

    pub async fn export_thread(&self, id: &str) -> Result<String> {
        let url = format!("{}/api/threads/{}/export", self.base_url, id);
        let response = self.client.get(&url).send().await?;
        if response.status().is_success() {
            let markdown = response.text().await?;
            Ok(markdown)
        } else {
            Err(anyhow::anyhow!(
                "Export failed with status: {}",
                response.status()
            ))
        }
    }

    pub async fn reorder_entries(&self, thread_id: &str, entries: Vec<(String, i32)>) -> Result<Thread> {
        let url = format!("{}/api/threads/{}/entries/reorder", self.base_url, thread_id);
        let request = ReorderEntriesRequest {
            entries: entries
                .into_iter()
                .map(|(id, order_num)| ReorderEntry { id, order_num })
                .collect(),
        };

        let response = self.client.put(&url).json(&request).send().await?;

        if response.status().is_success() {
            let api_response: ApiResponse<Thread> = response.json().await?;
            Ok(api_response.data)
        } else {
            Err(anyhow::anyhow!(
                "Reorder failed with status: {}",
                response.status()
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::prelude::*;
    use httpmock::Method::DELETE;
    // tokio_test not needed for these tests

    #[tokio::test]
    async fn test_api_client_new() {
        let client = ApiClient::new("http://localhost:8080");
        assert_eq!(client.base_url, "http://localhost:8080");
    }

    #[tokio::test]
    async fn test_get_threads_success() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET)
                .path("/api/threads");
            then.status(200)
                .header("content-type", "application/json")
                .body(r#"{"data": [{"id": "1", "title": "Test Thread", "inserted_at": "2024-01-01T00:00:00Z", "updated_at": "2024-01-01T00:00:00Z", "entries": []}]}"#);
        });

        let client = ApiClient::new(&server.base_url());
        let threads = client.get_threads().await.unwrap();

        assert_eq!(threads.len(), 1);
        assert_eq!(threads[0].id, "1");
        assert_eq!(threads[0].title, "Test Thread");
        mock.assert();
    }

    #[tokio::test]
    async fn test_get_threads_error() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET).path("/api/threads");
            then.status(500);
        });

        let client = ApiClient::new(&server.base_url());
        let result = client.get_threads().await;

        assert!(result.is_err());
        mock.assert();
    }

    #[tokio::test]
    async fn test_get_thread_success() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET)
                .path("/api/threads/123");
            then.status(200)
                .header("content-type", "application/json")
                .body(r#"{"data": {"id": "123", "title": "Specific Thread", "inserted_at": "2024-01-01T00:00:00Z", "updated_at": "2024-01-01T00:00:00Z", "entries": []}}"#);
        });

        let client = ApiClient::new(&server.base_url());
        let thread = client.get_thread("123").await.unwrap();

        assert_eq!(thread.id, "123");
        assert_eq!(thread.title, "Specific Thread");
        mock.assert();
    }

    #[tokio::test]
    async fn test_get_thread_not_found() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET).path("/api/threads/nonexistent");
            then.status(404);
        });

        let client = ApiClient::new(&server.base_url());
        let result = client.get_thread("nonexistent").await;

        assert!(result.is_err());
        mock.assert();
    }

    #[tokio::test]
    async fn test_create_thread_success() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(POST)
                .path("/api/threads")
                .header("content-type", "application/json")
                .json_body(serde_json::json!({
                    "thread": {
                        "title": "New Thread"
                    }
                }));
            then.status(201)
                .header("content-type", "application/json")
                .body(r#"{"data": {"id": "new-123", "title": "New Thread", "inserted_at": "2024-01-01T00:00:00Z", "updated_at": "2024-01-01T00:00:00Z", "entries": []}}"#);
        });

        let client = ApiClient::new(&server.base_url());
        let thread = client.create_thread("New Thread").await.unwrap();

        assert_eq!(thread.id, "new-123");
        assert_eq!(thread.title, "New Thread");
        mock.assert();
    }

    #[tokio::test]
    async fn test_create_thread_error() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(POST).path("/api/threads");
            then.status(422);
        });

        let client = ApiClient::new(&server.base_url());
        let result = client.create_thread("").await;

        assert!(result.is_err());
        mock.assert();
    }

    #[tokio::test]
    async fn test_create_entry_success() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(POST)
                .path("/api/entries")
                .header("content-type", "application/json")
                .json_body(serde_json::json!({
                    "entry": {
                        "content": "New entry content",
                        "order_num": 1,
                        "thread_id": "thread-123"
                    }
                }));
            then.status(201)
                .header("content-type", "application/json")
                .body(r#"{"data": {"id": "entry-456", "content": "New entry content", "order_num": 1, "thread_id": "thread-123", "inserted_at": "2024-01-01T00:00:00Z", "updated_at": "2024-01-01T00:00:00Z"}}"#);
        });

        let client = ApiClient::new(&server.base_url());
        let entry = client
            .create_entry("thread-123", "New entry content", 1, None)
            .await
            .unwrap();

        assert_eq!(entry.id, "entry-456");
        assert_eq!(entry.content, "New entry content");
        assert_eq!(entry.order_num, 1);
        assert_eq!(entry.thread_id, Some("thread-123".to_string()));
        mock.assert();
    }

    #[tokio::test]
    async fn test_create_entry_error() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(POST).path("/api/entries");
            then.status(422);
        });

        let client = ApiClient::new(&server.base_url());
        let result = client.create_entry("thread-123", "", 1, None).await;

        assert!(result.is_err());
        mock.assert();
    }

    #[tokio::test]
    async fn test_get_threads_empty_response() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET).path("/api/threads");
            then.status(200)
                .header("content-type", "application/json")
                .body(r#"{"data": []}"#);
        });

        let client = ApiClient::new(&server.base_url());
        let threads = client.get_threads().await.unwrap();

        assert_eq!(threads.len(), 0);
        mock.assert();
    }

    #[tokio::test]
    async fn test_get_thread_with_entries() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET)
                .path("/api/threads/123");
            then.status(200)
                .header("content-type", "application/json")
                .body(r#"{"data": {"id": "123", "title": "Thread with entries", "inserted_at": "2024-01-01T00:00:00Z", "updated_at": "2024-01-01T00:00:00Z", "entries": [{"id": "entry-1", "content": "First entry", "order_num": 1, "thread_id": "123", "inserted_at": "2024-01-01T00:00:00Z", "updated_at": "2024-01-01T00:00:00Z"}]}}"#);
        });

        let client = ApiClient::new(&server.base_url());
        let thread = client.get_thread("123").await.unwrap();

        assert_eq!(thread.id, "123");
        assert_eq!(thread.entries.len(), 1);
        assert_eq!(thread.entries[0].content, "First entry");
        mock.assert();
    }

    #[tokio::test]
    async fn test_api_client_with_different_base_url() {
        let client = ApiClient::new("https://api.example.com");
        assert_eq!(client.base_url, "https://api.example.com");
    }

    #[tokio::test]
    async fn test_create_thread_with_special_characters() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(POST)
                .path("/api/threads")
                .header("content-type", "application/json")
                .json_body(serde_json::json!({
                    "thread": {
                        "title": "Special: !@#$%^&*()_+ 测试"
                    }
                }));
            then.status(201)
                .header("content-type", "application/json")
                .body(r#"{"data": {"id": "special-123", "title": "Special: !@#$%^&*()_+ 测试", "inserted_at": "2024-01-01T00:00:00Z", "updated_at": "2024-01-01T00:00:00Z", "entries": []}}"#);
        });

        let client = ApiClient::new(&server.base_url());
        let thread = client
            .create_thread("Special: !@#$%^&*()_+ 测试")
            .await
            .unwrap();

        assert_eq!(thread.title, "Special: !@#$%^&*()_+ 测试");
        mock.assert();
    }

    #[tokio::test]
    async fn test_create_entry_with_multiline_content() {
        let server = MockServer::start();
        let content = "Line 1\nLine 2\nLine 3";
        let mock = server.mock(|when, then| {
            when.method(POST)
                .path("/api/entries")
                .header("content-type", "application/json")
                .json_body(serde_json::json!({
                    "entry": {
                        "content": content,
                        "order_num": 5,
                        "thread_id": "thread-456"
                    }
                }));
            then.status(201)
                .header("content-type", "application/json")
                .body(format!(r#"{{"data": {{"id": "entry-789", "content": "{}", "order_num": 5, "thread_id": "thread-456", "inserted_at": "2024-01-01T00:00:00Z", "updated_at": "2024-01-01T00:00:00Z"}}}}"#, content.replace("\n", "\\n")));
        });

        let client = ApiClient::new(&server.base_url());
        let entry = client.create_entry("thread-456", content, 5, None).await.unwrap();

        assert_eq!(entry.content, content);
        assert_eq!(entry.order_num, 5);
        mock.assert();
    }

    #[tokio::test]
    async fn test_invalid_json_response() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET).path("/api/threads");
            then.status(200)
                .header("content-type", "application/json")
                .body("invalid json");
        });

        let client = ApiClient::new(&server.base_url());
        let result = client.get_threads().await;

        assert!(result.is_err());
        mock.assert();
    }

    #[tokio::test]
    async fn test_delete_thread_success() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(DELETE).path("/api/threads/123");
            then.status(204);
        });

        let client = ApiClient::new(&server.base_url());
        let result = client.delete_thread("123").await;

        assert!(result.is_ok());
        mock.assert();
    }

    #[tokio::test]
    async fn test_delete_thread_not_found() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(DELETE).path("/api/threads/nonexistent");
            then.status(404);
        });

        let client = ApiClient::new(&server.base_url());
        let result = client.delete_thread("nonexistent").await;

        assert!(result.is_err());
        mock.assert();
    }

    #[tokio::test]
    async fn test_delete_entry_success() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(DELETE).path("/api/entries/456");
            then.status(204);
        });

        let client = ApiClient::new(&server.base_url());
        let result = client.delete_entry("456").await;

        assert!(result.is_ok());
        mock.assert();
    }

    #[tokio::test]
    async fn test_delete_entry_not_found() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(DELETE).path("/api/entries/nonexistent");
            then.status(404);
        });

        let client = ApiClient::new(&server.base_url());
        let result = client.delete_entry("nonexistent").await;

        assert!(result.is_err());
        mock.assert();
    }
}
