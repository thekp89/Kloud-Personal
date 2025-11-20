use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use std::io;

// Definimos nuestro enum de errores personalizados.
// Esto cubre los casos que esperamos que ocurran en un servidor de archivos.
#[derive(Debug)]
pub enum AppError {
    // El cliente pide algo que no existe
    NotFound,
    // El sistema operativo nos niega el acceso
    PermissionDenied,
    // Un error genérico para cosas que no deberían pasar (IO errors, etc.)
    InternalServerError(anyhow::Error),
    // Seguridad: Intentan acceder fuera de la carpeta permitida (Path Traversal)
    InvalidPath,
}

// Implementamos IntoResponse para que Axum sepa qué responder al navegador
// cuando ocurre uno de estos errores.
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        // Mapeamos el error interno a un Código de Estado HTTP y un mensaje
        let (status, error_message) = match self {
            AppError::NotFound => (StatusCode::NOT_FOUND, "Recurso no encontrado"),
            AppError::PermissionDenied => (StatusCode::FORBIDDEN, "Permiso denegado"),
            AppError::InvalidPath => (StatusCode::BAD_REQUEST, "Ruta inválida o insegura"),
            AppError::InternalServerError(err) => {
                // IMPORTANTE: Logueamos el error real en la terminal del servidor
                tracing::error!("Error interno: {:?}", err);
                // Pero al usuario le mostramos un mensaje genérico por seguridad
                (StatusCode::INTERNAL_SERVER_ERROR, "Error interno del servidor")
            }
        };

        // Construimos la respuesta final
        (status, error_message).into_response()
    }
}

// Esta implementación nos permite convertir automáticamente los errores de std::io
// a nuestro AppError. Gracias a esto podemos usar '?' en operaciones de archivo.
impl From<io::Error> for AppError {
    fn from(err: io::Error) -> Self {
        match err.kind() {
            io::ErrorKind::NotFound => AppError::NotFound,
            io::ErrorKind::PermissionDenied => AppError::PermissionDenied,
            _ => AppError::InternalServerError(anyhow::Error::new(err)),
        }
    }
}

// Esta implementación permite convertir errores genéricos de anyhow a nuestro tipo
impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::InternalServerError(err)
    }
}