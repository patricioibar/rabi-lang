use std::{cell::RefCell, fmt::Debug, rc::Rc};

pub trait FunctionValue: PartialEq + Debug {
    fn name(&self) -> &str;
}

type Elements<F> = RefCell<Vec<Value<F>>>;

#[derive(Debug)]
pub enum Value<F> {
    Integer(i64),
    Decimal(f64),
    String(Rc<String>),
    Boolean(bool),
    Array(Rc<Elements<F>>),
    Function(Rc<F>),
    Null,
}

impl<F> Clone for Value<F> {
    fn clone(&self) -> Self {
        match self {
            Value::Integer(i) => Value::Integer(*i),
            Value::Decimal(d) => Value::Decimal(*d),
            Value::String(s) => Value::String(Rc::clone(s)),
            Value::Boolean(b) => Value::Boolean(*b),
            Value::Array(a) => Value::Array(Rc::clone(a)),
            Value::Function(f) => Value::Function(Rc::clone(f)),
            Value::Null => Value::Null,
        }
    }
}

impl<F: FunctionValue> PartialEq for Value<F> {
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

fn overflow(operation: &str) -> String {
    format!("Integer overflow in {}", operation)
}

impl<F> Value<F> {
    pub fn array(elements: Vec<Value<F>>) -> Self {
        Value::Array(Rc::new(RefCell::new(elements)))
    }

    pub fn string(s: impl Into<String>) -> Self {
        Value::String(Rc::new(s.into()))
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

    pub fn element_at(&self, position: usize) -> Result<Option<Value<F>>, String> {
        match self {
            Value::Array(elements) => Ok(elements.borrow().get(position).cloned()),
            other => Err(format!(
                "Unsupported type for iteration: {}",
                other.type_name()
            )),
        }
    }

    pub fn len(&self) -> Result<Value<F>, String> {
        match self {
            Value::Array(elements) => Ok(Value::Integer(elements.borrow().len() as i64)),
            other => Err(format!("Unsupported type for len: {}", other.type_name())),
        }
    }

    pub fn get_index(&self, index: &Value<F>) -> Result<Value<F>, String> {
        let (elements, position) = self.index_target(index)?;
        Ok(elements.borrow()[position].clone())
    }

    pub fn set_index(&self, index: &Value<F>, value: Value<F>) -> Result<(), String> {
        let (elements, position) = self.index_target(index)?;
        elements.borrow_mut()[position] = value;
        Ok(())
    }

    fn index_target(&self, index: &Value<F>) -> Result<(&Elements<F>, usize), String> {
        let Value::Array(elements) = self else {
            return Err(format!(
                "Unsupported type for indexing: {}",
                self.type_name()
            ));
        };
        let Value::Integer(position) = *index else {
            return Err(format!(
                "Array index must be an Integer, found {}",
                index.type_name()
            ));
        };
        let length = elements.borrow().len();
        if position < 0 || position as usize >= length {
            return Err(format!(
                "Array index out of bounds: {} (length {})",
                position, length
            ));
        }
        Ok((elements, position as usize))
    }

    pub fn negate(self) -> Result<Value<F>, String> {
        match self {
            Value::Integer(i) => i
                .checked_neg()
                .map(Value::Integer)
                .ok_or_else(|| overflow("negation")),
            Value::Decimal(d) => Ok(Value::Decimal(-d)),
            other => Err(format!(
                "Unsupported operand type for unary minus: {}",
                other.type_name()
            )),
        }
    }

    pub fn greater_than(&self, other: &Value<F>) -> Result<bool, String> {
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

    pub fn less_than(&self, other: &Value<F>) -> Result<bool, String> {
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

    pub fn greater_equal_than(&self, other: &Value<F>) -> Result<bool, String> {
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

    pub fn less_equal_than(&self, other: &Value<F>) -> Result<bool, String> {
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

    pub fn modulo(self, other: Value<F>) -> Result<Value<F>, String> {
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
                Ok(Value::Decimal((a as f64) % b))
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

impl<F> std::ops::Add for Value<F> {
    type Output = Result<Value<F>, String>;

    fn add(self, other: Value<F>) -> Self::Output {
        match (self, other) {
            (Value::Integer(a), Value::Integer(b)) => a
                .checked_add(b)
                .map(Value::Integer)
                .ok_or_else(|| overflow("addition")),
            (Value::Decimal(a), Value::Decimal(b)) => Ok(Value::Decimal(a + b)),
            (Value::Integer(a), Value::Decimal(b)) => Ok(Value::Decimal(a as f64 + b)),
            (Value::Decimal(a), Value::Integer(b)) => Ok(Value::Decimal(a + b as f64)),
            (Value::String(a), Value::String(b)) => Ok(Value::string(format!("{}{}", a, b))),
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

impl<F> std::ops::Sub for Value<F> {
    type Output = Result<Value<F>, String>;

    fn sub(self, other: Value<F>) -> Self::Output {
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

impl<F> std::ops::Mul for Value<F> {
    type Output = Result<Value<F>, String>;

    fn mul(self, other: Value<F>) -> Self::Output {
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
                Ok(Value::string(s.repeat(n as usize)))
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

impl<F> std::ops::Div for Value<F> {
    type Output = Result<Value<F>, String>;

    fn div(self, other: Value<F>) -> Self::Output {
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

impl<F: FunctionValue> std::fmt::Display for Value<F> {
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
            Value::Function(function) => write!(f, "<function {}>", function.name()),
            Value::Null => write!(f, "null"),
        }
    }
}
