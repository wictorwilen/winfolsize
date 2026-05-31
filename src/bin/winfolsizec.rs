//! Console-subsystem twin of `winfolsize` for clean CLI usage on Windows.
//!
//! On Windows the main `winfolsize.exe` is linked with the `windows` subsystem
//! so launching the GUI doesn't flash a console. The shell then doesn't wait
//! for it, which makes CLI output appear after the prompt. `winfolsizec.exe`
//! is the same code linked as a normal console app, so `cmd`/PowerShell wait
//! for it and the output appears inline.
//!
//! On Linux/macOS this binary is functionally identical to `winfolsize`.

#[path = "../cli.rs"]
mod cli;
#[path = "../delete.rs"]
mod delete;
#[path = "../scanner/mod.rs"]
mod scanner;

use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        eprintln!(
            "winfolsizec: CLI binary. Run `winfolsizec --help` for usage, \
             or launch `winfolsize` (no args) for the GUI."
        );
        return ExitCode::from(2);
    }
    cli::run(args)
}
