//! `arrow` — apply one thing on another and produce a result file.
//!
//! Two behaviors, resolved automatically from the second argument:
//!
//!   arrow <program_a> <program_b> <output_file>
//!       program_b is an executable → pipe program_a's stdout into program_b's stdin.
//!
//!   arrow <program_a> <file> <output_file>
//!       file is not executable → feed it as stdin to program_a directly.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

// ── Entry point ───────────────────────────────────────────────────────────────

pub fn run(first: &str, second: &str, output_file: &str) {
    let first_path  = resolve(first);
    let second_path = resolve(second);
    let out_path    = resolve(output_file);

    if is_executable(&second_path) {
        run_pipe(&first_path, &second_path, &out_path);
    } else {
        run_apply(&first_path, &second_path, &out_path);
    }
}

// ── Pipe mode: program_a stdout → program_b stdin ────────────────────────────

fn run_pipe(prog_a: &PathBuf, prog_b: &PathBuf, out: &PathBuf) {
    let child_a = match Command::new(prog_a).stdout(Stdio::piped()).spawn() {
        Ok(c)  => c,
        Err(e) => { eprintln!("  Failed to launch '{}': {}", prog_a.display(), e); return; }
    };

    let stdout_a = match child_a.stdout {
        Some(s) => s,
        None    => { eprintln!("  Could not capture stdout of '{}'.", prog_a.display()); return; }
    };

    let child_b = match Command::new(prog_b)
        .stdin(Stdio::from(stdout_a))
        .stdout(Stdio::piped())
        .spawn()
    {
        Ok(c)  => c,
        Err(e) => { eprintln!("  Failed to launch '{}': {}", prog_b.display(), e); return; }
    };

    write_output(child_b, prog_b, out);
}

// ── Apply mode: file → program_a stdin ───────────────────────────────────────

fn run_apply(prog: &PathBuf, file: &PathBuf, out: &PathBuf) {
    let input = match fs::File::open(file) {
        Ok(f)  => f,
        Err(e) => { eprintln!("  Failed to open '{}': {}", file.display(), e); return; }
    };

    let child = match Command::new(prog)
        .stdin(Stdio::from(input))
        .stdout(Stdio::piped())
        .spawn()
    {
        Ok(c)  => c,
        Err(e) => { eprintln!("  Failed to launch '{}': {}", prog.display(), e); return; }
    };

    write_output(child, prog, out);
}

// ── Shared output writer ──────────────────────────────────────────────────────

fn write_output(child: std::process::Child, prog: &PathBuf, out: &PathBuf) {
    let output = match child.wait_with_output() {
        Ok(o)  => o,
        Err(e) => { eprintln!("  Error waiting for '{}': {}", prog.display(), e); return; }
    };

    match fs::write(out, &output.stdout) {
        Ok(()) => println!("  Done. Output written to '{}'.", out.display()),
        Err(e) => { eprintln!("  Failed to write '{}': {}", out.display(), e); return; }
    }

    if !output.status.success() {
        eprintln!("  Warning: '{}' exited with {}.", prog.display(), output.status);
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn is_executable(path: &PathBuf) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::metadata(path)
            .map(|m| m.permissions().mode() & 0o111 != 0)
            .unwrap_or(false)
    }
    #[cfg(not(unix))]
    {
        // On Windows, check for common executable extensions
        path.extension()
            .and_then(|e| e.to_str())
            .map(|e| matches!(e.to_lowercase().as_str(), "exe" | "cmd" | "bat"))
            .unwrap_or(false)
    }
}

fn resolve(s: &str) -> PathBuf {
    let p = Path::new(s);
    if p.is_absolute() {
        p.to_path_buf()
    } else {
        env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(p)
    }
}
