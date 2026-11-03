mod common;

use std::path::Path;

use rabi::interpreter::Interpreter;

#[test]
fn test_integration_programs() {
    common::check_all_programs(check_program);
}

fn check_program(path: &Path) -> Result<(), String> {
    let (output, result) = interpret_program(path);
    common::check_output(&common::program_name(path), &output, result)
}

fn interpret_program(path: &Path) -> (String, Result<(), String>) {
    let mut output = Vec::new();

    let result = common::parse_program(path)
        .and_then(|statements| Interpreter::with_output(&mut output).run(&statements));

    let output = String::from_utf8(output).expect("output should be valid UTF-8");
    (output, result)
}
