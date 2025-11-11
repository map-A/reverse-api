# Reverse-API

中文文档 | [English](./README.md)

一个用 Rust 编写的统一逆向 API 包装器，提供对多个 AI 服务的无缝访问，包括 ChatGPT、Grok、DeepSeek、Qwen 等。

## 🌟 特性

- **多模型支持**：统一接口支持 ChatGPT、Grok (XAI)、DeepSeek、Qwen 和 GLM 模型
- **多模态能力**：支持文本、图片、视频、音频和文档（通过 Qwen）
- **RESTful API**：简洁的、兼容 OpenAI 的 API 设计
- **流式响应**：实时流式传输，提供更好的用户体验
- **媒体生成**：图片和视频生成功能
- **文件上传**：支持上传和处理各种文件类型
- **代理支持**：可配置的代理设置，提供网络灵活性
- **Web 仪表板**：内置监控和统计仪表板
- **线程管理**：对话历史跟踪
- **浏览器模拟**：高级 HTTP 客户端，带有浏览器模拟功能

## 📋 系统要求

- **Rust**：1.70 或更高版本
- **操作系统**：Linux、macOS 或 Windows
- **网络**：互联网连接（支持代理）

## 🚀 快速开始

### 1. 克隆仓库

```bash
git clone https://github.com/map-A/reverse-api.git
cd reverse-api
```

### 2. 编译项目

```bash
cargo build --release
```

### 3. 配置 API Token

您需要从要使用的服务获取 token：

#### DeepSeek Token
1. 访问 https://chat.deepseek.com/
2. 登录并开始对话
3. 打开开发者工具（F12）→ Application → LocalStorage
4. 找到 `userToken` 并复制其值
5. 保存到 `.deepseek_token` 文件或通过 API 设置

#### Qwen Token
1. 访问 https://chat.qwen.ai/
2. 登录您的账户
3. 打开开发者工具（F12）→ Application → Cookies
4. 找到 `token` cookie 并复制其值
5. 保存到 `.qwen_token` 文件或通过 API 设置

### 4. 启动服务器

```bash
# 基础使用
./target/release/api_server

# 自定义端口和主机
./target/release/api_server --host 127.0.0.1 --port 8080

# 使用代理
./target/release/api_server --proxy http://127.0.0.1:7890

# 不使用代理
./target/release/api_server --no-proxy

# 使用环境变量
export API_HOST=0.0.0.0
export API_PORT=6969
export DEFAULT_PROXY=http://127.0.0.1:1082
./target/release/api_server
```

### 5. 通过 API 配置 Token

```bash
# 配置 DeepSeek token
curl -X POST http://localhost:6969/v1/config/deepseek \
  -H "Content-Type: application/json" \
  -d '{"token": "your_deepseek_token"}'

# 配置 Qwen token
curl -X POST http://localhost:6969/v1/config/qwen \
  -H "Content-Type: application/json" \
  -d '{"token": "your_qwen_token"}'
```

## 📖 API 文档

### 基础 URL

```
http://localhost:6969
```

### 接口端点

#### 健康检查

```bash
GET /health
```

**响应：**
```json
{
  "status": "ok",
  "default_proxy": "http://127.0.0.1:1082",
  "active_threads": 5,
  "version": "0.1.0"
}
```

#### 列出可用模型

```bash
GET /v1/models
```

**响应：**
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

#### 创建线程

```bash
POST /v1/threads
Content-Type: application/json

{
  "model": "qwen3-max",
  "messages": [
    {
      "role": "user",
      "content": "你好！"
    }
  ],
  "proxy": "http://proxy:port",  // 可选
  "metadata": {}                  // 可选
}
```

**响应：**
```json
{
  "id": "thread-uuid-123",
  "object": "thread",
  "created_at": 1234567890,
  "metadata": null
}
```

#### 向线程添加消息

```bash
POST /v1/threads/{thread_id}/messages
Content-Type: application/json

{
  "role": "user",
  "content": "你能做什么？"
}
```

#### 生成响应

```bash
POST /v1/responses
Content-Type: application/json

{
  "thread_id": "thread-uuid-123",
  "model": "qwen3-max",
  "file_ids": ["file-uuid-456"],  // 可选，用于多模态
  "proxy": "http://proxy:port"     // 可选
}
```

**响应：**
```json
{
  "response": "我可以帮助您完成各种任务...",
  "extra_data": { ... }
}
```

#### 上传文件（用于 Qwen 多模态）

```bash
POST /v1/files/upload
Content-Type: multipart/form-data

file: <二进制文件数据>
```

**响应：**
```json
{
  "id": "file-uuid-456",
  "filename": "image.jpg",
  "type": "image/jpeg"
}
```

#### 生成图片

```bash
POST /v1/images/generate
Content-Type: application/json

{
  "prompt": "一只可爱的橙色小猫",
  "size": "1:1",           // 选项：1:1, 16:9, 9:16
  "model": "qwen3-max",
  "download": true         // 自动下载到 ./generated/
}
```

**响应：**
```json
{
  "image_url": "https://...",
  "local_path": "./generated/images/1234567890.png"
}
```

#### 生成视频

```bash
POST /v1/videos/generate
Content-Type: application/json

{
  "prompt": "一只小猫在草地上玩耍",
  "size": "16:9",          // 选项：16:9, 9:16
  "model": "qwen3-max",
  "download": true         // 自动下载到 ./generated/
}
```

**响应：**
```json
{
  "video_url": "https://...",
  "local_path": "./generated/videos/1234567890.mp4"
}
```

#### 列出线程

```bash
GET /v1/threads
```

#### 获取线程详情

```bash
GET /v1/threads/{thread_id}
```

#### 删除线程

```bash
DELETE /v1/threads/{thread_id}
```

#### 列出线程消息

```bash
GET /v1/threads/{thread_id}/messages
```

## 💡 使用示例

### 示例 1：使用 DeepSeek 进行简单对话

```bash
# 创建线程
THREAD_ID=$(curl -s -X POST http://localhost:6969/v1/threads \
  -H "Content-Type: application/json" \
  -d '{"model": "deepseek-r1"}' | jq -r '.id')

# 添加消息
curl -s -X POST http://localhost:6969/v1/threads/$THREAD_ID/messages \
  -H "Content-Type: application/json" \
  -d '{"role": "user", "content": "用一句话解释量子计算"}'

# 获取响应
curl -s -X POST http://localhost:6969/v1/responses \
  -H "Content-Type: application/json" \
  -d "{\"thread_id\": \"$THREAD_ID\", \"model\": \"deepseek-r1\"}"
```

### 示例 2：使用 Qwen 进行图像识别

```bash
# 上传图片
FILE_ID=$(curl -s -X POST http://localhost:6969/v1/files/upload \
  -F "file=@test_image.jpg" | jq -r '.id')

# 创建线程
THREAD_ID=$(curl -s -X POST http://localhost:6969/v1/threads \
  -H "Content-Type: application/json" \
  -d '{"model": "qwen3-max"}' | jq -r '.id')

# 添加消息
curl -s -X POST http://localhost:6969/v1/threads/$THREAD_ID/messages \
  -H "Content-Type: application/json" \
  -d '{"role": "user", "content": "请描述这张图片"}'

# 获取带文件的响应
curl -s -X POST http://localhost:6969/v1/responses \
  -H "Content-Type: application/json" \
  -d "{\"thread_id\": \"$THREAD_ID\", \"model\": \"qwen3-max\", \"file_ids\": [\"$FILE_ID\"]}"
```

### 示例 3：生成图片

```bash
curl -s -X POST http://localhost:6969/v1/images/generate \
  -H "Content-Type: application/json" \
  -d '{
    "prompt": "海洋上美丽的日落",
    "size": "16:9",
    "model": "qwen3-max",
    "download": true
  }'
```

### 示例 4：代码调用

#### Rust 示例

```rust
use reverse_api::qwen::client::qwen::QwenClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 使用 token 创建客户端
    let client = QwenClient::with_token("your_token")?;
    
    // 开始对话
    let response = client.start_convo("你好，最近怎么样？", None, None).await?;
    println!("响应：{}", response.response.unwrap_or_default());
    
    Ok(())
}
```

#### Python 示例（使用 requests）

```python
import requests

BASE_URL = "http://localhost:6969"

# 创建线程
response = requests.post(f"{BASE_URL}/v1/threads", json={
    "model": "qwen3-max"
})
thread_id = response.json()["id"]

# 添加消息
requests.post(f"{BASE_URL}/v1/threads/{thread_id}/messages", json={
    "role": "user",
    "content": "你好，世界！"
})

# 获取响应
response = requests.post(f"{BASE_URL}/v1/responses", json={
    "thread_id": thread_id,
    "model": "qwen3-max"
})
print(response.json()["response"])
```

#### JavaScript 示例（使用 fetch）

```javascript
const BASE_URL = 'http://localhost:6969';

async function chat(message) {
  // 创建线程
  const threadRes = await fetch(`${BASE_URL}/v1/threads`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ model: 'qwen3-max' })
  });
  const { id: threadId } = await threadRes.json();
  
  // 添加消息
  await fetch(`${BASE_URL}/v1/threads/${threadId}/messages`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ role: 'user', content: message })
  });
  
  // 获取响应
  const responseRes = await fetch(`${BASE_URL}/v1/responses`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ thread_id: threadId, model: 'qwen3-max' })
  });
  const { response } = await responseRes.json();
  return response;
}

chat('你好！').then(console.log);
```

## 🤖 支持的模型

| 提供商 | 模型 ID | 功能 | 多模态 |
|----------|----------|--------------|------------|
| XAI | `grok-3-auto` | 文本生成 | ❌ |
| XAI | `grok-3-turbo` | 快速文本生成 | ❌ |
| XAI | `grok-3-mini` | 轻量级模型 | ❌ |
| OpenAI | `chatgpt` | 文本生成 | ❌ |
| OpenAI | `gpt-4` | 高级推理 | ❌ |
| DeepSeek | `deepseek-r1` | 推理模型 | ❌ |
| DeepSeek | `deepseek-chat` | 通用对话 | ❌ |
| 阿里巴巴 | `qwen3-max` | 高级多模态 | ✅ |
| 阿里巴巴 | `qwen3-plus` | 增强模型 | ✅ |
| 阿里巴巴 | `qwen3-turbo` | 快速模型 | ✅ |
| Z.ai | `glm-4.6` | 文本生成 | ❌ |

### Qwen 多模态支持

Qwen 模型支持以下文件类型：

- **图片**：JPG、PNG、GIF、WebP、BMP
- **文档**：PDF、TXT、DOC、DOCX
- **音频**：MP3、WAV、AAC、M4A
- **视频**：MP4、AVI、MOV、MKV

## 📁 项目结构

```
reverse-api/
├── src/
│   ├── bin/
│   │   ├── api_server.rs      # 主服务器程序
│   │   └── api/               # API 模块
│   │       ├── server.rs      # 服务器设置
│   │       ├── handlers.rs    # 请求处理器
│   │       ├── state.rs       # 应用状态
│   │       ├── dashboard.rs   # Web 仪表板
│   │       └── docs.rs        # API 文档
│   ├── chatgpt/               # ChatGPT 客户端
│   ├── grok/                  # Grok 客户端
│   ├── deepseek/              # DeepSeek 客户端
│   ├── qwen/                  # Qwen 客户端（多模态）
│   └── zto/                   # ZTO 客户端
├── examples/                  # 使用示例
├── generated/                 # 自动生成的媒体文件
│   ├── images/
│   └── videos/
├── Cargo.toml                 # Rust 依赖
└── README_CN.md               # 本文件
```

## 🔧 配置

### 命令行选项

```bash
api_server [OPTIONS]

选项：
  --host <HOST>      服务器主机（默认：0.0.0.0）
  --port <PORT>      服务器端口（默认：6969）
  --proxy <PROXY>    默认代理（默认：http://127.0.0.1:1082）
  --no-proxy         不使用任何默认代理
  --help             显示帮助信息
```

### 环境变量

- `API_HOST`：服务器主机（默认：0.0.0.0）
- `API_PORT`：服务器端口（默认：6969）
- `DEFAULT_PROXY`：默认代理 URL
- `DEEPSEEK_TOKEN`：DeepSeek 认证 token
- `QWEN_TOKEN`：Qwen 认证 token

### Token 文件

您也可以将 token 存储在文件中：
- `.deepseek_token` - DeepSeek token
- `.qwen_token` - Qwen token

## 🌐 仪表板和文档

服务器运行后，您可以访问：

- **API 文档**：http://localhost:6969/docs
- **仪表板**：http://localhost:6969/dashboard
  - 查看统计信息
  - 监控活动线程
  - 跟踪请求

## 🛠️ 开发

### 运行示例

```bash
# DeepSeek 示例
DEEPSEEK_TOKEN="your_token" cargo run --example deepseek_example

# Qwen 基础示例
QWEN_TOKEN="your_token" cargo run --example qwen_example

# Qwen 多模态示例
QWEN_TOKEN="your_token" cargo run --example qwen_multimodal_example

# Qwen 图片生成示例
QWEN_TOKEN="your_token" cargo run --example qwen_image_generation_example

# Grok 示例（需要代理）
cargo run --example grok_example
```

### 运行测试

```bash
cargo test
```

### 生产环境构建

```bash
cargo build --release
```

编译后的二进制文件位于 `./target/release/api_server`

## 🐛 故障排除

### 网络连接问题

如果遇到网络问题：

1. **使用代理**：设置 `--proxy` 参数或 `DEFAULT_PROXY` 环境变量
2. **检查防火墙**：确保防火墙允许出站连接
3. **验证 token**：确保您的 token 有效且未过期

### Token 不工作

1. **DeepSeek**：从 https://chat.deepseek.com/ 获取新的 token
   - 登录 → F12 → Application → LocalStorage → `userToken`
2. **Qwen**：从 https://chat.qwen.ai/ 获取新的 token
   - 登录 → F12 → Application → Cookies → `token`

### 端口已被占用

如果端口 6969 已被占用：

```bash
api_server --port 8080
```

### 代理问题

如果遇到代理连接问题：

```bash
# 尝试不使用代理
api_server --no-proxy

# 或使用不同的代理
api_server --proxy http://127.0.0.1:7890
```

### 视频生成时间过长

视频生成通常需要 1-3 分钟。这是正常的，因为视频渲染比较复杂。

## 📝 贡献

欢迎贡献！请随时提交 Pull Request。

1. Fork 仓库
2. 创建您的特性分支（`git checkout -b feature/amazing-feature`）
3. 提交您的更改（`git commit -m '添加一些很棒的功能'`）
4. 推送到分支（`git push origin feature/amazing-feature`）
5. 打开一个 Pull Request

## 📄 许可证

本项目采用 MIT 许可证 - 详见 LICENSE 文件。

## 🙏 致谢

- [rquest](https://github.com/penumbra-x/rquest) - 带有浏览器模拟的 HTTP 客户端
- [axum](https://github.com/tokio-rs/axum) - Web 框架
- [tokio](https://github.com/tokio-rs/tokio) - 异步运行时

## 📞 支持

如有问题、疑问或建议：
- 在 [GitHub](https://github.com/map-A/reverse-api/issues) 上开一个 issue
- 服务器运行时查看 [API 文档](http://localhost:6969/docs)

---

**注意**：这是一个用于教育目的的逆向工程 API 包装器。使用此工具时，请确保遵守各个 AI 提供商的服务条款。
