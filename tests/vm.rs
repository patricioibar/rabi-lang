mod common;

use std::path::Path;

use rabi::{compiler, interpreter::Interpreter, vm::Vm};

#[test]
fn test_integration_programs() {
    common::check_all_programs(check_program);
}

/// The VM must pass the program and print exactly what the interpreter prints.
fn check_program(path: &Path) -> Result<(), String> {
    let name = common::program_name(path);
    let (output, result) = run_program(path);
    common::check_output(&name, &output, result)?;

    let (expected, _) = interpret_program(path);
    if output != expected {
        return Err(format!(
            "{}: VM output differs from the interpreter\n--- vm ---\n{}\n--- interpreter ---\n{}",
            name, output, expected
        ));
    }

    Ok(())
}

fn run_program(path: &Path) -> (String, Result<(), String>) {
    let mut output = Vec::new();

    let result = common::parse_program(path)
        .and_then(compiler::compile)
        .and_then(|function| Vm::with_output(&mut output).run(function));

    let output = String::from_utf8(output).expect("output should be valid UTF-8");
    (output, result)
}

fn interpret_program(path: &Path) -> (String, Result<(), String>) {
    let mut output = Vec::new();

    let result = common::parse_program(path)
        .and_then(|statements| Interpreter::with_output(&mut output).run(&statements));

    let output = String::from_utf8(output).expect("output should be valid UTF-8");
    (output, result)
}
