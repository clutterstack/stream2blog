# Blog API Documentation

## Overview

Phoenix 1.18.0-rc3 headless API for the blog/threading system.

- **Base URL**: `http://localhost:4001`
- **Content-Type**: `application/json`
- **Database**: SQLite with binary IDs

## Data Models

### Thread
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "title": "My Blog Post",
  "inserted_at": "2025-07-04T05:04:52Z",
  "updated_at": "2025-07-04T05:04:52Z",
  "entries": [...]
}
```

### Entry
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440001",
  "content": "This is the content of the entry",
  "order_num": 1,
  "image_path": "/path/to/image.png",
  "thread_id": "550e8400-e29b-41d4-a716-446655440000",
  "inserted_at": "2025-07-04T05:14:54Z",
  "updated_at": "2025-07-04T05:14:54Z"
}
```

## API Endpoints

### Threads

#### GET /api/threads
List all threads (without entries).

**Response:**
```json
{
  "data": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "title": "My Blog Post",
      "inserted_at": "2025-07-04T05:04:52Z",
      "updated_at": "2025-07-04T05:04:52Z",
      "entries": []
    }
  ]
}
```

#### POST /api/threads
Create a new thread.

**Request:**
```json
{
  "thread": {
    "title": "My New Blog Post"
  }
}
```

**Response (201):**
```json
{
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "title": "My New Blog Post",
    "inserted_at": "2025-07-04T05:04:52Z",
    "updated_at": "2025-07-04T05:04:52Z",
    "entries": []
  }
}
```

#### GET /api/threads/:id
Get a specific thread with all its entries.

**Response:**
```json
{
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "title": "My Blog Post",
    "inserted_at": "2025-07-04T05:04:52Z",
    "updated_at": "2025-07-04T05:04:52Z",
    "entries": [
      {
        "id": "550e8400-e29b-41d4-a716-446655440001",
        "content": "First entry content",
        "order_num": 1,
        "image_path": null,
        "inserted_at": "2025-07-04T05:14:54Z",
        "updated_at": "2025-07-04T05:14:54Z"
      }
    ]
  }
}
```

#### PUT /api/threads/:id
Update a thread.

**Request:**
```json
{
  "thread": {
    "title": "Updated Blog Post Title"
  }
}
```

**Response (200):**
```json
{
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "title": "Updated Blog Post Title",
    "inserted_at": "2025-07-04T05:04:52Z",
    "updated_at": "2025-07-04T05:20:15Z",
    "entries": []
  }
}
```

#### DELETE /api/threads/:id
Delete a thread and all its entries.

**Response (204):** No content

#### GET /api/threads/:id/export
Export a thread as markdown.

**Response (200):**
```markdown
# My Blog Post

First entry content here.

Second entry content here.

Third entry content here.
```

**Headers:**
- `Content-Type: text/markdown`
- `Content-Disposition: attachment; filename="my_blog_post.md"`

### Entries

#### GET /api/entries
List all entries (across all threads).

**Response:**
```json
{
  "data": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440001",
      "content": "Entry content",
      "order_num": 1,
      "image_path": null,
      "thread_id": "550e8400-e29b-41d4-a716-446655440000",
      "inserted_at": "2025-07-04T05:14:54Z",
      "updated_at": "2025-07-04T05:14:54Z"
    }
  ]
}
```

#### POST /api/entries
Create a new entry.

**Request:**
```json
{
  "entry": {
    "content": "This is my entry content",
    "order_num": 1,
    "image_path": "/path/to/image.png",
    "thread_id": "550e8400-e29b-41d4-a716-446655440000"
  }
}
```

**Response (201):**
```json
{
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440001",
    "content": "This is my entry content",
    "order_num": 1,
    "image_path": "/path/to/image.png",
    "thread_id": "550e8400-e29b-41d4-a716-446655440000",
    "inserted_at": "2025-07-04T05:14:54Z",
    "updated_at": "2025-07-04T05:14:54Z"
  }
}
```

#### GET /api/entries/:id
Get a specific entry.

**Response:**
```json
{
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440001",
    "content": "Entry content",
    "order_num": 1,
    "image_path": null,
    "thread_id": "550e8400-e29b-41d4-a716-446655440000",
    "inserted_at": "2025-07-04T05:14:54Z",
    "updated_at": "2025-07-04T05:14:54Z"
  }
}
```

#### PUT /api/entries/:id
Update an entry.

**Request:**
```json
{
  "entry": {
    "content": "Updated entry content",
    "order_num": 2,
    "image_path": "/path/to/new_image.png"
  }
}
```

**Response (200):**
```json
{
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440001",
    "content": "Updated entry content",
    "order_num": 2,
    "image_path": "/path/to/new_image.png",
    "thread_id": "550e8400-e29b-41d4-a716-446655440000",
    "inserted_at": "2025-07-04T05:14:54Z",
    "updated_at": "2025-07-04T05:25:30Z"
  }
}
```

#### DELETE /api/entries/:id
Delete an entry.

**Response (204):** No content

## Error Responses

### 422 Unprocessable Entity
When validation fails:

```json
{
  "errors": {
    "title": ["can't be blank"],
    "content": ["can't be blank"]
  }
}
```

### 404 Not Found
When resource doesn't exist:

```json
{
  "errors": {
    "detail": "Not Found"
  }
}
```

## Development Notes

### Starting the Server
```bash
cd blog_api
PORT=4001 mix phx.server
```

### Database
- **File**: `blog_api_dev.db`
- **Type**: SQLite
- **Migrations**: Run `mix ecto.migrate`

### Testing Examples
```bash
# Create thread
curl -X POST http://localhost:4001/api/threads \
  -H "Content-Type: application/json" \
  -d '{"thread": {"title": "Test Thread"}}'

# Create entry
curl -X POST http://localhost:4001/api/entries \
  -H "Content-Type: application/json" \
  -d '{"entry": {"content": "Test content", "order_num": 1, "image_path": "/path/to/image.png", "thread_id": "YOUR_THREAD_ID"}}'

# Get thread with entries
curl http://localhost:4001/api/threads/YOUR_THREAD_ID
```

## Future Enhancements

### Planned Endpoints
- ✅ `GET /api/threads/:id/export` - Export thread as markdown
- `POST /api/threads/:id/entries` - Create entry for specific thread
- `PUT /api/threads/:id/entries/reorder` - Reorder entries

### WebSocket Support (Phase 2)
- Real-time updates via Phoenix channels
- `thread:*` channels for live collaboration