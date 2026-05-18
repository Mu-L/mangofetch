//! Theming: MonolithUI → egui visuals
//! Adapta el sistema de colores de MonolithUI a egui

use egui::{Color32, Stroke};

/// Presets de brand (colores primarios)
#[derive(Debug, Clone, Copy)]
pub enum BrandPreset {
    PlasmCore, // Cyan #22d3ee
               // Agregar más presets según lo necesite
}

impl BrandPreset {
    pub fn primary(&self) -> Color32 {
        match self {
            BrandPreset::PlasmCore => hex_to_color32("#22d3ee"),
        }
    }
}

/// Convierte hex string a Color32 (solo #RRGGBB)
fn hex_to_color32(hex: &str) -> Color32 {
    let hex = hex.trim_start_matches('#');
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
    Color32::from_rgb(r, g, b)
}

/// Agrega alpha a un color
fn with_alpha(color: Color32, alpha: f32) -> Color32 {
    let arr = color.to_array();
    let r = arr[0];
    let g = arr[1];
    let b = arr[2];
    Color32::from_rgba_unmultiplied(r, g, b, (alpha * 255.0) as u8)
}

/// Aplica el tema dark de MonolithUI al contexto egui
pub fn apply_monolith_dark(ctx: &egui::Context, brand: BrandPreset) {
    let mut visuals = egui::Visuals::dark();

    // Surface ramp → egui panels
    visuals.window_fill = hex_to_color32("#0c0e12"); // surface-1
    visuals.panel_fill = hex_to_color32("#060608"); // surface-0
    visuals.faint_bg_color = hex_to_color32("#131720"); // surface-2
    visuals.extreme_bg_color = hex_to_color32("#060608"); // surface-0

    // El color primario del brand activo
    let primary = brand.primary();

    // Widgets (solo propiedades seguras según la API actual)
    visuals.widgets.inactive.bg_fill = hex_to_color32("#1c2130"); // surface-3
    visuals.widgets.hovered.bg_fill = hex_to_color32("#252a3a"); // surface-4
    visuals.widgets.active.bg_fill = with_alpha(primary, 0.20); // accent-primary-bg

    // Selection
    visuals.selection.bg_fill = with_alpha(primary, 0.15);
    visuals.selection.stroke = Stroke::new(1.0, primary);

    // Hyperlinks → brand primary
    visuals.hyperlink_color = primary;

    ctx.set_visuals(visuals);
}

/// Carga fuentes embebidas (DM Sans + DM Mono)
/// Llamar en main.rs antes de crear el App
pub fn load_fonts(_ctx: &egui::Context) {
    // TODO: Copiar DM_Sans.ttf y DM_Mono.ttf a mangofetch-gui/assets/
    // y usar include_bytes!() para incrustarlas

    tracing::debug!("Fonts loaded (using egui defaults for now)");
}
