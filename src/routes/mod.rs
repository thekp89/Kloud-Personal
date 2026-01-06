use axum::{routing::get, Router};
use crate::AppState;
use std::sync::Arc;

mod download;
mod list;
mod upload;
mod assets;

pub fn app_router() -> Router<Arc<AppState>> {
    Router::new()
        // Ruta raíz: redirige a la lista principal
        .route("/", get(list::root_handler))
        
        // --- AGREGA ESTA LÍNEA ---
        // Esto maneja el caso base "/list/" donde no hay subcarpetas
        .route("/list/", get(list::list_handler)) 
        // ------------------------

        // Ruta para navegar subcarpetas: /list/carpeta/subcarpeta
        .route("/list/*path", get(list::list_handler))
        
        // Ruta para descargar
        .route("/download/*path", get(download::download_handler))

        // Ruta para subir archivos
        .route("/upload", axum::routing::post(upload::upload_handler))

        // Ruta para assets estáticos
        .route("/assets/*path", get(assets::assets_handler))
}