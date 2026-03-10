use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct FileNode {
    pub name: String,
    pub path: PathBuf,
    pub size: u64,
    pub is_dir: bool,
    pub extension: Option<String>,
    pub children: Vec<FileNode>,
}

impl FileNode {
    pub fn new_file(name: String, path: PathBuf, size: u64, extension: Option<String>) -> Self {
        Self {
            name,
            path,
            size,
            is_dir: false,
            extension,
            children: Vec::new(),
        }
    }

    pub fn new_dir(name: String, path: PathBuf) -> Self {
        Self {
            name,
            path,
            size: 0,
            is_dir: true,
            extension: None,
            children: Vec::new(),
        }
    }

    pub fn total_size(&self) -> u64 {
        self.size
    }

    pub fn file_count(&self) -> u64 {
        if !self.is_dir {
            return 1;
        }
        self.children.iter().map(|c| c.file_count()).sum()
    }

    pub fn dir_count(&self) -> u64 {
        if !self.is_dir {
            return 0;
        }
        1 + self.children.iter().map(|c| c.dir_count()).sum::<u64>()
    }

    /// Recalculate directory sizes from children (bottom-up).
    pub fn recalculate_sizes(&mut self) {
        if self.is_dir {
            for child in &mut self.children {
                child.recalculate_sizes();
            }
            self.size = self.children.iter().map(|c| c.size).sum();
        }
    }

    /// Sort children by size descending (recursive).
    pub fn sort_by_size(&mut self) {
        if self.is_dir {
            for child in &mut self.children {
                child.sort_by_size();
            }
            self.children.sort_by(|a, b| b.size.cmp(&a.size));
        }
    }
}

#[derive(Debug, Clone)]
pub struct ScanProgress {
    pub files_scanned: u64,
    pub dirs_scanned: u64,
    pub current_path: String,
    pub bytes_scanned: u64,
}

pub enum ScanMessage {
    Progress(ScanProgress),
    Complete(FileNode),
    Error(String),
}
