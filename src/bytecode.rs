#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Operation {
    Null,
    True,
    False,
    Return,
    Print,
    Len,
    Pop,
    Negate,
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Not,
    Equal,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    GetIndex,
    SetIndex,

    Call(u16),
    BuildArray(u16),

    GetConstant(u16),
    DefineGlobal(u16),
    GetGlobal(u16),
    SetGlobal(u16),

    GetLocal(u16),
    SetLocal(u16),

    Loop(u16),
    JumpIfFalse(u16),
    Jump(u16),
}

use crate::compiler::Chunk;
use crate::value::FunctionValue;

pub type Value = crate::value::Value<Function>;

#[derive(Debug, PartialEq)]
pub struct Function {
    pub name: String,
    pub arity: usize,
    pub chunk: Chunk,
}

impl FunctionValue for Function {
    fn name(&self) -> &str {
        &self.name
    }
}
