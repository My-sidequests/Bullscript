use std::fs;
use bullang::{ast::BuFile, checker, interpreter, parser};

pub fn run(path: &str) {
    let source = match fs::read_to_string(path) {
        Ok(s)  => s,
        Err(e) => {
            eprintln!("Error: could not read '{}': {}", path, e);
            return;
        }
    };

    let bu_file = match parser::parse_file(&source, false) {
        Ok(f)  => f,
        Err(e) => {
            eprintln!("Parse error in '{}': {}", path, e);
            return;
        }
    };

    let source_file = match bu_file {
        BuFile::Source(sf)    => sf,
        BuFile::Inventory(_)  => {
            eprintln!(
                "Error: '{}' is an inventory file. \
                 Only source files (.bu) can be run.",
                path
            );
            return;
        }
    };

    let violations = checker::check_no_escape(&source_file);
    if !violations.is_empty() {
        eprintln!("Error: the following bullets contain native escape blocks \
                   and cannot be interpreted:");
        for v in &violations {
            eprintln!("  '{}' — @{}", v.bullet, v.backends.join(", @"));
        }
        eprintln!();
        eprintln!("Remove the escape blocks or use 'bullarchy convert' \
                   to transpile instead.");
        return;
    }

    if let Err(e) = interpreter::run(&source_file) {
        eprintln!("Runtime error: {}", e);
    }
}
