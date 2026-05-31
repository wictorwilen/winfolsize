use std::path::Path;

/// Move a file or folder to the OS trash / recycle bin.
/// On Windows uses the Shell API directly (allows undo via Recycle Bin).
/// On Linux/macOS uses the `trash` crate (XDG trash spec / Finder trash).
pub fn recycle(path: &Path) -> Result<(), String> {
    #[cfg(windows)]
    {
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;
        use windows_sys::Win32::UI::Shell::{
            FO_DELETE, FOF_ALLOWUNDO, FOF_NOCONFIRMATION, FOF_SILENT, SHFILEOPSTRUCTW,
            SHFileOperationW,
        };

        let mut from: Vec<u16> = OsStr::new(path).encode_wide().collect();
        from.push(0);
        from.push(0);

        let file_op = SHFILEOPSTRUCTW {
            hwnd: std::ptr::null_mut(),
            wFunc: FO_DELETE as u32,
            pFrom: from.as_ptr(),
            pTo: std::ptr::null(),
            fFlags: FOF_ALLOWUNDO as u16 | FOF_NOCONFIRMATION as u16 | FOF_SILENT as u16,
            fAnyOperationsAborted: 0,
            hNameMappings: std::ptr::null_mut(),
            lpszProgressTitle: std::ptr::null(),
        };

        let result = unsafe { SHFileOperationW(&file_op as *const _ as *mut _) };
        if result == 0 {
            Ok(())
        } else {
            Err(format!("SHFileOperation failed with code {}", result))
        }
    }
    #[cfg(not(windows))]
    {
        trash::delete(path).map_err(|e| format!("Failed to move to trash: {}", e))
    }
}

/// Permanently delete a file or folder (bypasses trash / Recycle Bin).
pub fn permanent_delete(path: &Path) -> Result<(), String> {
    if path.is_dir() {
        std::fs::remove_dir_all(path).map_err(|e| format!("Failed to delete directory: {}", e))
    } else {
        std::fs::remove_file(path).map_err(|e| format!("Failed to delete file: {}", e))
    }
}
