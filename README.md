# 📥 yt-downloader

A high-performance, cross-platform command-line utility built in Rust to reliably download YouTube videos in pristine quality. This tool relies on an embedded async engine wrapper that auto-provisions its own operational binaries (`yt-dlp`), requiring zero configuration from the end-user.

## 🛠️ Tech Stack

* **Language:** Rust (Stable Edition)
* **Async Runtime:** `tokio` (Multi-threaded features enabled)
* **Downloader Core:** `youtube_dl` crate (Configured with native `rustls` TLS to eliminate dynamic OpenSSL dependencies on Windows/macOS)
* **CI/CD Pipeline:** GitHub Actions (Automated matrix workflows generating binaries for Linux, macOS, and Windows on every push to `main`)

---

## 🗺️ Product Roadmap

### Phase 1: Native CLI Core (Current)
- [x] Asynchronous download handling via Tokio.
- [x] Single-binary compilation across platforms without heavy bundlers (Nuitka/PyInstaller).
- [x] Stream selection to force standard, highly compatible `.mp4` structures over `.webm`.

### Phase 2: User Interface Expansion (Next Up)
- Transition the backend engine to support an intuitive Graphical User Interface (GUI).
- Maintain single-executable portability (no heavy Electron/Node runtimes).

---

## 🚀 Next Steps: Designing the GUI Architecture

To move from our current CLI to a application window matching your goal (Input Box, Destination Picker, and Execute Button), we will integrate a native Rust GUI framework.

### Recommended GUI Framework: `slint` or `egui`
For a lightweight, single-binary application, **Slint** or **egui** are ideal choices. They compile down to native machine code, don't require web wrappers, and load instantly on your desktop.

#### Draft UI Blueprint
+-------------------------------------------------------------+
|  yt-downloader (GUI Mode)                                   |
+-------------------------------------------------------------+
|  YouTube URL:                                               |
|  [ https://www.youtube.com/watch?v=...                   ]  |
|                                                             |
|  Save Destination:                                          |
|  [ /home/ink/downloads/videos/                           ]  |
|                                                             |
|                      [ DOWNLOAD VIDEO ]                     |
+-------------------------------------------------------------+


How the Data Will Flow inside Rust:

1. **State Struct:** We will define a mutable struct to hold the string values of your text input fields.
2. **Event Loop:** The framework captures the "Click" event from your Execute button.
3. **Thread Hand-off:** The UI thread handles rendering. When the download button is pressed, it will spawn a separate `tokio::spawn` worker thread to execute our `youtube_dl` logic so your graphical application window doesn't freeze or lag during network operations.
