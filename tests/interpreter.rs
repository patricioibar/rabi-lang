use std::path::{Path, PathBuf};

use rabi::{interpreter::Interpreter, parser, scanner};

#[test]
fn test_integration_programs() {
    let programs = get_test_program_files();
    assert!(!programs.is_empty(), "no test programs found");

    let mut failures = Vec::new();
    for program in &programs {
        if let Err(e) = check_program(program) {
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

fn check_program(path: &Path) -> Result<(), String> {
    let name = path.file_name().unwrap().to_string_lossy().into_owned();
    let (output, result) = interpret_program(path);

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

fn interpret_program(path: &Path) -> (String, Result<(), String>) {
    let mut output = Vec::new();

    let result = (|| {
        let source = std::fs::File::open(path).map_err(|e| format!("could not open: {}", e))?;
        let tokens = scanner::scan(std::io::BufReader::new(source))?;
        let statements = parser::parse(tokens)?;
        Interpreter::with_output(&mut output).run(&statements)
    })();

    let output = String::from_utf8(output).expect("output should be valid UTF-8");
    (output, result)
}

fn get_test_program_files() -> Vec<PathBuf> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/test-programs");
    let mut programs: Vec<PathBuf> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("could not read {}: {}", dir.display(), e))
        .map(|entry| entry.expect("could not read a directory entry").path())
        .filter(|path| path.is_file())
        .collect();
    programs.sort();
    programs
}
