# DiskRay - Advanced Disk Space Analyzer

[![Rust](https://img.shields.io/badge/rust-stable-brightgreen.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Version](https://img.shields.io/badge/version-0.3.0-orange.svg)](https://github.com/yourusername/diskray/releases)

DiskRay is a powerful, user-friendly disk space analyzer built with Rust and egui. It helps you visualize disk usage, identify large files and folders, and clean up unnecessary data with ease.

![DiskRay Screenshot](screenshot.png)

## ✨ Features

- **🚀 Fast Scanning** - Parallel filesystem traversal using `jwalk` for maximum performance
- **📊 Real-time Disk Information** - Always-visible panel with detailed disk usage statistics
- **🌳 Interactive Tree View** - Browse files and folders with size and percentage display
- **📈 Extension Analysis** - See which file types consume the most space with visual progress bars
- **💾 Duplicate Finder** - Identify potential duplicate files by size
- **🗑️ Old Files Detection** - Find files not accessed for over a year
- **🎨 Clean Interface** - Dark theme by default, resizable panels, intuitive navigation
- **⚡ Optimized for HDD/SSD** - Configurable parallelism to match your hardware
- **🪟 Windows Integration** - Custom icon, no console window, proper DPI scaling

## 🖥️ Screenshots

*Main interface with tree view and extension analysis panel*

![Main Window](screenshot-main.png)

*Disk information panel showing all mounted drives*

![Disks Panel](screenshot-disks.png)

## 📦 Installation

### From Source
```bash
git clone https://github.com/yourusername/diskray.git
cd diskray
cargo build --release
./target/release/diskray.exe  # Windows
# or
./target/release/diskray       # Linux/macOS
```

### Pre-built Binaries
Download the latest release from the [Releases page](https://github.com/yourusername/diskray/releases).

## 🚀 Usage

1. **Launch DiskRay** - The main window opens with a dark theme
2. **Select a directory** - Enter a path manually, click "Browse", or use quick scan buttons
3. **Click "Scan"** - The scanner starts, showing a spinning indicator
4. **Explore results**:
   - **Left panel**: Navigate the file tree, click to select items
   - **Right panel**: View extension statistics and details of selected items
5. **Double-click folders** in the tree to expand/collapse them
6. **Use the menu** to change theme, access tools, or get help

### Quick Scan Buttons
- Disk drives (C:\, D:\, etc.) - dynamically detected
- Home directory 🏠
- Desktop 🖥️

## ⚙️ Configuration

Performance settings can be adjusted in `src/scanner.rs`:

```rust
// Sequential vs parallel scanning
const USE_PARALLEL: bool = false;  // true for SSDs, false for HDDs

// Maximum scan depth (0 = unlimited)
const MAX_DEPTH: usize = 0;

// Ignored folders (add more as needed)
const IGNORED_DIRS: &[&str] = &[
    "$Recycle.Bin",
    "System Volume Information",
];
```

## 🛠️ Building for Release

```bash
cargo build --release --features "optimize"
```

The optimized build includes:
- LTO (Link Time Optimization)
- Abort on panic
- Strip symbols
- Small binary size

## 📁 Project Structure

```
diskray/
├── src/
│   ├── main.rs          # Entry point, window setup
│   ├── app.rs           # Main application state
│   ├── scanner.rs       # Filesystem scanner with jwalk
│   ├── analyzer.rs      # File categorization and analysis
│   ├── ui/              # UI components
│   │   ├── main_panel.rs # Menu and controls
│   │   ├── tree_panel.rs # File tree with percentages
│   │   └── disks_panel.rs # Disk information panel
│   └── lib.rs           # Module exports
├── assets/
│   └── diskray.ico      # Application icon
├── build.rs             # Windows resource compilation
└── Cargo.toml           # Dependencies and metadata
```

## 🔧 Dependencies

- **eframe/egui** - GUI framework
- **jwalk** - Parallel filesystem traversal
- **sysinfo** - Disk information
- **chrono** - Date/time handling
- **humansize** - Human-readable file sizes
- **image** - Icon loading
- **rfd** - File dialogs
- **parking_lot** - Efficient synchronization

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- The Rust community for excellent crates and documentation
- egui developers for the immediate-mode GUI framework
- All contributors who help improve DiskRay

---

**DiskRay** - Because your disk space matters! ⭐
