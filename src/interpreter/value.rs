use std::rc::Rc;

use crate::statement::Statement;
use crate::value::FunctionValue;

pub(super) type Value = crate::value::Value<Function>;

#[derive(Debug, PartialEq)]
pub(super) struct Function {
    pub name: String,
    pub parameters: Vec<String>,
    pub body: Vec<Statement>,
}

impl FunctionValue for Function {
    fn name(&self) -> &str {
        &self.name
    }
}

impl Value {
    pub fn function(name: String, parameters: Vec<String>, body: Vec<Statement>) -> Value {
        Value::Function(Rc::new(Function {
            name,
            parameters,
            body,
        }))
    }
}
