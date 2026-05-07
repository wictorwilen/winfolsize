<p align="center">
  <h1 align="center">🗂️ WinFolSize</h1>
  <p align="center">
    <strong>Where did all my disk space go?</strong><br>
    A blazing-fast Windows disk space visualizer built in Rust.
  </p>
</p>

<p align="center">
  <a href="https://github.com/wictorwilen/winfolsize/releases/latest"><img src="https://img.shields.io/github/v/release/wictorwilen/winfolsize?style=flat-square&color=blue" alt="Latest Release"></a>
  <a href="https://github.com/wictorwilen/winfolsize/actions/workflows/release.yml"><img src="https://img.shields.io/github/actions/workflow/status/wictorwilen/winfolsize/release.yml?style=flat-square&label=build" alt="Build Status"></a>
  <a href="LICENSE"><img src="https://img.shields.io/github/license/wictorwilen/winfolsize?style=flat-square" alt="License"></a>
  <img src="https://img.shields.io/badge/platform-Windows%20x64%20%7C%20ARM64-brightgreen?style=flat-square" alt="Platform">
  <img src="https://img.shields.io/badge/rust-1.85%2B-orange?style=flat-square&logo=rust" alt="Rust">
</p>

---

Scan any folder or drive and instantly see where your disk space is going with interactive **treemap** and **sunburst** visualizations. Powered by [egui](https://github.com/emilk/egui) for a native, GPU-accelerated experience.

<p align="center">
  <img src="assets/screenshot.png" alt="WinFolSize Screenshot" width="800">
</p>

## ✨ Features

- **🟩 Treemap view** — WinDirStat-style rectangles sized proportionally by file/folder size
- **🌀 Sunburst view** — concentric ring chart showing your directory hierarchy at a glance
- **🎨 Color by file type** — images, video, audio, documents, code, archives, and more
- **⚡ Background scanning** — UI stays buttery smooth with real-time progress updates
- **🔍 Drill-down navigation** — click folders to zoom in, back button to navigate out
- **💬 Hover tooltips** — instantly see file name, size, and type
- **📂 Native folder picker** — standard Windows file dialog
- **📊 Sidebar details** — file type legend, summary stats, and hovered item info

## 📦 Installation

Download the latest release for your architecture from the [Releases page](https://github.com/wictorwilen/winfolsize/releases/latest):

| Platform | Download |
|----------|----------|
| Windows x64 (Intel/AMD) | `winfolsize-*-windows-x86_64.zip` |
| Windows ARM64 | `winfolsize-*-windows-aarch64.zip` |

Extract the zip and run `winfolsize.exe` — no installation required.

## 🚀 Usage

1. Launch `winfolsize.exe`
2. Click **Select Folder** to choose a drive or directory
3. Click **Scan** to analyze disk usage
4. Toggle between **Treemap** and **Sunburst** views
5. Click on folders to drill in, use **Back** to navigate up
6. Hover over items to see details in the sidebar and tooltip

## 🎨 File Type Categories

| Color | Category | Extensions |
|-------|----------|------------|
| 🟢 Green | Images | .jpg, .png, .gif, .bmp, .svg, ... |
| 🟣 Purple | Video | .mp4, .mkv, .avi, .mov, ... |
| 🟡 Yellow | Audio | .mp3, .flac, .wav, .ogg, ... |
| 🔵 Blue | Documents | .pdf, .docx, .xlsx, .txt, ... |
| 🟠 Orange | Code | .rs, .py, .js, .ts, .c, ... |
| 🔴 Red | Archives | .zip, .7z, .tar, .gz, .rar, ... |
| 🩷 Pink | Executables | .exe, .dll, .msi, ... |
| 🩵 Teal | Data | .json, .xml, .csv, .db, ... |
| ⚪ Slate | System | .sys, .ini, .reg, .log, ... |
| ⬜ Gray | Other | Everything else |

## 🛠️ Building from Source

### Prerequisites

- [Rust](https://rustup.rs/) 1.85 or later
- [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) with C++ workload

### Build

```bash
cargo build --release
```

### Cross-compile for ARM64

```bash
rustup target add aarch64-pc-windows-msvc
cargo build --release --target aarch64-pc-windows-msvc
```

The binary will be in `target/release/winfolsize.exe` (or `target/<target>/release/`).

## 🤝 Contributing

Contributions are welcome! Feel free to open issues and pull requests.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📄 License

This project is licensed under the MIT License — see the [LICENSE](LICENSE) file for details.

---

<p align="center">
  Made with ❤️ and 🦀 by <a href="https://www.wictorwilen.se">Wictor Wilén</a>
</p>
