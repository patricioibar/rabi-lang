use super::value::Value;

fn int(i: i64) -> Value {
    Value::Integer(i)
}

fn dec(d: f64) -> Value {
    Value::Decimal(d)
}

fn string(s: &str) -> Value {
    Value::String(s.to_string())
}

fn array(elements: Vec<Value>) -> Value {
    Value::array(elements)
}

// ---------------------------------------------------------------- arithmetic

#[test]
fn adds_integers() {
    assert_eq!((int(2) + int(3)).unwrap(), int(5));
}

#[test]
fn adds_decimals() {
    assert_eq!((dec(0.5) + dec(0.25)).unwrap(), dec(0.75));
}

#[test]
fn adds_strings_by_concatenating() {
    assert_eq!((string("ra") + string("bi")).unwrap(), string("rabi"));
}

#[test]
fn addition_rejects_unrelated_types() {
    let error = (int(1) + Value::Boolean(true)).unwrap_err();
    assert!(error.contains("Integer"), "{error}");
    assert!(error.contains("Boolean"), "{error}");
}

#[test]
fn subtracts_integers_and_decimals() {
    assert_eq!((int(5) - int(3)).unwrap(), int(2));
    assert_eq!((dec(1.5) - dec(0.5)).unwrap(), dec(1.0));
}

#[test]
fn subtraction_rejects_strings() {
    assert!((string("a") - string("b")).is_err());
}

#[test]
fn multiplies_integers_and_decimals() {
    assert_eq!((int(4) * int(3)).unwrap(), int(12));
    assert_eq!((dec(1.5) * dec(2.0)).unwrap(), dec(3.0));
}

#[test]
fn multiplying_a_string_by_an_integer_repeats_it() {
    assert_eq!((string("ab") * int(3)).unwrap(), string("ababab"));
    assert_eq!((int(3) * string("ab")).unwrap(), string("ababab"));
}

#[test]
fn multiplying_a_string_by_zero_gives_the_empty_string() {
    assert_eq!((string("ab") * int(0)).unwrap(), string(""));
}

#[test]
fn multiplying_a_string_by_a_negative_integer_is_an_error() {
    let error = (string("ab") * int(-1)).unwrap_err();
    assert!(error.contains("negative"), "{error}");
}

#[test]
fn divides_integers_truncating_towards_zero() {
    assert_eq!((int(7) / int(2)).unwrap(), int(3));
    assert_eq!((int(-7) / int(2)).unwrap(), int(-3));
}

#[test]
fn divides_mixing_integers_and_decimals() {
    assert_eq!((dec(1.0) / dec(4.0)).unwrap(), dec(0.25));
    assert_eq!((int(1) / dec(4.0)).unwrap(), dec(0.25));
    assert_eq!((dec(1.0) / int(4)).unwrap(), dec(0.25));
}

#[test]
fn division_by_zero_is_an_error_for_every_numeric_combination() {
    assert_eq!((int(1) / int(0)).unwrap_err(), "Division by zero");
    assert_eq!((dec(1.0) / dec(0.0)).unwrap_err(), "Division by zero");
    assert_eq!((int(1) / dec(0.0)).unwrap_err(), "Division by zero");
    assert_eq!((dec(1.0) / int(0)).unwrap_err(), "Division by zero");
}

// --------------------------------------------------------------- comparisons

#[test]
fn compares_integers() {
    assert!(int(2).greater_than(&int(1)).unwrap());
    assert!(!int(1).greater_than(&int(2)).unwrap());
    assert!(int(1).less_than(&int(2)).unwrap());
    assert!(int(1).greater_equal_than(&int(1)).unwrap());
    assert!(int(1).less_equal_than(&int(1)).unwrap());
}

#[test]
fn compares_across_integers_and_decimals() {
    assert!(int(2).greater_than(&dec(1.5)).unwrap());
    assert!(dec(1.5).less_than(&int(2)).unwrap());
    assert!(dec(2.0).greater_equal_than(&int(2)).unwrap());
    assert!(int(2).less_equal_than(&dec(2.0)).unwrap());
}

#[test]
fn compares_strings_lexicographically() {
    assert!(string("b").greater_than(&string("a")).unwrap());
    assert!(string("a").less_than(&string("ab")).unwrap());
}

#[test]
fn comparing_unrelated_types_is_an_error() {
    for result in [
        int(1).greater_than(&string("a")),
        int(1).less_than(&Value::Boolean(true)),
        Value::Null.greater_equal_than(&int(1)),
        Value::Boolean(true).less_equal_than(&Value::Boolean(false)),
    ] {
        let error = result.unwrap_err();
        assert!(error.contains("Unsupported operand types"), "{error}");
    }
}

// ---------------------------------------------------------------- truthiness

#[test]
fn booleans_are_truthy_when_true() {
    assert!(Value::Boolean(true).is_truthy());
    assert!(!Value::Boolean(false).is_truthy());
}

#[test]
fn null_is_falsy() {
    assert!(!Value::Null.is_truthy());
}

#[test]
fn zero_is_falsy_and_other_integers_are_truthy() {
    assert!(!int(0).is_truthy());
    assert!(int(1).is_truthy());
    assert!(int(-1).is_truthy());
}

#[test]
fn non_empty_strings_are_truthy() {
    assert!(string("hello").is_truthy());
}

// ------------------------------------------------------------ names & display

#[test]
fn reports_type_names() {
    assert_eq!(int(1).type_name(), "Integer");
    assert_eq!(dec(1.0).type_name(), "Decimal");
    assert_eq!(string("a").type_name(), "String");
    assert_eq!(Value::Boolean(true).type_name(), "Boolean");
    assert_eq!(Value::Null.type_name(), "Null");
    assert_eq!(
        Value::Function {
            name: "f".to_string(),
            parameters: vec![],
            body: vec![],
        }
        .type_name(),
        "Function"
    );
}

#[test]
fn displays_values() {
    assert_eq!(int(42).to_string(), "42");
    assert_eq!(dec(1.5).to_string(), "1.5");
    assert_eq!(string("hi").to_string(), "hi");
    assert_eq!(Value::Boolean(true).to_string(), "true");
    assert_eq!(Value::Null.to_string(), "null");
    assert_eq!(
        Value::Function {
            name: "f".to_string(),
            parameters: vec!["a".to_string()],
            body: vec![],
        }
        .to_string(),
        "<function f>"
    );
}

// ------------------------------- regressions for bugs 5, 6, 8 and 9 -------

#[test]
fn arithmetic_mixes_integers_and_decimals() {
    assert_eq!((int(1) + dec(0.5)).unwrap(), dec(1.5));
    assert_eq!((dec(0.5) + int(1)).unwrap(), dec(1.5));
    assert_eq!((int(2) - dec(0.5)).unwrap(), dec(1.5));
    assert_eq!((dec(2.5) - int(1)).unwrap(), dec(1.5));
    assert_eq!((int(2) * dec(0.5)).unwrap(), dec(1.0));
    assert_eq!((dec(0.5) * int(2)).unwrap(), dec(1.0));
}

#[test]
fn an_integer_equals_a_decimal_holding_the_same_number() {
    assert_eq!(int(1), dec(1.0));
    assert_eq!(dec(1.0), int(1));
    assert_ne!(int(1), dec(1.5));
}

#[test]
fn values_of_different_types_are_never_equal() {
    assert_ne!(int(1), string("1"));
    assert_ne!(int(1), Value::Boolean(true));
    assert_ne!(Value::Null, int(0));
}

#[test]
fn zero_like_values_are_all_falsy() {
    assert!(!int(0).is_truthy());
    assert!(!dec(0.0).is_truthy());
    assert!(!string("").is_truthy());
}

#[test]
fn integer_overflow_is_a_runtime_error_not_a_panic() {
    for result in [
        int(i64::MAX) + int(1),
        int(i64::MIN) - int(1),
        int(i64::MAX) * int(2),
        int(i64::MIN) / int(-1),
    ] {
        let error = result.unwrap_err();
        assert!(error.contains("overflow"), "{error}");
    }
}

#[test]
fn overflow_is_reported_per_operation() {
    assert_eq!(
        (int(i64::MAX) + int(1)).unwrap_err(),
        "Integer overflow in addition"
    );
    assert_eq!(
        (int(i64::MIN) / int(-1)).unwrap_err(),
        "Integer overflow in division"
    );
}

#[test]
fn division_by_zero_still_reports_zero_not_overflow() {
    assert_eq!((int(i64::MIN) / int(0)).unwrap_err(), "Division by zero");
}

// -------------------------------------------------------------------- arrays

#[test]
fn adds_arrays_by_concatenating() {
    assert_eq!(
        (array(vec![int(1)]) + array(vec![int(2), string("a")])).unwrap(),
        array(vec![int(1), int(2), string("a")])
    );
    assert_eq!(
        (array(vec![]) + array(vec![int(1)])).unwrap(),
        array(vec![int(1)])
    );
}

#[test]
fn concatenation_does_not_alias_its_operands() {
    let left = array(vec![int(1)]);
    let joined = (left.clone() + array(vec![int(2)])).unwrap();

    let Value::Array(elements) = &left else {
        panic!("left should be an array");
    };
    elements.borrow_mut()[0] = int(99);

    assert_eq!(joined, array(vec![int(1), int(2)]));
}

#[test]
fn multiplying_an_array_by_an_integer_repeats_it() {
    assert_eq!(
        (array(vec![int(1), int(2)]) * int(2)).unwrap(),
        array(vec![int(1), int(2), int(1), int(2)])
    );
    assert_eq!(
        (int(2) * array(vec![int(1)])).unwrap(),
        array(vec![int(1), int(1)])
    );
    assert_eq!((array(vec![int(1)]) * int(0)).unwrap(), array(vec![]));
}

#[test]
fn multiplying_an_array_by_a_negative_integer_is_an_error() {
    let error = (array(vec![int(1)]) * int(-1)).unwrap_err();
    assert!(error.contains("negative"), "{error}");
}
