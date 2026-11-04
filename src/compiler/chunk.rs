use crate::bytecode::{Operation, Value};

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

    pub fn disassemble(&self, name: &str) {
        println!("== {name} ==");
        for (i, op) in self.code.iter().enumerate() {
            println!("{i:04} {op:?}");
        }
    }
}
