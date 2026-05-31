#![cfg_attr(all(not(debug_assertions), windows), windows_subsystem = "windows")]

mod app;
mod cli;
mod delete;
mod scanner;
mod ui;
mod viz;

use std::process::ExitCode;

fn make_icon() -> eframe::egui::IconData {
    let size = 64u32;
    let mut rgba = vec![0u8; (size * size * 4) as usize];
    let blocks: &[(u32, u32, u32, u32, [u8; 3])] = &[
        (0, 0, 38, 38, [66, 133, 244]),
        (40, 0, 24, 18, [234, 67, 53]),
        (40, 20, 24, 18, [251, 188, 4]),
        (0, 40, 20, 24, [52, 168, 83]),
        (22, 40, 42, 24, [171, 71, 188]),
    ];
    for &(bx, by, bw, bh, color) in blocks {
        for y in by..by + bh {
            for x in bx..bx + bw {
                let i = ((y * size + x) * 4) as usize;
                rgba[i] = color[0];
                rgba[i + 1] = color[1];
                rgba[i + 2] = color[2];
                rgba[i + 3] = 255;
            }
        }
    }
    eframe::egui::IconData {
        rgba,
        width: size,
        height: size,
    }
}

fn run_gui() -> ExitCode {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0])
            .with_title("WinFolSize — Disk Space Visualizer")
            .with_icon(std::sync::Arc::new(make_icon())),
        ..Default::default()
    };

    match eframe::run_native(
        "WinFolSize",
        options,
        Box::new(|cc| Ok(Box::new(app::WinFolSizeApp::new(cc)))),
    ) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: failed to launch GUI: {}", e);
            ExitCode::from(1)
        }
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        run_gui()
    } else {
        #[cfg(windows)]
        attach_parent_console();
        cli::run(args)
    }
}

/// On Windows release builds the binary is linked with the `windows`
/// subsystem (so the GUI doesn't pop a console window). That also means
/// the process starts with no attached console and Rust's stdio handles
/// point at NULL, so `winfolsize --version` would print nothing. For CLI
/// invocations we attach to the parent terminal (if any) AND rebind the
/// standard handles to it so println!/eprintln! become visible.
#[cfg(windows)]
fn attach_parent_console() {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Foundation::{HANDLE, INVALID_HANDLE_VALUE};
    use windows_sys::Win32::Storage::FileSystem::{
        CreateFileW, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING,
    };
    use windows_sys::Win32::System::Console::{
        ATTACH_PARENT_PROCESS, AttachConsole, GetStdHandle, STD_ERROR_HANDLE, STD_INPUT_HANDLE,
        STD_OUTPUT_HANDLE, SetStdHandle,
    };

    const GENERIC_READ: u32 = 0x8000_0000;
    const GENERIC_WRITE: u32 = 0x4000_0000;

    unsafe {
        if AttachConsole(ATTACH_PARENT_PROCESS) == 0 {
            return;
        }

        let to_wide = |s: &str| -> Vec<u16> {
            OsStr::new(s)
                .encode_wide()
                .chain(std::iter::once(0))
                .collect()
        };

        // Only rebind a std handle if it's currently null/invalid (i.e. the
        // process was started without any redirection on that stream).
        // If cmd/PowerShell redirected with '>' or '|', leave it alone.
        let needs_bind = |which: u32| -> bool {
            let h: HANDLE = GetStdHandle(which);
            h.is_null() || h == INVALID_HANDLE_VALUE
        };

        if needs_bind(STD_OUTPUT_HANDLE) || needs_bind(STD_ERROR_HANDLE) {
            let conout = to_wide("CONOUT$");
            let h_out = CreateFileW(
                conout.as_ptr(),
                GENERIC_WRITE,
                FILE_SHARE_READ | FILE_SHARE_WRITE,
                std::ptr::null(),
                OPEN_EXISTING,
                0,
                std::ptr::null_mut(),
            );
            if h_out != INVALID_HANDLE_VALUE {
                if needs_bind(STD_OUTPUT_HANDLE) {
                    SetStdHandle(STD_OUTPUT_HANDLE, h_out);
                }
                if needs_bind(STD_ERROR_HANDLE) {
                    SetStdHandle(STD_ERROR_HANDLE, h_out);
                }
            }
        }
        if needs_bind(STD_INPUT_HANDLE) {
            let conin = to_wide("CONIN$");
            let h_in = CreateFileW(
                conin.as_ptr(),
                GENERIC_READ,
                FILE_SHARE_READ | FILE_SHARE_WRITE,
                std::ptr::null(),
                OPEN_EXISTING,
                0,
                std::ptr::null_mut(),
            );
            if h_in != INVALID_HANDLE_VALUE {
                SetStdHandle(STD_INPUT_HANDLE, h_in);
            }
        }
    }
}
