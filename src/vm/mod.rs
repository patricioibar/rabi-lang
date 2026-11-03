use std::{
    collections::HashMap,
    io::{Stdout, Write},
    rc::Rc,
};

use crate::bytecode::{Function, Operation, Value};

#[cfg(test)]
mod vm_test;

const MAX_FRAMES: usize = 1024;

struct CallFrame {
    function: Rc<Function>,
    ip: usize,
    stack_slot: usize,
}

pub struct Vm<W: Write = Stdout> {
    stack: Vec<Value>,
    frames: Vec<CallFrame>,
    globals: HashMap<String, Value>,
    output: W,
}

impl Default for Vm<Stdout> {
    fn default() -> Self {
        Self::new()
    }
}

impl Vm<Stdout> {
    pub fn new() -> Self {
        Self::with_output(std::io::stdout())
    }
}

impl<W: Write> Vm<W> {
    pub fn with_output(output: W) -> Self {
        Vm {
            stack: Vec::new(),
            frames: Vec::new(),
            globals: HashMap::new(),
            output,
        }
    }

    pub fn run(&mut self, script: Function) -> Result<(), String> {
        let script = Rc::new(script);
        self.stack.clear();
        self.frames.clear();
        self.push(Value::Function(Rc::clone(&script)));
        self.frames.push(CallFrame {
            function: script,
            ip: 0,
            stack_slot: 0,
        });

        self.execute()
            .map_err(|e| format!("Error executing: {}", e))
    }

    #[rustfmt::skip]
    fn execute(&mut self) -> Result<(), String> {
        loop {
            let op = self.next_operation()?;
            match op {
                Operation::Null => self.push(Value::Null),
                Operation::True => self.push(Value::Boolean(true)),
                Operation::False => self.push(Value::Boolean(false)),

                Operation::GetConstant(i) => {
                    let value = self.constant(i)?;
                    self.push(value);
                }

                Operation::Pop => {
                    self.pop()?;
                }

                Operation::Print => {
                    let value = self.pop()?;
                    writeln!(self.output, "{}", value).map_err(|e| e.to_string())?;
                }

                Operation::Negate => {
                    let value = self.pop()?;
                    self.push(value.negate()?);
                }

                Operation::Not => {
                    let value = self.pop()?;
                    self.push(Value::Boolean(!value.is_truthy()));
                }

                Operation::Add => self.binary(|a, b| a + b)?,
                Operation::Subtract => self.binary(|a, b| a - b)?,
                Operation::Multiply => self.binary(|a, b| a * b)?,
                Operation::Divide => self.binary(|a, b| a / b)?,
                Operation::Modulo => self.binary(|a, b| a.modulo(b))?,
                Operation::Equal => self.binary(|a, b| Ok(Value::Boolean(a == b)))?,
                Operation::Greater => self.binary(|a, b| a.greater_than(&b).map(Value::Boolean))?,
                Operation::GreaterEqual => self.binary(|a, b| a.greater_equal_than(&b).map(Value::Boolean))?,
                Operation::Less => self.binary(|a, b| a.less_than(&b).map(Value::Boolean))?,
                Operation::LessEqual => self.binary(|a, b| a.less_equal_than(&b).map(Value::Boolean))?,

                Operation::DefineGlobal(i) => {
                    let name = self.constant_name(i)?;
                    let value = self.pop()?;
                    self.globals.insert(name, value);
                }

                Operation::GetGlobal(i) => {
                    let name = self.constant_name(i)?;
                    let value = self
                        .globals
                        .get(&name)
                        .cloned()
                        .ok_or_else(|| format!("Variable '{}' not defined", name))?;
                    self.push(value);
                }

                Operation::SetGlobal(i) => {
                    let name = self.constant_name(i)?;
                    let value = self.peek(0)?.clone();
                    match self.globals.get_mut(&name) {
                        Some(slot) => *slot = value,
                        None => return Err(format!("Variable '{}' not defined", name)),
                    }
                }

                Operation::GetLocal(slot) => {
                    let index = self.frame()?.stack_slot + slot as usize;
                    let value = self.stack.get(index).cloned().ok_or("Local slot out of bounds")?;
                    self.push(value);
                }

                Operation::SetLocal(slot) => {
                    let index = self.frame()?.stack_slot + slot as usize;
                    let value = self.peek(0)?.clone();
                    *self.stack.get_mut(index).ok_or("Local slot out of bounds")? = value;
                }

                Operation::Jump(offset) => {
                    self.frame_mut()?.ip += offset as usize;
                }

                Operation::JumpIfFalse(offset) => {
                    if !self.peek(0)?.is_truthy() {
                        self.frame_mut()?.ip += offset as usize;
                    }
                }

                Operation::Loop(offset) => {
                    self.frame_mut()?.ip -= offset as usize;
                }

                Operation::Call(arg_count) => {
                    self.call(arg_count as usize)?;
                }

                Operation::Return => {
                    let result = self.pop()?;
                    let frame = self.frames.pop().ok_or("Return outside of a frame")?;
                    self.stack.truncate(frame.stack_slot);
                    if self.frames.is_empty() {
                        return Ok(());
                    }
                    self.push(result);
                }
            }
        }
    }

    fn call(&mut self, arg_count: usize) -> Result<(), String> {
        let slots = self
            .stack
            .len()
            .checked_sub(arg_count + 1)
            .ok_or("Stack underflow")?;

        let function = match &self.stack[slots] {
            Value::Function(function) => Rc::clone(function),
            _ => return Err("Attempted to call a non-function value".to_string()),
        };
        if arg_count != function.arity {
            return Err(format!(
                "Expected {} arguments but got {}",
                function.arity, arg_count
            ));
        }
        if self.frames.len() >= MAX_FRAMES {
            return Err("Stack overflow".to_string());
        }

        self.frames.push(CallFrame {
            function,
            ip: 0,
            stack_slot: slots,
        });
        Ok(())
    }

    fn binary(
        &mut self,
        op: impl FnOnce(Value, Value) -> Result<Value, String>,
    ) -> Result<(), String> {
        let b = self.pop()?;
        let a = self.pop()?;
        self.push(op(a, b)?);
        Ok(())
    }

    fn next_operation(&mut self) -> Result<Operation, String> {
        let frame = self.frame_mut()?;
        let op = *frame
            .function
            .chunk
            .code
            .get(frame.ip)
            .ok_or("Reached end of bytecode without a Return")?;
        frame.ip += 1;
        Ok(op)
    }

    fn constant(&self, index: u16) -> Result<Value, String> {
        self.frame()?
            .function
            .chunk
            .constants
            .get(index as usize)
            .cloned()
            .ok_or_else(|| format!("Constant {} out of bounds", index))
    }

    fn constant_name(&self, index: u16) -> Result<String, String> {
        match self.constant(index)? {
            Value::String(name) => Ok(name.to_string()),
            other => Err(format!(
                "Expected a variable name constant, got {}",
                other.type_name()
            )),
        }
    }

    fn frame(&self) -> Result<&CallFrame, String> {
        self.frames
            .last()
            .ok_or_else(|| "No active frame".to_string())
    }

    fn frame_mut(&mut self) -> Result<&mut CallFrame, String> {
        self.frames
            .last_mut()
            .ok_or_else(|| "No active frame".to_string())
    }

    fn push(&mut self, value: Value) {
        self.stack.push(value);
    }

    fn pop(&mut self) -> Result<Value, String> {
        self.stack
            .pop()
            .ok_or_else(|| "Stack underflow".to_string())
    }

    fn peek(&self, distance: usize) -> Result<&Value, String> {
        self.stack
            .len()
            .checked_sub(distance + 1)
            .map(|i| &self.stack[i])
            .ok_or_else(|| "Stack underflow".to_string())
    }
}
