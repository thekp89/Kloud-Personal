# Local Share 🦀

Una herramienta de CLI ultrarrápida escrita en **Rust** para compartir archivos y carpetas dentro de una red local.

Diseñado para superar las limitaciones de `python -m http.server`, **Local Share** permite navegar por directorios y, lo más importante, **descargar carpetas completas comprimidas en ZIP al vuelo**, sin generar archivos temporales ni consumir memoria RAM excesiva.

## 🚀 Características Principales

- **Streaming de ZIP en tiempo real**: Al solicitar una carpeta, el servidor genera el flujo de bytes comprimidos (Deflate) directamente al socket TCP. Esto permite descargar gigabytes de datos comenzando instantáneamente y con un uso de memoria constante (buffer de 64KB), sin importar el tamaño de la carpeta.
- **Cero Dependencias en el Cliente**: Funciona con cualquier navegador web estándar.
- **Binario Estático**: Compila una sola vez y ejecuta en cualquier distro Linux sin instalar dependencias externas.
- **Concurrencia Asíncrona**: Construido sobre `Tokio` y `Axum`, capaz de manejar múltiples conexiones simultáneas eficientemente.

## 🛠️ Stack Tecnológico

Este proyecto utiliza un enfoque moderno del ecosistema asíncrono de Rust:

- **Axum**: Framework web ergonómico y modular.
- **Tokio**: Runtime asíncrono para operaciones no bloqueantes de I/O.
- **Async-zip**: Librería para la creación de archivos ZIP asíncronos.
- **Tokio-util (Compat)**: Puente para comunicar streams de `futures` con el ecosistema de `tokio`.
- **Clap**: Parseo de argumentos de línea de comandos.
- **Tracing**: Instrumentación y logs estructurados.

## 📦 Instalación y Uso

### Requisitos
Necesitas tener instalado Rust y Cargo.

### Compilación
Para desarrollo:
```bash
cargo run -- --path /ruta/a/tu/carpeta