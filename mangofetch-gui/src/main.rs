use eframe::egui;
use mangofetch_gui::{AppRuntime, BrandPreset};
use std::time::Duration;

fn main() -> Result<(), eframe::Error> {
    // Inicializar logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // Lanzar el runtime en thread separado
    let app_runtime = AppRuntime::start();

    let options = eframe::NativeOptions::default();

    eframe::run_native(
        "MangoFetch",
        options,
        Box::new(move |cc| {
            // Aplicar tema
            mangofetch_gui::theme::apply_monolith_dark(&cc.egui_ctx, BrandPreset::PlasmCore);
            mangofetch_gui::theme::load_fonts(&cc.egui_ctx);

            Ok(Box::new(MangoFetchApp::new(app_runtime)))
        }),
    )
}

struct MangoFetchApp {
    runtime: AppRuntime,
}

impl MangoFetchApp {
    fn new(runtime: AppRuntime) -> Self {
        Self { runtime }
    }
}

impl eframe::App for MangoFetchApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Drenar eventos pendientes (aunque por ahora no hacemos nada con ellos)
        let events = self.runtime.drain_events();
        for event in events {
            tracing::debug!("GUI received event: {:?}", event);
        }

        // Central panel vacío por ahora
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("🥭 MangoFetch GUI");
            ui.label("(work in progress - egui integration)");
            ui.separator();

            if ui.button("Send Test Command").clicked() {
                let cmd = mangofetch_gui::GuiCommand::RefreshQueue;
                if let Err(e) = self.runtime.send_command(cmd) {
                    tracing::error!("Failed to send command: {}", e);
                }
            }

            ui.label("Check console output for messages");
        });

        // Solicitar repaint cada 250ms para actualizar UI
        ctx.request_repaint_after(Duration::from_millis(250));
    }
}
