use std::env;
use std::path::Path;
use youtube_dl::YoutubeDl;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("❌ Error: Please provide a YouTube URL.");
        println!("Usage: cargo run -- <URL>");
        return Ok(());
    }

    let url = &args[1];
    let provider_path = Path::new(".");
    println!("🔍 Checking for yt-dlp engine in current folder...");

    let yt_dlp_executable = youtube_dl::download_yt_dlp(provider_path).await?;
    println!("📥 Starting download: {}", url);

    let mut downloader = YoutubeDl::new(url);
    downloader.youtube_dl_path(&yt_dlp_executable);
    downloader.format("bestvideo[ext=mp4]+bestaudio[ext=m4a]/best[ext=mp4]/best");

    downloader.download_to("downloads")?;
    println!("✅ Success! Your video is in the 'downloads' folder.");

    Ok(())
}
