use std::path::{Path, PathBuf};
use youtube_dl::YoutubeDl;

const WINDOWS_FFMPEG_URL: &str =
    "https://github.com/yt-dlp/FFmpeg-Builds/releases/latest/download/ffmpeg-master-latest-win64-gpl.zip";

pub enum MediaFormat {
    Mp4,
    Mp3,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VideoResolution {
    Res1080,
    Res720,
    Res480,
    Res360,
}

pub struct DownloadConfig {
    pub urls: Vec<String>,
    pub destination_path: String,
    pub target_format: MediaFormat,
    pub target_resolution: VideoResolution,
    pub engine_path: PathBuf,
    pub ffmpeg_path: Option<PathBuf>,
}

pub async fn ensure_windows_ffmpeg(install_dir: &Path) -> Result<Option<PathBuf>, String> {
    if !cfg!(windows) {
        return Ok(None);
    }

    std::fs::create_dir_all(install_dir)
        .map_err(|e| format!("Failed to create ffmpeg install directory: {}", e))?;

    let ffmpeg_path = install_dir.join("ffmpeg.exe");
    if ffmpeg_path.exists() {
        return std::path::absolute(ffmpeg_path)
            .map(Some)
            .map_err(|e| format!("Cannot resolve ffmpeg path: {}", e));
    }

    let zip_path = install_dir.join("ffmpeg-windows.zip");
    let extract_dir = install_dir.join("ffmpeg-windows");

    if extract_dir.exists() {
        std::fs::remove_dir_all(&extract_dir)
            .map_err(|e| format!("Failed to clear old ffmpeg directory: {}", e))?;
    }

    println!("Downloading Windows ffmpeg...");
    let response = reqwest::get(WINDOWS_FFMPEG_URL)
        .await
        .map_err(|e| format!("Failed to download ffmpeg: {}", e))?;
    if !response.status().is_success() {
        return Err(format!(
            "Failed to download ffmpeg: HTTP {}",
            response.status()
        ));
    }

    let archive_bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read ffmpeg download: {}", e))?;
    tokio::fs::write(&zip_path, &archive_bytes)
        .await
        .map_err(|e| format!("Failed to save ffmpeg download: {}", e))?;

    std::fs::create_dir_all(&extract_dir)
        .map_err(|e| format!("Failed to create ffmpeg extract directory: {}", e))?;

    let zip_path_arg = powershell_single_quoted_path(&zip_path);
    let extract_dir_arg = powershell_single_quoted_path(&extract_dir);
    let expand_archive_command = format!(
        "Expand-Archive -LiteralPath {} -DestinationPath {} -Force",
        zip_path_arg, extract_dir_arg
    );

    let status = std::process::Command::new("powershell.exe")
        .arg("-NoProfile")
        .arg("-ExecutionPolicy")
        .arg("Bypass")
        .arg("-Command")
        .arg(expand_archive_command)
        .status()
        .map_err(|e| format!("Failed to run PowerShell unzip for ffmpeg: {}", e))?;

    if !status.success() {
        return Err(String::from("Failed to extract ffmpeg archive."));
    }

    let extracted_ffmpeg = find_file_named(&extract_dir, "ffmpeg.exe")
        .ok_or_else(|| String::from("ffmpeg.exe was not found in the downloaded archive."))?;

    std::fs::copy(&extracted_ffmpeg, &ffmpeg_path)
        .map_err(|e| format!("Failed to install ffmpeg.exe: {}", e))?;

    let _ = std::fs::remove_file(&zip_path);
    let _ = std::fs::remove_dir_all(&extract_dir);

    std::path::absolute(ffmpeg_path)
        .map(Some)
        .map_err(|e| format!("Cannot resolve ffmpeg path: {}", e))
}

fn find_file_named(dir: &Path, file_name: &str) -> Option<PathBuf> {
    let entries = std::fs::read_dir(dir).ok()?;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file()
            && path
                .file_name()
                .is_some_and(|name| name.to_string_lossy().eq_ignore_ascii_case(file_name))
        {
            return Some(path);
        }

        if path.is_dir() {
            if let Some(found) = find_file_named(&path, file_name) {
                return Some(found);
            }
        }
    }

    None
}

fn powershell_single_quoted_path(path: &Path) -> String {
    let escaped = path.to_string_lossy().replace('\'', "''");
    format!("'{}'", escaped)
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

        if cfg!(windows) {
            let ffmpeg_path = config.ffmpeg_path.as_ref().ok_or_else(|| {
                String::from(
                    "ffmpeg engine not found. Restart the app while connected to the internet.",
                )
            })?;
            let ffmpeg_location = ffmpeg_path.to_string_lossy().to_string();
            downloader
                .extra_arg("--ffmpeg-location")
                .extra_arg(&ffmpeg_location);
        }

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
                let height_cap = match config.target_resolution {
                    VideoResolution::Res1080 => "1080",
                    VideoResolution::Res720 => "720",
                    VideoResolution::Res480 => "480",
                    VideoResolution::Res360 => "360",
                };
                let format_rule = format!(
                    "bestvideo[ext=mp4][height<=?{}]+bestaudio[ext=m4a]/mp4[height<=?{}]/best[height<=?{}]",
                    height_cap, height_cap, height_cap
                );
                downloader
                    .format(&format_rule)
                    .extra_arg("--merge-output-format")
                    .extra_arg("mp4")
                    .extra_arg("--force-overwrites");
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
