use crate::{
    bytecode::{Function, Operation, Value}, expression::Expression, statement::Statement, token::Token,
};

mod chunk;

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

/// Estado de compilación de UNA función. Cada declaración de función
/// crea un Compiler nuevo con su propio chunk y sus propias locales.
struct Compiler {
    function: Function,
    locals: Vec<Local>,
    scope_depth: usize,
}

impl Compiler {
    fn new(name: String, scope_depth: usize) -> Self {
        Compiler {
            function: Function {
                name,
                arity: 0,
                chunk: Chunk::new(),
            },
            locals: Vec::new(),
            scope_depth,
        }
    }

    fn finish(self) -> Result<Function, String> {
        if self.scope_depth != 0 {
            return Err("Unclosed scope at end of function.".to_string());
        }
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
        self.emit(Operation::Jump(0xffff))
    }

    fn patch_jump(&mut self, index: usize) -> Result<(), String> {
        let jump_offset = self.chunk().code.len() - index - 1;
        self.chunk().patch(index, Operation::Jump(jump_offset as u16));
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
        self.chunk().add_constant(Value::String(name.to_string().into()))
    }

    // ---------- statements ----------
    // Regla: un statement deja la pila como la encontró.

    #[rustfmt::skip]
    fn statement(&mut self, stmt: Statement) -> Result<(), String> {
        match stmt {
            Statement::FunctionDeclaration { name, parameters, body } => {
                self.function_declaration(name, parameters, body)
            }
            Statement::VariableDeclaration { name, initializer } => {
                self.variable_declaration(name, initializer.as_ref())
            }
            Statement::PrintStatement(expression) => {
                self.print_statement(expression)
            }
            Statement::IfStatement { condition, then_branch, else_branch } => {
                self.if_statement(condition, then_branch, else_branch.as_deref())
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
                self.return_statement(value.as_ref())
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
        parameters: &[String],
        body: Vec<Statement>,
    ) -> Result<(), String> {
        let name_index = self.chunk().add_constant(Value::String(name.to_string().into()))?;
        let mut compiler = Compiler::new(name.to_string(), self.scope_depth + 1);
        for statement in body {
            compiler.statement(statement)?;
        }
        let function = compiler.finish()?;
        let function_index = self.chunk().add_constant(Value::Function(function.into()))?;

        self.emit(Operation::GetConstant(function_index));
        //

        Ok(())
    }

    fn variable_declaration(
        &mut self,
        name: String,
        initializer: Option<Expression>,
    ) -> Result<(), String> {
        todo!()
    }

    fn print_statement(&mut self, expression: Expression) -> Result<(), String> {
        todo!()
    }

    fn if_statement(
        &mut self,
        condition: Expression,
        then_branch: Vec<Statement>,
        else_branch: Option<Vec<Statement>>,
    ) -> Result<(), String> {
        todo!()
    }

    fn while_statement(&mut self, condition: Expression, block: Vec<Statement>) -> Result<(), String> {
        todo!()
    }

    fn for_statement(
        &mut self,
        variable: String,
        iterable: Expression,
        block: Vec<Statement>,
    ) -> Result<(), String> {
        todo!()
    }

    /// Compila la expresión y descarta su valor con Pop.
    fn expression_statement(&mut self, expression: Expression) -> Result<(), String> {
        todo!()
    }

    fn return_statement(&mut self, value: Option<Expression>) -> Result<(), String> {
        todo!()
    }

    fn break_statement(&mut self) -> Result<(), String> {
        todo!()
    }

    fn continue_statement(&mut self) -> Result<(), String> {
        todo!()
    }

    // ---------- expressions ----------
    // Regla: una expresión deja exactamente un valor en la pila.

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
            Expression::Unary { operator, operand } => todo!(),
            Expression::Binary { left, operator, right } => todo!(),
            Expression::Grouping { expression } => todo!(),
            Expression::Variable { name } => todo!(),
            Expression::ArrayLiteral { elements } => todo!(),
            Expression::Index { collection, index } => todo!(),
            Expression::Len { operand } => todo!(),
            Expression::Assignment { name, value } => todo!(),
            Expression::IndexAssignment { collection, index, value } => todo!(),
            Expression::Call { function, arguments } => todo!(),
        }
        Ok(())
    }
}
