//! Custom reusable MonolithUI widgets for mangofetch-gui

use egui::{Ui, Color32, Vec2, Stroke, Frame, Margin, CornerRadius, RichText, FontFamily, FontId};
use crate::theme::with_alpha;

/// Renders a beautiful MonolithUI surface card with uniform padding and borders.
pub fn surface_card<R>(ui: &mut Ui, add_contents: impl FnOnce(&mut Ui) -> R) -> R {
    // --ui-surface-4 is default card bg (#252a3a)
    let fill_color = Color32::from_rgb(0x25, 0x2a, 0x3a);
    // --ui-surface-6 is border highlight (#3a4055)
    let stroke_color = Color32::from_rgb(0x3a, 0x40, 0x55);
    
    Frame::NONE
        .fill(fill_color)
        .inner_margin(Margin::same(12))
        .corner_radius(CornerRadius::same(6)) // --ui-r-md
        .stroke(Stroke::new(1.0, stroke_color))
        .show(ui, add_contents)
        .inner
}

/// Renders a sunken "well" panel for grouped parameters or backgrounds (e.g. Logs terminal, input well).
pub fn sunken_well<R>(ui: &mut Ui, add_contents: impl FnOnce(&mut Ui) -> R) -> R {
    // Sunken wells are darker (#060608 / SURFACE_0)
    let fill_color = Color32::from_rgb(0x06, 0x06, 0x08);
    let border_color = Color32::from_rgb(0x1c, 0x21, 0x30); // SURFACE_3
    
    Frame::NONE
        .fill(fill_color)
        .inner_margin(Margin::same(10))
        .corner_radius(CornerRadius::same(4)) // --ui-r-sm
        .stroke(Stroke::new(1.0, border_color))
        .show(ui, add_contents)
        .inner
}

/// A status indicator dot following the MonolithUI semantic specification.
/// - Active / Complete: Success Green (#34a853) with a pulse/glow
/// - Queued / Warning / Paused: Amber Yellow (#fbbf24)
/// - Error: Destructive Red (#f28b82)
pub fn status_dot(ui: &mut Ui, status: &str) {
    let color = match status {
        "Active" | "Complete" | "Online" => Color32::from_rgb(0x34, 0xa8, 0x53),
        "Queued" | "Warning" | "Paused" => Color32::from_rgb(0xfb, 0xbf, 0x24),
        _ => Color32::from_rgb(0xf2, 0x8b, 0x82),
    };

    let (rect, _response) = ui.allocate_exact_size(Vec2::new(14.0, 14.0), egui::Sense::hover());
    let painter = ui.painter();
    
    // Draw the main circle
    painter.circle_filled(rect.center(), 4.0, color);
    
    // Draw a tactile glow for Active states
    if status == "Active" || status == "Online" {
        painter.circle_filled(rect.center(), 6.5, with_alpha(color, 0.25));
    }
}

/// A MonolithUI pill tag showing text with a custom brand color background.
pub fn brand_pill(ui: &mut Ui, text: &str, color: Color32) {
    Frame::NONE
        .fill(with_alpha(color, 0.12))
        .stroke(Stroke::new(1.0, with_alpha(color, 0.25)))
        .inner_margin(Margin::symmetric(8, 3))
        .corner_radius(CornerRadius::same(4)) // --ui-r-sm
        .show(ui, |ui| {
            ui.label(
                RichText::new(text)
                    .color(color)
                    .font(FontId::new(10.5, FontFamily::Monospace)) // --ui-text-xs
            );
        });
}

/// Custom header title segment in Monolith Serif Display style.
pub fn section_header(ui: &mut Ui, title: &str) {
    ui.vertical(|ui| {
        ui.add_space(4.0);
        ui.label(
            RichText::new(title)
                .font(FontId::new(16.0, FontFamily::Proportional))
                .strong()
                .color(Color32::from_rgb(0xf3, 0xf4, 0xf6))
        );
        ui.add_space(6.0);
    });
}
