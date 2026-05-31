use std::fs;
use std::io::{self, Write};
use std::path::Path;

use bullang::ast::BuFile;
use bullang::fmt::format_source;
use bullang::parser::parse_file;

const MAX_PIPES: usize = 5;

// ── Entry point ───────────────────────────────────────────────────────────────

pub fn run() {
    // ── Step 1: filename ──────────────────────────────────────────────────────
    let filename = match ask("  file -> ") {
        s if s.is_empty() => { println!("  Aborted."); return; }
        s => s,
    };

    // Ensure .bu extension
    let filename = if filename.ends_with(".bu") {
        filename
    } else {
        format!("{}.bu", filename)
    };

    let file_exists = Path::new(&filename).exists();
    if file_exists {
        println!("  '{}' already exists — function will be appended.", filename);
    } else {
        println!("  '{}' will be created.", filename);
    }

    // ── Step 2: prototype ─────────────────────────────────────────────────────
    println!("  Enter the function prototype.");
    println!("  Example: let add(a: i32, b: i32) -> result: i32");
    let prototype = match ask("  prototype -> ") {
        s if s.is_empty() => { println!("  Aborted."); return; }
        s => s,
    };

    // ── Step 3: pipes ─────────────────────────────────────────────────────────
    println!(
        "  Enter up to {} pipes. Type 'conclude' to finish early.",
        MAX_PIPES
    );
    println!("  Example: (a, b) : a + b -> {{result}};");

    let mut pipes: Vec<String> = Vec::new();

    loop {
        if pipes.len() >= MAX_PIPES {
            println!("  Maximum of {} pipes reached.", MAX_PIPES);
            break;
        }

        let label = format!("  pipe {} -> ", pipes.len() + 1);
        let input = ask(&label);

        if input == "conclude" {
            break;
        }
        if input.is_empty() {
            println!("  (empty input — try again, or type 'conclude' to finish)");
            continue;
        }

        pipes.push(input);
    }

    if pipes.is_empty() {
        println!("  No pipes entered. Aborted.");
        return;
    }

    // ── Step 4: assemble + parse + pretty-print ───────────────────────────────
    let raw_src  = assemble(&prototype, &pipes);
    let (display, valid) = pretty_print(&raw_src);

    println!("\n  Preview:\n");
    for line in display.lines() {
        println!("  {}", line);
    }
    println!();

    if !valid {
        println!("  (Warning: the function above could not be parsed.");
        println!("   It will be saved as-is if you choose to continue.)");
        println!();
    }

    // ── Step 5: save prompt ───────────────────────────────────────────────────
    let answer = ask("  Save? (Y/n) -> ");

    if answer.eq_ignore_ascii_case("n") {
        println!("  Discarded. Back to command prompt.");
        return;
    }

    match write_function(&filename, &display, file_exists) {
        Ok(())  => println!("  Saved to '{}'.", filename),
        Err(e)  => eprintln!("  Error writing '{}': {}", filename, e),
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Read one line from stdin, trimmed, with a printed prompt.
fn ask(label: &str) -> String {
    print!("{}", label);
    io::stdout().flush().unwrap();
    let mut buf = String::new();
    io::stdin().read_line(&mut buf).unwrap_or(0);
    buf.trim().to_string()
}

/// Assemble prototype + pipe lines into a syntactically complete bullet source.
fn assemble(prototype: &str, pipes: &[String]) -> String {
    // Strip any trailing `{` the user may have added
    let proto = prototype.trim_end_matches(|c: char| c == '{' || c.is_whitespace());
    let mut src = format!("{} {{\n", proto);
    for pipe in pipes {
        src.push_str(&format!("    {}\n", pipe));
    }
    src.push_str("}\n");
    src
}

/// Try to parse and pretty-print `src`.
/// Returns (display_string, was_valid).
/// If parsing fails, returns the raw assembled source and false.
fn pretty_print(src: &str) -> (String, bool) {
    match parse_file(src, false) {
        Ok(BuFile::Source(sf)) => (format_source(&sf), true),
        _ => (src.to_string(), false),
    }
}

/// Append `content` to an existing file (separated by a blank line),
/// or create the file if it does not yet exist.
fn write_function(filename: &str, content: &str, append: bool) -> io::Result<()> {
    if append {
        let existing = fs::read_to_string(filename)?;
        // Ensure exactly one blank line between functions
        let trimmed  = existing.trim_end();
        let combined = format!("{}\n\n{}", trimmed, content);
        fs::write(filename, combined)
    } else {
        fs::write(filename, content)
    }
}
