use eframe::egui;

mod config;
mod engine;

#[tokio::main]
async fn main() -> eframe::Result<()> {
    // 1. Configure the native OS window settings
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([550.0, 400.0]) // Width, Height
            .with_resizable(true),
        ..Default::default()
    };
    let engine_path = youtube_dl::download_yt_dlp(std::path::Path::new("."))
        .await
        .ok();
    let ffmpeg_path = engine::ensure_windows_ffmpeg(std::path::Path::new("."))
        .await
        .ok()
        .flatten();
    let (tx, rx) = tokio::sync::mpsc::channel::<config::AppState>(32);

    // 2. Start the application loop
    eframe::run_native(
        "📥 YouTube Batch Downloader",
        native_options,
        Box::new(|_cc| Box::new(YtDownloaderApp::new(tx, rx, engine_path, ffmpeg_path))),
    )
}

// 3. This struct holds the live STATE of our user interface
struct YtDownloaderApp {
    url_input: String,
    destination_path: String,
    is_audio_only: bool,
    state: config::AppState,
    rx: tokio::sync::mpsc::Receiver<config::AppState>,
    tx: tokio::sync::mpsc::Sender<config::AppState>,
    engine_path: Option<std::path::PathBuf>,
    ffmpeg_path: Option<std::path::PathBuf>,
    selected_resolution: engine::VideoResolution,
}

// 4. Set the initial values when the app first launches
impl YtDownloaderApp {
    fn new(
        tx: tokio::sync::mpsc::Sender<config::AppState>,
        rx: tokio::sync::mpsc::Receiver<config::AppState>,
        engine_path: Option<std::path::PathBuf>,
        ffmpeg_path: Option<std::path::PathBuf>,
    ) -> Self {
        Self {
            url_input: String::new(),
            destination_path: String::from("Downloads"),
            is_audio_only: false,
            state: config::AppState::Idle(String::from("Ready")),
            rx,
            tx,
            engine_path,
            ffmpeg_path,
            selected_resolution: engine::VideoResolution::Res1080,
        }
    }
}

impl eframe::App for YtDownloaderApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        match self.rx.try_recv() {
            Ok(new_state) => {
                self.state = new_state;
                ctx.request_repaint();
            }
            Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => {
                // If the background thread dropped completely without sending a message,
                // force the UI out of the infinite loop
                if !matches!(self.state, config::AppState::Idle(_))
                    && !matches!(self.state, config::AppState::Error(_))
                {
                    self.state = config::AppState::Error(String::from(
                        "Background thread disconnected unexpectedly.",
                    ));
                    ctx.request_repaint();
                }
            }
            Err(tokio::sync::mpsc::error::TryRecvError::Empty) => {}
        }

        // Render a clean, central panel window layout
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("📥 YouTube Downloader");
            ui.add_space(10.0); // Margin space

            // Create a helper boolean to check if the app is currently idle
            let is_idle = matches!(self.state, config::AppState::Idle(_));

            // 🌟 CHANGE 2: Wrap inputs in an add_enabled_ui block so they lock during download
            ui.add_enabled_ui(is_idle, |ui| {
                // --- URL Input Component ---
                ui.label("YouTube URLs (One per line):");
                ui.add(
                    egui::TextEdit::multiline(&mut self.url_input)
                        .hint_text("https://youtube.com/...")
                        .desired_rows(4)
                        .desired_width(f32::INFINITY),
                );
                ui.add_space(10.0);

                // --- Destination Component ---
                ui.label("Save Destination:");
                ui.horizontal(|ui| {
                    ui.text_edit_singleline(&mut self.destination_path);
                    if ui.button("Browse...").clicked() {
                        // 1. Launch a native directory picker window
                        if let Some(path) = rfd::FileDialog::new().pick_folder() {
                            // 2. Convert the PathBuf securely into a lossy String and assign it to our state
                            self.destination_path = path.to_string_lossy().into_owned();
                        }
                    }
                });
                ui.add_space(10.0);

                // --- Toggle Checkbox Component ---
                ui.checkbox(&mut self.is_audio_only, "🎵 Audio Only (MP3)");
                ui.add_space(5.0);

                ui.add_enabled_ui(!self.is_audio_only, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Target Resolution Cap:");
                        egui::ComboBox::from_id_source("resolution_picker")
                            .selected_text(match self.selected_resolution {
                                engine::VideoResolution::Res1080 => "1080p (Full HD)",
                                engine::VideoResolution::Res720 => "720 (HD)",
                                engine::VideoResolution::Res480 => "480p",
                                engine::VideoResolution::Res360 => "360p",
                            })
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut self.selected_resolution,
                                    engine::VideoResolution::Res1080,
                                    "1080p (Full HD)",
                                );
                                ui.selectable_value(
                                    &mut self.selected_resolution,
                                    engine::VideoResolution::Res720,
                                    "720p (HD)",
                                );
                                ui.selectable_value(
                                    &mut self.selected_resolution,
                                    engine::VideoResolution::Res480,
                                    "480p",
                                );
                                ui.selectable_value(
                                    &mut self.selected_resolution,
                                    engine::VideoResolution::Res360,
                                    "360p",
                                );
                            })
                    })
                })
            });

            ui.add_space(20.0);

            // 🌟 CHANGE 3: Dynamic button state handling
            ui.add_enabled_ui(is_idle, |ui| {
                // Change the button text based on the active state
                let button_text = if is_idle {
                    "🚀 START DOWNLOAD"
                } else {
                    "⏳ DOWNLOADING..."
                };

                let start_button =
                    ui.add_sized([ui.available_width(), 40.0], egui::Button::new(button_text));

                if start_button.clicked() {
                    println!("Download triggered!");
                    println!("URLs to process:\n{}", self.url_input);
                    println!("Destination: {}", self.destination_path);
                    println!("Audio only mode: {}", self.is_audio_only);
                    let Some(engine_path) = self.engine_path.clone() else {
                        self.state = config::AppState::Error(String::from(
                            "yt-dlp engine not found. Restart the app.",
                        ));
                        return;
                    };
                    let ffmpeg_path = {
                        #[cfg(windows)]
                        {
                            let Some(ffmpeg_path) = self.ffmpeg_path.clone() else {
                                self.state = config::AppState::Error(String::from(
                                    "ffmpeg engine not found. Restart the app while connected to the internet.",
                                ));
                                return;
                            };
                            Some(ffmpeg_path)
                        }
                        #[cfg(not(windows))]
                        {
                            self.ffmpeg_path.clone()
                        }
                    };

                    self.state =
                        config::AppState::Downloading(String::from("Initializing engine..."));

                    let tx = self.tx.clone();
                    let parsed_urls: Vec<String> =
                        self.url_input.lines().map(|s| s.to_string()).collect();

                    let target_format = if self.is_audio_only {
                        engine::MediaFormat::Mp3
                    } else {
                        engine::MediaFormat::Mp4
                    };

                    let dest_path = self.destination_path.clone();
                    let target_res = self.selected_resolution;

                    tokio::spawn(async move {
                        let config = engine::DownloadConfig {
                            urls: parsed_urls,
                            destination_path: dest_path,
                            target_format,
                            engine_path,
                            ffmpeg_path,
                            target_resolution: target_res,
                        };

                        match engine::run_download_engine(config).await {
                            Ok(()) => {
                                let _ = tx
                                    .send(config::AppState::Idle(String::from(
                                        "Success! All files saved.",
                                    )))
                                    .await;
                            }
                            Err(e) => {
                                let _ = tx.send(config::AppState::Error(e)).await;
                            }
                        }
                    });
                }
            });
            ui.add_space(15.0);
            ui.separator(); // Draws a clean line separating the button from the status bar
            ui.add_space(5.0);

            match &self.state {
                config::AppState::Idle(msg) => {
                    ui.colored_label(
                        egui::Color32::from_rgb(100, 200, 100),
                        format!("🟢 Status: {}", msg),
                    );
                }
                config::AppState::Downloading(msg) => {
                    ui.horizontal(|ui| {
                        ui.add(egui::Spinner::new());
                        ui.colored_label(
                            egui::Color32::from_rgb(200, 200, 100),
                            format!(" {}", msg),
                        );
                    });
                }
                config::AppState::Error(msg) => {
                    // 🔴 This turns bright red and unlocks your input widgets!
                    ui.colored_label(
                        egui::Color32::from_rgb(230, 90, 90),
                        format!("❌ Error: {}", msg),
                    );
                }
            }
            if !is_idle {
                ui.add_space(5.0);
                if ui.button("⚠️ Force Stop / Reset UI").clicked() {
                    self.state = config::AppState::Idle(String::from("Engine reset manually."));
                }
            }
        });
    }
}
