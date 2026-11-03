use std::path::{Path, PathBuf};

use rabi::{parser, scanner, statement::Statement};

/// Every program in `tests/test-programs/`, sorted by name.
pub fn get_test_program_files() -> Vec<PathBuf> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/test-programs");
    let mut programs: Vec<PathBuf> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("could not read {}: {}", dir.display(), e))
        .map(|entry| entry.expect("could not read a directory entry").path())
        .filter(|path| path.is_file())
        .collect();
    programs.sort();
    programs
}

pub fn program_name(path: &Path) -> String {
    path.file_name().unwrap().to_string_lossy().into_owned()
}

pub fn parse_program(path: &Path) -> Result<Vec<Statement>, String> {
    let source = std::fs::File::open(path).map_err(|e| format!("could not open: {}", e))?;
    let tokens = scanner::scan(std::io::BufReader::new(source))?;
    parser::parse(tokens)
}

/// A program passes when it runs without errors and prints no line containing `ERROR`.
pub fn check_output(name: &str, output: &str, result: Result<(), String>) -> Result<(), String> {
    if let Err(e) = result {
        return Err(format!("{}: {}\n--- output ---\n{}", name, e, output));
    }

    let failed: Vec<&str> = output
        .lines()
        .filter(|line| line.contains("ERROR"))
        .collect();
    if !failed.is_empty() {
        return Err(format!(
            "{}: {} line(s) reported ERROR\n--- output ---\n{}",
            name,
            failed.len(),
            output
        ));
    }

    Ok(())
}

/// Runs `check` on every program and fails listing every program that failed.
pub fn check_all_programs(check: impl Fn(&Path) -> Result<(), String>) {
    let programs = get_test_program_files();
    assert!(!programs.is_empty(), "no test programs found");

    let mut failures = Vec::new();
    for program in &programs {
        if let Err(e) = check(program) {
            failures.push(e);
        }
    }

    assert!(
        failures.is_empty(),
        "{} of {} test programs failed:\n\n{}",
        failures.len(),
        programs.len(),
        failures.join("\n\n")
    );
}
