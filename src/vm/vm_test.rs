use std::rc::Rc;

use super::Vm;
use crate::bytecode::{Function, Operation as Op, Value};
use crate::compiler::{Chunk, compile};
use crate::{optimizer, parser, scanner};

/// Runs hand-assembled bytecode as the top-level script.
fn execute(constants: Vec<Value>, code: Vec<Op>) -> Result<String, String> {
    let function = Function {
        name: "<script>".to_string(),
        arity: 0,
        chunk: Chunk { code, constants },
    };
    let mut output = Vec::new();
    Vm::with_output(&mut output).run(function)?;
    String::from_utf8(output).map_err(|e| format!("output was not valid UTF-8: {e}"))
}

/// Compiles `source` and runs it on the VM.
fn run(source: &str) -> Result<String, String> {
    let (output, result) = run_keeping_output(source);
    result.map(|_| output)
}

/// Like [`run`], but also returns what was printed before an error.
fn run_keeping_output(source: &str) -> (String, Result<(), String>) {
    let mut output = Vec::new();
    let result = (|| {
        let tokens = scanner::scan(source.as_bytes())?;
        let statements = parser::parse(tokens)?;
        let function = compile(optimizer::optimize(statements))?;
        Vm::with_output(&mut output).run(function)
    })();
    let output = String::from_utf8(output).expect("output should be valid UTF-8");
    (output, result)
}

fn string(s: &str) -> Value {
    Value::String(Rc::new(s.to_string()))
}

// ================================================================ bytecode

// ------------------------------------------------------------------ constants

#[test]
fn prints_constants_and_literal_operations() {
    let output = execute(
        vec![Value::Integer(42), string("hi")],
        vec![
            Op::GetConstant(0),
            Op::Print,
            Op::GetConstant(1),
            Op::Print,
            Op::True,
            Op::Print,
            Op::Null,
            Op::Print,
            Op::Null,
            Op::Return,
        ],
    );
    assert_eq!(output.unwrap(), "42\nhi\ntrue\nnull\n");
}

#[test]
fn a_constant_out_of_bounds_is_an_error() {
    let error = execute(vec![], vec![Op::GetConstant(3), Op::Return]).unwrap_err();
    assert!(error.contains("Constant 3 out of bounds"), "{error}");
}

#[test]
fn running_past_the_end_of_the_code_is_an_error() {
    let error = execute(vec![], vec![Op::Null, Op::Pop]).unwrap_err();
    assert!(error.contains("without a Return"), "{error}");
}

#[test]
fn errors_are_prefixed() {
    let error = execute(vec![], vec![Op::GetConstant(0)]).unwrap_err();
    assert!(error.starts_with("Error executing: "), "{error}");
}

// ------------------------------------------------------------------- operators

#[test]
fn binary_operations_take_the_left_operand_from_deeper_in_the_stack() {
    let output = execute(
        vec![Value::Integer(10), Value::Integer(4)],
        vec![
            Op::GetConstant(0),
            Op::GetConstant(1),
            Op::Subtract,
            Op::Print,
            Op::GetConstant(0),
            Op::GetConstant(1),
            Op::Less,
            Op::Print,
            Op::Null,
            Op::Return,
        ],
    );
    assert_eq!(output.unwrap(), "6\nfalse\n");
}

#[test]
fn not_and_negate_replace_the_top_of_the_stack() {
    let output = execute(
        vec![Value::Integer(3)],
        vec![
            Op::GetConstant(0),
            Op::Negate,
            Op::Print,
            Op::GetConstant(0),
            Op::Not,
            Op::Print,
            Op::Null,
            Op::Return,
        ],
    );
    assert_eq!(output.unwrap(), "-3\nfalse\n");
}

#[test]
fn popping_an_empty_stack_is_an_error() {
    let error = execute(vec![], vec![Op::Pop, Op::Pop, Op::Return]).unwrap_err();
    assert!(error.contains("Stack underflow"), "{error}");
}

// --------------------------------------------------------------------- globals

#[test]
fn defines_reads_and_assigns_globals_by_name() {
    let output = execute(
        vec![Value::Integer(1), string("x"), Value::Integer(2)],
        vec![
            Op::GetConstant(0),
            Op::DefineGlobal(1),
            Op::GetConstant(2),
            Op::SetGlobal(1),
            Op::Print,
            Op::GetGlobal(1),
            Op::Print,
            Op::Null,
            Op::Return,
        ],
    );
    assert_eq!(output.unwrap(), "2\n2\n");
}

#[test]
fn reading_an_undefined_global_is_an_error() {
    let error = execute(vec![string("x")], vec![Op::GetGlobal(0), Op::Return]).unwrap_err();
    assert!(error.contains("'x' not defined"), "{error}");
}

#[test]
fn assigning_an_undefined_global_is_an_error() {
    let error = execute(
        vec![string("x")],
        vec![Op::Null, Op::SetGlobal(0), Op::Return],
    )
    .unwrap_err();
    assert!(error.contains("'x' not defined"), "{error}");
}

#[test]
fn a_global_name_must_be_a_string_constant() {
    let error = execute(
        vec![Value::Integer(1)],
        vec![Op::Null, Op::DefineGlobal(0), Op::Null, Op::Return],
    )
    .unwrap_err();
    assert!(error.contains("variable name"), "{error}");
}

// ---------------------------------------------------------------------- locals

#[test]
fn locals_are_addressed_relative_to_the_frame() {
    // slot 0 holds the script itself, so the first local lives in slot 1
    let output = execute(
        vec![Value::Integer(7), Value::Integer(8)],
        vec![
            Op::GetConstant(0),
            Op::GetConstant(1),
            Op::SetLocal(1),
            Op::Pop,
            Op::GetLocal(1),
            Op::Print,
            Op::Null,
            Op::Return,
        ],
    );
    assert_eq!(output.unwrap(), "8\n");
}

#[test]
fn a_local_slot_out_of_bounds_is_an_error() {
    let error = execute(vec![], vec![Op::GetLocal(5), Op::Return]).unwrap_err();
    assert!(error.contains("Local slot out of bounds"), "{error}");
    let error = execute(vec![], vec![Op::Null, Op::SetLocal(5), Op::Return]).unwrap_err();
    assert!(error.contains("Local slot out of bounds"), "{error}");
}

// ----------------------------------------------------------------------- jumps

#[test]
fn jump_skips_forward_relative_to_the_next_operation() {
    let output = execute(
        vec![Value::Integer(1), Value::Integer(2)],
        vec![
            Op::Jump(2),
            Op::GetConstant(0),
            Op::Print,
            Op::GetConstant(1),
            Op::Print,
            Op::Null,
            Op::Return,
        ],
    );
    assert_eq!(output.unwrap(), "2\n");
}

#[test]
fn jump_if_false_jumps_only_on_falsy_values_and_keeps_the_condition() {
    let output = execute(
        vec![],
        vec![
            Op::False,
            Op::JumpIfFalse(2),
            Op::True,
            Op::Print,
            Op::Print,
            Op::True,
            Op::JumpIfFalse(1),
            Op::Print,
            Op::Null,
            Op::Return,
        ],
    );
    assert_eq!(output.unwrap(), "false\ntrue\n");
}

#[test]
fn loop_jumps_backwards() {
    // counts a global from 0 to 2
    let constants = vec![
        Value::Integer(0),
        string("i"),
        Value::Integer(3),
        Value::Integer(1),
    ];
    let code = vec![
        Op::GetConstant(0),
        Op::DefineGlobal(1),
        Op::GetGlobal(1), // 2: loop start
        Op::GetConstant(2),
        Op::Less,
        Op::JumpIfFalse(9),
        Op::Pop,
        Op::GetGlobal(1),
        Op::Print,
        Op::GetGlobal(1),
        Op::GetConstant(3),
        Op::Add,
        Op::SetGlobal(1),
        Op::Pop,
        Op::Loop(13),
        Op::Pop,
        Op::Null,
        Op::Return,
    ];
    assert_eq!(execute(constants, code).unwrap(), "0\n1\n2\n");
}

// ----------------------------------------------------------------------- calls

fn function(name: &str, arity: usize, constants: Vec<Value>, code: Vec<Op>) -> Value {
    Value::Function(Rc::new(Function {
        name: name.to_string(),
        arity,
        chunk: Chunk { code, constants },
    }))
}

#[test]
fn a_call_passes_arguments_in_the_slots_after_the_function() {
    let add = function(
        "add",
        2,
        vec![],
        vec![Op::GetLocal(1), Op::GetLocal(2), Op::Add, Op::Return],
    );
    let output = execute(
        vec![add, Value::Integer(2), Value::Integer(3)],
        vec![
            Op::GetConstant(0),
            Op::GetConstant(1),
            Op::GetConstant(2),
            Op::Call(2),
            Op::Print,
            Op::Null,
            Op::Return,
        ],
    );
    assert_eq!(output.unwrap(), "5\n");
}

#[test]
fn return_discards_the_frame_and_leaves_only_the_result() {
    let seven = function(
        "seven",
        0,
        vec![Value::Integer(7), Value::Integer(99)],
        vec![
            Op::GetConstant(1),
            Op::GetConstant(1),
            Op::GetConstant(0),
            Op::Return,
        ],
    );
    // the local below the call must still be in slot 1 after the call returns
    let output = execute(
        vec![Value::Integer(1), seven],
        vec![
            Op::GetConstant(0),
            Op::GetConstant(1),
            Op::Call(0),
            Op::Print,
            Op::GetLocal(1),
            Op::Print,
            Op::Null,
            Op::Return,
        ],
    );
    assert_eq!(output.unwrap(), "7\n1\n");
}

#[test]
fn calling_a_non_function_is_an_error() {
    let error = execute(
        vec![Value::Integer(1)],
        vec![Op::GetConstant(0), Op::Call(0), Op::Return],
    )
    .unwrap_err();
    assert!(error.contains("non-function"), "{error}");
}

#[test]
fn calling_with_the_wrong_number_of_arguments_is_an_error() {
    let identity = function("identity", 1, vec![], vec![Op::GetLocal(1), Op::Return]);
    let error = execute(
        vec![identity],
        vec![Op::GetConstant(0), Op::Call(0), Op::Return],
    )
    .unwrap_err();
    assert!(error.contains("Expected 1 arguments but got 0"), "{error}");
}

// ---------------------------------------------------------------------- arrays

#[test]
fn build_array_collects_the_top_values_in_push_order() {
    let output = execute(
        vec![Value::Integer(1), Value::Integer(2)],
        vec![
            Op::GetConstant(0),
            Op::GetConstant(1),
            Op::BuildArray(2),
            Op::Print,
            Op::Null,
            Op::Return,
        ],
    );
    assert_eq!(output.unwrap(), "[1, 2]\n");
}

#[test]
fn set_index_leaves_the_assigned_value_on_the_stack() {
    let output = execute(
        vec![Value::Integer(0), Value::Integer(7)],
        vec![
            Op::Null,
            Op::BuildArray(1),
            Op::GetConstant(0),
            Op::GetConstant(1),
            Op::SetIndex,
            Op::Print,
            Op::Null,
            Op::Return,
        ],
    );
    assert_eq!(output.unwrap(), "7\n");
}

#[test]
fn build_array_with_too_few_values_is_a_stack_underflow() {
    let error = execute(vec![], vec![Op::BuildArray(5), Op::Return]).unwrap_err();
    assert!(error.contains("Stack underflow"), "{error}");
}

#[test]
fn for_iter_on_an_empty_array_jumps_without_pushing() {
    let output = execute(
        vec![Value::Integer(0)],
        vec![
            Op::BuildArray(0),
            Op::GetConstant(0),
            Op::ForIter(2),
            Op::Print,
            Op::Return,
            Op::Print,
            Op::Print,
            Op::Null,
            Op::Return,
        ],
    );
    assert_eq!(output.unwrap(), "0\n[]\n");
}

#[test]
fn for_iter_pushes_the_element_and_advances_the_counter() {
    let output = execute(
        vec![Value::Integer(7), Value::Integer(0)],
        vec![
            Op::GetConstant(0),
            Op::BuildArray(1),
            Op::GetConstant(1),
            Op::ForIter(4),
            Op::Print,
            Op::Print,
            Op::Null,
            Op::Return,
        ],
    );
    assert_eq!(output.unwrap(), "7\n1\n");
}

#[test]
fn for_iter_rejects_an_iterable_that_is_not_an_array() {
    let error = execute(
        vec![Value::Integer(0)],
        vec![Op::True, Op::GetConstant(0), Op::ForIter(0), Op::Return],
    )
    .unwrap_err();
    assert!(
        error.contains("Unsupported type for iteration: Boolean"),
        "{error}"
    );
}

#[test]
fn for_iter_rejects_a_counter_that_is_not_an_integer() {
    let error = execute(
        vec![],
        vec![Op::BuildArray(0), Op::Null, Op::ForIter(0), Op::Return],
    )
    .unwrap_err();
    assert!(error.contains("Corrupted for loop"), "{error}");
}

// ================================================================== programs

// ---------------------------------------------------------------------- print

#[test]
fn print_writes_one_line_per_statement_in_order() {
    assert_eq!(run("print 1\nprint 2\nprint 3\n").unwrap(), "1\n2\n3\n");
}

#[test]
fn a_program_without_print_writes_nothing() {
    assert_eq!(run("let x = 1\n").unwrap(), "");
}

#[test]
fn prints_every_kind_of_value() {
    assert_eq!(run("print 42\n").unwrap(), "42\n");
    assert_eq!(run("print 1.5\n").unwrap(), "1.5\n");
    assert_eq!(run("print 2.0\n").unwrap(), "2\n");
    assert_eq!(run("print \"hi\"\n").unwrap(), "hi\n");
    assert_eq!(run("print true\n").unwrap(), "true\n");
    assert_eq!(run("print null\n").unwrap(), "null\n");
    assert_eq!(
        run("func f(): return 1\nprint f\n").unwrap(),
        "<function f>\n"
    );
}

// ----------------------------------------------------------------- expressions

#[test]
fn evaluates_grouping_and_precedence() {
    assert_eq!(run("print 2 + 3 * 4\n").unwrap(), "14\n");
    assert_eq!(run("print (2 + 3) * 4\n").unwrap(), "20\n");
    assert_eq!(run("print 10 - 2 - 3\n").unwrap(), "5\n");
    assert_eq!(run("print -7 % 2\n").unwrap(), "-1\n");
}

#[test]
fn evaluates_unary_operators() {
    assert_eq!(run("print -5\n").unwrap(), "-5\n");
    assert_eq!(run("print - -5\n").unwrap(), "5\n");
    assert_eq!(run("print !true\n").unwrap(), "false\n");
    assert_eq!(run("print !0\n").unwrap(), "true\n");
    assert_eq!(run("print !null\n").unwrap(), "true\n");
}

#[test]
fn unary_minus_on_a_non_number_is_an_error() {
    let error = run("print -\"hi\"\n").unwrap_err();
    assert!(error.contains("unary minus: String"), "{error}");
}

#[test]
fn evaluates_comparisons_and_equality() {
    assert_eq!(run("print 1 < 2\n").unwrap(), "true\n");
    assert_eq!(run("print 1 <= 1\n").unwrap(), "true\n");
    assert_eq!(run("print 3 > 2\n").unwrap(), "true\n");
    assert_eq!(run("print 3 >= 4\n").unwrap(), "false\n");
    assert_eq!(run("print 1 == 1\n").unwrap(), "true\n");
    assert_eq!(run("print 1 != 1\n").unwrap(), "false\n");
    assert_eq!(run("print \"a\" < \"b\"\n").unwrap(), "true\n");
    assert_eq!(run("print 1 == \"1\"\n").unwrap(), "false\n");
}

#[test]
fn arithmetic_mixes_integers_and_decimals() {
    assert_eq!(run("print 1 + 0.5\n").unwrap(), "1.5\n");
    assert_eq!(run("print 3 / 2.0\n").unwrap(), "1.5\n");
    assert_eq!(run("print 7 / -2\n").unwrap(), "-3\n");
    assert_eq!(run("print 1 == 1.0\n").unwrap(), "true\n");
}

#[test]
fn strings_concatenate_and_repeat() {
    assert_eq!(run("print \"ra\" + \"bi\"\n").unwrap(), "rabi\n");
    assert_eq!(run("print 3 * \"ab\"\n").unwrap(), "ababab\n");
}

#[test]
fn arithmetic_errors_abort_the_program() {
    let error = run("print 9223372036854775807 + 1\n").unwrap_err();
    assert!(error.contains("overflow"), "{error}");
    let error = run("print 1 / 0\n").unwrap_err();
    assert!(error.contains("Division by zero"), "{error}");
    let error = run("print \"a\" + 1\n").unwrap_err();
    assert!(error.contains("String and Integer"), "{error}");
}

#[test]
fn logical_operators_do_not_evaluate_the_second_operand_when_the_first_decides() {
    assert_eq!(run("print false and undeclared\n").unwrap(), "false\n");
    assert_eq!(run("print true or undeclared\n").unwrap(), "true\n");
}

#[test]
fn or_yields_the_deciding_operand() {
    assert_eq!(run("print null or 0\n").unwrap(), "0\n");
    assert_eq!(run("print 5 or 0\n").unwrap(), "5\n");
}

#[test]
fn and_yields_a_boolean() {
    assert_eq!(run("print 1 and 5\n").unwrap(), "true\n");
    assert_eq!(run("print 0 and 5\n").unwrap(), "false\n");
}

// ------------------------------------------------------------------- variables

#[test]
fn declares_and_reads_a_variable() {
    assert_eq!(run("let x = 10\nlet y = x + 5\nprint y\n").unwrap(), "15\n");
}

#[test]
fn a_declaration_without_an_initializer_is_null() {
    assert_eq!(run("let x\nprint x\n").unwrap(), "null\n");
}

#[test]
fn redeclaring_a_variable_replaces_it() {
    assert_eq!(run("let x = 1\nlet x = 2\nprint x\n").unwrap(), "2\n");
}

#[test]
fn assignment_evaluates_to_the_assigned_value() {
    let source = "let x = 1\nlet y = x = 9\nprint y\nprint x\n";
    assert_eq!(run(source).unwrap(), "9\n9\n");
}

#[test]
fn reading_an_undeclared_variable_is_an_error() {
    let error = run("print nope\n").unwrap_err();
    assert!(error.contains("'nope' not defined"), "{error}");
}

#[test]
fn assigning_to_an_undeclared_variable_is_an_error() {
    let error = run("nope = 1\n").unwrap_err();
    assert!(error.contains("'nope' not defined"), "{error}");
}

// ---------------------------------------------------------------------- scopes

#[test]
fn a_variable_declared_in_a_block_is_not_readable_after_it() {
    let error = run("if true:\n    let x = 1\nprint x\n").unwrap_err();
    assert!(error.contains("'x' not defined"), "{error}");
}

#[test]
fn a_block_can_shadow_an_outer_variable_without_overwriting_it() {
    let source = "let x = 1\nif true:\n    let x = 2\n    print x\nprint x\n";
    assert_eq!(run(source).unwrap(), "2\n1\n");
}

#[test]
fn a_block_can_still_assign_to_an_outer_variable() {
    let source = "let x = 1\nif true:\n    x = 2\nprint x\n";
    assert_eq!(run(source).unwrap(), "2\n");
}

#[test]
fn a_local_initializer_reads_the_shadowed_variable() {
    let source = "let x = 1\nif true:\n    let x = x + 1\n    print x\n";
    assert_eq!(run(source).unwrap(), "2\n");
}

#[test]
fn nested_blocks_keep_their_locals_apart() {
    let source = "if true:\n    let a = 1\n    if true:\n        let b = 2\n        print a + b\n    let c = 3\n    print a + c\n";
    assert_eq!(run(source).unwrap(), "3\n4\n");
}

// ----------------------------------------------------------------- control flow

#[test]
fn takes_the_matching_branch() {
    assert_eq!(run("if true: print 1\nelse: print 2\n").unwrap(), "1\n");
    assert_eq!(run("if false: print 1\nelse: print 2\n").unwrap(), "2\n");
    assert_eq!(run("if false: print 1\nprint 3\n").unwrap(), "3\n");
}

#[test]
fn else_if_chains_pick_the_first_truthy_condition() {
    let source = "let x = 2\nif x == 1: print 1\nelse if x == 2: print 2\nelse: print 3\n";
    assert_eq!(run(source).unwrap(), "2\n");
}

#[test]
fn conditions_use_truthiness() {
    assert_eq!(run("if 0: print 1\nelse: print 2\n").unwrap(), "2\n");
    assert_eq!(run("if \"\": print 1\nelse: print 2\n").unwrap(), "2\n");
    assert_eq!(run("if \"a\": print 1\nelse: print 2\n").unwrap(), "1\n");
    assert_eq!(run("if null: print 1\nelse: print 2\n").unwrap(), "2\n");
}

#[test]
fn runs_a_while_loop_until_its_condition_is_false() {
    let source = "let i = 0\nwhile i < 3:\n    print i\n    i = i + 1\n";
    assert_eq!(run(source).unwrap(), "0\n1\n2\n");
}

#[test]
fn a_while_whose_condition_starts_false_never_runs() {
    assert_eq!(run("while false: print 1\nprint 2\n").unwrap(), "2\n");
}

#[test]
fn every_loop_iteration_can_redeclare_the_same_variable() {
    let source = "let i = 0\nwhile i < 3:\n    let x = i * 10\n    print x\n    i = i + 1\n";
    assert_eq!(run(source).unwrap(), "0\n10\n20\n");
}

#[test]
fn break_leaves_the_loop_from_a_nested_block() {
    let source = "let i = 0\nwhile true:\n    let x = i\n    if x == 2:\n        let y = x\n        break\n    i = i + 1\nprint i\n";
    assert_eq!(run(source).unwrap(), "2\n");
}

#[test]
fn continue_skips_the_rest_of_the_iteration() {
    let source = "let i = 0\nwhile i < 4:\n    i = i + 1\n    let x = i\n    if x == 2: continue\n    print x\n";
    assert_eq!(run(source).unwrap(), "1\n3\n4\n");
}

#[test]
fn break_only_leaves_the_innermost_loop() {
    let source = "let i = 0\nwhile i < 2:\n    let j = 0\n    while true:\n        j = j + 1\n        if j > 1: break\n    print i * 10 + j\n    i = i + 1\n";
    assert_eq!(run(source).unwrap(), "2\n12\n");
}

#[test]
fn loops_inside_functions_keep_the_stack_balanced() {
    let source = "func f():\n    let a = 1\n    let i = 0\n    while i < 3:\n        let x = 10\n        i = i + 1\n        if i == 2:\n            let z = 5\n            continue\n        if i == 3: break\n    let b = 7\n    return a + b\nprint f()\n";
    assert_eq!(run(source).unwrap(), "8\n");
}

// -------------------------------------------------------------------- functions

#[test]
fn calls_a_function_and_returns_a_value() {
    assert_eq!(run("func f(): return 7\nprint f()\n").unwrap(), "7\n");
}

#[test]
fn passes_several_arguments() {
    assert_eq!(
        run("func sub(a, b): return a - b\nprint sub(5, 3)\n").unwrap(),
        "2\n"
    );
}

#[test]
fn a_function_without_a_return_evaluates_to_null() {
    assert_eq!(run("func f(): let x = 1\nprint f()\n").unwrap(), "null\n");
}

#[test]
fn a_bare_return_evaluates_to_null() {
    assert_eq!(run("func f(): return\nprint f()\n").unwrap(), "null\n");
}

#[test]
fn return_stops_executing_the_body() {
    let source = "func f():\n    print 1\n    return 2\n    print 3\nprint f()\n";
    assert_eq!(run(source).unwrap(), "1\n2\n");
}

#[test]
fn returns_from_inside_a_loop() {
    let source = "func f():\n    let i = 0\n    while true:\n        let x = i\n        if x == 3: return x\n        i = i + 1\nprint f()\n";
    assert_eq!(run(source).unwrap(), "3\n");
}

#[test]
fn recursion_works() {
    let source =
        "func fib(n):\n    if n < 2: return n\n    return fib(n - 1) + fib(n - 2)\nprint fib(10)\n";
    assert_eq!(run(source).unwrap(), "55\n");
}

#[test]
fn functions_are_first_class_values() {
    let source = "func twice(f, x): return f(f(x))\nfunc inc(n): return n + 1\nlet g = inc\nprint twice(g, 1)\nprint g == inc\n";
    assert_eq!(run(source).unwrap(), "3\ntrue\n");
}

#[test]
fn a_function_can_read_and_assign_globals() {
    let source = "let x = 1\nfunc f():\n    x = x + 1\nf()\nf()\nprint x\n";
    assert_eq!(run(source).unwrap(), "3\n");
}

#[test]
fn a_function_reads_globals_declared_after_it() {
    assert_eq!(
        run("func f(): return later\nlet later = 7\nprint f()\n").unwrap(),
        "7\n"
    );
}

#[test]
fn a_parameter_shadows_a_global_without_overwriting_it() {
    let source = "let x = 1\nfunc f(x): return x\nprint f(2)\nprint x\n";
    assert_eq!(run(source).unwrap(), "2\n1\n");
}

#[test]
fn a_callee_cannot_read_a_callers_local() {
    let source =
        "func g(): return secret\nfunc f():\n    let secret = 1\n    return g()\nprint f()\n";
    let error = run(source).unwrap_err();
    assert!(error.contains("'secret' not defined"), "{error}");
}

#[test]
fn a_nested_function_is_callable_from_the_function_that_declared_it() {
    let source = "func f():\n    func g(): return 2\n    return g()\nprint f()\n";
    assert_eq!(run(source).unwrap(), "2\n");
}

#[test]
fn a_nested_function_is_not_callable_from_the_global_scope() {
    let source = "func f():\n    func g(): return 2\nf()\ng()\n";
    let error = run(source).unwrap_err();
    assert!(error.contains("'g' not defined"), "{error}");
}

#[test]
fn a_nested_function_cannot_read_the_locals_of_the_function_that_declared_it() {
    let source =
        "func f():\n    let secret = 1\n    func g(): return secret\n    return g()\nprint f()\n";
    let error = run(source).unwrap_err();
    assert!(error.contains("'secret' not defined"), "{error}");
}

#[test]
fn calling_with_the_wrong_number_of_arguments_is_an_error_in_programs() {
    let error = run("func f(a, b): return a\nf(1)\n").unwrap_err();
    assert!(error.contains("Expected 2 arguments but got 1"), "{error}");
}

#[test]
fn arguments_are_evaluated_before_checking_the_callee() {
    let source = "func touch():\n    print \"touched\"\n    return 0\n5(touch())\n";
    let (output, result) = run_keeping_output(source);
    assert!(result.unwrap_err().contains("non-function"));
    assert_eq!(output, "touched\n");
}

#[test]
fn unbounded_recursion_is_a_stack_overflow() {
    let error = run("func f(): return f()\nf()\n").unwrap_err();
    assert!(error.contains("Stack overflow"), "{error}");
}

// ---------------------------------------------------------------------- arrays

#[test]
fn array_literals_index_and_len() {
    assert_eq!(
        run("print [1, \"dos\", [3], null]\n").unwrap(),
        "[1, dos, [3], null]\n"
    );
    assert_eq!(run("print [1, 2,]\n").unwrap(), "[1, 2]\n");
    assert_eq!(run("print [10, 20][1]\n").unwrap(), "20\n");
    assert_eq!(run("print len [[1, 2], [3]][0]\n").unwrap(), "2\n");
}

#[test]
fn arrays_are_shared_by_reference() {
    let source = "let a = [1]\nlet b = a\nb[0] = 2\nprint a\n";
    assert_eq!(run(source).unwrap(), "[2]\n");
}

#[test]
fn repetition_copies_references_to_the_elements() {
    let source = "let m = [[0]] * 2\nm[0][0] = 9\nprint m\n";
    assert_eq!(run(source).unwrap(), "[[9], [9]]\n");
}

#[test]
fn a_literal_builds_a_new_array_each_time_it_runs() {
    let source = concat!(
        "func make(): return [0]\n",
        "let a = make()\n",
        "a[0] = 1\n",
        "print make()\n",
    );
    assert_eq!(run(source).unwrap(), "[0]\n");
}

#[test]
fn an_index_assignment_yields_the_assigned_value() {
    assert_eq!(
        run("let a = [0, 0]\nprint a[1] = 5\nprint a\n").unwrap(),
        "5\n[0, 5]\n"
    );
}

#[test]
fn arrays_passed_to_a_function_can_be_mutated_through_locals() {
    let source = concat!(
        "func fill(arr, v):\n",
        "    let i = 0\n",
        "    while i < len arr:\n",
        "        arr[i] = v\n",
        "        i = i + 1\n",
        "let a = [1, 2, 3]\n",
        "fill(a, 7)\n",
        "print a\n",
    );
    assert_eq!(run(source).unwrap(), "[7, 7, 7]\n");
}

#[test]
fn array_errors_match_the_interpreter() {
    let error = run("print 1[0]\n").unwrap_err();
    assert!(
        error.contains("Unsupported type for indexing: Integer"),
        "{error}"
    );
    let error = run("print [1][\"0\"]\n").unwrap_err();
    assert!(
        error.contains("Array index must be an Integer, found String"),
        "{error}"
    );
    let error = run("print [1][-1]\n").unwrap_err();
    assert!(
        error.contains("Array index out of bounds: -1 (length 1)"),
        "{error}"
    );
    let error = run("let a = [1]\na[1] = 0\n").unwrap_err();
    assert!(
        error.contains("Array index out of bounds: 1 (length 1)"),
        "{error}"
    );
    let error = run("print len \"abc\"\n").unwrap_err();
    assert!(
        error.contains("Unsupported type for len: String"),
        "{error}"
    );
}

#[test]
fn an_index_assignment_evaluates_its_value_before_checking_the_bounds() {
    let source =
        "let a = [1]\nfunc touch():\n    print \"touched\"\n    return 0\na[5] = touch()\n";
    let (output, result) = run_keeping_output(source);
    assert!(result.unwrap_err().contains("out of bounds"));
    assert_eq!(output, "touched\n");
}

// ------------------------------------------------------------------------- for

#[test]
fn a_for_loop_visits_every_element_in_order() {
    assert_eq!(
        run("for x in [1, 2, 3]:\n    print x\n").unwrap(),
        "1\n2\n3\n"
    );
    assert_eq!(run("for x in []:\n    print x\n").unwrap(), "");
}

#[test]
fn continue_from_a_nested_block_skips_to_the_next_element() {
    let source = concat!(
        "for x in [1, 2, 3]:\n",
        "    let y = x * 2\n",
        "    if y == 4:\n",
        "        let z = 0\n",
        "        continue\n",
        "    print y\n",
        "print \"done\"\n",
    );
    assert_eq!(run(source).unwrap(), "2\n6\ndone\n");
}

#[test]
fn break_leaves_a_for_loop() {
    let source = concat!(
        "for x in [1, 2, 3]:\n",
        "    let y = x\n",
        "    if x == 2: break\n",
        "    print y\n",
        "print \"done\"\n",
    );
    assert_eq!(run(source).unwrap(), "1\ndone\n");
}

#[test]
fn nested_for_loops_keep_their_own_state() {
    let source = concat!(
        "for a in [1, 2]:\n",
        "    for b in [10, 20]:\n",
        "        if b == 20: continue\n",
        "        print a + b\n",
        "    print a\n",
    );
    assert_eq!(run(source).unwrap(), "11\n1\n12\n2\n");
}

#[test]
fn a_for_loop_inside_a_function_can_return() {
    let source = concat!(
        "func find(arr, target):\n",
        "    let i = 0\n",
        "    for x in arr:\n",
        "        if x == target: return i\n",
        "        i = i + 1\n",
        "    return -1\n",
        "print find([5, 6, 7], 7)\n",
        "print find([5, 6, 7], 8)\n",
    );
    assert_eq!(run(source).unwrap(), "2\n-1\n");
}

#[test]
fn a_for_loop_reads_the_live_array() {
    let source = "let arr = [1, 2, 3]\nfor x in arr:\n    arr[2] = 30\n    print x\n";
    assert_eq!(run(source).unwrap(), "1\n2\n30\n");
}

#[test]
fn reassigning_the_iterated_variable_does_not_change_the_loop() {
    let source = "let arr = [1, 2]\nfor x in arr:\n    arr = arr + [9]\n    print x\nprint arr\n";
    assert_eq!(run(source).unwrap(), "1\n2\n[1, 2, 9, 9]\n");
}

#[test]
fn assigning_the_loop_variable_does_not_change_the_array() {
    let source = "let arr = [1, 2]\nfor x in arr:\n    x = 0\nprint arr\n";
    assert_eq!(run(source).unwrap(), "[1, 2]\n");
}

#[test]
fn the_loop_variable_is_not_visible_after_the_loop() {
    let error = run("for x in [1]:\n    print x\nprint x\n").unwrap_err();
    assert!(error.contains("Variable 'x' not defined"), "{error}");
}

#[test]
fn the_iterable_resolves_before_the_loop_variable_is_declared() {
    assert_eq!(
        run("let x = [1, 2]\nfor x in x:\n    print x\n").unwrap(),
        "1\n2\n"
    );
}

#[test]
fn iterating_something_that_is_not_an_array_is_an_error() {
    let error = run("for x in 5:\n    print x\n").unwrap_err();
    assert!(
        error.contains("Unsupported type for iteration: Integer"),
        "{error}"
    );
}

// ---------------------------------------------------------------------- errors

#[test]
fn a_runtime_error_aborts_the_program_after_earlier_output() {
    let (output, result) = run_keeping_output("print 1\nprint nope\nprint 2\n");
    assert!(result.is_err());
    assert_eq!(output, "1\n");
}

#[test]
fn an_error_inside_a_function_propagates_to_the_caller() {
    let error = run("func f(): return 1 / 0\nprint f()\n").unwrap_err();
    assert!(error.contains("Division by zero"), "{error}");
}

// ===================================================================== input

fn run_with_input(source: &str, input: &str) -> Result<String, String> {
    let tokens = scanner::scan(source.as_bytes())?;
    let statements = parser::parse(tokens)?;
    let function = compile(optimizer::optimize(statements))?;
    let mut output = Vec::new();
    let input = Box::new(std::io::Cursor::new(input.to_string()));
    Vm::with_io(&mut output, input).run(function)?;
    String::from_utf8(output).map_err(|e| format!("output was not valid UTF-8: {e}"))
}

#[test]
fn input_writes_the_prompt_and_returns_the_line_without_its_terminator() {
    assert_eq!(
        run_with_input("let n = input \"name: \"\nprint \"hi \" + n\n", "Ana\r\nBob\n").unwrap(),
        "name: hi Ana\n"
    );
}

#[test]
fn input_reads_consecutive_lines_inside_functions() {
    assert_eq!(
        run_with_input(
            "func ask():\n    return input \"\"\nprint ask()\nprint ask()\n",
            "a\nb"
        )
        .unwrap(),
        "a\nb\n"
    );
}

#[test]
fn input_returns_null_at_end_of_input() {
    assert_eq!(run_with_input("print input \"\"\n", "").unwrap(), "null\n");
}

#[test]
fn input_without_a_prompt_writes_nothing_before_reading() {
    assert_eq!(run_with_input("let a = input\nprint a\n", "x\n").unwrap(), "x\n");
}
