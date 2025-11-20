use crate::{error::AppError, AppState};
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
    let mut html = String::from(r#"
        <!DOCTYPE html>
        <html>
        <head>
            <meta charset="utf-8">
            <title>Local Cloud</title>
            <style>
                body { font-family: sans-serif; max-width: 800px; margin: 2rem auto; padding: 0 1rem; background: #f0f0f0; }
                h1 { color: #333; }
                ul { list-style: none; padding: 0; background: white; border-radius: 8px; box-shadow: 0 2px 5px rgba(0,0,0,0.1); }
                li { border-bottom: 1px solid #eee; }
                li:last-child { border-bottom: none; }
                a { display: block; padding: 1rem; text-decoration: none; color: #2c3e50; display: flex; justify-content: space-between; }
                a:hover { background-color: #f8f9fa; color: #007bff; }
                .type { color: #888; font-size: 0.9em; }
                .back { margin-bottom: 1rem; display: inline-block; }
            </style>
        </head>
        <body>
            <h1>Archivos</h1>
            <a href="../" class="back">⬅ Subir un nivel</a>
            <ul>
    "#);

    // Recopilamos items para ordenarlos (directorios primero)
    let mut items = Vec::new();
    while let Some(entry) = entries.next_entry().await? {
        let metadata = entry.metadata().await?;
        let name = entry.file_name().to_string_lossy().to_string();
        // Ignorar archivos ocultos unix
        if name.starts_with('.') { continue; }
        items.push((name, metadata.is_dir()));
    }

    // Ordenar: Carpetas primero, luego alfabético
    items.sort_by(|a, b| {
        b.1.cmp(&a.1).then(a.0.cmp(&b.0))
    });

    for (name, is_dir) in items {
        // Construimos los links
        // Si es carpeta, link a /list/... para navegar
        // Si es archivo, link a /download/... para bajar
        
        // El truco: Para descargar una carpeta como ZIP, el usuario debe navegar a ella
        // y podríamos poner un botón de "descargar todo", pero por ahora,
        // vamos a permitir navegar.
        // TRUCO ADICIONAL: Agregamos un link específico de descarga para carpetas.
        
        let current_url_path = if req_path.is_empty() { 
            String::new() 
        } else { 
            format!("{}/", req_path.trim_end_matches('/')) 
        };

        if is_dir {
            // Link para entrar en la carpeta
            let browse_link = format!("/list/{}{}", current_url_path, name);
            // Link para descargar la carpeta como ZIP
            let zip_link = format!("/download/{}{}", current_url_path, name);
            
            html.push_str(&format!(
                r#"<li>
                    <div style="display:flex; width:100%">
                        <a href="{}" style="flex-grow:1">📁 {}/</a>
                        <a href="{}" style="flex-grow:0; border-left:1px solid #eee">⬇ ZIP</a>
                    </div>
                   </li>"#, 
                browse_link, name, zip_link
            ));
        } else {
            let link = format!("/download/{}{}", current_url_path, name);
            html.push_str(&format!(
                r#"<li><a href="{}">📄 {} <span class="type">Archivo</span></a></li>"#, 
                link, name
            ));
        }
    }

    html.push_str("</ul></body></html>");

    Ok(Html(html))
}