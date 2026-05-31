use std::io::{BufRead, Read, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::mpsc;

use crate::delete;
use crate::scanner::types::{FileNode, ScanMessage};
use crate::scanner::walker;

const HELP: &str = "\
winfolsize v{VERSION} — Disk space visualizer (CLI + GUI)
(C) Wictor Wilén — MIT License — https://github.com/wictorwilen/winfolsize

USAGE:
    winfolsize                              Launch the GUI
    winfolsize scan <PATH> [OPTIONS]        Scan a directory and list largest entries
    winfolsize delete [OPTIONS]             Read paths from stdin and delete them
    winfolsize --help | -h                  Show this help
    winfolsize --version | -V               Show version

SCAN OPTIONS:
    -f, --files N           List the N largest files (default: 0)
    -d, --folders N         List the N largest folders (default: 0)
    -n, --top N             Shortcut for --files N --folders N
        --bytes             Print raw byte sizes instead of human-readable
        --paths-only        Print only paths (no size column)
    -0, --null              Separate entries with NUL bytes (for `xargs -0`)
        --json              Output as a JSON array
        --no-progress       Suppress progress output on stderr

DELETE OPTIONS:
        --recycle           Move to Recycle Bin / Trash (default)
        --permanent         Permanently delete (skip trash)
    -y, --yes               Skip confirmation prompt
    -0, --null              Read NUL-terminated input
        --dry-run           Print what would be deleted without acting

EXAMPLES:
    # 20 largest files under D:\\
    winfolsize scan D:\\ --files 20

    # 10 largest folders under /home, pipe through fzf, then send to trash
    winfolsize scan /home -d 10 | fzf -m | winfolsize delete --recycle -y

    # Permanently nuke the 5 largest files under ./build (with NUL-safe paths)
    winfolsize scan ./build -f 5 -0 | winfolsize delete --permanent -0 -y
";

const BANNER: &str = "\
winfolsize v{VERSION}
(C) Wictor Wilén — MIT License
https://github.com/wictorwilen/winfolsize
";

fn render(template: &str) -> String {
    template.replace("{VERSION}", env!("CARGO_PKG_VERSION"))
}

fn print_help() {
    println!("{}", render(HELP));
}

fn print_help_err() {
    eprintln!("{}", render(HELP));
}

pub fn run(args: Vec<String>) -> ExitCode {
    let mut iter = args.into_iter();
    let cmd = match iter.next() {
        Some(c) => c,
        None => {
            print_help_err();
            return ExitCode::from(2);
        }
    };

    match cmd.as_str() {
        "-h" | "--help" | "help" => {
            print_help();
            ExitCode::SUCCESS
        }
        "-V" | "--version" | "version" => {
            print!("{}", render(BANNER));
            ExitCode::SUCCESS
        }
        "scan" => run_scan(iter.collect()),
        "delete" | "rm" => run_delete(iter.collect()),
        other => {
            eprintln!("error: unknown command '{}'\n", other);
            print_help_err();
            ExitCode::from(2)
        }
    }
}

#[derive(Default)]
struct ScanOpts {
    path: Option<PathBuf>,
    files: usize,
    folders: usize,
    bytes: bool,
    paths_only: bool,
    null: bool,
    json: bool,
    no_progress: bool,
}

fn run_scan(args: Vec<String>) -> ExitCode {
    let mut opts = ScanOpts::default();
    let mut i = 0;
    while i < args.len() {
        let a = &args[i];
        match a.as_str() {
            "-f" | "--files" => {
                i += 1;
                opts.files = match args.get(i).and_then(|v| v.parse().ok()) {
                    Some(n) => n,
                    None => {
                        eprintln!("error: --files requires a number");
                        return ExitCode::from(2);
                    }
                };
            }
            "-d" | "--folders" | "--dirs" => {
                i += 1;
                opts.folders = match args.get(i).and_then(|v| v.parse().ok()) {
                    Some(n) => n,
                    None => {
                        eprintln!("error: --folders requires a number");
                        return ExitCode::from(2);
                    }
                };
            }
            "-n" | "--top" => {
                i += 1;
                let n: usize = match args.get(i).and_then(|v| v.parse().ok()) {
                    Some(n) => n,
                    None => {
                        eprintln!("error: --top requires a number");
                        return ExitCode::from(2);
                    }
                };
                opts.files = n;
                opts.folders = n;
            }
            "--bytes" => opts.bytes = true,
            "--paths-only" => opts.paths_only = true,
            "-0" | "--null" => opts.null = true,
            "--json" => opts.json = true,
            "--no-progress" => opts.no_progress = true,
            "-h" | "--help" => {
                print_help();
                return ExitCode::SUCCESS;
            }
            other if other.starts_with('-') => {
                eprintln!("error: unknown scan option '{}'", other);
                return ExitCode::from(2);
            }
            _ => {
                if opts.path.is_some() {
                    eprintln!("error: multiple paths given");
                    return ExitCode::from(2);
                }
                opts.path = Some(PathBuf::from(a));
            }
        }
        i += 1;
    }

    let path = match opts.path.take() {
        Some(p) => p,
        None => {
            eprintln!("error: scan requires a <PATH>\n");
            print_help_err();
            return ExitCode::from(2);
        }
    };
    if !path.exists() {
        eprintln!("error: path does not exist: {}", path.display());
        return ExitCode::from(1);
    }

    if opts.files == 0 && opts.folders == 0 {
        opts.files = 20;
    }

    let root = match scan_blocking(&path, !opts.no_progress) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("error: scan failed: {}", e);
            return ExitCode::from(1);
        }
    };

    let mut files: Vec<&FileNode> = Vec::new();
    let mut folders: Vec<&FileNode> = Vec::new();
    collect(&root, &mut files, &mut folders);
    files.sort_by(|a, b| b.size.cmp(&a.size));
    folders.sort_by(|a, b| b.size.cmp(&a.size));
    files.truncate(opts.files);
    folders.truncate(opts.folders);

    if opts.json {
        return emit_json(&files, &folders);
    }

    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    let sep: u8 = if opts.null { 0 } else { b'\n' };

    let header = !opts.paths_only && !opts.null;

    if !folders.is_empty() {
        if header {
            let _ = writeln!(out, "# Top {} folders", folders.len());
        }
        for n in &folders {
            write_entry(&mut out, n, &opts, sep);
        }
    }
    if !files.is_empty() {
        if header {
            let _ = writeln!(out, "# Top {} files", files.len());
        }
        for n in &files {
            write_entry(&mut out, n, &opts, sep);
        }
    }

    ExitCode::SUCCESS
}

fn write_entry<W: Write>(out: &mut W, node: &FileNode, opts: &ScanOpts, sep: u8) {
    if opts.paths_only {
        let _ = out.write_all(node.path.to_string_lossy().as_bytes());
    } else if opts.bytes {
        let _ = write!(out, "{}\t{}", node.size, node.path.display());
    } else {
        let human = humansize::format_size(node.size, humansize::BINARY);
        let _ = write!(out, "{:>10}\t{}", human, node.path.display());
    }
    let _ = out.write_all(&[sep]);
}

fn collect<'a>(node: &'a FileNode, files: &mut Vec<&'a FileNode>, folders: &mut Vec<&'a FileNode>) {
    let mut stack = vec![node];
    while let Some(n) = stack.pop() {
        if n.is_dir {
            // Don't include the scan root itself
            if !std::ptr::eq(n, node) {
                folders.push(n);
            }
            for c in &n.children {
                stack.push(c);
            }
        } else {
            files.push(n);
        }
    }
}

fn emit_json(files: &[&FileNode], folders: &[&FileNode]) -> ExitCode {
    fn esc(s: &str) -> String {
        let mut out = String::with_capacity(s.len() + 2);
        for c in s.chars() {
            match c {
                '"' => out.push_str("\\\""),
                '\\' => out.push_str("\\\\"),
                '\n' => out.push_str("\\n"),
                '\r' => out.push_str("\\r"),
                '\t' => out.push_str("\\t"),
                c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
                c => out.push(c),
            }
        }
        out
    }
    fn arr(items: &[&FileNode], kind: &str) -> String {
        let parts: Vec<String> = items
            .iter()
            .map(|n| {
                format!(
                    "{{\"path\":\"{}\",\"size\":{},\"type\":\"{}\"}}",
                    esc(&n.path.to_string_lossy()),
                    n.size,
                    kind
                )
            })
            .collect();
        format!("[{}]", parts.join(","))
    }
    println!(
        "{{\"folders\":{},\"files\":{}}}",
        arr(folders, "dir"),
        arr(files, "file")
    );
    ExitCode::SUCCESS
}

fn scan_blocking(path: &Path, progress: bool) -> Result<FileNode, String> {
    let (tx, rx) = mpsc::channel();
    let handle = walker::start_scan(path, tx);
    let mut last_progress = std::time::Instant::now();
    loop {
        match rx.recv() {
            Ok(ScanMessage::Progress(p)) => {
                if progress && last_progress.elapsed().as_millis() > 200 {
                    last_progress = std::time::Instant::now();
                    eprint!(
                        "\rscanning… {} files, {} dirs, {} ",
                        p.files_scanned,
                        p.dirs_scanned,
                        humansize::format_size(p.bytes_scanned, humansize::BINARY)
                    );
                    let _ = std::io::stderr().flush();
                }
            }
            Ok(ScanMessage::Complete(root)) => {
                if progress {
                    eprintln!();
                }
                let _ = handle.join();
                return Ok(root);
            }
            Ok(ScanMessage::Error(e)) => return Err(e),
            Err(e) => return Err(e.to_string()),
        }
    }
}

#[derive(Default)]
struct DeleteOpts {
    permanent: bool,
    yes: bool,
    null: bool,
    dry_run: bool,
}

fn run_delete(args: Vec<String>) -> ExitCode {
    let mut opts = DeleteOpts::default();
    for a in &args {
        match a.as_str() {
            "--recycle" | "--trash" => opts.permanent = false,
            "--permanent" | "--force" => opts.permanent = true,
            "-y" | "--yes" => opts.yes = true,
            "-0" | "--null" => opts.null = true,
            "--dry-run" => opts.dry_run = true,
            "-h" | "--help" => {
                print_help();
                return ExitCode::SUCCESS;
            }
            other => {
                eprintln!("error: unknown delete option '{}'", other);
                return ExitCode::from(2);
            }
        }
    }

    let paths = match read_paths(opts.null) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("error: failed to read paths: {}", e);
            return ExitCode::from(1);
        }
    };

    if paths.is_empty() {
        eprintln!("error: no paths read from stdin");
        return ExitCode::from(1);
    }

    if !opts.yes && !opts.dry_run {
        let action = if opts.permanent {
            "PERMANENTLY DELETE"
        } else {
            "move to trash"
        };
        eprintln!("About to {} {} item(s):", action, paths.len());
        for p in &paths {
            eprintln!("  {}", p.display());
        }
        eprint!("Proceed? [y/N] ");
        let _ = std::io::stderr().flush();
        let mut line = String::new();
        if std::io::stdin().read_line(&mut line).is_err()
            || !matches!(line.trim().to_lowercase().as_str(), "y" | "yes")
        {
            eprintln!("Aborted.");
            return ExitCode::from(1);
        }
    }

    let mut errors = 0usize;
    for p in &paths {
        if opts.dry_run {
            println!(
                "would {} {}",
                if opts.permanent { "delete" } else { "recycle" },
                p.display()
            );
            continue;
        }
        let res = if opts.permanent {
            delete::permanent_delete(p)
        } else {
            delete::recycle(p)
        };
        match res {
            Ok(()) => eprintln!("ok  {}", p.display()),
            Err(e) => {
                errors += 1;
                eprintln!("err {} — {}", p.display(), e);
            }
        }
    }

    if errors > 0 {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}

fn read_paths(null: bool) -> std::io::Result<Vec<PathBuf>> {
    let stdin = std::io::stdin();
    let mut paths = Vec::new();
    if null {
        let mut buf = Vec::new();
        stdin.lock().read_to_end(&mut buf)?;
        for chunk in buf.split(|&b| b == 0) {
            if chunk.is_empty() {
                continue;
            }
            paths.push(PathBuf::from(String::from_utf8_lossy(chunk).to_string()));
        }
    } else {
        for line in stdin.lock().lines() {
            let line = line?;
            let line = line.trim_end_matches(['\r', '\n']);
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            // Accept "SIZE\tPATH" rows from `winfolsize scan` by taking the last tab field.
            let path = line.rsplit('\t').next().unwrap_or(line).trim();
            if path.is_empty() {
                continue;
            }
            paths.push(PathBuf::from(path));
        }
    }
    Ok(paths)
}
