use qrcode::QrCode;
use qrcode::render::svg;
use serde::{Deserialize, Serialize};
use urlencoding::encode;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyLinks {
    pub tg_link: String,
    pub https_link: String,
    pub qr_svg: String,
}

/// Generates valid tg:// and https:// links and SVG QR code for a given proxy.
pub fn generate_links(server: &str, port: u16, secret: &str) -> Result<ProxyLinks, String> {
    let clean_server = server.trim();
    let clean_secret = secret.trim();

    let encoded_server = encode(clean_server);
    // Secret is kept as-is or url-encoded if containing special chars
    let encoded_secret = clean_secret;

    let tg_link = format!(
        "tg://proxy?server={}&port={}&secret={}",
        encoded_server, port, encoded_secret
    );

    let https_link = format!(
        "https://t.me/proxy?server={}&port={}&secret={}",
        encoded_server, port, encoded_secret
    );

    // Generate QR code for the tg:// link so mobile users can scan directly
    let qr = QrCode::new(tg_link.as_bytes())
        .map_err(|e| format!("Failed to generate QR code: {}", e))?;

    let qr_svg = qr
        .render::<svg::Color>()
        .min_dimensions(200, 200)
        .dark_color(svg::Color("#38bdf8")) // Cyan / Blue accent
        .light_color(svg::Color("#0f172a")) // Dark slate background
        .build();

    Ok(ProxyLinks {
        tg_link,
        https_link,
        qr_svg,
    })
}

/// Formats a proxy into a TXT export block.
pub fn format_proxy_txt(server: &str, port: u16, secret: &str) -> String {
    format!(
        "MTProxy Desk Export\n===================\nServer: {}\nPort: {}\nSecret: {}\n\nTelegram Direct Link:\ntg://proxy?server={}&port={}&secret={}\n\nWeb Link:\nhttps://t.me/proxy?server={}&port={}&secret={}\n\nWarning: Sharing this proxy information allows anyone with the secret to connect through this proxy server.\n",
        server, port, secret, server, port, secret, server, port, secret
    )
}
