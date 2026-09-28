use std::{cell::RefCell, rc::Rc};

use crate::statement::Statement;

#[derive(Debug, Clone)]
pub(super) enum Value {
    Integer(i64),
    Decimal(f64),
    String(String),
    Boolean(bool),
    Array(Rc<RefCell<Vec<Value>>>),
    Function(Rc<Function>),
    Null,
}

#[derive(Debug, PartialEq)]
pub(super) struct Function {
    pub name: String,
    pub parameters: Vec<String>,
    pub body: Vec<Statement>,
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Integer(a), Value::Integer(b)) => a == b,
            (Value::Decimal(a), Value::Decimal(b)) => a == b,
            (Value::Integer(a), Value::Decimal(b)) | (Value::Decimal(b), Value::Integer(a)) => {
                (*a as f64) == *b
            }
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::Array(a), Value::Array(b)) => a == b,
            (Value::Null, Value::Null) => true,
            (Value::Function(a), Value::Function(b)) => a == b,
            _ => false,
        }
    }
}

const DIVISION_BY_ZERO: &str = "Division by zero";

pub(super) fn overflow(operation: &str) -> String {
    format!("Integer overflow in {}", operation)
}

impl Value {
    pub fn array(elements: Vec<Value>) -> Value {
        Value::Array(Rc::new(RefCell::new(elements)))
    }

    pub fn function(name: String, parameters: Vec<String>, body: Vec<Statement>) -> Value {
        Value::Function(Rc::new(Function {
            name,
            parameters,
            body,
        }))
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Boolean(b) => *b,
            Value::Null => false,
            Value::Integer(i) => *i != 0,
            Value::Decimal(d) => *d != 0.0,
            Value::String(s) => !s.is_empty(),
            Value::Array(elements) => !elements.borrow().is_empty(),
            Value::Function(_) => true,
        }
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Integer(_) => "Integer",
            Value::Decimal(_) => "Decimal",
            Value::String(_) => "String",
            Value::Boolean(_) => "Boolean",
            Value::Array(_) => "Array",
            Value::Function(_) => "Function",
            Value::Null => "Null",
        }
    }

    pub fn greater_than(&self, other: &Value) -> Result<bool, String> {
        match (self, other) {
            (Value::Integer(a), Value::Integer(b)) => Ok(a > b),
            (Value::Decimal(a), Value::Decimal(b)) => Ok(a > b),
            (Value::Integer(a), Value::Decimal(b)) => Ok((*a as f64) > *b),
            (Value::Decimal(a), Value::Integer(b)) => Ok(*a > (*b as f64)),
            (Value::String(a), Value::String(b)) => Ok(a > b),
            _ => Err(format!(
                "Unsupported operand types for comparison: {} and {}",
                self.type_name(),
                other.type_name()
            )),
        }
    }

    pub fn less_than(&self, other: &Value) -> Result<bool, String> {
        match (self, other) {
            (Value::Integer(a), Value::Integer(b)) => Ok(a < b),
            (Value::Decimal(a), Value::Decimal(b)) => Ok(a < b),
            (Value::Integer(a), Value::Decimal(b)) => Ok((*a as f64) < *b),
            (Value::Decimal(a), Value::Integer(b)) => Ok(*a < (*b as f64)),
            (Value::String(a), Value::String(b)) => Ok(a < b),
            _ => Err(format!(
                "Unsupported operand types for comparison: {} and {}",
                self.type_name(),
                other.type_name()
            )),
        }
    }

    pub fn greater_equal_than(&self, other: &Value) -> Result<bool, String> {
        match (self, other) {
            (Value::Integer(a), Value::Integer(b)) => Ok(a >= b),
            (Value::Decimal(a), Value::Decimal(b)) => Ok(a >= b),
            (Value::Integer(a), Value::Decimal(b)) => Ok((*a as f64) >= *b),
            (Value::Decimal(a), Value::Integer(b)) => Ok(*a >= (*b as f64)),
            (Value::String(a), Value::String(b)) => Ok(a >= b),
            _ => Err(format!(
                "Unsupported operand types for comparison: {} and {}",
                self.type_name(),
                other.type_name()
            )),
        }
    }

    pub fn less_equal_than(&self, other: &Value) -> Result<bool, String> {
        match (self, other) {
            (Value::Integer(a), Value::Integer(b)) => Ok(a <= b),
            (Value::Decimal(a), Value::Decimal(b)) => Ok(a <= b),
            (Value::Integer(a), Value::Decimal(b)) => Ok((*a as f64) <= *b),
            (Value::Decimal(a), Value::Integer(b)) => Ok(*a <= (*b as f64)),
            (Value::String(a), Value::String(b)) => Ok(a <= b),
            _ => Err(format!(
                "Unsupported operand types for comparison: {} and {}",
                self.type_name(),
                other.type_name()
            )),
        }
    }

    pub fn modulo(&self, other: Value) -> Result<Value, String> {
        match (self, other) {
            (Value::Integer(a), Value::Integer(b)) => {
                if b == 0 {
                    return Err(DIVISION_BY_ZERO.to_string());
                }
                Ok(Value::Integer(a % b))
            }
            (Value::Decimal(a), Value::Decimal(b)) => {
                if b == 0.0 {
                    return Err(DIVISION_BY_ZERO.to_string());
                }
                Ok(Value::Decimal(a % b))
            }
            (Value::Integer(a), Value::Decimal(b)) => {
                if b == 0.0 {
                    return Err(DIVISION_BY_ZERO.to_string());
                }
                Ok(Value::Decimal((*a as f64) % b))
            }
            (Value::Decimal(a), Value::Integer(b)) => {
                if b == 0 {
                    return Err(DIVISION_BY_ZERO.to_string());
                }
                Ok(Value::Decimal(a % (b as f64)))
            }
            (a, b) => Err(format!(
                "Unsupported operand types for modulo: {} and {}",
                a.type_name(),
                b.type_name()
            )),
        }
    }
}

impl std::ops::Add for Value {
    type Output = Result<Value, String>;

    fn add(self, other: Value) -> Self::Output {
        match (self, other) {
            (Value::Integer(a), Value::Integer(b)) => a
                .checked_add(b)
                .map(Value::Integer)
                .ok_or_else(|| overflow("addition")),
            (Value::Decimal(a), Value::Decimal(b)) => Ok(Value::Decimal(a + b)),
            (Value::Integer(a), Value::Decimal(b)) => Ok(Value::Decimal(a as f64 + b)),
            (Value::Decimal(a), Value::Integer(b)) => Ok(Value::Decimal(a + b as f64)),
            (Value::String(a), Value::String(b)) => Ok(Value::String(format!("{}{}", a, b))),
            (Value::Array(a), Value::Array(b)) => {
                let mut elements = a.borrow().clone();
                elements.extend(b.borrow().iter().cloned());
                Ok(Value::array(elements))
            }
            (a, b) => Err(format!(
                "Unsupported operand types for addition: {} and {}",
                a.type_name(),
                b.type_name()
            )),
        }
    }
}

impl std::ops::Sub for Value {
    type Output = Result<Value, String>;

    fn sub(self, other: Value) -> Self::Output {
        match (self, other) {
            (Value::Integer(a), Value::Integer(b)) => a
                .checked_sub(b)
                .map(Value::Integer)
                .ok_or_else(|| overflow("subtraction")),
            (Value::Decimal(a), Value::Decimal(b)) => Ok(Value::Decimal(a - b)),
            (Value::Integer(a), Value::Decimal(b)) => Ok(Value::Decimal(a as f64 - b)),
            (Value::Decimal(a), Value::Integer(b)) => Ok(Value::Decimal(a - b as f64)),
            (a, b) => Err(format!(
                "Unsupported operand types for subtraction: {} and {}",
                a.type_name(),
                b.type_name()
            )),
        }
    }
}

impl std::ops::Mul for Value {
    type Output = Result<Value, String>;

    fn mul(self, other: Value) -> Self::Output {
        match (self, other) {
            (Value::Integer(a), Value::Integer(b)) => a
                .checked_mul(b)
                .map(Value::Integer)
                .ok_or_else(|| overflow("multiplication")),
            (Value::Decimal(a), Value::Decimal(b)) => Ok(Value::Decimal(a * b)),
            (Value::Integer(a), Value::Decimal(b)) => Ok(Value::Decimal(a as f64 * b)),
            (Value::Decimal(a), Value::Integer(b)) => Ok(Value::Decimal(a * b as f64)),
            (Value::String(s), Value::Integer(n)) | (Value::Integer(n), Value::String(s)) => {
                if n < 0 {
                    return Err("Cannot multiply string by negative integer".to_string());
                }
                Ok(Value::String(s.repeat(n as usize)))
            }
            (Value::Array(a), Value::Integer(n)) | (Value::Integer(n), Value::Array(a)) => {
                if n < 0 {
                    return Err("Cannot multiply array by negative integer".to_string());
                }
                let mut elements = Vec::with_capacity(a.borrow().len() * n as usize);
                for _ in 0..n {
                    elements.extend(a.borrow().iter().cloned());
                }
                Ok(Value::array(elements))
            }
            (a, b) => Err(format!(
                "Unsupported operand types for multiplication: {} and {}",
                a.type_name(),
                b.type_name()
            )),
        }
    }
}

impl std::ops::Div for Value {
    type Output = Result<Value, String>;

    fn div(self, other: Value) -> Self::Output {
        match (self, other) {
            (Value::Integer(a), Value::Integer(b)) => {
                if b == 0 {
                    return Err(DIVISION_BY_ZERO.to_string());
                }
                a.checked_div(b)
                    .map(Value::Integer)
                    .ok_or_else(|| overflow("division"))
            }
            (Value::Decimal(a), Value::Decimal(b)) => {
                if b == 0.0 {
                    return Err(DIVISION_BY_ZERO.to_string());
                }
                Ok(Value::Decimal(a / b))
            }
            (Value::Integer(a), Value::Decimal(b)) => {
                if b == 0.0 {
                    return Err(DIVISION_BY_ZERO.to_string());
                }
                Ok(Value::Decimal(a as f64 / b))
            }
            (Value::Decimal(a), Value::Integer(b)) => {
                if b == 0 {
                    return Err(DIVISION_BY_ZERO.to_string());
                }
                Ok(Value::Decimal(a / b as f64))
            }
            (a, b) => Err(format!(
                "Unsupported operand types for division: {} and {}",
                a.type_name(),
                b.type_name()
            )),
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Integer(i) => write!(f, "{}", i),
            Value::Decimal(d) => write!(f, "{}", d),
            Value::String(s) => write!(f, "{}", s),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Array(elements) => write!(
                f,
                "[{}]",
                elements
                    .borrow()
                    .iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Value::Function(function) => write!(f, "<function {}>", function.name),
            Value::Null => write!(f, "null"),
        }
    }
}
