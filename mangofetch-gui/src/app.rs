//! MangoFetchApp core implementation conforming to the MonolithUI design system.

use egui::{
    Ui, Color32, Vec2, Stroke, Frame, Margin, CornerRadius, RichText, FontFamily, FontId,
    Align, Layout, ProgressBar, ScrollArea, Button, SelectableLabel
};
use egui_extras::{TableBuilder, Column};
use crate::bridge::{CoreEvent, GuiCommand, QueueItemInfo, MediaInfo};
use crate::runtime::AppRuntime;
use crate::theme::BrandPreset;
use crate::widgets::{surface_card, sunken_well, status_dot, brand_pill, section_header};

/// Active tabs in the orbital navigation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Home,
    Queue,
    Settings,
    Logs,
    About,
}

pub struct MangoFetchApp {
    runtime: AppRuntime,
    current_tab: Tab,
    theme: BrandPreset,
    
    // Core states
    items: Vec<QueueItemInfo>,
    logs: Vec<String>,
    ytdlp_installed: bool,
    ffmpeg_installed: bool,
    
    // Inputs & Forms
    input_url: String,
    output_dir: String,
    audio_only: bool,
    selected_quality: String,
    
    // Media Pre-fetch
    media_info_loading: bool,
    media_info: Option<MediaInfo>,
    media_info_error: Option<String>,
    
    // Settings parameters
    concurrent_limit: usize,
    auto_retry: bool,
    
    // Telemetry
    sys: sysinfo::System,
}

impl MangoFetchApp {
    pub fn new(runtime: AppRuntime) -> Self {
        let default_output_dir = dirs::download_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| "C:\\Downloads".to_string());

        let mut sys = sysinfo::System::new_all();
        sys.refresh_all();

        let app = Self {
            runtime,
            current_tab: Tab::Home,
            theme: BrandPreset::PlasmCore,
            items: Vec::new(),
            logs: Vec::new(),
            ytdlp_installed: false,
            ffmpeg_installed: false,
            input_url: String::new(),
            output_dir: default_output_dir,
            audio_only: false,
            selected_quality: "Best".to_string(),
            media_info_loading: false,
            media_info: None,
            media_info_error: None,
            concurrent_limit: 3,
            auto_retry: true,
            sys,
        };

        // Trigger initial core checks
        let _ = app.runtime.send_command(GuiCommand::CheckDependencies);
        let _ = app.runtime.send_command(GuiCommand::RefreshQueue);

        app
    }

    /// Drains all incoming asynchronous events from the Tokio background engine
    fn drain_events(&mut self) {
        let events = self.runtime.drain_events();
        for event in events {
            match event {
                CoreEvent::QueueUpdated(queue_items) => {
                    self.items = queue_items;
                }
                CoreEvent::DownloadProgress { id, progress, speed, eta } => {
                    if let Some(item) = self.items.iter_mut().find(|i| i.id == id) {
                        item.progress = progress;
                        item.speed = speed;
                        item.eta = eta;
                    }
                }
                CoreEvent::DownloadComplete { id, title } => {
                    if let Some(item) = self.items.iter_mut().find(|i| i.id == id) {
                        item.status = "Complete".to_string();
                        item.progress = 100.0;
                    }
                    self.logs.push(format!("✓ [{}] Completed successfully", title));
                }
                CoreEvent::DownloadError { id, error } => {
                    if let Some(item) = self.items.iter_mut().find(|i| i.id == id) {
                        item.status = "Error".to_string();
                        self.logs.push(format!("✗ [ID #{}] Error: {}", id, error));
                    }
                }
                CoreEvent::MediaInfoFetched(result) => {
                    self.media_info_loading = false;
                    match result {
                        Ok(info) => {
                            self.media_info = Some(info);
                            self.media_info_error = None;
                        }
                        Err(err) => {
                            self.media_info = None;
                            self.media_info_error = Some(err);
                        }
                    }
                }
                CoreEvent::DependencyStatus { ytdlp, ffmpeg } => {
                    self.ytdlp_installed = ytdlp;
                    self.ffmpeg_installed = ffmpeg;
                }
                CoreEvent::LogLine(line) => {
                    self.logs.push(line);
                    if self.logs.len() > 800 {
                        self.logs.remove(0);
                    }
                }
            }
        }
    }

    /// Renders the sidebar navigation panel (left)
    fn render_sidebar(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.add_space(24.0);
            
            // Premium Brand Logo and name
            ui.horizontal(|ui| {
                ui.add_space(12.0);
                ui.label(
                    RichText::new("🥭 mangofetch")
                        .font(FontId::new(20.0, FontFamily::Proportional))
                        .strong()
                        .color(self.theme.primary())
                );
            });
            
            ui.add_space(32.0);

            // Tab navigation list
            let nav_tabs = [
                (Tab::Home, "📥  Home"),
                (Tab::Queue, "📊  Queue"),
                (Tab::Settings, "⚙  Settings"),
                (Tab::Logs, "📝  Logs"),
                (Tab::About, "ℹ  About"),
            ];

            for (tab_enum, label) in nav_tabs {
                let is_active = self.current_tab == tab_enum;
                
                ui.horizontal(|ui| {
                    ui.add_space(6.0);

                    // Physical indicator on active hover
                    if is_active {
                        let (rect, _) = ui.allocate_exact_size(Vec2::new(3.0, 24.0), egui::Sense::hover());
                        ui.painter().rect_filled(rect, CornerRadius::same(2), self.theme.primary());
                        ui.add_space(4.0);
                    } else {
                        ui.add_space(7.0);
                    }

                    let btn_text = if is_active {
                        RichText::new(label).strong().color(self.theme.primary())
                    } else {
                        RichText::new(label).color(Color32::from_rgb(0x9c, 0xa3, 0xaf))
                    };

                    let response = ui.add(
                        SelectableLabel::new(is_active, btn_text)
                    );
                    
                    if response.clicked() {
                        self.current_tab = tab_enum;
                        if tab_enum == Tab::Queue {
                            let _ = self.runtime.send_command(GuiCommand::RefreshQueue);
                        }
                    }
                });
                
                ui.add_space(8.0);
            }
        });
    }

    /// Home Tab: Entry input, options, media preview, download triggers
    fn draw_home_tab(&mut self, ui: &mut Ui) {
        section_header(ui, "📥  Command Center");
        ui.add_space(6.0);

        surface_card(ui, |ui| {
            ui.label(RichText::new("URL to Download").color(Color32::from_rgb(0xd1, 0xd5, 0xdb)));
            ui.add_space(4.0);

            ui.horizontal(|ui| {
                // Input well
                let text_edit = ui.add_sized(
                    Vec2::new(ui.available_width() - 110.0, 28.0),
                    egui::TextEdit::singleline(&mut self.input_url)
                        .hint_text("Paste YouTube, Twitch, Magnet, TikTok or direct link...")
                );
                
                if text_edit.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    self.fetch_preview();
                }

                if ui.add_sized(Vec2::new(100.0, 28.0), Button::new("Inspect")).clicked() {
                    self.fetch_preview();
                }
            });
            ui.add_space(8.0);
        });

        ui.add_space(16.0);

        // Pre-fetch video details preview
        if self.media_info_loading {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label(RichText::new("Analyzing media details...").italics().color(Color32::from_rgb(0x9c, 0xa3, 0xaf)));
            });
            ui.add_space(16.0);
        } else if let Some(ref info) = self.media_info {
            surface_card(ui, |ui| {
                ui.label(RichText::new("📋 Media Metadata Preview").strong().color(self.theme.primary()));
                ui.add_space(8.0);
                
                ui.horizontal(|ui| {
                    ui.label("Title:");
                    ui.label(RichText::new(&info.title).strong().color(Color32::WHITE));
                });
                ui.add_space(4.0);

                ui.horizontal(|ui| {
                    ui.label("Duration:");
                    if let Some(sec) = info.duration {
                        let min = sec / 60;
                        let s = sec % 60;
                        ui.label(RichText::new(format!("{:02}:{:02}", min, s)).color(Color32::WHITE));
                    } else {
                        ui.label("Live Stream / Unknown");
                    }
                });
                ui.add_space(4.0);

                ui.horizontal(|ui| {
                    ui.label("Platform detected:");
                    brand_pill(ui, &info.platform, self.theme.secondary());
                });

                ui.add_space(8.0);
            });
            ui.add_space(16.0);
        } else if let Some(ref err) = self.media_info_error {
            Frame::NONE
                .fill(Color32::from_rgba_unmultiplied(242, 139, 130, 15))
                .stroke(Stroke::new(1.0, Color32::from_rgba_unmultiplied(242, 139, 130, 60)))
                .inner_margin(Margin::same(10))
                .corner_radius(CornerRadius::same(4))
                .show(ui, |ui| {
                    ui.label(RichText::new(format!("⚠️ Metadata check failed: {}", err)).color(Color32::from_rgb(0xf2, 0x8b, 0x82)));
                });
            ui.add_space(16.0);
        }

        // Configuration well and Trigger
        surface_card(ui, |ui| {
            ui.label(RichText::new("Download Options").strong().color(Color32::from_rgb(0xd1, 0xd5, 0xdb)));
            ui.add_space(12.0);

            ui.horizontal(|ui| {
                ui.checkbox(&mut self.audio_only, "Extract Audio Only (MP3/M4A)");
                ui.add_space(20.0);
                
                ui.label("Video Quality:");
                egui::ComboBox::from_id_salt("quality_combo")
                    .selected_text(&self.selected_quality)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.selected_quality, "Best".to_string(), "Best (Default)");
                        ui.selectable_value(&mut self.selected_quality, "1080p".to_string(), "1080p HD");
                        ui.selectable_value(&mut self.selected_quality, "720p".to_string(), "720p");
                        ui.selectable_value(&mut self.selected_quality, "480p".to_string(), "480p");
                        ui.selectable_value(&mut self.selected_quality, "Audio-Only".to_string(), "Pure Audio");
                    });
            });

            ui.add_space(12.0);
            ui.label("Output Directory:");
            ui.add_space(4.0);
            
            ui.horizontal(|ui| {
                ui.add_sized(
                    Vec2::new(ui.available_width() - 80.0, 24.0),
                    egui::TextEdit::singleline(&mut self.output_dir)
                );
                
                if ui.button("Browse...").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                        self.output_dir = path.to_string_lossy().to_string();
                    }
                }
            });

            ui.add_space(16.0);

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let start_btn = ui.add_sized(
                    Vec2::new(180.0, 36.0),
                    Button::new(RichText::new("📥 Enqueue Download").strong().color(Color32::BLACK))
                        .fill(self.theme.primary())
                );

                if start_btn.clicked() && !self.input_url.is_empty() {
                    let cmd = GuiCommand::StartDownload {
                        url: self.input_url.clone(),
                        output_dir: self.output_dir.clone(),
                        quality: Some(self.selected_quality.clone()),
                        audio_only: self.audio_only,
                    };
                    let _ = self.runtime.send_command(cmd);
                    
                    // Add message and jump to Queue
                    self.logs.push(format!("Enqueued download: {}", self.input_url));
                    self.input_url.clear();
                    self.media_info = None;
                    self.current_tab = Tab::Queue;
                }
            });
        });
    }

    /// Queue Tab: interactive grid with progress bars
    fn draw_queue_tab(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            section_header(ui, "📊 Active Download Queue");
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui.button("Refresh Queue").clicked() {
                    let _ = self.runtime.send_command(GuiCommand::RefreshQueue);
                }
            });
        });
        ui.add_space(8.0);

        if self.items.is_empty() {
            sunken_well(ui, |ui| {
                ui.centered_and_justified(|ui| {
                    ui.label(RichText::new("No active or completed downloads in the queue.").color(Color32::from_rgb(0x9c, 0xa3, 0xaf)));
                });
            });
            return;
        }

        // Render queue in a rich table
        ScrollArea::vertical().show(ui, |ui| {
            TableBuilder::new(ui)
                .striped(true)
                .cell_layout(Layout::left_to_right(Align::Center))
                .column(Column::exact(40.0))    // ID
                .column(Column::exact(80.0))    // Platform
                .column(Column::remainder())   // Title
                .column(Column::exact(100.0))   // Status
                .column(Column::exact(140.0))   // Progress
                .column(Column::exact(80.0))    // Actions
                .header(24.0, |mut header| {
                    header.col(|ui| { ui.label("#"); });
                    header.col(|ui| { ui.label("Platform"); });
                    header.col(|ui| { ui.label("Media Title"); });
                    header.col(|ui| { ui.label("Status"); });
                    header.col(|ui| { ui.label("Progress"); });
                    header.col(|ui| { ui.label("Controls"); });
                })
                .body(|body| {
                    let items_clone = self.items.clone();
                    body.rows(32.0, items_clone.len(), |mut row| {
                        let item = &items_clone[row.index()];
                        
                        // ID
                        row.col(|ui| {
                            ui.label(RichText::new(format!("{:02}", item.id)).font(FontId::monospace(11.0)));
                        });

                        // Platform
                        row.col(|ui| {
                            brand_pill(ui, &item.platform, self.theme.secondary());
                        });

                        // Title
                        row.col(|ui| {
                            ui.label(RichText::new(&item.title).strong().color(Color32::WHITE));
                        });

                        // Status & Dot
                        row.col(|ui| {
                            ui.horizontal(|ui| {
                                status_dot(ui, &item.status);
                                ui.add_space(2.0);
                                ui.label(&item.status);
                            });
                        });

                        // Progress Bar & Speed
                        row.col(|ui| {
                            ui.vertical(|ui| {
                                ui.add_space(2.0);
                                let p = item.progress / 100.0;
                                ui.add(ProgressBar::new(p).show_percentage());
                                
                                if item.status == "Active" && item.speed > 0.0 {
                                    let speed_str = format!("{:.1} MB/s", item.speed / 1_048_576.0);
                                    ui.label(RichText::new(speed_str).font(FontId::monospace(9.0)).color(Color32::from_rgb(0x9c, 0xa3, 0xaf)));
                                }
                            });
                        });

                        // Action button controls
                        row.col(|ui| {
                            ui.horizontal(|ui| {
                                if item.status == "Active" {
                                    if ui.small_button("⏸").clicked() {
                                        let _ = self.runtime.send_command(GuiCommand::PauseDownload { id: item.id });
                                    }
                                } else if item.status == "Paused" {
                                    if ui.small_button("▶").clicked() {
                                        let _ = self.runtime.send_command(GuiCommand::ResumeDownload { id: item.id });
                                    }
                                }
                                
                                if ui.small_button("❌").clicked() {
                                    let _ = self.runtime.send_command(GuiCommand::RemoveDownload { id: item.id });
                                }
                            });
                        });
                    });
                });
        });
    }

    /// Settings Tab: engine config
    fn draw_settings_tab(&mut self, ui: &mut Ui) {
        section_header(ui, "⚙ Engine Preferences");
        ui.add_space(8.0);

        ScrollArea::vertical().show(ui, |ui| {
            surface_card(ui, |ui| {
                ui.label(RichText::new("Concurrency & Limits").strong().color(self.theme.primary()));
                ui.add_space(12.0);

                ui.horizontal(|ui| {
                    ui.label("Max Concurrent Downloads:");
                    ui.add(egui::Slider::new(&mut self.concurrent_limit, 1..=8));
                });
                
                ui.add_space(8.0);
                ui.checkbox(&mut self.auto_retry, "Automatically retry failed downloads");
            });

            ui.add_space(16.0);

            surface_card(ui, |ui| {
                ui.label(RichText::new("Graphical Customization").strong().color(self.theme.primary()));
                ui.add_space(12.0);

                ui.label("Active Brand Preset Theme:");
                ui.add_space(6.0);

                let presets = [
                    BrandPreset::PlasmCore,
                    BrandPreset::OxidizedGold,
                    BrandPreset::VioletReaction,
                    BrandPreset::CoolantLiquid,
                    BrandPreset::CriticalMass,
                ];

                ui.horizontal(|ui| {
                    for preset in presets {
                        let active = self.theme == preset;
                        let text = RichText::new(preset.name()).color(preset.primary());
                        
                        if ui.selectable_label(active, text).clicked() {
                            self.theme = preset;
                            crate::theme::apply_monolith_dark(ui.ctx(), preset);
                        }
                    }
                });
            });

            ui.add_space(16.0);

            // Engine status checks
            surface_card(ui, |ui| {
                ui.label(RichText::new("External Dependencies").strong().color(self.theme.primary()));
                ui.add_space(12.0);

                ui.horizontal(|ui| {
                    ui.label("yt-dlp Core Downloader:");
                    if self.ytdlp_installed {
                        brand_pill(ui, "INSTALLED", Color32::from_rgb(0x34, 0xa8, 0x53));
                    } else {
                        brand_pill(ui, "MISSING / RECOVERY", Color32::from_rgb(0xf2, 0x8b, 0x82));
                    }
                });
                
                ui.add_space(6.0);
                
                ui.horizontal(|ui| {
                    ui.label("ffmpeg Converter Suite:");
                    if self.ffmpeg_installed {
                        brand_pill(ui, "INSTALLED", Color32::from_rgb(0x34, 0xa8, 0x53));
                    } else {
                        brand_pill(ui, "MISSING", Color32::from_rgb(0xf2, 0x8b, 0x82));
                    }
                });

                ui.add_space(12.0);
                if ui.button("Force Re-Check Dependencies").clicked() {
                    let _ = self.runtime.send_command(GuiCommand::CheckDependencies);
                }
            });
        });
    }

    /// Logs Tab: scrollable terminal mockup
    fn draw_logs_tab(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            section_header(ui, "📝 Engine Activity Shell");
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui.button("Clear Buffer").clicked() {
                    self.logs.clear();
                }
            });
        });
        ui.add_space(6.0);

        sunken_well(ui, |ui| {
            ScrollArea::vertical()
                .auto_shrink([false, false])
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    if self.logs.is_empty() {
                        ui.label(
                            RichText::new("[SYSTEM] Idle - Listening for download tasks...")
                                .font(FontId::monospace(11.0))
                                .color(Color32::from_rgb(0x9c, 0xa3, 0xaf))
                        );
                    } else {
                        for line in &self.logs {
                            let text_color = if line.starts_with('✓') {
                                Color32::from_rgb(0x81, 0xc9, 0x95) // Success green
                            } else if line.starts_with('✗') {
                                Color32::from_rgb(0xf2, 0x8b, 0x82) // Danger red
                            } else if line.starts_with('⚙') {
                                self.theme.primary()
                            } else {
                                Color32::from_rgb(0xe5, 0xe7, 0xeb) // Neutral
                            };

                            ui.label(
                                RichText::new(line)
                                    .font(FontId::monospace(11.0))
                                    .color(text_color)
                            );
                        }
                    }
                });
        });
    }

    /// About Tab: information block
    fn draw_about_tab(&mut self, ui: &mut Ui) {
        section_header(ui, "ℹ  About MangoFetch");
        ui.add_space(8.0);

        ScrollArea::vertical().show(ui, |ui| {
            surface_card(ui, |ui| {
                ui.centered_and_justified(|ui| {
                    ui.label(
                        RichText::new("🥭")
                            .font(FontId::new(48.0, FontFamily::Proportional))
                    );
                });
                ui.add_space(12.0);

                ui.label(
                    RichText::new("MangoFetch v0.5.5")
                        .font(FontId::new(20.0, FontFamily::Proportional))
                        .strong()
                        .color(self.theme.primary())
                );
                
                ui.label("Industrial-grade high-speed concurrent media downloading utility.");
                ui.add_space(12.0);
                
                ui.label("Credits & Contributors:");
                ui.label(RichText::new("• Core Architecture & GUI: Jules Martins").strong().color(Color32::WHITE));
                ui.label("• Framework: egui + eframe (Immediate mode Desktop Suite)");
                ui.label("• Async Engine: Tokio multi-threaded runtime");
                
                ui.add_space(16.0);
                ui.separator();
                ui.add_space(8.0);

                ui.label(
                    RichText::new("LICENSE AND LEGAL")
                        .font(FontId::new(12.0, FontFamily::Monospace))
                        .strong()
                        .color(self.theme.secondary())
                );
                ui.add_space(4.0);
                ui.label("This software is licensed under the GPL-3.0-or-later License.");
            });
        });
    }

    /// Triggers url inspections
    fn fetch_preview(&mut self) {
        if !self.input_url.is_empty() {
            self.media_info_loading = true;
            self.media_info = None;
            self.media_info_error = None;
            let _ = self.runtime.send_command(GuiCommand::FetchMediaInfo { url: self.input_url.clone() });
        }
    }
}

impl eframe::App for MangoFetchApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 1. Drain pending core events
        self.drain_events();

        // Refresh system metrics occasionally
        self.sys.refresh_cpu();
        self.sys.refresh_memory();

        // 2. Render Top Command Bar
        egui::TopBottomPanel::top("top_bar")
            .frame(Frame::NONE.fill(Color32::from_rgb(0x0c, 0x0e, 0x12))) // SURFACE_1
            .show(ctx, |ui| {
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.add_space(12.0);
                    ui.label(
                        RichText::new("MANGOFETCH TERMINAL STATION")
                            .font(FontId::new(10.5, FontFamily::Monospace))
                            .color(Color32::from_rgb(0x6b, 0x72, 0x80))
                    );

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.add_space(12.0);
                        // Status connection dot
                        status_dot(ui, "Online");
                        ui.label(RichText::new("CONNECTED").font(FontId::new(10.5, FontFamily::Monospace)).color(Color32::from_rgb(0x34, 0xa8, 0x53)));
                        
                        ui.separator();
                        
                        let active_cnt = self.items.iter().filter(|i| i.status == "Active").count();
                        ui.label(format!("ACTIVE DOWNLOADS: {}", active_cnt));
                    });
                });
                ui.add_space(4.0);
            });

        // 3. Render Left Sidebar Navigation (Orbital Layout)
        egui::SidePanel::left("left_sidebar")
            .frame(Frame::NONE.fill(Color32::from_rgb(0x0c, 0x0e, 0x12))) // SURFACE_1
            .exact_width(180.0)
            .show(ctx, |ui| {
                self.render_sidebar(ui);
            });

        // 4. Render Bottom telemetry status bar
        egui::TopBottomPanel::bottom("bottom_bar")
            .frame(Frame::NONE.fill(Color32::from_rgb(0x0c, 0x0e, 0x12))) // SURFACE_1
            .show(ctx, |ui| {
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.add_space(12.0);
                    
                    // CPU and Memory Telemetry
                    let cpu_usage = self.sys.global_cpu_info().cpu_usage();
                    let total_mem = self.sys.total_memory() / 1_048_576; // MB
                    let used_mem = self.sys.used_memory() / 1_048_576; // MB
                    
                    ui.label(
                        RichText::new(format!("CPU: {:.1}% | MEM: {}/{} MB", cpu_usage, used_mem, total_mem))
                            .font(FontId::new(10.0, FontFamily::Monospace))
                            .color(Color32::from_rgb(0x9c, 0xa3, 0xaf))
                    );

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.add_space(12.0);
                        let build_lbl = format!("v0.5.5 | PRESET: {}", self.theme.name().to_uppercase());
                        ui.label(
                            RichText::new(build_lbl)
                                .font(FontId::new(10.0, FontFamily::Monospace))
                                .color(Color32::from_rgb(0x6b, 0x72, 0x80))
                        );
                    });
                });
                ui.add_space(4.0);
            });

        // 5. Render Central Content panel with Dot Background Grid
        egui::CentralPanel::default()
            .frame(Frame::NONE.fill(Color32::from_rgb(0x06, 0x06, 0x08))) // SURFACE_0
            .show(ctx, |ui| {
                // Background Dot Grid painting
                let painter = ui.painter();
                let rect = ui.max_rect();
                let dot_color = Color32::from_rgba_unmultiplied(255, 255, 255, 10); // ~4% opacity
                
                let start_x = (rect.min.x / 20.0).floor() * 20.0;
                let start_y = (rect.min.y / 20.0).floor() * 20.0;
                
                let mut x = start_x;
                while x < rect.max.x {
                    let mut y = start_y;
                    while y < rect.max.y {
                        painter.circle_filled(egui::pos2(x, y), 0.75, dot_color);
                        y += 20.0;
                    }
                    x += 20.0;
                }

                // Inner contents container with standard margin
                Frame::NONE
                    .inner_margin(Margin::same(16))
                    .show(ui, |ui| {
                        match self.current_tab {
                            Tab::Home => self.draw_home_tab(ui),
                            Tab::Queue => self.draw_queue_tab(ui),
                            Tab::Settings => self.draw_settings_tab(ui),
                            Tab::Logs => self.draw_logs_tab(ui),
                            Tab::About => self.draw_about_tab(ui),
                        }
                    });
            });

        // Loop redraw every 250ms for telemetry and queue updates
        ctx.request_repaint_after(std::time::Duration::from_millis(250));
    }
}
