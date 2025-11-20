// src/utils/archiver.rs

use async_zip::tokio::write::ZipFileWriter;
use async_zip::{Compression, ZipEntryBuilder};
use futures::stream::Stream;
use std::path::{Path, PathBuf};
use tokio_util::io::ReaderStream;
use tokio_util::compat::FuturesAsyncWriteCompatExt; 

const BUFFER_SIZE: usize = 64 * 1024;

pub fn archive_directory_stream(
    root_path: PathBuf,
    dir_name: String,
) -> impl Stream<Item = std::io::Result<bytes::Bytes>> {
    
    let (w, r) = tokio::io::duplex(BUFFER_SIZE);

    tokio::spawn(async move {
        if let Err(e) = zip_directory(w, &root_path, &dir_name).await {
            tracing::error!("Error comprimiendo el directorio: {:?}", e);
        }
    });

    ReaderStream::new(r)
}

async fn zip_directory<W>(writer: W, root_path: &Path, _dir_prefix: &str) -> anyhow::Result<()>
where
    W: tokio::io::AsyncWrite + Unpin + Send + 'static,
{
    let mut writer = ZipFileWriter::with_tokio(writer);
    let mut stack = vec![root_path.to_path_buf()];
    let parent_dir = root_path.parent().unwrap_or(Path::new("/"));

    while let Some(path) = stack.pop() {
        let mut entries = tokio::fs::read_dir(&path).await?;

        while let Some(entry) = entries.next_entry().await? {
            let entry_path = entry.path();
            let metadata = entry.metadata().await?;

            if metadata.is_dir() {
                stack.push(entry_path);
            } else {
                let relative_path = entry_path.strip_prefix(parent_dir)?;
                let entry_name = relative_path.to_string_lossy().to_string();

                let builder = ZipEntryBuilder::new(entry_name.into(), Compression::Deflate);
                let mut file = tokio::fs::File::open(&entry_path).await?;

                // --- SOLUCIÓN AL ERROR E0382 ---
                
                // 1. Obtenemos el escritor original (Futures)
                let entry_writer = writer.write_entry_stream(builder).await?;
                
                // 2. Lo "envolvemos" para que funcione con Tokio
                let mut compat_writer = entry_writer.compat_write();
                
                // 3. Copiamos los datos usando Tokio
                tokio::io::copy(&mut file, &mut compat_writer).await?;
                
                // 4. IMPORTANTE: Recuperamos el escritor original "desenvolviendo" el wrapper
                let entry_writer = compat_writer.into_inner();
                
                // 5. Ahora sí podemos cerrarlo porque recuperamos la propiedad (ownership)
                entry_writer.close().await?;
            }
        }
    }

    writer.close().await?;
    Ok(())
}