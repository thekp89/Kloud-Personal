use crate::{error::AppError, utils::html, AppState, assets::Assets};
use axum::{
    extract::{Path, State, Query},
    response::{Html, IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct ListParams {
    format: Option<String>, // "json" or empty
    mode: Option<String>,   // "legacy" or empty
}

#[derive(Serialize)]
struct FileEntry {
    name: String,
    is_dir: bool,
    size: u64,
}

#[derive(Serialize)]
struct DirectoryListing {
    current_path: String,
    entries: Vec<FileEntry>,
}

// Redirige "/" a "/list/"
pub async fn root_handler() -> axum::response::Redirect {
    axum::response::Redirect::to("/list/")
}

// Muestra el contenido del directorio
pub async fn list_handler(
    State(state): State<Arc<AppState>>,
    path: Option<Path<String>>,
    Query(params): Query<ListParams>,
) -> Result<Response, AppError> { // Changed return type to Response to allow mix of Html and Json
    // Manejar caso donde path es None (la raíz /list/)
    let req_path = path.map(|p| p.0).unwrap_or_default();
    
    // Saneamiento básico
    if req_path.contains("..") {
        return Err(AppError::InvalidPath);
    }

    let full_path = state.base_path.join(req_path.trim_start_matches('/'));

    if !full_path.exists() {
        return Err(AppError::NotFound);
    }

    // Leemos el directorio
    let mut entries = tokio::fs::read_dir(&full_path).await?;
    
    // Recopilamos items
    let mut items = Vec::new();
    while let Some(entry) = entries.next_entry().await? {
        let metadata = entry.metadata().await?;
        let name = entry.file_name().to_string_lossy().to_string();
        // Ignorar archivos ocultos unix
        if name.starts_with('.') { continue; }
        items.push((name, metadata.is_dir(), metadata.len()));
    }

    // Ordenar: Carpetas primero, luego alfabético
    items.sort_by(|a, b| {
        b.1.cmp(&a.1).then(a.0.cmp(&b.0))
    });

    // Check format
    if params.format.as_deref() == Some("json") {
        let file_entries: Vec<FileEntry> = items.into_iter().map(|(name, is_dir, size)| FileEntry {
            name,
            is_dir,
            size,
        }).collect();

        let listing = DirectoryListing {
            current_path: if req_path.is_empty() { "/".to_string() } else { format!("/{}", req_path) },
            entries: file_entries,
        };

        return Ok(Json(listing).into_response());
    }

    // Check Mode
    if params.mode.as_deref() == Some("legacy") {
        let html = html::generate_file_list_html(items, &req_path, state.max_upload_size);
        return Ok(Html(html).into_response());
    }

    // Modern Mode (Default)
    // 1. Get DirectoryListing struct for injection
    let file_entries: Vec<FileEntry> = items.into_iter().map(|(name, is_dir, size)| FileEntry {
        name,
        is_dir,
        size,
    }).collect();

    let listing = DirectoryListing {
        current_path: if req_path.is_empty() { "/".to_string() } else { format!("/{}", req_path) },
        entries: file_entries,
    };
    
    let initial_data_json = serde_json::to_string(&listing).unwrap_or_default();

    // 2. Load index.html (Disk First -> Embedded Fallback)
    let index_content = if let Some(theme_dir) = &state.theme_path {
        let custom_path = theme_dir.join("index.html");
        if custom_path.exists() {
             tokio::fs::read_to_string(custom_path).await
                .map_err(|e| AppError::InternalServerError(anyhow::anyhow!("Error reading custom index.html: {}", e)))?
        } else {
             let index_file = Assets::get("index.html").ok_or(AppError::NotFound)?;
             std::str::from_utf8(index_file.data.as_ref())
                .map_err(|e| AppError::InternalServerError(anyhow::anyhow!("Asset encoding error: {}", e)))?
                .to_string()
        }
    } else {
         let index_file = Assets::get("index.html").ok_or(AppError::NotFound)?;
         std::str::from_utf8(index_file.data.as_ref())
            .map_err(|e| AppError::InternalServerError(anyhow::anyhow!("Asset encoding error: {}", e)))?
            .to_string()
    };

    // 3. Inject Data
    // Note: We used `__INITIAL_DATA__` placeholder in `index.html`. 
    // We should strictly replace it. Since it's a script variable, we don't need quotes if we just replace the identifier, 
    // BUT serde_json stringifies it, so it becomes a string or object.
    // In JS: `window.INITIAL_DATA = __INITIAL_DATA__;`
    // If we replace `__INITIAL_DATA__` with `{"entries":...}`, it becomes `window.INITIAL_DATA = {"entries":...};` -> Valid JS.
    
    let html_string = index_content.replace("__INITIAL_DATA__", &initial_data_json);

    Ok(Html(html_string).into_response())
}