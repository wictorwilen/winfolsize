use eframe::egui::Color32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FileCategory {
    Images,
    Video,
    Audio,
    Documents,
    Code,
    Archives,
    Executables,
    Data,
    System,
    Other,
}

impl FileCategory {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Images => "Images",
            Self::Video => "Video",
            Self::Audio => "Audio",
            Self::Documents => "Documents",
            Self::Code => "Code",
            Self::Archives => "Archives",
            Self::Executables => "Executables",
            Self::Data => "Data",
            Self::System => "System",
            Self::Other => "Other",
        }
    }

    pub fn color(&self) -> Color32 {
        match self {
            Self::Images => Color32::from_rgb(46, 204, 113),     // green
            Self::Video => Color32::from_rgb(155, 89, 182),      // purple
            Self::Audio => Color32::from_rgb(241, 196, 15),      // yellow
            Self::Documents => Color32::from_rgb(52, 152, 219),  // blue
            Self::Code => Color32::from_rgb(230, 126, 34),       // orange
            Self::Archives => Color32::from_rgb(231, 76, 60),    // red
            Self::Executables => Color32::from_rgb(236, 72, 153),// pink
            Self::Data => Color32::from_rgb(20, 184, 166),       // teal
            Self::System => Color32::from_rgb(148, 163, 184),    // slate
            Self::Other => Color32::from_rgb(107, 114, 128),     // gray
        }
    }

    pub fn all() -> &'static [FileCategory] {
        &[
            Self::Images,
            Self::Video,
            Self::Audio,
            Self::Documents,
            Self::Code,
            Self::Archives,
            Self::Executables,
            Self::Data,
            Self::System,
            Self::Other,
        ]
    }
}

pub fn categorize_extension(ext: &str) -> FileCategory {
    match ext {
        // Images
        "jpg" | "jpeg" | "png" | "gif" | "bmp" | "svg" | "webp" | "ico" | "tiff" | "tif"
        | "raw" | "cr2" | "nef" | "psd" | "ai" | "heic" | "heif" | "avif" => FileCategory::Images,

        // Video
        "mp4" | "avi" | "mkv" | "mov" | "wmv" | "flv" | "webm" | "m4v" | "mpg" | "mpeg"
        | "3gp" | "ts" | "mts" => FileCategory::Video,

        // Audio
        "mp3" | "wav" | "flac" | "aac" | "ogg" | "wma" | "m4a" | "opus" | "mid" | "midi" => {
            FileCategory::Audio
        }

        // Documents
        "pdf" | "doc" | "docx" | "xls" | "xlsx" | "ppt" | "pptx" | "odt" | "ods" | "odp"
        | "txt" | "rtf" | "csv" | "md" | "epub" | "pages" | "numbers" | "key" => {
            FileCategory::Documents
        }

        // Code
        "rs" | "py" | "js" | "jsx" | "tsx" | "c" | "cpp" | "h" | "hpp" | "java"
        | "go" | "rb" | "php" | "swift" | "kt" | "cs" | "html" | "css" | "scss" | "sass"
        | "less" | "vue" | "svelte" | "sh" | "bash" | "ps1" | "psm1" | "json" | "yaml"
        | "yml" | "toml" | "xml" | "sql" | "r" | "lua" | "dart" | "zig" | "nim" | "ex"
        | "exs" | "erl" | "hs" | "ml" | "clj" | "scala" | "v" | "vhdl" => FileCategory::Code,

        // Archives
        "zip" | "rar" | "7z" | "tar" | "gz" | "bz2" | "xz" | "zst" | "lz4" | "cab"
        | "iso" | "dmg" | "img" => FileCategory::Archives,

        // Executables
        "exe" | "msi" | "dll" | "so" | "dylib" | "app" | "bat" | "cmd" | "com" | "scr"
        | "apk" | "deb" | "rpm" | "appimage" => FileCategory::Executables,

        // Data / databases
        "db" | "sqlite" | "sqlite3" | "mdb" | "accdb" | "parquet" | "arrow" | "feather"
        | "hdf5" | "h5" | "npy" | "npz" | "pkl" | "pickle" | "dat" | "bin" | "bak"
        | "log" | "tmp" => FileCategory::Data,

        // System
        "sys" | "drv" | "inf" | "ini" | "cfg" | "conf" | "reg" | "lnk" | "url"
        | "desktop" | "plist" => FileCategory::System,

        _ => FileCategory::Other,
    }
}

pub fn color_for_extension(ext: Option<&str>) -> Color32 {
    match ext {
        Some(e) => categorize_extension(e).color(),
        None => FileCategory::Other.color(),
    }
}

pub fn category_for_extension(ext: Option<&str>) -> FileCategory {
    match ext {
        Some(e) => categorize_extension(e),
        None => FileCategory::Other,
    }
}
