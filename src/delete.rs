use std::path::Path;

#[cfg(windows)]
use std::ffi::OsStr;
#[cfg(windows)]
use std::os::windows::ffi::OsStrExt;

/// Move a file or folder to the Windows Recycle Bin.
/// Returns Ok(()) on success, Err with a description on failure.
pub fn recycle(path: &Path) -> Result<(), String> {
    #[cfg(windows)]
    {
        use windows_sys::Win32::UI::Shell::{
            SHFileOperationW, SHFILEOPSTRUCTW, FO_DELETE, FOF_ALLOWUNDO, FOF_NOCONFIRMATION,
            FOF_SILENT,
        };

        let mut from: Vec<u16> = OsStr::new(path).encode_wide().collect();
        from.push(0); // null terminator
        from.push(0); // double-null terminator

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
        Err("Recycle bin is only supported on Windows".to_string())
    }
}

/// Permanently delete a file or folder (bypasses the Recycle Bin).
pub fn permanent_delete(path: &Path) -> Result<(), String> {
    if path.is_dir() {
        std::fs::remove_dir_all(path).map_err(|e| format!("Failed to delete directory: {}", e))
    } else {
        std::fs::remove_file(path).map_err(|e| format!("Failed to delete file: {}", e))
    }
}
