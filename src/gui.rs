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
    theme_enabled: bool,
    theme_path: Option<PathBuf>,
    
    // Runtime State
    server_handle: Option<JoinHandle<()>>,
    mdns_handle: Option<crate::utils::mdns::MdnsGuard>,
    status_msg: String,
    
    // QR Code
    qr_texture: Option<egui::TextureHandle>,
    show_auth_in_qr: bool,
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
            theme_enabled: false,
            theme_path: None,
            server_handle: None,
            mdns_handle: None,
            status_msg: "Ready".to_string(),
            qr_texture: None,
            show_auth_in_qr: false,
        }
    }
}

impl LocalShareApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Default::default()
    }

    fn start_server(&mut self, ctx: &egui::Context) {
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
            max_upload_size: 10,
            tls: self.tls_enabled,
            cert: None,
            key: None,
            username: if self.auth_enabled { Some(self.username.clone()) } else { None },
            password: if self.auth_enabled { Some(self.password.clone()) } else { None },
            theme: if self.theme_enabled { self.theme_path.clone() } else { None },
            dump_theme: None,
        };

        self.status_msg = format!("Running on port {}", port);
        
        // --- Identity & Discovery ---
        // 1. mDNS
        let mdns = crate::utils::mdns::register_service(port, "local-share", self.tls_enabled);
        self.mdns_handle = Some(mdns);

        // 2. Generate QR
        self.update_qr_code(ctx);

        // Spawn server task
        let handle = tokio::spawn(async move {
            server::start_server(args).await;
        });
        
        self.server_handle = Some(handle);
    }

    fn update_qr_code(&mut self, ctx: &egui::Context) {
        // Regenerate QR based on current settings
        // Only if we have valid settings to generate a URL
         let port = match self.port.parse::<u16>() {
            Ok(p) => p,
            Err(_) => return,
        };
        
        let local_ip = crate::utils::net::get_local_ip();
        
        let url = crate::utils::net::build_connection_url(
            self.tls_enabled,
            &local_ip,
            port,
            Some(&self.username),
            Some(&self.password),
            self.show_auth_in_qr && self.auth_enabled,
        );

        if let Ok((w, h, rgb)) = crate::utils::qr::generate_qr_image(&url) {
             let image = egui::ColorImage::from_rgb([w as usize, h as usize], &rgb);
             self.qr_texture = Some(ctx.load_texture("qr_code", image, Default::default()));
        }
    }

    fn stop_server(&mut self) {
        if let Some(handle) = self.server_handle.take() {
            handle.abort();
        }
        // dropping the guard stops the service
        self.mdns_handle = None;
        
        self.status_msg = "Stopped".to_string();
        self.qr_texture = None;
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
            
            // Theming
            ui.heading("Theming");
            ui.checkbox(&mut self.theme_enabled, "Use Custom Theme");
            if self.theme_enabled {
                 ui.indent("theme_indent", |ui| {
                    ui.horizontal(|ui| {
                        let path_str = self.theme_path.as_ref()
                            .map(|p| p.to_string_lossy().to_string())
                            .unwrap_or_else(|| "None".to_string());
                        
                        ui.label("Theme Path:");
                        ui.label(path_str);
                        
                        if ui.button("Select Theme Folder").clicked() {
                            if let Some(path) = rfd::FileDialog::new().pick_folder() {
                                self.theme_path = Some(path);
                            }
                        }
                    });
                    ui.small("Select a folder containing 'index.html', 'css/', etc.");
                });
            }

            ui.add_space(20.0);
            ui.separator();
            ui.add_space(10.0);

            // Actions
            ui.horizontal(|ui| {
                if self.server_handle.is_none() {
                    if ui.button("Start Server").clicked() {
                        self.start_server(ctx);
                    }
                } else {
                    if ui.button("Stop Server").clicked() {
                        self.stop_server();
                    }
                    ui.spinner();
                }
            });

            // QR Panel
            if self.server_handle.is_some() {
                ui.add_space(20.0);
                ui.separator();
                ui.heading("Connect Mobile");
                
                if self.auth_enabled {
                    if ui.checkbox(&mut self.show_auth_in_qr, "Include Credentials in QR").changed() {
                        self.update_qr_code(ctx);
                    }
                }

                if let Some(texture) = &self.qr_texture {
                    ui.image((texture.id(), texture.size_vec2()));
                }
            }

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
