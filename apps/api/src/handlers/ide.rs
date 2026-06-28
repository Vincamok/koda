use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response, Sse},
    Extension, Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

use crate::{
    ai::{
        context_builder::{builtin_framework_pack, builtin_lang_pack, AiContextBuilder},
        provider::ChatMessage,
    },
    error::AppError,
    middleware::auth::{AuthUser, OrgContext},
    AppState,
};

fn is_secret_file(path: &str) -> bool {
    let lower = path.to_lowercase();
    let name = lower.rsplit('/').next().unwrap_or(&lower);
    name == ".env"
        || name.starts_with(".env.")
        || name.ends_with(".key")
        || name.ends_with(".pem")
        || name.ends_with(".p12")
        || name.ends_with(".pfx")
        || name == ".netrc"
        || name == "id_rsa"
        || name == "id_ed25519"
        || name.contains("secret")
        || name.contains("credential")
        || name.contains("password")
}

fn detect_packs_from_extension(file_path: &str) -> (Vec<String>, Vec<String>) {
    let ext = file_path.rsplit('.').next().unwrap_or("").to_lowercase();
    match ext.as_str() {
        "rs" => (vec!["rust".into()], vec!["axum".into(), "sqlx".into()]),
        "ts" | "tsx" => (vec!["typescript".into()], vec!["react".into(), "nextjs".into()]),
        "py" => (vec!["python".into()], vec![]),
        "go" => (vec!["go".into()], vec![]),
        "sql" => (vec!["sql".into()], vec![]),
        _ => (vec![], vec![]),
    }
}

// ── File browser ──────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct FileNode {
    pub name: String,
    pub path: String,
    #[serde(rename = "type")]
    pub node_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<FileNode>>,
}

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{org_id}/workspaces/{workspace_id}/files",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID"),
        ("workspace_id" = Uuid, Path, description = "Workspace ID"),
    ),
    responses(
        (status = 200, description = "File tree", body = Vec<FileNode>),
        (status = 404, description = "Workspace not found"),
    ),
    tag = "ide",
    security(("session" = []))
)]
pub async fn get_workspace_files(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Extension(org): Extension<OrgContext>,
    Path((_org_id, workspace_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    // Verify workspace belongs to org
    let ws = sqlx::query!(
        "SELECT id, uid FROM workspaces WHERE id = $1 AND organization_id = $2",
        workspace_id,
        org.id,
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::NotFound)?;

    // In Phase 2 this will query the actual volume via the git-manager API.
    // For now return a placeholder tree.
    let _ = ws;
    let placeholder: Vec<FileNode> = vec![
        FileNode {
            name: "src".into(),
            path: "src".into(),
            node_type: "dir".into(),
            children: Some(vec![
                FileNode { name: "main.rs".into(), path: "src/main.rs".into(), node_type: "file".into(), children: None },
                FileNode { name: "lib.rs".into(), path: "src/lib.rs".into(), node_type: "file".into(), children: None },
            ]),
        },
        FileNode { name: "Cargo.toml".into(), path: "Cargo.toml".into(), node_type: "file".into(), children: None },
        FileNode { name: "README.md".into(), path: "README.md".into(), node_type: "file".into(), children: None },
    ];

    Ok(Json(json!({ "data": placeholder })))
}

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{org_id}/workspaces/{workspace_id}/files/{file_path}",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID"),
        ("workspace_id" = Uuid, Path, description = "Workspace ID"),
        ("file_path" = String, Path, description = "Path to file within workspace"),
    ),
    responses(
        (status = 200, description = "File content"),
        (status = 404, description = "Not found"),
    ),
    tag = "ide",
    security(("session" = []))
)]
pub async fn get_workspace_file_content(
    State(state): State<AppState>,
    Extension(_auth): Extension<AuthUser>,
    Extension(org): Extension<OrgContext>,
    Path((_org_id, workspace_id, file_path)): Path<(Uuid, Uuid, String)>,
) -> Result<impl IntoResponse, AppError> {
    sqlx::query!(
        "SELECT id FROM workspaces WHERE id = $1 AND organization_id = $2",
        workspace_id,
        org.id,
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::NotFound)?;

    // Phase 2: read from volume. Stub returns placeholder content.
    Ok(Json(json!({
        "data": {
            "path": file_path,
            "content": "// File content will be served from workspace volume in Phase 2\n"
        }
    })))
}

// ── AI Chat SSE ───────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct AiChatRequest {
    pub message: String,
    pub context: Option<AiChatContext>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct AiChatContext {
    pub file_path: Option<String>,
    pub file_content: Option<String>,
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{org_id}/workspaces/{workspace_id}/ai/chat",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID"),
        ("workspace_id" = Uuid, Path, description = "Workspace ID"),
    ),
    request_body = AiChatRequest,
    responses(
        (status = 200, description = "SSE stream of AI response chunks"),
        (status = 404, description = "Workspace not found"),
    ),
    tag = "ide",
    security(("session" = []))
)]
pub async fn post_workspace_ai_chat(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Extension(org): Extension<OrgContext>,
    Path((_org_id, workspace_id)): Path<(Uuid, Uuid)>,
    Json(body): Json<AiChatRequest>,
) -> Result<Response, AppError> {
    use futures::StreamExt;

    // Verify workspace membership
    sqlx::query!(
        "SELECT id FROM workspaces WHERE id = $1 AND organization_id = $2",
        workspace_id,
        org.id,
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::NotFound)?;

    // Fetch user settings for locale
    let settings = sqlx::query!(
        "SELECT locale FROM user_settings WHERE user_id = $1",
        auth.id
    )
    .fetch_optional(&state.pool)
    .await?;
    let locale = settings.map(|s| s.locale).unwrap_or_else(|| "fr".into());

    // Fetch org-level KODA.md if it exists (placeholder)
    let koda_md: Option<String> = None;

    // Fetch active MCP bindings + their tool definitions for this workspace
    let mcp_tools_context: Option<String> = {
        let bindings = sqlx::query!(
            r#"SELECT cd.name AS connector_name, cd.tools AS connector_tools
               FROM workspace_mcp_bindings wb
               JOIN mcp_connector_definitions cd ON cd.id = wb.connector_definition_id
               WHERE wb.workspace_id = $1 AND wb.enabled = TRUE"#,
            workspace_id,
        )
        .fetch_all(&state.pool)
        .await
        .unwrap_or_default();

        if bindings.is_empty() {
            None
        } else {
            let tools_doc = bindings
                .into_iter()
                .filter_map(|b| {
                    let tools = b.connector_tools?;
                    if tools.as_array().map(|a| a.is_empty()).unwrap_or(true) {
                        return None;
                    }
                    Some(format!(
                        "## MCP Connector: {}\nAvailable tools:\n```json\n{}\n```",
                        b.connector_name,
                        serde_json::to_string_pretty(&tools).unwrap_or_default()
                    ))
                })
                .collect::<Vec<_>>()
                .join("\n\n");

            if tools_doc.is_empty() {
                None
            } else {
                Some(format!(
                    "You have access to the following external tool connectors via MCP:\n\n{tools_doc}\n\nWhen the user asks to use a tool, describe which connector and tool you would invoke."
                ))
            }
        }
    };

    // Secret filter — never forward sensitive file contents to LLM
    let safe_context = body.context.as_ref().and_then(|ctx| {
        match (&ctx.file_path, &ctx.file_content) {
            (Some(path), Some(content)) if !is_secret_file(path) => {
                Some((path.clone(), content.clone()))
            }
            _ => None,
        }
    });

    // Auto-detect lang/framework packs from current file extension
    let (lang_names, fw_names) = safe_context
        .as_ref()
        .map(|(path, _)| detect_packs_from_extension(path))
        .unwrap_or_default();

    let lang_pack_content: Vec<String> = lang_names
        .iter()
        .filter_map(|l| builtin_lang_pack(l))
        .map(str::to_string)
        .collect();

    let fw_pack_content: Vec<String> = fw_names
        .iter()
        .filter_map(|f| builtin_framework_pack(f))
        .map(str::to_string)
        .collect();

    // Build context using AiContextBuilder
    let mut builder = AiContextBuilder::new()
        .locale(&locale)
        .lang_packs(lang_pack_content)
        .framework_packs(fw_pack_content);

    if let Some(km) = koda_md {
        builder = builder.koda_md(&km);
    }

    if let Some(mcp_ctx) = mcp_tools_context {
        builder = builder.personal_instructions(&mcp_ctx);
    }

    // Assemble user message with (safe) file context
    let mut user_message = body.message.clone();
    if let Some((path, content)) = safe_context {
        if !content.is_empty() {
            user_message = format!(
                "Current file: {path}\n```\n{}\n```\n\n{user_message}",
                content.chars().take(8000).collect::<String>(),
            );
        }
    }

    let context = builder.build(
        vec![ChatMessage { role: "user".into(), content: user_message }],
        vec![],
    );

    let adapter = state.config.ai.build_adapter(&state.http)
        .map_err(|e| AppError::Internal(e))?;

    let stream = adapter
        .chat_stream(context)
        .await
        .map_err(|e| AppError::Internal(e))?;

    let sse_stream = stream.map(|event| {
        match event {
            Ok(text) => {
                let data = json!({ "delta": { "text": text } }).to_string();
                Ok::<_, std::convert::Infallible>(axum::response::sse::Event::default().data(data))
            }
            Err(_) => Ok(axum::response::sse::Event::default().data("[DONE]")),
        }
    });

    Ok(Sse::new(sse_stream).into_response())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn secret_file_detection() {
        assert!(is_secret_file(".env"));
        assert!(is_secret_file(".env.local"));
        assert!(is_secret_file(".env.production"));
        assert!(is_secret_file("private.key"));
        assert!(is_secret_file("cert.pem"));
        assert!(is_secret_file("bundle.p12"));
        assert!(is_secret_file("store.pfx"));
        assert!(is_secret_file(".netrc"));
        assert!(is_secret_file("id_rsa"));
        assert!(is_secret_file("id_ed25519"));
        assert!(is_secret_file("api_secret.json"));
        assert!(is_secret_file("db_password.txt"));
        assert!(is_secret_file("credentials.json"));
    }

    #[test]
    fn non_secret_files_not_filtered() {
        assert!(!is_secret_file("main.rs"));
        assert!(!is_secret_file("config.yaml"));
        assert!(!is_secret_file("README.md"));
        assert!(!is_secret_file("Cargo.toml"));
        assert!(!is_secret_file("src/lib.rs"));
        assert!(!is_secret_file("env_utils.rs")); // contains "env" but not .env
    }

    #[test]
    fn pack_detection_from_extension() {
        let (langs, fw) = detect_packs_from_extension("src/main.rs");
        assert_eq!(langs, vec!["rust"]);
        assert!(fw.contains(&"axum".to_string()));

        let (langs, fw) = detect_packs_from_extension("app/page.tsx");
        assert_eq!(langs, vec!["typescript"]);
        assert!(fw.contains(&"react".to_string()));
        assert!(fw.contains(&"nextjs".to_string()));

        let (langs, _fw) = detect_packs_from_extension("main.py");
        assert_eq!(langs, vec!["python"]);

        let (langs, _fw) = detect_packs_from_extension("main.go");
        assert_eq!(langs, vec!["go"]);

        let (langs, _fw) = detect_packs_from_extension("schema.sql");
        assert_eq!(langs, vec!["sql"]);

        let (langs, fw) = detect_packs_from_extension("README.md");
        assert!(langs.is_empty());
        assert!(fw.is_empty());
    }
}
