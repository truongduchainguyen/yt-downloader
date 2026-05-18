use std::path::{Path, PathBuf};
use youtube_dl::YoutubeDl;

pub enum MediaFormat {
    Mp4,
    Mp3,
}

pub struct DownloadConfig {
    pub urls: Vec<String>,
    pub destination_path: String,
    pub target_format: MediaFormat,
    pub engine_path: PathBuf,
}

pub async fn run_download_engine(config: DownloadConfig) -> Result<(), String> {
    // Cross-platform home dir expansion
    let raw_path = if config.destination_path.starts_with('~') {
        let home = if cfg!(windows) {
            std::env::var("USERPROFILE")
                .or_else(|_| std::env::var("HOMEPATH"))
                .unwrap_or_default()
        } else {
            // Works for both Linux and macOS
            std::env::var("HOME").unwrap_or_default()
        };
        config.destination_path.replacen("~", &home, 1)
    } else {
        config.destination_path.clone()
    };

    if !Path::new(&raw_path).exists() {
        std::fs::create_dir_all(&raw_path)
            .map_err(|e| format!("Failed to create destination directory: {}", e))?;
    }

    let dest_path = if cfg!(windows) {
        // Avoid UNC paths (\\?\C:\...) that yt-dlp can't handle
        std::path::absolute(&raw_path)
            .map_err(|e| format!("Cannot resolve destination path: {}", e))?
    } else {
        // Works correctly on both macOS and Linux
        std::fs::canonicalize(&raw_path)
            .map_err(|e| format!("Cannot resolve destination path: {}", e))?
    };

    let absolute_dest_str = dest_path.to_string_lossy().to_string();
    println!("📂 Absolute destination: {}", absolute_dest_str);

    for url in config.urls {
        let clean_url = url.trim();
        if clean_url.is_empty() {
            continue;
        }

        println!("🚀 Downloading: {}", clean_url);

        let template_path = absolute_dest_str.replace('\\', "/"); // no-op on macOS/Linux
        let output_template = format!("{}/%(title)s.%(ext)s", template_path);

        let mut downloader = YoutubeDl::new(clean_url);
        downloader
            .youtube_dl_path(&config.engine_path)
            .extra_arg("--output")
            .extra_arg(&output_template);

        // Windows: illegal chars. macOS/Linux: unicode edge cases
        if cfg!(windows) {
            downloader.extra_arg("--restrict-filenames");
        } else {
            // Handles unicode normalization differences on macOS HFS+
            // and special chars on Linux without being as aggressive as --restrict-filenames
            downloader.extra_arg("--trim-filenames").extra_arg("200");
        }

        match config.target_format {
            MediaFormat::Mp3 => {
                downloader
                    .extract_audio(true)
                    .extra_arg("--audio-format")
                    .extra_arg("mp3")
                    .extra_arg("--audio-quality")
                    .extra_arg("0");
            }
            MediaFormat::Mp4 => {
                downloader
                    .format("bestvideo[ext=mp4]+bestaudio[ext=m4a]/mp4/best")
                    .extra_arg("--merge-output-format")
                    .extra_arg("mp4");
            }
        }

        downloader
            .download_to_async(&dest_path)
            .await
            .map_err(|e| format!("Download failed for {}: {}", clean_url, e))?;

        println!("✅ Downloaded to: {}", absolute_dest_str);
    }

    Ok(())
}
