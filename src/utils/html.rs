use humansize::{format_size, DECIMAL};

pub fn generate_file_list_html(
    entries: Vec<(String, bool, u64)>, // (name, is_dir, size)
    current_path: &str,
    max_upload_size: u64,
) -> String {
    let parent_link = if current_path.is_empty() {
        String::new()
    } else {
        format!(r#"<a href="../" class="back">⬅ Subir un nivel</a>"#)
    };

    let mut list_items = String::new();
    let current_url_path = if current_path.is_empty() { 
        String::new() 
    } else { 
        format!("{}/", current_path.trim_end_matches('/')) 
    };

    for (name, is_dir, size) in entries {
        if is_dir {
            let browse_link = format!("/list/{}{}", current_url_path, name);
            let zip_link = format!("/download/{}{}", current_url_path, name);
            list_items.push_str(&format!(
                r#"<li>
                    <div class="file-row">
                        <a href="{}" class="file-link">📁 {}/</a>
                        <a href="{}" class="action-link">⬇ ZIP</a>
                    </div>
                   </li>"#, 
                browse_link, name, zip_link
            ));
        } else {
            let link = format!("/download/{}{}", current_url_path, name);
            let size_str = format_size(size, DECIMAL);
            list_items.push_str(&format!(
                r#"<li>
                    <div class="file-row">
                        <a href="{}" class="file-link">📄 {}</a>
                        <span class="meta">{}</span>
                    </div>
                   </li>"#, 
                link, name, size_str
            ));
        }
    }

    format!(r#"
        <!DOCTYPE html>
        <html>
        <head>
            <meta charset="utf-8">
            <meta name="viewport" content="width=device-width, initial-scale=1">
            <title>Local Cloud</title>
            <style>
                :root {{
                    --primary: #007bff;
                    --bg: #f8f9fa;
                    --surface: #ffffff;
                    --border: #dee2e6;
                    --text: #212529;
                    --text-muted: #6c757d;
                }}
                body {{ font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif; max-width: 900px; margin: 2rem auto; padding: 0 1rem; background: var(--bg); color: var(--text); }}
                h1 {{ color: var(--text); margin-bottom: 1.5rem; }}
                ul {{ list-style: none; padding: 0; background: var(--surface); border-radius: 8px; box-shadow: 0 2px 5px rgba(0,0,0,0.05); border: 1px solid var(--border); }}
                li {{ border-bottom: 1px solid var(--border); }}
                li:last-child {{ border-bottom: none; }}
                .file-row {{ display: flex; align-items: center; padding: 0.75rem 1rem; }}
                .file-row:hover {{ background-color: #f1f3f5; }}
                .file-link {{ flex-grow: 1; text-decoration: none; color: var(--text); font-weight: 500; }}
                .file-link:hover {{ color: var(--primary); }}
                .action-link {{ text-decoration: none; color: var(--primary); font-size: 0.9em; padding: 0.25rem 0.5rem; border-radius: 4px; background: #e7f1ff; }}
                .action-link:hover {{ background: #d0e2ff; }}
                .meta {{ color: var(--text-muted); font-size: 0.85em; margin-left: 1rem; }}
                .back {{ display: inline-block; margin-bottom: 1rem; text-decoration: none; color: var(--text-muted); font-weight: 500; }}
                .back:hover {{ color: var(--primary); }}

                /* Upload Zone */
                #drop-zone {{
                    border: 2px dashed var(--border);
                    border-radius: 8px;
                    padding: 2rem;
                    text-align: center;
                    margin-bottom: 2rem;
                    background: var(--surface);
                    transition: all 0.2s;
                    cursor: pointer;
                }}
                #drop-zone.dragover {{
                    border-color: var(--primary);
                    background: #e7f1ff;
                }}
                #drop-zone p {{ margin: 0; color: var(--text-muted); }}
                #progress {{ margin-top: 1rem; display: none; }}
                .bar {{ height: 4px; background: #e9ecef; border-radius: 2px; overflow: hidden; }}
                .fill {{ height: 100%; background: var(--primary); width: 0%; transition: width 0.2s; }}
            </style>
        </head>
        <body>
            <h1>Local Share</h1>
            
            <div id="drop-zone">
                <p>Arrastra archivos aquí o haz clic para subir</p>
                <p style="font-size: 0.8em; margin-top: 0.5rem;">Máximo: {}</p>
                <input type="file" id="file-input" multiple style="display: none">
                <div id="progress">
                    <div class="bar"><div class="fill" id="progress-fill"></div></div>
                    <p id="status-text" style="font-size: 0.8em; margin-top: 0.5rem;">Subiendo...</p>
                </div>
            </div>

            {}
            <ul>
                {}
            </ul>

            <script>
                const dropZone = document.getElementById('drop-zone');
                const fileInput = document.getElementById('file-input');
                const progress = document.getElementById('progress');
                const progressFill = document.getElementById('progress-fill');
                const statusText = document.getElementById('status-text');
                const currentPath = "{}";

                // Click to upload
                dropZone.addEventListener('click', () => fileInput.click());
                fileInput.addEventListener('change', (e) => handleFiles(e.target.files));

                // Drag & Drop
                dropZone.addEventListener('dragover', (e) => {{
                    e.preventDefault();
                    dropZone.classList.add('dragover');
                }});
                dropZone.addEventListener('dragleave', () => dropZone.classList.remove('dragover'));
                dropZone.addEventListener('drop', (e) => {{
                    e.preventDefault();
                    dropZone.classList.remove('dragover');
                    handleFiles(e.dataTransfer.files);
                }});

                async function handleFiles(files) {{
                    if (files.length === 0) return;

                    progress.style.display = 'block';
                    let uploaded = 0;
                    let total = files.length;

                    for (let file of files) {{
                        statusText.textContent = `Subiendo ${{file.name}}...`;
                        const formData = new FormData();
                        formData.append('file', file);

                        try {{
                            // Usamos encodeURIComponent para manejar espacios y caracteres especiales en la ruta
                            const pathParam = currentPath ? `?path=${{encodeURIComponent(currentPath)}}` : '?path=/';
                            const response = await fetch('/upload' + pathParam, {{
                                method: 'POST',
                                body: formData
                            }});

                            if (!response.ok) throw new Error('Error en subida');
                            uploaded++;
                            progressFill.style.width = `${{(uploaded / total) * 100}}%`;
                        }} catch (err) {{
                            console.error(err);
                            alert(`Error subiendo ${{file.name}}`);
                        }}
                    }}

                    statusText.textContent = '¡Completado! Recargando...';
                    setTimeout(() => window.location.reload(), 500);
                }}
            </script>
        </body>
        </html>
    "#, format_size(max_upload_size, DECIMAL), parent_link, list_items, current_path)
}
