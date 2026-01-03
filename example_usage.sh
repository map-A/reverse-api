#!/bin/bash

# 统一 API 使用示例
# 启动服务器: ./target/release/api_server 

API_URL="http://localhost:6969"


echo "=================================================="
echo "统一逆向 API 使用示例"
echo "=================================================="
echo ""

# 配置 tokens
echo "📝 配置 API Tokens..."
# DEEPSEEK_TOKEN=$(cat .deepseek_token 2>/dev/null)
QWEN_TOKEN=$(cat .qwen_token 2>/dev/null)

# if [ -n "$DEEPSEEK_TOKEN" ]; then
#     curl -s -X POST "$API_URL/v1/config/deepseek" \
#         -H "Content-Type: application/json" \
#         -d "{\"token\": \"$DEEPSEEK_TOKEN\"}" > /dev/null
#     echo "✅ DeepSeek token 已配置"
# fi

if [ -n "$QWEN_TOKEN" ]; then
    curl -s -X POST "$API_URL/v1/config/qwen" \
        -H "Content-Type: application/json" \
        -d "{\"token\": \"$QWEN_TOKEN\"}" > /dev/null
    echo "✅ Qwen token 已配置"
fi
echo ""

# 示例 1: Qwen 对话
echo "=== 示例 2: Qwen 对话 ==="
THREAD=$(curl -s -X POST "$API_URL/v1/threads" \
    -H "Content-Type: application/json" \
    -d '{"model": "qwen3-max"}' | jq -r '.id')

curl -s -X POST "$API_URL/v1/threads/$THREAD/messages" \
    -H "Content-Type: application/json" \
    -d '{"role": "user", "content": "推荐三本科幻小说"}' > /dev/null

echo "问题: 推荐三本科幻小说"
echo "回答:"
curl -s -X POST "$API_URL/v1/responses" \
    -H "Content-Type: application/json" \
    -d "{\"thread_id\": \"$THREAD\", \"model\": \"qwen3-max\"}" | jq -r '.response'
echo ""

# 示例 3: Qwen 多模态 - 图片识别
if [ -f "test_image.jpg" ]; then
    echo "=== 示例 3: Qwen 多模态 (图片识别) ==="
    FILE_ID=$(curl -s -X POST "$API_URL/v1/files/upload" -F "file=@test_image.jpg" | jq -r '.id')
    echo "✅ 图片已上传: $FILE_ID"
    
    THREAD=$(curl -s -X POST "$API_URL/v1/threads" \
        -H "Content-Type: application/json" \
        -d '{"model": "qwen3-max"}' | jq -r '.id')
    
    curl -s -X POST "$API_URL/v1/threads/$THREAD/messages" \
        -H "Content-Type: application/json" \
        -d '{"role": "user", "content": "请描述这张图片"}' > /dev/null
    
    echo "问题: 请描述这张图片"
    echo "回答:"
    curl -s -X POST "$API_URL/v1/responses" \
        -H "Content-Type: application/json" \
        -d "{\"thread_id\": \"$THREAD\", \"model\": \"qwen3-max\", \"file_ids\": [\"$FILE_ID\"]}" | jq -r '.response'
    echo ""
fi

# 示例 2: Qwen 图片生成（自动下载）
echo "=== 示例 2: Qwen 图片生成（自动下载） ==="
echo "提示词: 一只可爱的橙色小猫"
IMAGE_RESULT=$(curl -s -X POST "$API_URL/v1/images/generate" \
    -H "Content-Type: application/json" \
    -d '{"prompt": "一只可爱的橙色小猫", "size": "1:1", "model": "qwen3-max", "download": true}')

IMAGE_URL=$(echo "$IMAGE_RESULT" | jq -r '.image_url')
LOCAL_PATH=$(echo "$IMAGE_RESULT" | jq -r '.local_path')
echo "✅ 图片已生成并保存"
echo "🖼️  图片 URL: ${IMAGE_URL:0:80}..."
if [ "$LOCAL_PATH" != "null" ]; then
    echo "💾 本地路径: $LOCAL_PATH"
fi
echo ""

# 示例 3: Qwen 视频生成（自动下载）
echo "=== 示例 3: Qwen 视频生成（自动下载） ==="
echo "提示词: 一只小猫在草地上玩耍"
echo "⏳ 视频生成需要 1-3 分钟，请稍候..."
VIDEO_RESULT=$(curl -s -X POST "$API_URL/v1/videos/generate" \
    -H "Content-Type: application/json" \
    -d '{"prompt": "一只小猫在草地上玩耍", "size": "16:9", "model": "qwen3-max", "download": true}')

VIDEO_URL=$(echo "$VIDEO_RESULT" | jq -r '.video_url')
LOCAL_PATH=$(echo "$VIDEO_RESULT" | jq -r '.local_path')
echo "✅ 视频已生成并保存"
echo "🎬 视频 URL: ${VIDEO_URL:0:80}..."
if [ "$LOCAL_PATH" != "null" ]; then
    echo "💾 本地路径: $LOCAL_PATH"
fi
echo ""

# 示例 4: 查看可用模型
echo "=== 示例 4: 查看所有可用模型 ==="
curl -s "$API_URL/v1/models" | jq -r '.data[] | "  - \(.id) (\(.owned_by))"'
echo ""

echo "=================================================="
echo "✅ 所有示例执行完成"
echo "=================================================="
echo ""
echo "💡 提示："
echo "  • 多模态功能支持图片、文档、音频、视频"
echo "  • 设置 download=true 自动下载生成的内容到 ./generated/ 目录"
echo "  • 使用 file_ids 参数传递已上传的文件"
echo "  • 视频生成需要 1-3 分钟时间"
echo "  • 更多文档请访问: http://localhost:6969/docs"
