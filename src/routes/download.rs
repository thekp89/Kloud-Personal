use crate::{error::AppError, utils::archiver, AppState};
use axum::{
    extract::{Path, State},
    response::IntoResponse,
    http::header,
    body::Body,
};
use std::sync::Arc;
use tower_http::services::ServeFile;

pub async fn download_handler(
    State(state): State<Arc<AppState>>,
    Path(request_path): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    // 1. Saneamiento de ruta (Ciberseguridad básica: evitar Path Traversal ../../etc/passwd)
    // Nota: En producción, esto requiere validación más estricta.
    let clean_path = request_path.trim_start_matches('/');
    if clean_path.contains("..") {
        return Err(AppError::InvalidPath);
    }

    // 2. Construir la ruta absoluta en el sistema de archivos
    let full_path = state.base_path.join(clean_path);

    // 3. Verificar existencia
    if !full_path.exists() {
        return Err(AppError::NotFound);
    }

    // 4. Lógica de decisión: ¿Archivo o Carpeta?
    if full_path.is_dir() {
        // CASO CARPETA: Streaming de ZIP al vuelo
        let dir_name = full_path.file_name()
            .unwrap_or_else(|| std::ffi::OsStr::new("download"))
            .to_string_lossy()
            .to_string();
        
        let zip_filename = format!("{}.zip", dir_name);

        // Iniciamos el stream (aquí se conecta con utils/archiver.rs)
        let stream = archiver::archive_directory_stream(full_path, dir_name);
        let body = Body::from_stream(stream);

        // Configuramos cabeceras para forzar descarga
        let headers = [
            (header::CONTENT_TYPE, "application/zip"),
            (header::CONTENT_DISPOSITION, &format!("attachment; filename=\"{}\"", zip_filename)),
        ];

        Ok((headers, body).into_response())
    } else {
        // CASO ARCHIVO: Servir directamente
        // ServeFile maneja eficientemente la lectura del disco
        let mut service = ServeFile::new(full_path);
        let result = service.try_call(axum::http::Request::new(Body::empty())).await;
        
        match result {
            Ok(res) => Ok(res.into_response()),
            Err(e) => {
                tracing::error!("Error sirviendo archivo: {}", e);
                Err(AppError::InternalServerError(anyhow::anyhow!(e)))
            }
        }
    }
}