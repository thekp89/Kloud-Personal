use eframe::egui;
use std::path::PathBuf;
use tokio::task::JoinHandle;
use crate::server::{self, Args};

pub struct LocalShareApp {
    // Configuration State
    path: PathBuf,
    port: String,
    tls_enabled: bool,
    auth_enabled: bool,
    username: String,
    password: String,
    
    // Runtime State
    server_handle: Option<JoinHandle<()>>,
    status_msg: String,
}

impl Default for LocalShareApp {
    fn default() -> Self {
        Self {
            path: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            port: "3000".to_string(),
            tls_enabled: false,
            auth_enabled: false,
            username: "admin".to_string(),
            password: "password".to_string(),
            server_handle: None,
            status_msg: "Ready".to_string(),
        }
    }
}

impl LocalShareApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Default::default()
    }

    fn start_server(&mut self) {
        if self.server_handle.is_some() {
            return;
        }

        let port = match self.port.parse::<u16>() {
            Ok(p) => p,
            Err(_) => {
                self.status_msg = "Invalid Port".to_string();
                return;
            }
        };

        let args = Args {
            path: self.path.clone(),
            port,
            max_upload_size: 10, // Default fixed for GUI, could be added
            tls: self.tls_enabled,
            cert: None,
            key: None,
            username: if self.auth_enabled { Some(self.username.clone()) } else { None },
            password: if self.auth_enabled { Some(self.password.clone()) } else { None },
        };

        self.status_msg = format!("Running on port {}", port);
        
        // Spawn server task
        let handle = tokio::spawn(async move {
            server::start_server(args).await;
        });
        
        self.server_handle = Some(handle);
    }

    fn stop_server(&mut self) {
        if let Some(handle) = self.server_handle.take() {
            handle.abort();
            self.status_msg = "Stopped".to_string();
        }
    }
}

impl eframe::App for LocalShareApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Local Share Launcher");
            ui.add_space(10.0);

            // Path Selection
            ui.horizontal(|ui| {
                ui.label("Path:");
                ui.label(self.path.to_string_lossy());
                if ui.button("Select Folder").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                        self.path = path;
                    }
                }
            });

            // Port
            ui.horizontal(|ui| {
                ui.label("Port:");
                ui.text_edit_singleline(&mut self.port);
            });

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);

            // Security
            ui.heading("Security");
            ui.checkbox(&mut self.tls_enabled, "Enable HTTPS (TLS)");
            
            ui.checkbox(&mut self.auth_enabled, "Enable Authentication");
            if self.auth_enabled {
                ui.indent("auth_indent", |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Username:");
                        ui.text_edit_singleline(&mut self.username);
                    });
                    ui.horizontal(|ui| {
                        ui.label("Password:");
                        ui.text_edit_singleline(&mut self.password);
                    });
                });
            }

            ui.add_space(20.0);
            ui.separator();
            ui.add_space(10.0);

            // Actions
            ui.horizontal(|ui| {
                if self.server_handle.is_none() {
                    if ui.button("Start Server").clicked() {
                        self.start_server();
                    }
                } else {
                    if ui.button("Stop Server").clicked() {
                        self.stop_server();
                    }
                    ui.spinner();
                }
            });

            ui.label(&self.status_msg);
            
            if self.server_handle.is_some() {
                 let protocol = if self.tls_enabled { "https" } else { "http" };
                 let url = format!("{}://localhost:{}", protocol, self.port);
                 if ui.link(url.clone()).clicked() {
                     let _ = open::that(url);
                 }
            }
        });
    }
}
