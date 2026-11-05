use std::rc::Rc;

use super::compile;
use crate::bytecode::{Function, Operation as Op, Value};
use crate::{parser, scanner};

fn compile_source(source: &str) -> Result<Function, String> {
    let tokens = scanner::scan(source.as_bytes())?;
    let statements = parser::parse(tokens)?;
    compile(statements)
}

fn script(source: &str) -> Function {
    compile_source(source).unwrap()
}

fn code(source: &str) -> Vec<Op> {
    script(source).chunk.code
}

fn compile_error(source: &str) -> String {
    match compile_source(source) {
        Ok(_) => panic!("expected a compile error for {source:?}"),
        Err(error) => error,
    }
}

fn function_constant<'a>(function: &'a Function, name: &str) -> &'a Function {
    function
        .chunk
        .constants
        .iter()
        .find_map(|constant| match constant {
            Value::Function(inner) if inner.name == name => Some(inner.as_ref()),
            _ => None,
        })
        .unwrap_or_else(|| panic!("no function constant named {name}"))
}

fn string(s: &str) -> Value {
    Value::String(Rc::new(s.to_string()))
}

// -------------------------------------------------------------------- script

#[test]
fn an_empty_program_returns_null() {
    assert_eq!(code(""), vec![Op::Null, Op::Return]);
}

#[test]
fn the_script_is_named_and_takes_no_arguments() {
    let function = script("");
    assert_eq!(function.name, "<script>");
    assert_eq!(function.arity, 0);
}

#[test]
fn an_expression_statement_discards_its_value() {
    assert_eq!(
        code("1\n"),
        vec![Op::GetConstant(0), Op::Pop, Op::Null, Op::Return]
    );
}

#[test]
fn print_consumes_its_value() {
    assert_eq!(
        code("print 1\n"),
        vec![Op::GetConstant(0), Op::Print, Op::Null, Op::Return]
    );
}

// ------------------------------------------------------------------ literals

#[test]
fn numbers_and_strings_are_loaded_from_the_constant_table() {
    let function = script("print 42\nprint 1.5\nprint \"hi\"\n");
    assert!(function.chunk.constants[0] == Value::Integer(42));
    assert!(function.chunk.constants[1] == Value::Decimal(1.5));
    assert!(function.chunk.constants[2] == string("hi"));
    assert_eq!(function.chunk.code[0], Op::GetConstant(0));
    assert_eq!(function.chunk.code[2], Op::GetConstant(1));
    assert_eq!(function.chunk.code[4], Op::GetConstant(2));
}

#[test]
fn booleans_and_null_have_their_own_operations() {
    assert_eq!(code("print true\n")[0], Op::True);
    assert_eq!(code("print false\n")[0], Op::False);
    assert_eq!(code("print null\n")[0], Op::Null);
    assert!(
        script("print true\nprint false\nprint null\n")
            .chunk
            .constants
            .is_empty()
    );
}

// --------------------------------------------------------------- expressions

#[test]
fn unary_operators_follow_their_operand() {
    assert_eq!(
        code("print -1\n"),
        vec![
            Op::GetConstant(0),
            Op::Negate,
            Op::Print,
            Op::Null,
            Op::Return
        ]
    );
    assert_eq!(
        code("print !true\n"),
        vec![Op::True, Op::Not, Op::Print, Op::Null, Op::Return]
    );
}

#[test]
fn each_binary_operator_compiles_to_its_operation() {
    let cases = [
        ("+", Op::Add),
        ("-", Op::Subtract),
        ("*", Op::Multiply),
        ("/", Op::Divide),
        ("%", Op::Modulo),
        ("==", Op::Equal),
        (">", Op::Greater),
        (">=", Op::GreaterEqual),
        ("<", Op::Less),
        ("<=", Op::LessEqual),
    ];
    for (operator, operation) in cases {
        assert_eq!(
            code(&format!("print 1 {operator} 2\n")),
            vec![
                Op::GetConstant(0),
                Op::GetConstant(1),
                operation,
                Op::Print,
                Op::Null,
                Op::Return
            ],
            "operator {operator}"
        );
    }
}

#[test]
fn not_equal_is_the_negation_of_equal() {
    assert_eq!(
        code("print 1 != 2\n"),
        vec![
            Op::GetConstant(0),
            Op::GetConstant(1),
            Op::Equal,
            Op::Not,
            Op::Print,
            Op::Null,
            Op::Return
        ]
    );
}

#[test]
fn operands_are_emitted_following_precedence() {
    assert_eq!(
        code("print 1 + 2 * 3\n"),
        vec![
            Op::GetConstant(0),
            Op::GetConstant(1),
            Op::GetConstant(2),
            Op::Multiply,
            Op::Add,
            Op::Print,
            Op::Null,
            Op::Return
        ]
    );
}

#[test]
fn grouping_emits_no_operation_of_its_own() {
    assert_eq!(code("print (1)\n"), code("print 1\n"));
}

#[test]
fn and_skips_the_right_operand_and_normalizes_to_a_boolean() {
    assert_eq!(
        code("print true and false\n"),
        vec![
            Op::True,
            Op::JumpIfFalse(2),
            Op::Pop,
            Op::False,
            Op::Not,
            Op::Not,
            Op::Print,
            Op::Null,
            Op::Return
        ]
    );
}

#[test]
fn or_keeps_the_left_operand_when_it_is_truthy() {
    assert_eq!(
        code("print false or true\n"),
        vec![
            Op::False,
            Op::JumpIfFalse(1),
            Op::Jump(2),
            Op::Pop,
            Op::True,
            Op::Print,
            Op::Null,
            Op::Return
        ]
    );
}

#[test]
fn a_call_pushes_the_function_then_its_arguments() {
    assert_eq!(
        code("f(1, 2)\n"),
        vec![
            Op::GetGlobal(0),
            Op::GetConstant(1),
            Op::GetConstant(2),
            Op::Call(2),
            Op::Pop,
            Op::Null,
            Op::Return
        ]
    );
}

// ------------------------------------------------------------------- globals

#[test]
fn a_top_level_declaration_defines_a_global_after_its_initializer() {
    let function = script("let x = 1\n");
    assert!(function.chunk.constants[0] == Value::Integer(1));
    assert!(function.chunk.constants[1] == string("x"));
    assert_eq!(
        function.chunk.code,
        vec![
            Op::GetConstant(0),
            Op::DefineGlobal(1),
            Op::Null,
            Op::Return
        ]
    );
}

#[test]
fn a_declaration_without_an_initializer_defines_null() {
    assert_eq!(
        code("let x\n"),
        vec![Op::Null, Op::DefineGlobal(0), Op::Null, Op::Return]
    );
}

#[test]
fn reading_and_assigning_an_unknown_name_uses_globals() {
    assert_eq!(
        code("print x\n"),
        vec![Op::GetGlobal(0), Op::Print, Op::Null, Op::Return]
    );
    assert_eq!(
        code("x = 2\n"),
        vec![
            Op::GetConstant(0),
            Op::SetGlobal(1),
            Op::Pop,
            Op::Null,
            Op::Return
        ]
    );
}

#[test]
fn a_global_name_is_stored_once_in_the_constant_table() {
    let function = script("let x = 1\nprint x\nx = 2\n");
    let names = function
        .chunk
        .constants
        .iter()
        .filter(|constant| **constant == string("x"))
        .count();
    assert_eq!(names, 1);
}

// -------------------------------------------------------------------- locals

#[test]
fn a_declaration_inside_a_block_is_a_local_popped_at_the_end_of_the_block() {
    assert_eq!(
        code("if true:\n    let a = 1\n    print a\n"),
        vec![
            Op::True,
            Op::JumpIfFalse(6),
            Op::Pop,
            Op::GetConstant(0),
            Op::GetLocal(1),
            Op::Print,
            Op::Pop,
            Op::Jump(1),
            Op::Pop,
            Op::Null,
            Op::Return
        ]
    );
}

#[test]
fn slot_zero_is_reserved_for_the_running_function() {
    let function = script("func f(a):\n    return a\n");
    let f = function_constant(&function, "f");
    assert_eq!(f.chunk.code[0], Op::GetLocal(1));
}

#[test]
fn a_local_initializer_reads_the_outer_variable_it_shadows() {
    let function = script("func f(a):\n    if true:\n        let a = a\n        print a\n");
    let f = function_constant(&function, "f");
    assert_eq!(f.chunk.code[3], Op::GetLocal(1));
    assert_eq!(f.chunk.code[4], Op::GetLocal(2));
}

#[test]
fn assigning_a_local_uses_its_slot() {
    let function = script("func f(a):\n    a = 2\n");
    let f = function_constant(&function, "f");
    assert_eq!(
        f.chunk.code,
        vec![
            Op::GetConstant(0),
            Op::SetLocal(1),
            Op::Pop,
            Op::Null,
            Op::Return
        ]
    );
}

// ----------------------------------------------------------------- functions

#[test]
fn a_function_declaration_loads_the_function_and_binds_its_name() {
    let function = script("func f(a, b):\n    return a + b\n");
    assert_eq!(
        function.chunk.code,
        vec![
            Op::GetConstant(0),
            Op::DefineGlobal(1),
            Op::Null,
            Op::Return
        ]
    );
    assert!(function.chunk.constants[1] == string("f"));
}

#[test]
fn a_function_knows_its_name_and_arity() {
    let function = script("func f(a, b):\n    return a + b\n");
    let f = function_constant(&function, "f");
    assert_eq!(f.name, "f");
    assert_eq!(f.arity, 2);
}

#[test]
fn parameters_are_the_first_locals_of_a_function() {
    let function = script("func f(a, b):\n    return a + b\n");
    let f = function_constant(&function, "f");
    assert_eq!(
        f.chunk.code,
        vec![
            Op::GetLocal(1),
            Op::GetLocal(2),
            Op::Add,
            Op::Return,
            Op::Null,
            Op::Return
        ]
    );
}

#[test]
fn a_bare_return_returns_null() {
    let function = script("func f():\n    return\n");
    let f = function_constant(&function, "f");
    assert_eq!(
        f.chunk.code,
        vec![Op::Null, Op::Return, Op::Null, Op::Return]
    );
}

#[test]
fn a_function_without_return_ends_returning_null() {
    let function = script("func f():\n    print 1\n");
    let f = function_constant(&function, "f");
    assert_eq!(f.chunk.code[2..], [Op::Null, Op::Return]);
}

#[test]
fn a_declaration_inside_a_function_is_local() {
    let function = script("func f():\n    let x = 1\n    print x\n");
    let f = function_constant(&function, "f");
    assert_eq!(
        f.chunk.code,
        vec![
            Op::GetConstant(0),
            Op::GetLocal(1),
            Op::Print,
            Op::Null,
            Op::Return
        ]
    );
}

#[test]
fn a_function_reads_names_it_does_not_declare_as_globals() {
    let function = script("let x = 1\nfunc f():\n    return x\n");
    let f = function_constant(&function, "f");
    assert_eq!(f.chunk.code[0], Op::GetGlobal(0));
    assert!(f.chunk.constants[0] == string("x"));
}

#[test]
fn a_nested_function_is_a_local_of_the_function_that_declares_it() {
    let function = script("func f():\n    func g():\n        return 1\n    return g()\n");
    let f = function_constant(&function, "f");
    assert_eq!(
        f.chunk.code,
        vec![
            Op::GetConstant(0),
            Op::GetLocal(1),
            Op::Call(0),
            Op::Return,
            Op::Null,
            Op::Return
        ]
    );
}

#[test]
fn a_nested_function_cannot_see_the_locals_of_its_enclosing_function() {
    let function = script("func f():\n    let x = 1\n    func g():\n        return x\n");
    let f = function_constant(&function, "f");
    let g = function_constant(f, "g");
    assert_eq!(g.chunk.code[0], Op::GetGlobal(0));
}

// --------------------------------------------------------------------- if

#[test]
fn an_if_without_else_jumps_over_its_branch() {
    assert_eq!(
        code("if true: print 1\n"),
        vec![
            Op::True,
            Op::JumpIfFalse(4),
            Op::Pop,
            Op::GetConstant(0),
            Op::Print,
            Op::Jump(1),
            Op::Pop,
            Op::Null,
            Op::Return
        ]
    );
}

#[test]
fn an_if_else_runs_exactly_one_branch() {
    assert_eq!(
        code("if true: print 1\nelse: print 2\n"),
        vec![
            Op::True,
            Op::JumpIfFalse(4),
            Op::Pop,
            Op::GetConstant(0),
            Op::Print,
            Op::Jump(3),
            Op::Pop,
            Op::GetConstant(1),
            Op::Print,
            Op::Null,
            Op::Return
        ]
    );
}

// --------------------------------------------------------------------- loops

#[test]
fn a_while_loops_back_to_its_condition() {
    assert_eq!(
        code("while false:\n    print 1\n"),
        vec![
            Op::False,
            Op::JumpIfFalse(4),
            Op::Pop,
            Op::GetConstant(0),
            Op::Print,
            Op::Loop(6),
            Op::Pop,
            Op::Null,
            Op::Return
        ]
    );
}

#[test]
fn break_discards_the_loop_locals_and_jumps_past_the_loop() {
    assert_eq!(
        code("while true:\n    let a = 1\n    break\n"),
        vec![
            Op::True,
            Op::JumpIfFalse(6),
            Op::Pop,
            Op::GetConstant(0),
            Op::Pop,
            Op::Jump(3),
            Op::Pop,
            Op::Loop(8),
            Op::Pop,
            Op::Null,
            Op::Return
        ]
    );
}

#[test]
fn continue_discards_the_loop_locals_and_jumps_to_the_condition() {
    assert_eq!(
        code("while true:\n    let a = 1\n    continue\n"),
        vec![
            Op::True,
            Op::JumpIfFalse(6),
            Op::Pop,
            Op::GetConstant(0),
            Op::Pop,
            Op::Loop(6),
            Op::Pop,
            Op::Loop(8),
            Op::Pop,
            Op::Null,
            Op::Return
        ]
    );
}

#[test]
fn break_inside_a_nested_block_discards_every_local_of_the_loop() {
    assert_eq!(
        code("while true:\n    let a = 1\n    if true:\n        let b = 2\n        break\n"),
        vec![
            Op::True,
            Op::JumpIfFalse(14),
            Op::Pop,
            Op::GetConstant(0),
            Op::True,
            Op::JumpIfFalse(7),
            Op::Pop,
            Op::GetConstant(1),
            Op::Pop,
            Op::Pop,
            Op::Jump(6),
            Op::Pop,
            Op::Jump(1),
            Op::Pop,
            Op::Pop,
            Op::Loop(16),
            Op::Pop,
            Op::Null,
            Op::Return
        ]
    );
}

#[test]
fn break_leaves_only_the_innermost_loop() {
    let code = code("while true:\n    while true:\n        break\n    print 1\n");
    let print_index = code.iter().position(|op| *op == Op::Print).unwrap();
    let Op::Jump(offset) = code[6] else {
        panic!("expected the break jump at index 6, got {:?}", code[6]);
    };
    // the break lands right after the inner loop, before `print 1`
    assert!(7 + (offset as usize) < print_index);
}

// --------------------------------------------------------------------- arrays

#[test]
fn an_array_literal_builds_from_its_elements_in_order() {
    assert_eq!(
        code("print [1, true]\n"),
        vec![
            Op::GetConstant(0),
            Op::True,
            Op::BuildArray(2),
            Op::Print,
            Op::Null,
            Op::Return
        ]
    );
    assert_eq!(
        code("print []\n"),
        vec![Op::BuildArray(0), Op::Print, Op::Null, Op::Return]
    );
}

#[test]
fn an_array_literal_is_never_a_constant() {
    // arrays are mutable and shared, so each evaluation must build a new one
    let function = script("print [1]\n");
    assert!(
        !function
            .chunk
            .constants
            .iter()
            .any(|c| matches!(c, Value::Array(_))),
        "{:?}",
        function.chunk.constants
    );
}

#[test]
fn indexing_pushes_collection_then_index() {
    let function = script("let a = [1]\nprint a[0]\n");
    assert_eq!(
        function.chunk.code[3..],
        [
            Op::GetGlobal(1),
            Op::GetConstant(2),
            Op::GetIndex,
            Op::Print,
            Op::Null,
            Op::Return
        ]
    );
}

#[test]
fn an_index_assignment_pushes_collection_index_and_value() {
    let function = script("let a = [1]\na[0] = 2\n");
    assert_eq!(
        function.chunk.code[3..],
        [
            Op::GetGlobal(1),
            Op::GetConstant(2),
            Op::GetConstant(3),
            Op::SetIndex,
            Op::Pop,
            Op::Null,
            Op::Return,
        ]
    );
}

#[test]
fn len_follows_its_operand() {
    assert_eq!(
        code("print len []\n"),
        vec![Op::BuildArray(0), Op::Len, Op::Print, Op::Null, Op::Return]
    );
}

// ------------------------------------------------------------------------ for

#[test]
fn a_for_loop_keeps_the_iterable_and_counter_below_the_loop_variable() {
    let function = script("for x in [1, 2]:\n    print x\n");
    assert_eq!(
        function.chunk.code,
        vec![
            Op::GetConstant(0),
            Op::GetConstant(1),
            Op::BuildArray(2),
            Op::GetConstant(2),
            Op::ForIter(4),
            Op::GetLocal(3),
            Op::Print,
            Op::Pop,
            Op::Loop(5),
            Op::Pop,
            Op::Pop,
            Op::Null,
            Op::Return,
        ]
    );
    assert_eq!(function.chunk.constants[2], Value::Integer(0));
}

#[test]
fn break_in_a_for_loop_discards_the_loop_variable_and_lands_on_the_exit() {
    assert_eq!(
        code("for x in []:\n    if x: break\n"),
        vec![
            Op::BuildArray(0),
            Op::GetConstant(0),
            Op::ForIter(9),
            Op::GetLocal(3),
            Op::JumpIfFalse(4),
            Op::Pop,
            Op::Pop,
            Op::Jump(4),
            Op::Jump(1),
            Op::Pop,
            Op::Pop,
            Op::Loop(10),
            Op::Pop,
            Op::Pop,
            Op::Null,
            Op::Return,
        ]
    );
}

#[test]
fn continue_in_a_for_loop_returns_to_for_iter() {
    let code = code("for x in []:\n    continue\n");
    assert_eq!(code[2], Op::ForIter(4));
    assert_eq!(code[3..6], [Op::Pop, Op::Loop(3), Op::Pop]);
}

#[test]
fn a_for_loop_inside_a_function_uses_slots_after_the_parameters() {
    let function = script("func f(a):\n    for x in a:\n        print x\n");
    let f = function_constant(&function, "f");
    assert_eq!(
        f.chunk.code[..6],
        [
            Op::GetLocal(1),
            Op::GetConstant(0),
            Op::ForIter(4),
            Op::GetLocal(4),
            Op::Print,
            Op::Pop,
        ]
    );
}

// -------------------------------------------------------------------- errors

#[test]
fn break_outside_a_loop_is_a_compile_error() {
    assert!(compile_error("break\n").contains("Break statement not allowed"));
    assert!(compile_error("if true: break\n").contains("Break statement not allowed"));
}

#[test]
fn continue_outside_a_loop_is_a_compile_error() {
    assert!(compile_error("continue\n").contains("Continue statement not allowed"));
}

#[test]
fn break_inside_a_function_declared_in_a_loop_is_a_compile_error() {
    let error = compile_error("while true:\n    func f(): break\n");
    assert!(error.contains("Break statement not allowed"), "{error}");
}

#[test]
fn return_outside_a_function_is_a_compile_error() {
    let error = compile_error("return 1\n");
    assert!(error.contains("Return statement not allowed"), "{error}");
    let error = compile_error("while true: return\n");
    assert!(error.contains("Return statement not allowed"), "{error}");
}

#[test]
fn input_compiles_its_prompt_then_the_input_operation() {
    assert_eq!(
        code("print input null\n"),
        vec![Op::Null, Op::Input, Op::Print, Op::Null, Op::Return]
    );
}
