# Local Share 🦀

Una herramienta ultrarrápida y versátil escrita en **Rust** para compartir archivos y carpetas dentro de una red local, ahora con una interfaz gráfica intuitiva.

Diseñado para superar las limitaciones de `python -m http.server`, **Local Share** permite navegar por directorios, **descargar carpetas completas comprimidas en ZIP al vuelo** y ahora también **recibir archivos** de forma segura.

## 🚀 Características Principales

- **Streaming de ZIP en tiempo real**: Generación de ZIP al vuelo sin archivos temporales ni consumo excesivo de RAM.
- **Doble Modo (GUI/CLI)**: Lanzador nativo para uso visual o terminal para automatización.
- **Subida de Archivos**: Interfaz web drag-and-drop para recibir archivos de otros dispositivos.
- **Seguridad TLS (HTTPS)**: Soporte para conexiones cifradas con certificados propios o generados automáticamente.
- **Autenticación**: Protección mediante Basic Auth (usuario/contraseña).
- **Zero Config**: Binario único sin dependencias externas.

## 🖥️ Interfaz Gráfica (GUI) vs CLI

**Local Share** detecta automáticamente el modo de ejecución:

- **Modo GUI**: Se activa al ejecutar sin argumentos. Ideal para uso personal rápido.
- **Modo CLI**: Se activa al pasar argumentos (ej. `--path`). Ideal para scripts y servidores.

### Iniciar la GUI:
```bash
cargo run
```

Desde la interfaz gráfica puedes seleccionar carpetas, cambiar el puerto, activar la seguridad y abrir el servidor en tu navegador con un solo clic.

## 🛠️ Stack Tecnológico

- **egui / eframe**: UI nativa inmediata y ligera.
- **Axum & Tokio**: El estándar de oro para servicios web asíncronos en Rust.
- **Axum-server**: Manejo robusto de TLS.
- **rcgen**: Generación de certificados X.509 efímeros.
- **Async-zip**: Compresión en streaming de alto rendimiento.

## 📦 Uso vía CLI

### Requisitos
- Rust y Cargo instalados.

### Comandos de ejemplo
```bash
# Compartir carpeta actual en el puerto 3000
cargo run -- --path .

# Compartir con seguridad y límite de subida de 50MB
cargo run -- --path /Descargas -P 8080 --tls -S 50 --username admin --password secreto
```

### Argumentos disponibles

| Argumento | Corto | Descripción | Default |
|-----------|-------|-------------|---------|
| `--path`  | `-p`  | Ruta del directorio a compartir | `.` |
| `--port`  | `-P`  | Puerto del servidor | `3000` |
| `--max-upload-size` | `-S` | Límite de subida en MB | `10` |
| `--tls`   | | Habilita HTTPS (Genera cert. si no hay) | `false` |
| `--username`| `-u` | Usuario para autenticación | - |
| `--password`| `-w` | Contraseña para autenticación | - |
| `--cert`  | | Ruta al certificado .pem | - |
| `--key`   | | Ruta a la clave privada .key | - |