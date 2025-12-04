use crate::{error::AppError, utils::html, AppState};
use axum::{
    extract::{Path, State},
    response::Html,
};
use std::sync::Arc;

// Redirige "/" a "/list/"
pub async fn root_handler() -> axum::response::Redirect {
    axum::response::Redirect::to("/list/")
}

// Muestra el contenido del directorio
pub async fn list_handler(
    State(state): State<Arc<AppState>>,
    path: Option<Path<String>>,
) -> Result<Html<String>, AppError> {
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
    
    // Construimos HTML simple a mano (sin plantillas para no añadir dependencias extra)
    // Recopilamos items para ordenarlos (directorios primero)
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

    let html = html::generate_file_list_html(items, &req_path, state.max_upload_size);

    Ok(Html(html))
}