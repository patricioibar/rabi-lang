use crate::{
    bytecode::{Function, Operation, Value},
    expression::Expression,
    statement::Statement,
    token::Token,
};

mod chunk;

#[cfg(test)]
mod compiler_test;

pub use chunk::Chunk;

/// Compila el programa entero como una función "script" de nivel superior.
pub fn compile(statements: Vec<Statement>) -> Result<Function, String> {
    let mut compiler = Compiler::new("<script>".to_string(), 0);
    for stmt in statements {
        compiler.statement(stmt)?;
    }
    compiler.finish()
}

struct Local {
    name: String,
    depth: usize,
}

struct Loop {
    start: usize,
    depth: usize,
    break_jumps: Vec<usize>,
}

struct Compiler {
    function: Function,
    locals: Vec<Local>,
    scope_depth: usize,
    loops: Vec<Loop>,
}

impl Compiler {
    fn new(name: String, scope_depth: usize) -> Self {
        Compiler {
            function: Function {
                name,
                arity: 0,
                chunk: Chunk::new(),
            },
            locals: vec![Local {
                name: String::new(),
                depth: scope_depth,
            }],
            scope_depth,
            loops: Vec::new(),
        }
    }

    fn finish(mut self) -> Result<Function, String> {
        if self.scope_depth != self.locals[0].depth {
            return Err("Unclosed scope at end of function.".to_string());
        }
        self.emit(Operation::Null);
        self.emit(Operation::Return);
        Ok(self.function)
    }

    // ---------- emisión ----------

    fn chunk(&mut self) -> &mut Chunk {
        &mut self.function.chunk
    }

    fn emit(&mut self, op: Operation) -> usize {
        self.chunk().write(op)
    }

    fn emit_constant(&mut self, value: Value) -> Result<(), String> {
        let index = self.chunk().add_constant(value)?;
        self.emit(Operation::GetConstant(index));
        Ok(())
    }

    fn emit_jump(&mut self, op: fn(u16) -> Operation) -> usize {
        self.emit(op(0xffff))
    }

    fn patch_jump(&mut self, index: usize) -> Result<(), String> {
        let jump_offset = (self.chunk().code.len() - index - 1) as u16;
        let op = match self.chunk().code[index] {
            Operation::Jump(_) => Operation::Jump(jump_offset),
            Operation::JumpIfFalse(_) => Operation::JumpIfFalse(jump_offset),
            Operation::ForIter(_) => Operation::ForIter(jump_offset),
            _ => return Err("Expected jump operation".to_string()),
        };
        self.chunk().patch(index, op);
        Ok(())
    }

    fn emit_loop(&mut self, loop_start: usize) -> Result<(), String> {
        let offset = self.chunk().code.len() - loop_start + 1;
        self.emit(Operation::Loop(offset as u16));
        Ok(())
    }

    // ---------- scopes y variables ----------

    fn begin_scope(&mut self) {
        self.scope_depth += 1;
    }

    fn end_scope(&mut self) {
        self.scope_depth -= 1;
        while let Some(local) = self.locals.last() {
            if local.depth > self.scope_depth {
                self.emit(Operation::Pop);
                self.locals.pop();
            } else {
                break;
            }
        }
    }

    fn resolve_local(&self, name: String) -> Option<u16> {
        let index = self.locals.iter().rposition(|local| local.name == name)?;
        Some(index as u16)
    }

    fn identifier_constant(&mut self, name: String) -> Result<u16, String> {
        let existing = self.chunk().constants.iter().position(
            |constant| matches!(constant, Value::String(constant_name) if **constant_name == name),
        );
        match existing {
            Some(index) => Ok(index as u16),
            None => self.chunk().add_constant(Value::String(name.into())),
        }
    }

    fn define_variable(&mut self, name: String) -> Result<(), String> {
        if self.scope_depth == 0 {
            let index = self.identifier_constant(name)?;
            self.emit(Operation::DefineGlobal(index));
        } else {
            let depth = self.scope_depth;
            self.locals.push(Local { name, depth });
        }
        Ok(())
    }

    // ---------- statements ----------

    #[rustfmt::skip]
    fn statement(&mut self, stmt: Statement) -> Result<(), String> {
        match stmt {
            Statement::FunctionDeclaration { name, parameters, body } => {
                self.function_declaration(name, parameters, body)
            }
            Statement::VariableDeclaration { name, initializer } => {
                self.variable_declaration(name, initializer)
            }
            Statement::PrintStatement(expression) => {
                self.print_statement(expression)
            }
            Statement::IfStatement { condition, then_branch, else_branch } => {
                self.if_statement(condition, then_branch, else_branch)
            }
            Statement::WhileStatement { condition, block } => {
                self.while_statement(condition, block)
            }
            Statement::ForStatement { variable, iterable, block } => {
                self.for_statement(variable, iterable, block)
            }
            Statement::ExpressionStatement(expression) => {
                self.expression_statement(expression)
            }
            Statement::ReturnStatement { value } => {
                self.return_statement(value)
            }
            Statement::BreakStatement => {
                self.break_statement()
            }
            Statement::ContinueStatement => {
                self.continue_statement()
            }
        }
    }

    fn function_declaration(
        &mut self,
        name: String,
        parameters: Vec<String>,
        body: Vec<Statement>,
    ) -> Result<(), String> {
        let mut compiler = Compiler::new(name.clone(), 1);
        compiler.function.arity = parameters.len();
        for parameter in parameters {
            compiler.define_variable(parameter)?;
        }
        for statement in body {
            compiler.statement(statement)?;
        }
        let function = compiler.finish()?;
        self.emit_constant(Value::Function(function.into()))?;
        self.define_variable(name)
    }

    fn variable_declaration(
        &mut self,
        name: String,
        initializer: Option<Expression>,
    ) -> Result<(), String> {
        match initializer {
            Some(expression) => self.expression(expression)?,
            None => {
                self.emit(Operation::Null);
            }
        }
        self.define_variable(name)
    }

    fn print_statement(&mut self, expression: Expression) -> Result<(), String> {
        self.expression(expression)?;
        self.emit(Operation::Print);
        Ok(())
    }

    fn if_statement(
        &mut self,
        condition: Expression,
        then_branch: Vec<Statement>,
        else_branch: Option<Vec<Statement>>,
    ) -> Result<(), String> {
        self.expression(condition)?;
        let else_jump = self.emit_jump(Operation::JumpIfFalse);
        self.emit(Operation::Pop);
        self.begin_scope();
        for stmt in then_branch {
            self.statement(stmt)?;
        }
        self.end_scope();
        let end_jump = self.emit_jump(Operation::Jump);
        self.patch_jump(else_jump)?;
        self.emit(Operation::Pop);
        if let Some(else_branch) = else_branch {
            self.begin_scope();
            for stmt in else_branch {
                self.statement(stmt)?;
            }
            self.end_scope();
        }
        self.patch_jump(end_jump)?;
        Ok(())
    }

    fn while_statement(
        &mut self,
        condition: Expression,
        block: Vec<Statement>,
    ) -> Result<(), String> {
        let loop_start = self.chunk().code.len();
        self.expression(condition)?;
        let exit_jump = self.emit_jump(Operation::JumpIfFalse);
        self.emit(Operation::Pop);
        self.loops.push(Loop {
            start: loop_start,
            depth: self.scope_depth,
            break_jumps: Vec::new(),
        });
        self.begin_scope();
        for stmt in block {
            self.statement(stmt)?;
        }
        self.end_scope();
        self.emit_loop(loop_start)?;
        self.patch_jump(exit_jump)?;
        self.emit(Operation::Pop);
        let finished_loop = self.loops.pop().ok_or("Loop stack underflow")?;
        for jump in finished_loop.break_jumps {
            self.patch_jump(jump)?;
        }
        Ok(())
    }

    fn expression_statement(&mut self, expression: Expression) -> Result<(), String> {
        self.expression(expression)?;
        self.emit(Operation::Pop);
        Ok(())
    }

    fn return_statement(&mut self, value: Option<Expression>) -> Result<(), String> {
        if self.locals[0].depth == 0 {
            return Err("Return statement not allowed outside of a function".to_string());
        }
        if let Some(expression) = value {
            self.expression(expression)?;
        } else {
            self.emit(Operation::Null);
        }
        self.emit(Operation::Return);
        Ok(())
    }

    fn break_statement(&mut self) -> Result<(), String> {
        let depth = match self.loops.last() {
            Some(innermost) => innermost.depth,
            None => return Err("Break statement not allowed outside of a loop".to_string()),
        };
        self.discard_locals_above(depth);
        let jump = self.emit_jump(Operation::Jump);
        if let Some(innermost) = self.loops.last_mut() {
            innermost.break_jumps.push(jump);
        }
        Ok(())
    }

    fn continue_statement(&mut self) -> Result<(), String> {
        let (start, depth) = match self.loops.last() {
            Some(innermost) => (innermost.start, innermost.depth),
            None => return Err("Continue statement not allowed outside of a loop".to_string()),
        };
        self.discard_locals_above(depth);
        self.emit_loop(start)
    }

    fn discard_locals_above(&mut self, depth: usize) {
        let count = self
            .locals
            .iter()
            .rev()
            .take_while(|local| local.depth > depth)
            .count();
        for _ in 0..count {
            self.emit(Operation::Pop);
        }
    }

    fn for_statement(
        &mut self,
        variable: String,
        iterable: Expression,
        block: Vec<Statement>,
    ) -> Result<(), String> {
        self.expression(iterable)?;
        self.emit_constant(Value::Integer(0))?;
        self.begin_scope();
        self.define_variable("(iter)".to_string())?;
        self.define_variable("(index)".to_string())?;

        let start = self.chunk().code.len();
        let exit = self.emit_jump(Operation::ForIter);
        self.loops.push(Loop {
            start,
            depth: self.scope_depth,
            break_jumps: Vec::new(),
        });

        self.begin_scope();
        self.define_variable(variable)?;
        for stmt in block {
            self.statement(stmt)?;
        }
        self.end_scope();
        self.emit_loop(start)?;

        self.patch_jump(exit)?;
        let finished_loop = self.loops.pop().ok_or("Loop stack underflow")?;
        for jump in finished_loop.break_jumps {
            self.patch_jump(jump)?;
        }
        self.end_scope();
        Ok(())
    }

    // ---------- expressions ----------

    #[rustfmt::skip]
    fn expression(&mut self, expression: Expression) -> Result<(), String> {
        match expression {
            Expression::Literal { value } => {
                match value {
                    Token::Integer(int) => {
                        let idx = self.chunk().add_constant(Value::Integer(int))?;
                        self.emit(Operation::GetConstant(idx));
                    },
                    Token::Decimal(dec) => {
                        let idx = self.chunk().add_constant(Value::Decimal(dec))?;
                        self.emit(Operation::GetConstant(idx));
                    },
                    Token::StringLiteral(str) => {
                        let idx = self.chunk().add_constant(Value::String(str.into()))?;
                        self.emit(Operation::GetConstant(idx));
                    },
                    Token::True => {self.emit(Operation::True);},
                    Token::False => {self.emit(Operation::False);},
                    Token::Null => {self.emit(Operation::Null);},
                    _ => return Err("Expected literal".to_string()),
                }
            },
            Expression::Unary { operator, operand } => {
                self.expression(*operand)?;
                match operator {
                    Token::Minus => self.emit(Operation::Negate),
                    Token::Bang => self.emit(Operation::Not),
                    _ => return Err("Expected unary operator".to_string()),
                };
            },
            Expression::Binary { left, operator: Token::And, right } => {
                self.expression(*left)?;
                let end_jump = self.emit_jump(Operation::JumpIfFalse);
                self.emit(Operation::Pop);
                self.expression(*right)?;
                self.patch_jump(end_jump)?;
                self.emit(Operation::Not);
                self.emit(Operation::Not);
            },
            Expression::Binary { left, operator: Token::Or, right } => {
                self.expression(*left)?;
                let else_jump = self.emit_jump(Operation::JumpIfFalse);
                let end_jump = self.emit_jump(Operation::Jump);
                self.patch_jump(else_jump)?;
                self.emit(Operation::Pop);
                self.expression(*right)?;
                self.patch_jump(end_jump)?;
            },
            Expression::Binary { left, operator, right } => {
                self.expression(*left)?;
                self.expression(*right)?;
                match operator {
                    Token::Plus => self.emit(Operation::Add),
                    Token::Minus => self.emit(Operation::Subtract),
                    Token::Asterisk => self.emit(Operation::Multiply),
                    Token::Slash => self.emit(Operation::Divide),
                    Token::Percent => self.emit(Operation::Modulo),
                    Token::EqualEqual => self.emit(Operation::Equal),
                    Token::BangEqual => {
                        self.emit(Operation::Equal);
                        self.emit(Operation::Not)
                    },
                    Token::Greater => self.emit(Operation::Greater),
                    Token::GreaterEqual => self.emit(Operation::GreaterEqual),
                    Token::Less => self.emit(Operation::Less),
                    Token::LessEqual => self.emit(Operation::LessEqual),
                    _ => return Err("Expected binary operator".to_string()),
                };
            },
            Expression::Grouping { expression } => {
                self.expression(*expression)?;
            },
            Expression::Variable { name } => {
                if let Some(index) = self.resolve_local(name.clone()) {
                    self.emit(Operation::GetLocal(index));
                } else {
                    let index = self.identifier_constant(name.clone())?;
                    self.emit(Operation::GetGlobal(index));
                }
            },
            Expression::Assignment { name, value } => {
                self.expression(*value)?;
                if let Some(index) = self.resolve_local(name.clone()) {
                    self.emit(Operation::SetLocal(index));
                } else {
                    let index = self.identifier_constant(name.clone())?;
                    self.emit(Operation::SetGlobal(index));
                }
            },
            Expression::Call { function, arguments } => {
                self.expression(*function)?;
                let arg_count = arguments.len() as u16;
                for arg in arguments {
                    self.expression(arg)?;
                }
                self.emit(Operation::Call(arg_count));
            },
            Expression::ArrayLiteral { elements } => {
                let count = elements.len() as u16;
                for element in elements {
                    self.expression(element)?;
                }
                self.emit(Operation::BuildArray(count));
            },
            Expression::Index { collection, index } => {
                self.expression(*collection)?;
                self.expression(*index)?;
                self.emit(Operation::GetIndex);
            },
            Expression::Len { operand } => {
                self.expression(*operand)?;
                self.emit(Operation::Len);
            },
            Expression::IndexAssignment { collection, index, value } => {
                self.expression(*collection)?;
                self.expression(*index)?;
                self.expression(*value)?;
                self.emit(Operation::SetIndex);
            },
        }
        Ok(())
    }
}
