use qrcode::QrCode;

/// Generates a string containing the QR code for printing in the terminal.
/// Uses Unicode blocks for high contrast and readability.
pub fn generate_ascii_qr(content: &str) -> Result<String, String> {
    let code = QrCode::new(content).map_err(|e| e.to_string())?;
    
    let string = code.render::<char>()
        .quiet_zone(true)
        .module_dimensions(2, 1)
        .build();

    Ok(string)
}

/// Generates a raw RGB buffer for the QR code, suitable for GUI rendering.
/// Returns (width, height, bytes).
pub fn generate_qr_image(content: &str) -> Result<(u32, u32, Vec<u8>), String> {
    let code = QrCode::new(content).map_err(|e| e.to_string())?;
    
    // We'll manually construct the image to avoid dependency hell with `image` crate versions.
    // Let's say we want each module to be 4x4 pixels.
    let scale = 4;
    let quiet_zone = 2; // modules
    
    let logical_width = code.width();
    let width = (logical_width + 2 * quiet_zone) * scale;
    let height = width; // QR codes are square
    
    let mut buffer = vec![255u8; (width * height * 3) as usize]; // White background (255)
    
    // Function to set a pixel (x, y) to color (r, g, b)
    let set_pixel = |buf: &mut Vec<u8>, x: usize, y: usize, r: u8, g: u8, b: u8| {
        let idx = (y * width + x) * 3;
        if idx + 2 < buf.len() {
            buf[idx] = r;
            buf[idx+1] = g;
            buf[idx+2] = b;
        }
    };
    
    // Fill the modules
    // Start drawing at offset
    let start_offset = quiet_zone * scale;
    
    for y in 0..logical_width {
        for x in 0..logical_width {
            let color = code[(x, y)];
            match color {
                qrcode::Color::Dark => {
                    // Draw a black square of size `scale`
                    let start_x = start_offset + x * scale;
                    let start_y = start_offset + y * scale;
                    
                    for py in 0..scale {
                        for px in 0..scale {
                            set_pixel(&mut buffer, start_x + px, start_y + py, 0, 0, 0);
                        }
                    }
                },
                qrcode::Color::Light => {
                    // Already white, do nothing
                }
            }
        }
    }
    
    Ok((width as u32, height as u32, buffer))
}
