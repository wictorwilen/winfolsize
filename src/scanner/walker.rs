use std::path::Path;
use std::sync::mpsc;
use std::thread;

use super::types::{FileNode, ScanMessage, ScanProgress};

pub fn start_scan(
    root: &Path,
    sender: mpsc::Sender<ScanMessage>,
) -> thread::JoinHandle<()> {
    let root = root.to_path_buf();
    thread::spawn(move || {
        let mut files_scanned: u64 = 0;
        let mut dirs_scanned: u64 = 0;
        let mut bytes_scanned: u64 = 0;

        match scan_dir(&root, &sender, &mut files_scanned, &mut dirs_scanned, &mut bytes_scanned) {
            Ok(mut node) => {
                node.recalculate_sizes();
                node.sort_by_size();
                let _ = sender.send(ScanMessage::Complete(node));
            }
            Err(e) => {
                let _ = sender.send(ScanMessage::Error(format!("Scan failed: {}", e)));
            }
        }
    })
}

fn scan_dir(
    path: &Path,
    sender: &mpsc::Sender<ScanMessage>,
    files_scanned: &mut u64,
    dirs_scanned: &mut u64,
    bytes_scanned: &mut u64,
) -> Result<FileNode, std::io::Error> {
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string_lossy().to_string());

    let mut node = FileNode::new_dir(name, path.to_path_buf());
    *dirs_scanned += 1;

    let entries = match std::fs::read_dir(path) {
        Ok(entries) => entries,
        Err(_) => return Ok(node), // skip inaccessible directories
    };

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        let entry_path = entry.path();
        let metadata = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };

        if metadata.is_dir() {
            match scan_dir(&entry_path, sender, files_scanned, dirs_scanned, bytes_scanned) {
                Ok(child) => node.children.push(child),
                Err(_) => continue,
            }
        } else {
            let file_size = metadata.len();
            let file_name = entry_path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            let extension = entry_path
                .extension()
                .map(|e| e.to_string_lossy().to_lowercase());

            node.children.push(FileNode::new_file(
                file_name,
                entry_path,
                file_size,
                extension,
            ));

            *files_scanned += 1;
            *bytes_scanned += file_size;
        }

        // Send progress every 500 files
        if *files_scanned % 500 == 0 {
            let _ = sender.send(ScanMessage::Progress(ScanProgress {
                files_scanned: *files_scanned,
                dirs_scanned: *dirs_scanned,
                current_path: path.to_string_lossy().to_string(),
                bytes_scanned: *bytes_scanned,
            }));
        }
    }

    Ok(node)
}
