# 📥 yt-downloader

A high-performance, cross-platform command-line utility built in Rust to reliably download YouTube videos in pristine quality. This tool relies on an embedded async engine wrapper that auto-provisions its own operational binaries (`yt-dlp`), requiring zero configuration from the end-user.

## 🛠️ Tech Stack

* **Language:** Rust (Stable Edition)
* **Async Runtime:** `tokio` (Multi-threaded features enabled)
* **Downloader Core:** `youtube_dl` crate (Configured with native `rustls` TLS to eliminate dynamic OpenSSL dependencies on Windows/macOS)
* **CI/CD Pipeline:** GitHub Actions (Automated matrix workflows generating binaries for Linux, macOS, and Windows on every push to `main`)

---

## 🗺️ Product Roadmap

### Phase 1: Native CLI Core (Complete)
- [x] Asynchronous download handling via Tokio.
- [x] Stream selection to force standard, highly compatible `.mp4` structures over `.webm`.
- [x] Automatic local provisioning of the underlying operational binary (`yt-dlp`).

### Phase 2: Desktop User Interface Development (Current Focus)
- [x] Build a lightweight, native desktop window using a compiled Rust GUI framework (`slint` or `egui`).
- [x] Design an interactive control center featuring:
  - A multi-line text input field supporting batch processing (one URL per line).
  - A native OS file dialog button ("Browse...") to dynamically set paths.
- [x] Offload download logic to background async tasks using `tokio::spawn` to keep the UI responsive.

### Phase 3: Audio Extraction, Resolution, & Batch Parsing (Next Up)
- [ ] Implement `MediaFormat` enum parsing to natively support **M4A (Direct Stream Copy)** and **MP3 (High-Quality VBR)**.
- [ ] Integrate explicit target resolution restrictions (`360p`, `480p`, `720p`, `1080p`) via strict stream height filters.
- [ ] Add contextual UI state management (automatically graying out/disabling the Resolution dropdown when "Audio Only" is checked).
- [ ] Implement string-splitting (`.lines()`) and sanitation to process multiple URLs sequentially or concurrently.

### Phase 4: Media Post-Processing & Upscaling (Future Expansion)
- [ ] Introduce a modular "Sidecar Pattern" pipeline to invoke localized transcoders without expanding the base binary size.
- [ ] Implement resolution modifications (like basic upscaling or conversion to legacy containers) via a decoupled execution process.

---

#### Draft UI Blueprint
+-----------------------------------------------------------------+
| 📥 yt-downloader                                                |
+-----------------------------------------------------------------+
|                                                                 |
|  YouTube URLs (One per line):                                   |
|  +-----------------------------------------------------------+  |
|  | https://www.youtube.com/watch?v=dQw4w9WgXcQ               |  |
|  | https://www.youtube.com/watch?v=kJQP7kiw5Fk               |  |
|  |                                                           |  |
|  +-----------------------------------------------------------+  |
|                                                                 |
|  Save Destination:                                              |
|  [ /home/ink/downloads/videos/                      ] [Browse..] |
|                                                                 |
|  Download Options:                                              |
|  (•) Video                  ( ) Audio Only                      |
|                                                                 |
|  Resolution:                 Output Format:                     |
|  [ 1080p v ]                 [ MP4  v ]                         |
|                                                                 |
|  Progress: [=========================>-------------] 3/5 Videos |
|                                                                 |
|                       [ START DOWNLOAD ]                        |
+-----------------------------------------------------------------+


How the Data Will Flow inside Rust:

1. **State Struct:** We will define a mutable struct to hold the string values of your text input fields.
2. **Event Loop:** The framework captures the "Click" event from your Execute button.
3. **Thread Hand-off:** The UI thread handles rendering. When the download button is pressed, it will spawn a separate `tokio::spawn` worker thread to execute our `youtube_dl` logic so your graphical application window doesn't freeze or lag during network operations.
