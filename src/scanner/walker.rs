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

        let mut root_node = {
            let name = root
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| root.to_string_lossy().to_string());
            FileNode::new_dir(name, root.clone())
        };

        // Iterative scan using an explicit stack.
        // Each entry is (filesystem path, index path into root_node tree).
        let mut stack: Vec<(std::path::PathBuf, Vec<usize>)> = Vec::new();
        stack.push((root.clone(), Vec::new()));

        while let Some((dir_path, index_path)) = stack.pop() {
            dirs_scanned += 1;

            let entries = match std::fs::read_dir(&dir_path) {
                Ok(entries) => entries,
                Err(_) => continue,
            };

            // Collect children for this directory
            let mut child_dirs: Vec<(std::path::PathBuf, Vec<usize>)> = Vec::new();

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

                let file_name = entry_path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();

                if metadata.is_dir() {
                    let child = FileNode::new_dir(file_name, entry_path.clone());
                    let node = get_node_mut(&mut root_node, &index_path);
                    let child_idx = node.children.len();
                    node.children.push(child);

                    let mut child_path = index_path.clone();
                    child_path.push(child_idx);
                    child_dirs.push((entry_path, child_path));
                } else {
                    let file_size = metadata.len();
                    let extension = entry_path
                        .extension()
                        .map(|e| e.to_string_lossy().to_lowercase());

                    let node = get_node_mut(&mut root_node, &index_path);
                    node.children.push(FileNode::new_file(
                        file_name,
                        entry_path,
                        file_size,
                        extension,
                    ));

                    files_scanned += 1;
                    bytes_scanned += file_size;
                }

                // Send progress every 500 files
                if files_scanned % 500 == 0 {
                    let _ = sender.send(ScanMessage::Progress(ScanProgress {
                        files_scanned,
                        dirs_scanned,
                        current_path: dir_path.to_string_lossy().to_string(),
                        bytes_scanned,
                    }));
                }
            }

            // Push child dirs onto the stack (reversed so first child is processed first)
            for item in child_dirs.into_iter().rev() {
                stack.push(item);
            }
        }

        root_node.recalculate_sizes();
        root_node.sort_by_size();
        let _ = sender.send(ScanMessage::Complete(root_node));
    })
}

/// Navigate into the tree by index path to get a mutable reference.
fn get_node_mut<'a>(root: &'a mut FileNode, path: &[usize]) -> &'a mut FileNode {
    let mut node = root;
    for &idx in path {
        node = &mut node.children[idx];
    }
    node
}
