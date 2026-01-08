use axum::{
    extract::{Multipart, State},
    response::{IntoResponse, Response as AxumResponse},
    Json,
};
use reverse_api::{Logger, QwenClient};

use super::error::ApiError;
use super::state::AppState;
use super::types::*;

pub async fn create_thread(
    State(state): State<AppState>,
    Json(payload): Json<CreateThreadRequest>,
) -> std::result::Result<AxumResponse, ApiError> {
    let start_time = std::time::Instant::now();
    Logger::info(&format!(
        "Creating new thread with {} initial messages",
        payload.messages.len()
    ));

    let (thread_id, thread_state) = state
        .create_thread(payload.messages, payload.metadata, &payload.model)
        .await?;

    let response = Thread {
        id: thread_id,
        object: "thread".to_string(),
        created_at: thread_state.created_at,
        metadata: thread_state.metadata,
    };

    state
        .record_request("POST", "/v1/threads", 200, start_time.elapsed(), "")
        .await;
    Ok(Json(response).into_response())
}

pub async fn get_thread(
    State(state): State<AppState>,
    axum::extract::Path(params): axum::extract::Path<ThreadPath>,
) -> std::result::Result<impl IntoResponse, ApiError> {
    let thread_id = params.thread_id;
    let thread_state = state.get_thread(&thread_id).await?;

    let response = Thread {
        id: thread_id,
        object: "thread".to_string(),
        created_at: thread_state.created_at,
        metadata: thread_state.metadata,
    };

    Ok(Json(response).into_response())
}

pub async fn list_threads(
    State(state): State<AppState>,
) -> std::result::Result<AxumResponse, ApiError> {
    let threads = state.list_threads().await;

    let data: Vec<Thread> = threads
        .into_iter()
        .map(|(id, state)| Thread {
            id,
            object: "thread".to_string(),
            created_at: state.created_at,
            metadata: state.metadata,
        })
        .collect();

    let response = ListThreadsResponse {
        object: "list".to_string(),
        data,
        has_more: false,
    };

    Ok(Json(response).into_response())
}

pub async fn delete_thread(
    State(state): State<AppState>,
    axum::extract::Path(params): axum::extract::Path<ThreadPath>,
) -> std::result::Result<AxumResponse, ApiError> {
    let thread_id = params.thread_id;
    state.delete_thread(&thread_id).await?;

    Ok(Json(serde_json::json!({
        "id": thread_id,
        "object": "thread.deleted",
        "deleted": true
    }))
    .into_response())
}

pub async fn add_message(
    State(state): State<AppState>,
    axum::extract::Path(params): axum::extract::Path<ThreadPath>,
    Json(payload): Json<AddMessageRequest>,
) -> std::result::Result<AxumResponse, ApiError> {
    let thread_id = params.thread_id;
    if payload.content.trim().is_empty() {
        return Err(ApiError::bad_request("Message content cannot be empty"));
    }

    state
        .add_message_to_thread(&thread_id, payload.role.clone(), payload.content.clone())
        .await?;

    let message_id = uuid::Uuid::new_v4().to_string();
    let created_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let response = Message {
        id: message_id,
        object: "thread.message".to_string(),
        created_at,
        thread_id,
        role: payload.role,
        content: vec![ContentPart {
            content_type: "text".to_string(),
            text: TextContent {
                value: payload.content,
                annotations: vec![],
            },
        }],
    };

    Ok(Json(response).into_response())
}

pub async fn list_messages(
    State(state): State<AppState>,
    axum::extract::Path(params): axum::extract::Path<ThreadPath>,
) -> std::result::Result<AxumResponse, ApiError> {
    let thread_id = params.thread_id;
    let thread_state = state.get_thread(&thread_id).await?;

    let data: Vec<Message> = thread_state
        .get_messages()
        .iter()
        .enumerate()
        .map(|(idx, msg)| Message {
            id: format!("msg_{}_{}", thread_id, idx),
            object: "thread.message".to_string(),
            created_at: msg.created_at.unwrap_or(thread_state.created_at),
            thread_id: thread_id.clone(),
            role: msg.role.clone(),
            content: vec![ContentPart {
                content_type: "text".to_string(),
                text: TextContent {
                    value: msg.content.clone(),
                    annotations: vec![],
                },
            }],
        })
        .collect();

    let response = ListMessagesResponse {
        object: "list".to_string(),
        data,
        has_more: false,
    };

    Ok(Json(response).into_response())
}

pub async fn configure_qwen(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> std::result::Result<AxumResponse, ApiError> {
    let token = payload["token"]
        .as_str()
        .ok_or_else(|| ApiError::bad_request("Missing 'token' field"))?;

    state.set_qwen_token(token.to_string()).await;

    // Try to fetch and cache models
    match QwenClient::with_token(token.to_string()) {
        Ok(client) => match client.get_models().await {
            Ok(models) => {
                Logger::info(&format!("✅ Fetched {} Qwen models", models.len()));
                state.set_qwen_models(models).await;
            }
            Err(e) => {
                Logger::info(&format!("⚠️  Could not fetch Qwen models: {}", e));
            }
        },
        Err(e) => {
            Logger::info(&format!("⚠️  Could not create Qwen client: {}", e));
        }
    }

    Ok(Json(serde_json::json!({
        "status": "success",
        "message": "Qwen token configured"
    }))
    .into_response())
}

pub async fn create_response(
    State(state): State<AppState>,
    Json(payload): Json<CreateResponseRequest>,
) -> std::result::Result<AxumResponse, ApiError> {
    let start_time = std::time::Instant::now();
    let thread_id = payload.thread_id.clone();

    Logger::info(&format!("Creating response for thread: {}", thread_id));

    let mut thread_state = state.get_thread(&thread_id).await?;

    let last_user_message = thread_state
        .get_messages()
        .iter()
        .rev()
        .find(|m| m.role == "user")
        .ok_or_else(|| ApiError::bad_request("No user message found in thread"))?;

    let message_content = last_user_message.content.clone();
    if message_content.trim().is_empty() {
        return Err(ApiError::bad_request("Last user message content is empty"));
    }

    let model = thread_state.model.clone();

    Logger::info(&format!("Using model: {}", model));

    // Determine which client to use based on model name
    let answer = if model.starts_with("qwen") {
        Logger::info("Starting Qwen conversation");

        // Acquire a client by rotating tokens: get next token and create a client for it
        let token = state
            .next_qwen_token()
            .await
            .ok_or_else(|| ApiError::bad_request("Qwen token not configured. Please configure it via POST /v1/config/qwen"))?;
        let client = reverse_api::QwenClient::with_token(token).map_err(|e| ApiError::internal_error(format!("Could not create Qwen client: {}", e)))?;
        // Check for special instructions
        let use_search = payload
            .instructions
            .as_ref()
            .map(|s| s.contains("search"))
            .unwrap_or(false);
        let use_thinking = payload
            .instructions
            .as_ref()
            .map(|s| s.contains("thinking"))
            .unwrap_or(false);

        // Build extra_data for continuous conversation
        let extra_data = if let (Some(chat_id), Some(parent_id)) =
            (&thread_state.qwen_chat_id, &thread_state.qwen_parent_id)
        {
            Some(reverse_api::qwen::models::ExtraData {
                chat_id: chat_id.clone(),
                model_id: model.clone(),
                parent_id: Some(parent_id.clone()),
            })
        } else {
            None
        };

        let result = if let Some(file_ids) = &payload.file_ids {
            if !file_ids.is_empty() {
                Logger::info(&format!("Using {} files with Qwen", file_ids.len()));

                // Get uploaded files from state
                let files = state.get_uploaded_files(file_ids).await;

                if files.is_empty() {
                    return Err(ApiError::bad_request(
                        "No valid files found for provided file_ids",
                    ));
                }

                // Execute conversation with files
                client
                    .start_convo_with_files(
                        &message_content,
                        files,
                        None, // Auto-select model
                        extra_data.as_ref(),
                    )
                    .await
                    .map_err(|e| {
                        ApiError::internal_error(format!("Qwen multimodal error: {}", e))
                    })?
            } else if use_search {
                Logger::info("Using Qwen with search");
                client
                    .start_convo_with_search(&message_content, Some(&model), extra_data.as_ref())
                    .await
                    .map_err(|e| ApiError::internal_error(format!("Qwen search error: {}", e)))?
            } else if use_thinking {
                Logger::info("Using Qwen with thinking");
                client
                    .start_convo_with_thinking(
                        &message_content,
                        Some(&model),
                        extra_data.as_ref(),
                        None, // Use default thinking budget
                    )
                    .await
                    .map_err(|e| ApiError::internal_error(format!("Qwen thinking error: {}", e)))?
            } else {
                // Normal text conversation
                Logger::info(&format!("Calling start_convo with model: {}", model));
                client
                    .start_convo(&message_content, Some(&model), extra_data.as_ref())
                    .await
                    .map_err(|e| ApiError::internal_error(format!("Qwen error: {}", e)))?
            }
        } else if use_search {
            Logger::info("Using Qwen with search");
            client
                .start_convo_with_search(&message_content, Some(&model), extra_data.as_ref())
                .await
                .map_err(|e| ApiError::internal_error(format!("Qwen search error: {}", e)))?
        } else if use_thinking {
            Logger::info("Using Qwen with thinking");
            client
                .start_convo_with_thinking(
                    &message_content,
                    Some(&model),
                    extra_data.as_ref(),
                    None, // Use default thinking budget
                )
                .await
                .map_err(|e| ApiError::internal_error(format!("Qwen thinking error: {}", e)))?
        } else {
            // Normal text conversation with context
            Logger::info(&format!("Calling start_convo, model: {}", model));
            client
                .start_convo(&message_content, Some(&model), extra_data.as_ref())
                .await
                .map_err(|e| ApiError::internal_error(format!("Qwen error: {}", e)))?
        };

        // Update thread state with Qwen session info for continuous conversation
        if let Some(chat_id) = &result.chat_id {
            thread_state.qwen_chat_id = Some(chat_id.clone());
            thread_state.qwen_parent_id = Some(result.response_id.clone());
        }

        result.content
    } else {
        return Err(ApiError::bad_request(format!(
            "Unsupported model: {}. Use 'qwen-*'",
            model
        )));
    };

    thread_state.add_message("assistant".to_string(), answer.clone());

    let model = thread_state.model.clone();
    state.update_thread(&thread_id, thread_state).await?;

    let response_id = uuid::Uuid::new_v4().to_string();
    let created_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let response = Response {
        id: response_id,
        object: "thread.response".to_string(),
        created_at,
        thread_id,
        status: "completed".to_string(),
        model,
        response: Some(answer),
    };

    state
        .record_request("POST", "/v1/responses", 200, start_time.elapsed(), "")
        .await;
    Ok(Json(response).into_response())
}

pub async fn upload_file_for_qwen(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> std::result::Result<AxumResponse, ApiError> {
    let token = state
        .next_qwen_token()
        .await
        .ok_or_else(|| ApiError::bad_request("Qwen token not configured. Please configure it via POST /v1/config/qwen"))?;
    let client = reverse_api::QwenClient::with_token(token)
        .map_err(|e| ApiError::internal_error(format!("Could not create Qwen client: {}", e)))?;

    let mut file_data: Option<(String, Vec<u8>)> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| ApiError::internal_error(format!("Multipart error: {}", e)))?
    {
        let field_name = field.name().unwrap_or("file").to_string();
        if field_name == "file" {
            let filename = field.file_name().unwrap_or("unknown").to_string();
            let data = field
                .bytes()
                .await
                .map_err(|e| ApiError::internal_error(format!("Failed to read file: {}", e)))?;

            file_data = Some((filename, data.to_vec()));
            break;
        }
    }

    let (filename, data) =
        file_data.ok_or_else(|| ApiError::bad_request("No file provided in multipart request"))?;

    // Save file temporarily
    let temp_path = format!("/tmp/{}", filename);
    std::fs::write(&temp_path, &data)
        .map_err(|e| ApiError::internal_error(format!("Failed to save file: {}", e)))?;

    // Upload to Qwen
    let file = client
        .upload_file(&temp_path)
        .await
        .map_err(|e| ApiError::internal_error(format!("File upload failed: {}", e)))?;

    // Clean up temp file
    let _ = std::fs::remove_file(&temp_path);

    // Store file in state for later use
    let file_id = state.store_uploaded_file(file.clone()).await;

    let response = FileUploadResponse {
        id: file_id,
        name: file.name.clone(),
        size: file.size,
        file_class: file.file_class.clone(),
    };

    Ok(Json(response).into_response())
}

pub async fn generate_image(
    State(state): State<AppState>,
    Json(payload): Json<GenerateImageRequest>,
) -> std::result::Result<AxumResponse, ApiError> {
    Logger::info(&format!("Generating image with prompt: {}", payload.prompt));

    let token = state
        .next_qwen_token()
        .await
        .ok_or_else(|| ApiError::bad_request("Qwen token not configured. Please configure it via POST /v1/config/qwen"))?;
    let client = reverse_api::QwenClient::with_token(token)
        .map_err(|e| ApiError::internal_error(format!("Could not create Qwen client: {}", e)))?;

    // Get thread state if provided for continuous generation
    let extra_data = if let Some(thread_id) = &payload.thread_id {
        let thread_state = state.get_thread(thread_id).await?;

        // Get chat_id and parent_id from thread
        if let Some(last_msg) = thread_state.messages.last() {
            // Parse metadata if it contains qwen session info
            Some(reverse_api::qwen::models::ExtraData {
                chat_id: thread_id.clone(),
                model_id: payload.model.clone(),
                parent_id: Some(last_msg.content.clone()),
            })
        } else {
            None
        }
    } else {
        None
    };

    // Generate image
    let result = client
        .generate_image(
            &payload.prompt,
            payload.size.as_deref(),
            Some(&payload.model),
            extra_data.as_ref(),
        )
        .await
        .map_err(|e| ApiError::internal_error(format!("Image generation failed: {}", e)))?;

    Logger::info(&format!("Image generated: {}", result.content));

    // Download image if requested
    let local_path = if payload.download {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let filename = format!("generated_image_{}.png", timestamp);
        let filepath = format!("./generated/{}", filename);

        // Create directory if it doesn't exist
        std::fs::create_dir_all("./generated")
            .map_err(|e| ApiError::internal_error(format!("Failed to create directory: {}", e)))?;

        Logger::info(&format!("Downloading image to: {}", filepath));
        client
            .download_media(&result.content, &filepath)
            .await
            .map_err(|e| ApiError::internal_error(format!("Failed to download image: {}", e)))?;

        Logger::info(&format!("Image saved to: {}", filepath));
        Some(filepath)
    } else {
        None
    };

    let response = GenerateImageResponse {
        image_url: result.content,
        prompt: payload.prompt,
        chat_id: result.chat_id,
        response_id: result.response_id,
        local_path,
    };

    Ok(Json(response).into_response())
}

pub async fn generate_video(
    State(state): State<AppState>,
    Json(payload): Json<GenerateVideoRequest>,
) -> std::result::Result<AxumResponse, ApiError> {
    Logger::info(&format!("Generating video with prompt: {}", payload.prompt));

    let token = state
        .next_qwen_token()
        .await
        .ok_or_else(|| ApiError::bad_request("Qwen token not configured. Please configure it via POST /v1/config/qwen"))?;
    let client = reverse_api::QwenClient::with_token(token)
        .map_err(|e| ApiError::internal_error(format!("Could not create Qwen client: {}", e)))?;

    // Get thread state if provided for continuous generation
    let extra_data = if let Some(thread_id) = &payload.thread_id {
        let thread_state = state.get_thread(thread_id).await?;

        // Get chat_id and parent_id from thread
        if let Some(last_msg) = thread_state.messages.last() {
            Some(reverse_api::qwen::models::ExtraData {
                chat_id: thread_id.clone(),
                model_id: payload.model.clone(),
                parent_id: Some(last_msg.content.clone()),
            })
        } else {
            None
        }
    } else {
        None
    };

    // Generate video with progress logging
    Logger::info("Starting video generation (this may take 1-3 minutes)...");
    let result = client
        .generate_video_with_progress(
            &payload.prompt,
            payload.size.as_deref(),
            Some(&payload.model),
            extra_data.as_ref(),
            |status, percent| {
                if percent % 20 == 0 || status == "success" {
                    Logger::info(&format!("Video generation: {} - {}%", status, percent));
                }
            },
        )
        .await
        .map_err(|e| ApiError::internal_error(format!("Video generation failed: {}", e)))?;

    Logger::info(&format!("Video generated: {}", result.content));

    // Download video if requested
    let local_path = if payload.download {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let filename = format!("generated_video_{}.mp4", timestamp);
        let filepath = format!("./generated/{}", filename);

        // Create directory if it doesn't exist
        std::fs::create_dir_all("./generated")
            .map_err(|e| ApiError::internal_error(format!("Failed to create directory: {}", e)))?;

        Logger::info(&format!("Downloading video to: {}", filepath));
        client
            .download_media(&result.content, &filepath)
            .await
            .map_err(|e| ApiError::internal_error(format!("Failed to download video: {}", e)))?;

        Logger::info(&format!("Video saved to: {}", filepath));
        Some(filepath)
    } else {
        None
    };

    let response = GenerateVideoResponse {
        video_url: result.content,
        prompt: payload.prompt,
        chat_id: result.chat_id,
        response_id: result.response_id,
        local_path,
    };

    Ok(Json(response).into_response())
}
