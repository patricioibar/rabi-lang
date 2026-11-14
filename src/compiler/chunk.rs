use std::fmt::Write;

use crate::bytecode::{Operation, Value};

const NAME_WIDTH: usize = 13;

#[derive(Debug, Default, PartialEq)]
pub struct Chunk {
    pub code: Vec<Operation>,
    pub constants: Vec<Value>,
}

impl Chunk {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn write(&mut self, op: Operation) -> usize {
        self.code.push(op);
        self.code.len() - 1
    }

    pub fn patch(&mut self, index: usize, op: Operation) {
        self.code[index] = op;
    }

    pub fn add_constant(&mut self, value: Value) -> Result<u16, String> {
        if self.constants.len() >= u16::MAX as usize {
            return Err("Exceded constants limit in chunk.".to_string());
        }
        self.constants.push(value);
        Ok((self.constants.len() - 1) as u16)
    }

    pub fn disassemble(&self, name: &str) -> String {
        let mut output = String::new();
        self.write_disassembly(name, &mut output)
            .expect("writing to a String never fails");
        output
    }

    fn write_disassembly(&self, name: &str, output: &mut String) -> std::fmt::Result {
        writeln!(output, "== {name} ==")?;
        for (offset, op) in self.code.iter().enumerate() {
            writeln!(output, "{offset:04} {}", self.operation(offset, *op))?;
        }
        for constant in &self.constants {
            if let Value::Function(function) = constant {
                writeln!(output)?;
                function.chunk.write_disassembly(&function.name, output)?;
            }
        }
        Ok(())
    }

    #[rustfmt::skip]
    fn operation(&self, offset: usize, op: Operation) -> String {
        match op {
            Operation::GetConstant(i) => self.constant_operation("GetConstant", i),
            Operation::DefineGlobal(i) => self.constant_operation("DefineGlobal", i),
            Operation::GetGlobal(i) => self.constant_operation("GetGlobal", i),
            Operation::SetGlobal(i) => self.constant_operation("SetGlobal", i),

            Operation::GetLocal(i) => format!("{:<NAME_WIDTH$} slot {i}", "GetLocal"),
            Operation::SetLocal(i) => format!("{:<NAME_WIDTH$} slot {i}", "SetLocal"),
            Operation::Call(n) => format!("{:<NAME_WIDTH$} args {n}", "Call"),
            Operation::BuildArray(n) => format!("{:<NAME_WIDTH$} elements {n}", "BuildArray"),

            Operation::Jump(n) => jump("Jump", offset + 1 + n as usize),
            Operation::JumpIfFalse(n) => jump("JumpIfFalse", offset + 1 + n as usize),
            Operation::ForIter(n) => jump("ForIter", offset + 1 + n as usize),
            Operation::Loop(n) => jump("Loop", (offset + 1).saturating_sub(n as usize)),

            simple => format!("{simple:?}"),
        }
    }

    fn constant_operation(&self, name: &str, index: u16) -> String {
        match self.constants.get(index as usize) {
            Some(value) => format!("{name:<NAME_WIDTH$} {index:>3} {}", constant(value)),
            None => format!("{name:<NAME_WIDTH$} {index:>3} <out of bounds>"),
        }
    }
}

/// Formats a jump as the absolute offset it lands on.
fn jump(name: &str, target: usize) -> String {
    format!("{name:<NAME_WIDTH$} -> {target:04}")
}

/// Formats a constant, quoting strings so names read as names.
fn constant(value: &Value) -> String {
    match value {
        Value::String(string) => format!("\"{string}\""),
        other => other.to_string(),
    }
}
