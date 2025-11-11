# Reverse-API

[中文文档](./README_CN.md) | English

A unified reverse API wrapper written in Rust that provides seamless access to multiple AI services including ChatGPT, Grok, DeepSeek, Qwen, and more.

## 🌟 Features

- **Multi-Model Support**: Unified interface for ChatGPT, Grok (XAI), DeepSeek, Qwen, and GLM models
- **Multimodal Capabilities**: Support for text, images, videos, audio, and documents (via Qwen)
- **RESTful API**: Clean, OpenAI-compatible API design
- **Streaming Responses**: Real-time streaming for better user experience
- **Media Generation**: Image and video generation capabilities
- **File Upload**: Support for uploading and processing various file types
- **Proxy Support**: Configurable proxy settings for network flexibility
- **Web Dashboard**: Built-in monitoring and statistics dashboard
- **Thread Management**: Conversation history tracking
- **Browser Impersonation**: Advanced HTTP client with browser emulation

## 📋 Requirements

- **Rust**: 1.70 or higher
- **Operating System**: Linux, macOS, or Windows
- **Network**: Internet connection (proxy support available)

## 🚀 Quick Start

### 1. Clone the Repository

```bash
git clone https://github.com/map-A/reverse-api.git
cd reverse-api
```

### 2. Build the Project

```bash
cargo build --release
```

### 3. Configure API Tokens

You need to obtain tokens from the services you want to use:

#### DeepSeek Token
1. Visit https://chat.deepseek.com/
2. Login and start a conversation
3. Open Developer Tools (F12) → Application → LocalStorage
4. Find `userToken` and copy its value
5. Save to `.deepseek_token` file or set via API

#### Qwen Token
1. Visit https://chat.qwen.ai/
2. Login to your account
3. Open Developer Tools (F12) → Application → Cookies
4. Find `token` cookie and copy its value
5. Save to `.qwen_token` file or set via API

### 4. Start the Server

```bash
# Basic usage
./target/release/api_server

# Custom port and host
./target/release/api_server --host 127.0.0.1 --port 8080

# With proxy
./target/release/api_server --proxy http://127.0.0.1:7890

# Without proxy
./target/release/api_server --no-proxy

# Environment variables
export API_HOST=0.0.0.0
export API_PORT=6969
export DEFAULT_PROXY=http://127.0.0.1:1082
./target/release/api_server
```

### 5. Configure Tokens via API

```bash
# Configure DeepSeek token
curl -X POST http://localhost:6969/v1/config/deepseek \
  -H "Content-Type: application/json" \
  -d '{"token": "your_deepseek_token"}'

# Configure Qwen token
curl -X POST http://localhost:6969/v1/config/qwen \
  -H "Content-Type: application/json" \
  -d '{"token": "your_qwen_token"}'
```

## 📖 API Documentation

### Base URL

```
http://localhost:6969
```

### Endpoints

#### Health Check

```bash
GET /health
```

**Response:**
```json
{
  "status": "ok",
  "default_proxy": "http://127.0.0.1:1082",
  "active_threads": 5,
  "version": "0.1.0"
}
```

#### List Available Models

```bash
GET /v1/models
```

**Response:**
```json
{
  "data": [
    {
      "id": "grok-3-auto",
      "object": "model",
      "created": 1677610602,
      "owned_by": "xai"
    },
    {
      "id": "deepseek-r1",
      "object": "model",
      "created": 1677610602,
      "owned_by": "deepseek"
    },
    {
      "id": "qwen3-max",
      "object": "model",
      "created": 1677610602,
      "owned_by": "alibaba"
    }
  ]
}
```

#### Create Thread

```bash
POST /v1/threads
Content-Type: application/json

{
  "model": "qwen3-max",
  "messages": [
    {
      "role": "user",
      "content": "Hello!"
    }
  ],
  "proxy": "http://proxy:port",  // Optional
  "metadata": {}                  // Optional
}
```

**Response:**
```json
{
  "id": "thread-uuid-123",
  "object": "thread",
  "created_at": 1234567890,
  "metadata": null
}
```

#### Add Message to Thread

```bash
POST /v1/threads/{thread_id}/messages
Content-Type: application/json

{
  "role": "user",
  "content": "What can you do?"
}
```

#### Generate Response

```bash
POST /v1/responses
Content-Type: application/json

{
  "thread_id": "thread-uuid-123",
  "model": "qwen3-max",
  "file_ids": ["file-uuid-456"],  // Optional, for multimodal
  "proxy": "http://proxy:port"     // Optional
}
```

**Response:**
```json
{
  "response": "I can help you with various tasks...",
  "extra_data": { ... }
}
```

#### Upload File (for Qwen multimodal)

```bash
POST /v1/files/upload
Content-Type: multipart/form-data

file: <binary file data>
```

**Response:**
```json
{
  "id": "file-uuid-456",
  "filename": "image.jpg",
  "type": "image/jpeg"
}
```

#### Generate Image

```bash
POST /v1/images/generate
Content-Type: application/json

{
  "prompt": "A cute orange cat",
  "size": "1:1",           // Options: 1:1, 16:9, 9:16
  "model": "qwen3-max",
  "download": true         // Auto-download to ./generated/
}
```

**Response:**
```json
{
  "image_url": "https://...",
  "local_path": "./generated/images/1234567890.png"
}
```

#### Generate Video

```bash
POST /v1/videos/generate
Content-Type: application/json

{
  "prompt": "A cat playing in the grass",
  "size": "16:9",          // Options: 16:9, 9:16
  "model": "qwen3-max",
  "download": true         // Auto-download to ./generated/
}
```

**Response:**
```json
{
  "video_url": "https://...",
  "local_path": "./generated/videos/1234567890.mp4"
}
```

#### List Threads

```bash
GET /v1/threads
```

#### Get Thread Details

```bash
GET /v1/threads/{thread_id}
```

#### Delete Thread

```bash
DELETE /v1/threads/{thread_id}
```

#### List Thread Messages

```bash
GET /v1/threads/{thread_id}/messages
```

## 💡 Usage Examples

### Example 1: Simple Conversation with DeepSeek

```bash
# Create thread
THREAD_ID=$(curl -s -X POST http://localhost:6969/v1/threads \
  -H "Content-Type: application/json" \
  -d '{"model": "deepseek-r1"}' | jq -r '.id')

# Add message
curl -s -X POST http://localhost:6969/v1/threads/$THREAD_ID/messages \
  -H "Content-Type: application/json" \
  -d '{"role": "user", "content": "Explain quantum computing in one sentence"}'

# Get response
curl -s -X POST http://localhost:6969/v1/responses \
  -H "Content-Type: application/json" \
  -d "{\"thread_id\": \"$THREAD_ID\", \"model\": \"deepseek-r1\"}"
```

### Example 2: Image Recognition with Qwen

```bash
# Upload image
FILE_ID=$(curl -s -X POST http://localhost:6969/v1/files/upload \
  -F "file=@test_image.jpg" | jq -r '.id')

# Create thread
THREAD_ID=$(curl -s -X POST http://localhost:6969/v1/threads \
  -H "Content-Type: application/json" \
  -d '{"model": "qwen3-max"}' | jq -r '.id')

# Add message
curl -s -X POST http://localhost:6969/v1/threads/$THREAD_ID/messages \
  -H "Content-Type: application/json" \
  -d '{"role": "user", "content": "Describe this image"}'

# Get response with file
curl -s -X POST http://localhost:6969/v1/responses \
  -H "Content-Type: application/json" \
  -d "{\"thread_id\": \"$THREAD_ID\", \"model\": \"qwen3-max\", \"file_ids\": [\"$FILE_ID\"]}"
```

### Example 3: Generate Image

```bash
curl -s -X POST http://localhost:6969/v1/images/generate \
  -H "Content-Type: application/json" \
  -d '{
    "prompt": "A beautiful sunset over the ocean",
    "size": "16:9",
    "model": "qwen3-max",
    "download": true
  }'
```

### Example 4: Using with Code

#### Rust Example

```rust
use reverse_api::qwen::client::qwen::QwenClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create client with token
    let client = QwenClient::with_token("your_token")?;
    
    // Start conversation
    let response = client.start_convo("Hello, how are you?", None, None).await?;
    println!("Response: {}", response.response.unwrap_or_default());
    
    Ok(())
}
```

#### Python Example (using requests)

```python
import requests

BASE_URL = "http://localhost:6969"

# Create thread
response = requests.post(f"{BASE_URL}/v1/threads", json={
    "model": "qwen3-max"
})
thread_id = response.json()["id"]

# Add message
requests.post(f"{BASE_URL}/v1/threads/{thread_id}/messages", json={
    "role": "user",
    "content": "Hello, world!"
})

# Get response
response = requests.post(f"{BASE_URL}/v1/responses", json={
    "thread_id": thread_id,
    "model": "qwen3-max"
})
print(response.json()["response"])
```

#### JavaScript Example (using fetch)

```javascript
const BASE_URL = 'http://localhost:6969';

async function chat(message) {
  // Create thread
  const threadRes = await fetch(`${BASE_URL}/v1/threads`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ model: 'qwen3-max' })
  });
  const { id: threadId } = await threadRes.json();
  
  // Add message
  await fetch(`${BASE_URL}/v1/threads/${threadId}/messages`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ role: 'user', content: message })
  });
  
  // Get response
  const responseRes = await fetch(`${BASE_URL}/v1/responses`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ thread_id: threadId, model: 'qwen3-max' })
  });
  const { response } = await responseRes.json();
  return response;
}

chat('Hello!').then(console.log);
```

## 🤖 Supported Models

| Provider | Model ID | Capabilities | Multimodal |
|----------|----------|--------------|------------|
| XAI | `grok-3-auto` | Text generation | ❌ |
| XAI | `grok-3-turbo` | Fast text generation | ❌ |
| XAI | `grok-3-mini` | Lightweight model | ❌ |
| OpenAI | `chatgpt` | Text generation | ❌ |
| OpenAI | `gpt-4` | Advanced reasoning | ❌ |
| DeepSeek | `deepseek-r1` | Reasoning model | ❌ |
| DeepSeek | `deepseek-chat` | General chat | ❌ |
| Alibaba | `qwen3-max` | Advanced multimodal | ✅ |
| Alibaba | `qwen3-plus` | Enhanced model | ✅ |
| Alibaba | `qwen3-turbo` | Fast model | ✅ |
| Z.ai | `glm-4.6` | Text generation | ❌ |

### Qwen Multimodal Support

Qwen models support the following file types:

- **Images**: JPG, PNG, GIF, WebP, BMP
- **Documents**: PDF, TXT, DOC, DOCX
- **Audio**: MP3, WAV, AAC, M4A
- **Video**: MP4, AVI, MOV, MKV

## 📁 Project Structure

```
reverse-api/
├── src/
│   ├── bin/
│   │   ├── api_server.rs      # Main server binary
│   │   └── api/               # API modules
│   │       ├── server.rs      # Server setup
│   │       ├── handlers.rs    # Request handlers
│   │       ├── state.rs       # Application state
│   │       ├── dashboard.rs   # Web dashboard
│   │       └── docs.rs        # API documentation
│   ├── chatgpt/               # ChatGPT client
│   ├── grok/                  # Grok client
│   ├── deepseek/              # DeepSeek client
│   ├── qwen/                  # Qwen client (multimodal)
│   └── zto/                   # ZTO client
├── examples/                  # Usage examples
├── generated/                 # Auto-generated media
│   ├── images/
│   └── videos/
├── Cargo.toml                 # Rust dependencies
└── README.md                  # This file
```

## 🔧 Configuration

### Command Line Options

```bash
api_server [OPTIONS]

Options:
  --host <HOST>      Server host (default: 0.0.0.0)
  --port <PORT>      Server port (default: 6969)
  --proxy <PROXY>    Default proxy (default: http://127.0.0.1:1082)
  --no-proxy         Don't use any default proxy
  --help             Show help message
```

### Environment Variables

- `API_HOST`: Server host (default: 0.0.0.0)
- `API_PORT`: Server port (default: 6969)
- `DEFAULT_PROXY`: Default proxy URL
- `DEEPSEEK_TOKEN`: DeepSeek authentication token
- `QWEN_TOKEN`: Qwen authentication token

### Token Files

You can also store tokens in files:
- `.deepseek_token` - DeepSeek token
- `.qwen_token` - Qwen token

## 🌐 Dashboard & Documentation

Once the server is running, you can access:

- **API Documentation**: http://localhost:6969/docs
- **Dashboard**: http://localhost:6969/dashboard
  - View statistics
  - Monitor active threads
  - Track requests

## 🛠️ Development

### Run Examples

```bash
# DeepSeek example
DEEPSEEK_TOKEN="your_token" cargo run --example deepseek_example

# Qwen basic example
QWEN_TOKEN="your_token" cargo run --example qwen_example

# Qwen multimodal example
QWEN_TOKEN="your_token" cargo run --example qwen_multimodal_example

# Qwen image generation
QWEN_TOKEN="your_token" cargo run --example qwen_image_generation_example

# Grok example (requires proxy)
cargo run --example grok_example
```

### Run Tests

```bash
cargo test
```

### Build for Production

```bash
cargo build --release
```

The binary will be located at `./target/release/api_server`

## 🐛 Troubleshooting

### Network Connection Issues

If you encounter network issues:

1. **Use a proxy**: Set the `--proxy` flag or `DEFAULT_PROXY` environment variable
2. **Check firewall**: Ensure your firewall allows outbound connections
3. **Verify tokens**: Make sure your tokens are valid and not expired

### Token Not Working

1. **DeepSeek**: Get a fresh token from https://chat.deepseek.com/
   - Login → F12 → Application → LocalStorage → `userToken`
2. **Qwen**: Get a fresh token from https://chat.qwen.ai/
   - Login → F12 → Application → Cookies → `token`

### Port Already in Use

If port 6969 is already in use:

```bash
api_server --port 8080
```

### Proxy Issues

If you have proxy connection issues:

```bash
# Try without proxy
api_server --no-proxy

# Or use a different proxy
api_server --proxy http://127.0.0.1:7890
```

### Video Generation Takes Too Long

Video generation typically takes 1-3 minutes. This is normal due to the complexity of video rendering.

## 📝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details.

## 🙏 Acknowledgments

- [rquest](https://github.com/penumbra-x/rquest) - HTTP client with browser impersonation
- [axum](https://github.com/tokio-rs/axum) - Web framework
- [tokio](https://github.com/tokio-rs/tokio) - Async runtime

## 📞 Support

For issues, questions, or suggestions:
- Open an issue on [GitHub](https://github.com/map-A/reverse-api/issues)
- Check the [API documentation](http://localhost:6969/docs) when server is running

---

**Note**: This is a reverse-engineered API wrapper for educational purposes. Please ensure you comply with the terms of service of each AI provider when using this tool.
