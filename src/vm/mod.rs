use std::{
    collections::HashMap,
    hash::{BuildHasherDefault, Hasher},
    io::{Stdout, Write},
    rc::Rc,
};

use crate::bytecode::{Function, Operation, Value};

#[cfg(test)]
mod vm_test;

const MAX_FRAMES: usize = 1024;

#[derive(Default)]
struct FxHasher {
    hash: u64,
}

impl Hasher for FxHasher {
    fn write(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            self.hash =
                (self.hash.rotate_left(5) ^ byte as u64).wrapping_mul(0x51_7c_c1_b7_27_22_0a_95);
        }
    }

    fn finish(&self) -> u64 {
        self.hash
    }
}

type Globals = HashMap<String, Value, BuildHasherDefault<FxHasher>>;

struct CallFrame {
    function: Rc<Function>,
    ip: usize,
    stack_slot: usize,
}

pub struct Vm<W: Write = Stdout> {
    stack: Vec<Value>,
    frames: Vec<CallFrame>,
    globals: Globals,
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
            globals: Globals::default(),
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
                    let frame = self.frames.last().ok_or("No active frame")?;
                    let constant = frame
                        .function
                        .chunk
                        .constants
                        .get(i as usize)
                        .ok_or_else(|| format!("Constant {} out of bounds", i))?;
                    self.stack.extend_from_slice(std::slice::from_ref(constant));
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

                Operation::BuildArray(count) => {
                    let start = self
                        .stack
                        .len()
                        .checked_sub(count as usize)
                        .ok_or("Stack underflow")?;
                    let elements = self.stack.split_off(start);
                    self.push(Value::array(elements));
                }

                Operation::GetIndex => {
                    let index = self.pop()?;
                    let collection = self.pop()?;
                    self.push(collection.get_index(&index)?);
                }

                Operation::SetIndex => {
                    let value = self.pop()?;
                    let index = self.pop()?;
                    let collection = self.pop()?;
                    collection.set_index(&index, value.clone())?;
                    self.push(value);
                }

                Operation::Len => {
                    let value = self.pop()?;
                    self.push(value.len()?);
                }

                Operation::DefineGlobal(i) => {
                    let name = self.constant_name(i)?;
                    let value = self.pop()?;
                    self.globals.insert(name.to_string(), value);
                }

                Operation::GetGlobal(i) => {
                    let name = self.constant_name(i)?;
                    let value = self
                        .globals
                        .get(name.as_str())
                        .ok_or_else(|| format!("Variable '{}' not defined", name))?;
                    self.stack.extend_from_slice(std::slice::from_ref(value));
                }

                Operation::SetGlobal(i) => {
                    let name = self.constant_name(i)?;
                    let value = self.stack.last().ok_or("Stack underflow")?;
                    match self.globals.get_mut(name.as_str()) {
                        Some(slot) => slot.clone_from(value),
                        None => return Err(format!("Variable '{}' not defined", name)),
                    }
                }

                Operation::GetLocal(slot) => {
                    let index = self.frame()?.stack_slot + slot as usize;
                    if index >= self.stack.len() {
                        return Err("Local slot out of bounds".to_string());
                    }
                    self.stack.extend_from_within(index..=index);
                }

                Operation::SetLocal(slot) => {
                    let index = self.frame()?.stack_slot + slot as usize;
                    let top = self.stack.len().checked_sub(1).ok_or("Stack underflow")?;
                    let (below, rest) = self.stack.split_at_mut(top);
                    below
                        .get_mut(index)
                        .ok_or("Local slot out of bounds")?
                        .clone_from(&rest[0]);
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

                Operation::ForIter(offset) => {
                    let position = match self.peek(0)? {
                        Value::Integer(i) => *i as usize,
                        other => {
                            return Err(format!(
                                "Corrupted for loop: expected an Integer counter, found {}",
                                other.type_name()
                            ))
                        }
                    };
                    match self.peek(1)?.element_at(position)? {
                        Some(element) => {
                            *self.stack.last_mut().ok_or("Stack underflow")? =
                                Value::Integer(position as i64 + 1);
                            self.push(element);
                        }
                        None => self.frame_mut()?.ip += offset as usize,
                    }
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

    fn constant(&self, index: u16) -> Result<&Value, String> {
        self.frame()?
            .function
            .chunk
            .constants
            .get(index as usize)
            .ok_or_else(|| format!("Constant {} out of bounds", index))
    }

    fn constant_name(&self, index: u16) -> Result<Rc<String>, String> {
        match self.constant(index)? {
            Value::String(name) => Ok(Rc::clone(name)),
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
