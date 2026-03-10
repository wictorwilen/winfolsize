# WinFolSize

A Windows disk space visualizer built in Rust with [egui](https://github.com/emilk/egui). Scan any folder or drive and explore where your disk space is going using interactive **treemap** and **sunburst** visualizations.

![Rust](https://img.shields.io/badge/Rust-1.75%2B-orange)
![Platform](https://img.shields.io/badge/Platform-Windows%20x64%20%7C%20ARM64-blue)

## Features

- **Treemap view** — classic WinDirStat-style rectangles sized by file/folder size
- **Sunburst view** — concentric ring chart showing directory hierarchy
- **Color by file type** — images, video, audio, documents, code, archives, etc.
- **Background scanning** — UI stays responsive with live progress updates
- **Drill-down navigation** — click folders to zoom in, back button to navigate out
- **Hover tooltips** — see file name, size, and type on hover
- **Native folder picker** — Windows file dialog for selecting scan targets
- **Sidebar details** — file type legend, summary stats, hovered item details

## Building

### Prerequisites

- [Rust](https://rustup.rs/) 1.75 or later
- [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) with C++ workload

### Build for current platform

```bash
cargo build --release
```

### Cross-compile for x64

```bash
rustup target add x86_64-pc-windows-msvc
cargo build --release --target x86_64-pc-windows-msvc
```

### Cross-compile for ARM64

```bash
rustup target add aarch64-pc-windows-msvc
cargo build --release --target aarch64-pc-windows-msvc
```

The binary will be in `target/release/winfolsize.exe` (or `target/<target>/release/`).

## Usage

1. Launch `winfolsize.exe`
2. Click **Select Folder** to choose a drive or directory
3. Click **Scan** to analyze disk usage
4. Toggle between **Treemap** and **Sunburst** views
5. Click on folders to drill in, use **Back** to navigate up
6. Hover over items to see details in the sidebar and tooltip

## File Type Categories

| Color | Category |
|-------|----------|
| 🟢 Green | Images |
| 🟣 Purple | Video |
| 🟡 Yellow | Audio |
| 🔵 Blue | Documents |
| 🟠 Orange | Code |
| 🔴 Red | Archives |
| 🩷 Pink | Executables |
| 🩵 Teal | Data |
| ⚪ Slate | System |
| ⬜ Gray | Other |

## License

MIT
