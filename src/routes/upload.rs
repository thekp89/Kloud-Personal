use crate::{error::AppError, AppState};
use axum::{
    extract::{Multipart, Query, State},
    response::IntoResponse,
    http::StatusCode,
};
use std::sync::Arc;

use tokio::fs::File;
use tokio::io::AsyncWriteExt;

#[derive(serde::Deserialize)]
pub struct UploadParams {
    path: String,
}

pub async fn upload_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<UploadParams>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
    // 1. Saneamiento de ruta base
    let clean_path = params.path.trim_start_matches('/');
    if clean_path.contains("..") {
        return Err(AppError::InvalidPath);
    }

    let target_dir = state.base_path.join(clean_path);
    if !target_dir.exists() {
        return Err(AppError::NotFound);
    }

    // 2. Procesar cada campo del multipart
    while let Some(field) = multipart.next_field().await.map_err(|e| AppError::InternalServerError(anyhow::anyhow!(e)))? {
        let file_name = if let Some(name) = field.file_name() {
            name.to_string()
        } else {
            continue; // Ignorar campos que no son archivos
        };

        // Saneamiento básico del nombre de archivo
        let file_name = std::path::Path::new(&file_name)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("uploaded_file");

        // 3. Manejo de colisiones
        let mut dest_path = target_dir.join(file_name);
        let mut counter = 1;
        let file_stem = std::path::Path::new(file_name)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(file_name);
        let extension = std::path::Path::new(file_name)
            .extension()
            .and_then(|s| s.to_str())
            .map(|e| format!(".{}", e))
            .unwrap_or_default();

        while dest_path.exists() {
            let new_name = format!("{}({}){}", file_stem, counter, extension);
            dest_path = target_dir.join(new_name);
            counter += 1;
        }

        // 4. Guardar el archivo
        let data = field.bytes().await.map_err(|e| AppError::InternalServerError(anyhow::anyhow!(e)))?;
        
        let mut file = File::create(&dest_path).await.map_err(|e| AppError::InternalServerError(anyhow::anyhow!(e)))?;
        file.write_all(&data).await.map_err(|e| AppError::InternalServerError(anyhow::anyhow!(e)))?;
        
        tracing::info!("Archivo subido: {:?}", dest_path);
    }

    Ok((StatusCode::OK, "Subida completada"))
}
