use super::value::Value;

#[test]
fn function_values_are_equal_when_their_ast_is_equal() {
    let f = || Value::function("f".to_string(), vec!["a".to_string()], vec![]);
    assert_eq!(f(), f());
    assert_ne!(
        f(),
        Value::function("f".to_string(), vec!["b".to_string()], vec![])
    );
}

#[test]
fn function_values_report_type_and_display() {
    let f = Value::function("f".to_string(), vec!["a".to_string()], vec![]);
    assert_eq!(f.type_name(), "Function");
    assert_eq!(f.to_string(), "<function f>");
    assert!(f.is_truthy());
}
