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
        // Iterative count to avoid stack overflow on deep trees
        let mut count = 0u64;
        let mut stack = vec![self];
        while let Some(node) = stack.pop() {
            if node.is_dir {
                for child in &node.children {
                    stack.push(child);
                }
            } else {
                count += 1;
            }
        }
        count
    }

    pub fn dir_count(&self) -> u64 {
        let mut count = 0u64;
        let mut stack = vec![self];
        while let Some(node) = stack.pop() {
            if node.is_dir {
                count += 1;
                for child in &node.children {
                    stack.push(child);
                }
            }
        }
        count
    }

    /// Recalculate directory sizes from children (bottom-up, iterative).
    pub fn recalculate_sizes(&mut self) {
        // Post-order traversal: collect all dir nodes in pre-order, then process in reverse.
        let mut dir_indices: Vec<Vec<usize>> = Vec::new();
        let mut stack: Vec<Vec<usize>> = vec![vec![]];
        while let Some(path) = stack.pop() {
            let node = Self::get_ref(self, &path);
            if node.is_dir {
                dir_indices.push(path.clone());
                for i in 0..node.children.len() {
                    let mut child_path = path.clone();
                    child_path.push(i);
                    stack.push(child_path);
                }
            }
        }
        // Process in reverse (deepest dirs first)
        for path in dir_indices.into_iter().rev() {
            let node = Self::get_mut(self, &path);
            if node.is_dir {
                node.size = node.children.iter().map(|c| c.size).sum();
            }
        }
    }

    /// Sort children by size descending (iterative).
    pub fn sort_by_size(&mut self) {
        let mut stack: Vec<*mut FileNode> = vec![self as *mut FileNode];
        while let Some(ptr) = stack.pop() {
            // SAFETY: we own the tree exclusively via &mut self, no aliasing
            let node = unsafe { &mut *ptr };
            if node.is_dir {
                node.children.sort_by(|a, b| b.size.cmp(&a.size));
                for child in &mut node.children {
                    stack.push(child as *mut FileNode);
                }
            }
        }
    }

    fn get_ref<'a>(root: &'a FileNode, path: &[usize]) -> &'a FileNode {
        let mut node = root;
        for &i in path {
            node = &node.children[i];
        }
        node
    }

    fn get_mut<'a>(root: &'a mut FileNode, path: &[usize]) -> &'a mut FileNode {
        let mut node = root;
        for &i in path {
            node = &mut node.children[i];
        }
        node
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
