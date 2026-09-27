use super::runtime::Runtime;
use crate::{parser, scanner};

fn interpret(source: &str) -> Result<String, String> {
    let mut output = Vec::new();
    let tokens = scanner::scan(source.as_bytes())?;
    let statements = parser::parse(tokens)?;
    Runtime::new(&mut output).run_plain(statements)?;
    String::from_utf8(output).map_err(|e| format!("output was not valid UTF-8: {e}"))
}

// --------------------------------------------------------------------- print

#[test]
fn print_writes_one_line_per_statement_in_order() {
    assert_eq!(
        interpret("print 1\nprint 2\nprint 3\n").unwrap(),
        "1\n2\n3\n"
    );
}

#[test]
fn a_program_without_print_writes_nothing() {
    assert_eq!(interpret("let x = 1\n").unwrap(), "");
}

// ------------------------------------------------------------------ literals

#[test]
fn evaluates_literals() {
    assert_eq!(interpret("print 42\n").unwrap(), "42\n");
    assert_eq!(interpret("print 1.5\n").unwrap(), "1.5\n");
    assert_eq!(interpret("print \"hi\"\n").unwrap(), "hi\n");
    assert_eq!(interpret("print true\n").unwrap(), "true\n");
    assert_eq!(interpret("print false\n").unwrap(), "false\n");
    assert_eq!(interpret("print null\n").unwrap(), "null\n");
}

#[test]
fn evaluates_grouping_and_precedence() {
    assert_eq!(interpret("print 2 + 3 * 4\n").unwrap(), "14\n");
    assert_eq!(interpret("print (2 + 3) * 4\n").unwrap(), "20\n");
    assert_eq!(interpret("print 10 - 2 - 3\n").unwrap(), "5\n");
}

#[test]
fn evaluates_unary_operators() {
    assert_eq!(interpret("print -5\n").unwrap(), "-5\n");
    assert_eq!(interpret("print - -5\n").unwrap(), "5\n");
    assert_eq!(interpret("print !true\n").unwrap(), "false\n");
    assert_eq!(interpret("print !0\n").unwrap(), "true\n");
    assert_eq!(interpret("print !null\n").unwrap(), "true\n");
}

#[test]
fn unary_minus_on_a_non_number_is_an_error() {
    let error = interpret("print -\"hi\"\n").unwrap_err();
    assert!(error.contains("unary minus"), "{error}");
}

#[test]
fn evaluates_comparisons_and_equality() {
    assert_eq!(interpret("print 1 < 2\n").unwrap(), "true\n");
    assert_eq!(interpret("print 1 <= 1\n").unwrap(), "true\n");
    assert_eq!(interpret("print 3 > 2\n").unwrap(), "true\n");
    assert_eq!(interpret("print 3 >= 4\n").unwrap(), "false\n");
    assert_eq!(interpret("print 1 == 1\n").unwrap(), "true\n");
    assert_eq!(interpret("print 1 != 1\n").unwrap(), "false\n");
    assert_eq!(interpret("print \"x\" == \"x\"\n").unwrap(), "true\n");
}

#[test]
fn arithmetic_mixes_integers_and_decimals() {
    assert_eq!(interpret("print 1 + 0.5\n").unwrap(), "1.5\n");
    assert_eq!(interpret("print 1 - 0.5\n").unwrap(), "0.5\n");
    assert_eq!(interpret("print 3 * 0.5\n").unwrap(), "1.5\n");
    assert_eq!(interpret("print 3 / 2.0\n").unwrap(), "1.5\n");
}

#[test]
fn an_integer_equals_a_decimal_holding_the_same_number() {
    assert_eq!(interpret("print 1 == 1.0\n").unwrap(), "true\n");
    assert_eq!(interpret("print 1 != 1.0\n").unwrap(), "false\n");
}

#[test]
fn integer_overflow_reports_an_error() {
    let error = interpret("print 9223372036854775807 + 1\n").unwrap_err();
    assert!(error.contains("overflow"), "{error}");
}

#[test]
fn logical_operators_do_not_evaluate_the_second_operand_when_the_first_decides() {
    assert_eq!(
        interpret("print false and undeclared\n").unwrap(),
        "false\n"
    );
    assert_eq!(interpret("print true or undeclared\n").unwrap(), "true\n");
}

#[test]
fn evaluates_logical_operators() {
    assert_eq!(interpret("print true and false\n").unwrap(), "false\n");
    assert_eq!(interpret("print false or true\n").unwrap(), "true\n");
}

// ----------------------------------------------------------------- variables

#[test]
fn declares_and_reads_a_variable() {
    let source = "let x = 10\nlet y = x + 5\nprint x\nprint y\n";
    assert_eq!(interpret(source).unwrap(), "10\n15\n");
}

#[test]
fn a_declaration_without_an_initializer_is_null() {
    assert_eq!(interpret("let x\nprint x\n").unwrap(), "null\n");
}

#[test]
fn a_variable_survives_across_statements() {
    let source = "let x = 1\nx = x + 1\nx = x + 1\nprint x\n";
    assert_eq!(interpret(source).unwrap(), "3\n");
}

#[test]
fn redeclaring_a_variable_replaces_it() {
    assert_eq!(interpret("let x = 1\nlet x = 2\nprint x\n").unwrap(), "2\n");
}

#[test]
fn assignment_evaluates_to_the_assigned_value() {
    let source = "let x = 1\nlet y = x = 9\nprint y\nprint x\n";
    assert_eq!(interpret(source).unwrap(), "9\n9\n");
}

#[test]
fn reading_an_undeclared_variable_is_an_error() {
    let error = interpret("print nope\n").unwrap_err();
    assert!(error.contains("'nope' not defined"), "{error}");
}

#[test]
fn assigning_to_an_undeclared_variable_is_an_error() {
    let error = interpret("nope = 1\n").unwrap_err();
    assert!(error.contains("'nope' not defined"), "{error}");
}

// ------------------------------------------------------------------ branches

#[test]
fn takes_the_then_branch() {
    assert_eq!(interpret("if true:\n\tprint \"then\"\n").unwrap(), "then\n");
}

#[test]
fn takes_the_else_branch() {
    let source = "if false:\n\tprint \"then\"\nelse:\n\tprint \"else\"\n";
    assert_eq!(interpret(source).unwrap(), "else\n");
}

#[test]
fn skips_an_if_without_else() {
    assert_eq!(interpret("if false:\n\tprint \"then\"\n").unwrap(), "");
}

#[test]
fn truthy_conditions_enter_the_if() {
    assert_eq!(interpret("if 5:\n\tprint \"in\"\n").unwrap(), "in\n");
    assert_eq!(interpret("if \"s\":\n\tprint \"in\"\n").unwrap(), "in\n");
}

#[test]
fn falsy_conditions_skip_the_if() {
    assert_eq!(interpret("if 0:\n\tprint \"in\"\n").unwrap(), "");
    assert_eq!(interpret("if 0.0:\n\tprint \"in\"\n").unwrap(), "");
    assert_eq!(interpret("if \"\":\n\tprint \"in\"\n").unwrap(), "");
    assert_eq!(interpret("if null:\n\tprint \"in\"\n").unwrap(), "");
}

#[test]
fn nests_branches() {
    let source = "if true:\n\tif false:\n\t\tprint 1\n\telse:\n\t\tprint 2\n";
    assert_eq!(interpret(source).unwrap(), "2\n");
}

// --------------------------------------------------------------------- loops

#[test]
fn runs_a_while_loop_until_its_condition_is_false() {
    let source = concat!(
        "let i = 0\n",
        "while i < 3:\n",
        "\ti = i + 1\n",
        "\tprint i\n",
    );
    assert_eq!(interpret(source).unwrap(), "1\n2\n3\n");
}

#[test]
fn a_while_whose_condition_starts_false_never_runs() {
    assert_eq!(interpret("while false:\n\tprint 1\n").unwrap(), "");
}

#[test]
fn break_leaves_the_loop() {
    let source = concat!(
        "let i = 0\n",
        "while true:\n",
        "\ti = i + 1\n",
        "\tprint i\n",
        "\tif i == 3:\n",
        "\t\tbreak\n",
    );
    assert_eq!(interpret(source).unwrap(), "1\n2\n3\n");
}

#[test]
fn continue_skips_the_rest_of_the_iteration() {
    let source = concat!(
        "let i = 0\n",
        "while i < 4:\n",
        "\ti = i + 1\n",
        "\tif i == 2:\n",
        "\t\tcontinue\n",
        "\tprint i\n",
    );
    assert_eq!(interpret(source).unwrap(), "1\n3\n4\n");
}

#[test]
fn break_only_leaves_the_innermost_loop() {
    let source = concat!(
        "let outer = 0\n",
        "while outer < 2:\n",
        "\touter = outer + 1\n",
        "\tlet inner = 0\n",
        "\twhile true:\n",
        "\t\tinner = inner + 1\n",
        "\t\tprint outer * 10 + inner\n",
        "\t\tif inner == 2:\n",
        "\t\t\tbreak\n",
    );
    assert_eq!(interpret(source).unwrap(), "11\n12\n21\n22\n");
}

// ----------------------------------------------------------------- functions

#[test]
fn a_function_is_a_printable_value() {
    let source = "func f():\n\treturn 1\nprint f\n";
    assert_eq!(interpret(source).unwrap(), "<function f>\n");
}

#[test]
fn calls_a_function_and_returns_a_value() {
    let source = "func double(n):\n\treturn n * 2\nprint double(21)\n";
    assert_eq!(interpret(source).unwrap(), "42\n");
}

#[test]
fn passes_several_arguments() {
    let source = "func add(a, b):\n\treturn a + b\nprint add(2, 3)\n";
    assert_eq!(interpret(source).unwrap(), "5\n");
}

#[test]
fn a_function_without_a_return_evaluates_to_null() {
    let source = "func f():\n\tlet ignored = 1\nprint f()\n";
    assert_eq!(interpret(source).unwrap(), "null\n");
}

#[test]
fn a_bare_return_evaluates_to_null() {
    let source = "func f():\n\treturn\nprint f()\n";
    assert_eq!(interpret(source).unwrap(), "null\n");
}

#[test]
fn return_stops_executing_the_body() {
    let source = concat!(
        "func f():\n",
        "\tprint \"before\"\n",
        "\treturn 1\n",
        "\tprint \"after\"\n",
        "print f()\n",
    );
    assert_eq!(interpret(source).unwrap(), "before\n1\n");
}

#[test]
fn returns_from_inside_a_loop() {
    let source = concat!(
        "func first_over(limit):\n",
        "\tlet i = 0\n",
        "\twhile true:\n",
        "\t\ti = i + 1\n",
        "\t\tif i > limit:\n",
        "\t\t\treturn i\n",
        "print first_over(4)\n",
    );
    assert_eq!(interpret(source).unwrap(), "5\n");
}

#[test]
fn recursion_works() {
    let source = concat!(
        "func fact(n):\n",
        "\tif n <= 1:\n",
        "\t\treturn 1\n",
        "\treturn n * fact(n - 1)\n",
        "print fact(5)\n",
    );
    assert_eq!(interpret(source).unwrap(), "120\n");
}

#[test]
fn functions_are_first_class_values() {
    let source = concat!(
        "func double(n):\n",
        "\treturn n * 2\n",
        "func apply(f, v):\n",
        "\treturn f(v)\n",
        "print apply(double, 5)\n",
    );
    assert_eq!(interpret(source).unwrap(), "10\n");
}

#[test]
fn a_function_can_read_a_global() {
    let source = "let base = 40\nfunc f():\n\treturn base + 2\nprint f()\n";
    assert_eq!(interpret(source).unwrap(), "42\n");
}

#[test]
fn a_function_can_assign_to_a_global() {
    let source = concat!(
        "let counter = 0\n",
        "func bump():\n",
        "\tcounter = counter + 1\n",
        "bump()\n",
        "bump()\n",
        "print counter\n",
    );
    assert_eq!(interpret(source).unwrap(), "2\n");
}

#[test]
fn a_function_can_assign_to_a_global_from_inside_a_block() {
    let source = concat!(
        "let counter = 0\n",
        "func bump():\n",
        "\tif true:\n",
        "\t\tcounter = 1\n",
        "bump()\n",
        "print counter\n",
    );
    assert_eq!(interpret(source).unwrap(), "1\n");
}

#[test]
fn a_local_shadows_a_global_without_overwriting_it() {
    let source = concat!(
        "let counter = 0\n",
        "func f():\n",
        "\tlet counter = 9\n",
        "\treturn counter\n",
        "print f()\n",
        "print counter\n",
    );
    assert_eq!(interpret(source).unwrap(), "9\n0\n");
}

#[test]
fn a_parameter_shadows_a_global_without_overwriting_it() {
    let source = "let n = 1\nfunc f(n):\n\treturn n\nprint f(9)\nprint n\n";
    assert_eq!(interpret(source).unwrap(), "9\n1\n");
}

#[test]
fn a_parameter_is_not_readable_after_the_call() {
    let error = interpret("func f(n):\n\treturn n\nlet x = f(1)\nprint n\n").unwrap_err();
    assert!(error.contains("'n' not defined"), "{error}");
}

#[test]
fn a_local_is_not_readable_after_the_call() {
    let source = "func f():\n\tlet local = 1\n\treturn local\nlet x = f()\nprint local\n";
    let error = interpret(source).unwrap_err();
    assert!(error.contains("'local' not defined"), "{error}");
}

#[test]
fn calling_with_the_wrong_number_of_arguments_is_an_error() {
    let too_few = interpret("func f(a, b):\n\treturn a\nf(1)\n").unwrap_err();
    assert!(
        too_few.contains("Expected 2 arguments but got 1"),
        "{too_few}"
    );

    let too_many = interpret("func f(a):\n\treturn a\nf(1, 2)\n").unwrap_err();
    assert!(
        too_many.contains("Expected 1 arguments but got 2"),
        "{too_many}"
    );
}

#[test]
fn calling_a_non_function_is_an_error() {
    let error = interpret("let x = 1\nx()\n").unwrap_err();
    assert!(error.contains("non-function"), "{error}");
}

#[test]
fn calling_an_undeclared_function_is_an_error() {
    let error = interpret("nope()\n").unwrap_err();
    assert!(error.contains("'nope' not defined"), "{error}");
}

// ------------------------------------------------------------------- scoping

#[test]
fn a_variable_declared_in_an_if_block_is_not_readable_after_it() {
    let error = interpret("if true:\n\tlet inside = 1\nprint inside\n").unwrap_err();
    assert!(error.contains("'inside' not defined"), "{error}");
}

#[test]
fn a_variable_declared_in_a_while_block_is_not_readable_after_it() {
    let source = concat!(
        "let i = 0\n",
        "while i < 1:\n",
        "\ti = i + 1\n",
        "\tlet inside = 1\n",
        "print inside\n",
    );
    let error = interpret(source).unwrap_err();
    assert!(error.contains("'inside' not defined"), "{error}");
}

#[test]
fn a_block_can_shadow_an_outer_variable_without_overwriting_it() {
    let source = "let x = 1\nif true:\n\tlet x = 2\n\tprint x\nprint x\n";
    assert_eq!(interpret(source).unwrap(), "2\n1\n");
}

#[test]
fn a_block_can_still_assign_to_an_outer_variable() {
    let source = "let x = 1\nif true:\n\tx = 2\nprint x\n";
    assert_eq!(interpret(source).unwrap(), "2\n");
}

#[test]
fn every_loop_iteration_can_redeclare_the_same_variable() {
    let source = concat!(
        "let i = 0\n",
        "while i < 3:\n",
        "\ti = i + 1\n",
        "\tlet step = i\n",
        "\tprint step\n",
    );
    assert_eq!(interpret(source).unwrap(), "1\n2\n3\n");
}

#[test]
fn a_callee_cannot_read_a_callers_local() {
    let source = concat!(
        "func g():\n",
        "\treturn caller_local\n",
        "func f():\n",
        "\tlet caller_local = 7\n",
        "\treturn g()\n",
        "print f()\n",
    );
    let error = interpret(source).unwrap_err();
    assert!(error.contains("'caller_local' not defined"), "{error}");
}

#[test]
fn a_callee_cannot_write_to_a_callers_local() {
    let source = concat!(
        "func g():\n",
        "\tcaller_local = 99\n",
        "func f():\n",
        "\tlet caller_local = 1\n",
        "\tg()\n",
        "\treturn caller_local\n",
        "print f()\n",
    );
    let error = interpret(source).unwrap_err();
    assert!(error.contains("'caller_local' not defined"), "{error}");
}

#[test]
fn a_nested_function_is_callable_from_the_function_that_declared_it() {
    let source = concat!(
        "func outer():\n",
        "\tfunc inner():\n",
        "\t\treturn 1\n",
        "\treturn inner()\n",
        "print outer()\n",
    );
    assert_eq!(interpret(source).unwrap(), "1\n");
}

#[test]
fn a_nested_function_is_not_callable_from_the_global_scope() {
    let source = concat!(
        "func outer():\n",
        "\tfunc inner():\n",
        "\t\treturn 1\n",
        "\treturn inner()\n",
        "let x = outer()\n",
        "print inner()\n",
    );
    let error = interpret(source).unwrap_err();
    assert!(error.contains("'inner' not defined"), "{error}");
}

#[test]
fn a_nested_function_cannot_read_the_locals_of_the_function_that_declared_it() {
    let source = concat!(
        "func outer(n):\n",
        "\tfunc inner():\n",
        "\t\treturn n * 2\n",
        "\treturn inner()\n",
        "print outer(21)\n",
    );
    let error = interpret(source).unwrap_err();
    assert!(error.contains("Variable 'n' not defined"), "{error}");
}

// -------------------------------------------------------------------- errors

#[test]
fn a_runtime_error_aborts_the_program() {
    let error = interpret("print 1\nlet y = 1 / 0\nprint 2\n").unwrap_err();
    assert_eq!(error, "Division by zero");
}

#[test]
fn an_error_inside_a_function_propagates_to_the_caller() {
    let error = interpret("func f():\n\treturn 1 / 0\nf()\n").unwrap_err();
    assert_eq!(error, "Division by zero");
}

#[test]
fn a_return_outside_a_function_is_an_error() {
    let error = interpret("return 1\n").unwrap_err();
    assert_eq!(error, "'return' outside of a function");
}

#[test]
fn break_and_continue_outside_a_loop_are_errors() {
    assert_eq!(
        interpret("break\n").unwrap_err(),
        "'break' outside of a loop"
    );
    assert_eq!(
        interpret("continue\n").unwrap_err(),
        "'continue' outside of a loop"
    );
}

#[test]
fn break_and_continue_inside_a_function_but_outside_a_loop_are_errors() {
    assert_eq!(
        interpret("func f():\n\tbreak\nf()\n").unwrap_err(),
        "'break' outside of a loop"
    );
    assert_eq!(
        interpret("func f():\n\tcontinue\nf()\n").unwrap_err(),
        "'continue' outside of a loop"
    );
}

#[test]
fn logical_or_yield_the_deciding_operand() {
    assert_eq!(
        interpret("print 0 or \"fallback\"\n").unwrap(),
        "fallback\n"
    );
}

#[test]
fn logical_and_dont_yield_the_deciding_operand() {
    assert_eq!(interpret("print 0 and \"fallback\"\n").unwrap(), "false\n");
}
