//! MangoFetch GUI — Interfaz gráfica con egui + eframe

pub mod app;
pub mod bridge;
pub mod runtime;
pub mod theme;
pub mod widgets;

pub use app::MangoFetchApp;
pub use bridge::{CoreEvent, GuiCommand, MediaInfo, QueueItemInfo};
pub use runtime::AppRuntime;
pub use theme::BrandPreset;
