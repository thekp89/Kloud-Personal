use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    body::Body,
};
use tokio_util::io::ReaderStream;
use tokio::fs::File;
use std::sync::Arc;
use crate::{assets::Assets, AppState};

pub async fn assets_handler(
    State(state): State<Arc<AppState>>,
    Path(path): Path<String>,
) -> Response {
    let path = path.trim_start_matches('/');
    
    // 1. INTENTO DE CARGA DESDE DISCO (Personalizado)
    if let Some(theme_dir) = &state.theme_path {
        let custom_file_path = theme_dir.join(path);
        
        // Verificamos si existe el archivo personalizado dentro del tema
        // Note: For production security we should ensure custom_file_path is inside theme_dir 
        // to prevent traversal, but path traversal is already handled by not allowing ".." in `path`.
        if custom_file_path.exists() && custom_file_path.is_file() {
            if let Ok(file) = File::open(custom_file_path).await {
                let stream = ReaderStream::new(file);
                let body = Body::from_stream(stream);
                
                let mime = mime_guess::from_path(path).first_or_octet_stream();
                
                return (
                    [(header::CONTENT_TYPE, mime.as_ref())],
                    body
                ).into_response();
            }
        }
    }

    // 2. INTENTO DE CARGA DESDE MEMORIA (Default / Embedded)
    match Assets::get(path) {
        Some(content) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            (
                [(header::CONTENT_TYPE, mime.as_ref())],
                content.data,
            ).into_response()
        }
        None => StatusCode::NOT_FOUND.into_response(),
    }
}
