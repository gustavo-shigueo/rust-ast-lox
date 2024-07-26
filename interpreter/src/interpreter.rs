use std::{
    cell::RefCell,
    rc::Rc,
    time::{SystemTime, UNIX_EPOCH},
};

use lox_core::{report, Error, Result};
use parser::{
    BinaryOperator, BinaryOperatorKind, Expression, LogicalOperator, LogicalOperatorKind,
    Statement, UnaryOperatorKind,
};

use crate::{Callable, CallableKind, Environment, RuntimeError, Value};

#[derive(Debug, Default)]
pub struct Interpreter {
    pub environment: Rc<RefCell<Environment>>,
}

impl Interpreter {
    #[must_use]
    pub fn new() -> Self {
        let mut environment = Environment::new();

        environment.define(
            &"clock".into(),
            Some(Value::Callable(Callable {
                arity: 0,
                kind: CallableKind::NativeFunction(Rc::new(|_| {
                    let now = SystemTime::now();
                    let elapsed = now.duration_since(UNIX_EPOCH).unwrap_or_default();

                    Value::Number(1_000.0 * elapsed.as_secs_f64())
                })),
            })),
        );

        environment.define(
            &"print".into(),
            Some(Value::Callable(Callable {
                arity: 1,
                kind: CallableKind::NativeFunction(Rc::new(|args| {
                    println!("{}", args[0]);
                    Value::Nil
                })),
            })),
        );

        environment.define(
            &"readLine".into(),
            Some(Value::Callable(Callable {
                arity: 0,
                kind: CallableKind::NativeFunction(Rc::new(|_| {
                    let stdin = std::io::stdin();
                    let mut buffer = String::new();
                    _ = stdin.read_line(&mut buffer);

                    Value::String(buffer.trim_end_matches(&['\r', '\n']).into())
                })),
            })),
        );

        let environment = Rc::new(RefCell::new(environment));

        Self { environment }
    }

    pub fn interpret(&mut self, source: &str, program: &[Statement]) {
        for statement in program {
            if let Err(error) = self.execute(statement) {
                report(source, &error);
                break;
            }
        }
    }

    fn execute(&mut self, statement: &Statement) -> Result<(), RuntimeError> {
        match statement {
            Statement::Expression(expression) => {
                self.evaluate(expression)?;
            }
            Statement::Declaration {
                ref identifier,
                initializer,
                ..
            } => {
                let value = initializer.as_ref().map(|x| self.evaluate(x)).transpose()?;
                self.environment.borrow_mut().define(identifier, value);
            }
            Statement::Block(statements) => self.execute_block(statements)?,
            Statement::If {
                condition,
                then_branch,
                else_branch,
            } => {
                if self.evaluate(condition)?.is_truthy() {
                    self.execute(then_branch)?;
                } else if let Some(else_branch) = else_branch {
                    self.execute(else_branch)?;
                }
            }
            Statement::While {
                condition,
                body: statement,
            } => {
                while self.evaluate(condition)?.is_truthy() {
                    match self.execute(statement) {
                        Ok(()) => (),
                        Err(Error {
                            source: RuntimeError::UnexpectedBreakStatement,
                            ..
                        }) => break,
                        Err(Error {
                            source: RuntimeError::UnexpectedContinueStatement,
                            ..
                        }) => continue,
                        Err(e) => return Err(e),
                    }
                }
            }
            Statement::Break { line, column } => {
                return Err(Error {
                    line: *line,
                    column: *column,
                    source: RuntimeError::UnexpectedBreakStatement,
                })
            }
            Statement::Continue { line, column } => {
                return Err(Error {
                    line: *line,
                    column: *column,
                    source: RuntimeError::UnexpectedContinueStatement,
                })
            }
            Statement::Function {
                identifier,
                parameters,
                body,
            } => {
                self.environment.borrow_mut().define(
                    identifier,
                    Some(Value::Callable(Callable {
                        arity: parameters.len(),
                        kind: CallableKind::LoxFunction {
                            identifier: Some(Rc::clone(identifier)),
                            parameters: Rc::clone(parameters),
                            body: Rc::clone(body),
                            closure: Rc::clone(&self.environment),
                        },
                    })),
                );
            }
            Statement::Return {
                line,
                column,
                expression,
            } => {
                return Err(Error {
                    line: *line,
                    column: *column,
                    source: RuntimeError::UnexpectedReturnStatement(
                        expression
                            .as_ref()
                            .map_or(Ok(Value::Nil), |x| self.evaluate(x))?,
                    ),
                })
            }
        };

        Ok(())
    }

    fn execute_block(&mut self, statements: &[Statement]) -> Result<(), RuntimeError> {
        let current = Rc::clone(&self.environment);

        self.environment = Environment::spawn_child(&current);
        for statement in statements {
            if let Err(error) = self.execute(statement) {
                self.environment = current;
                return Err(error);
            }
        }
        self.environment = current;

        Ok(())
    }

    fn evaluate(&mut self, expression: &Expression) -> Result<Value, RuntimeError> {
        Ok(match expression {
            Expression::Ternary {
                condition,
                truthy,
                falsey,
            } => self.evaluate_ternary_expression(condition, truthy, falsey)?,
            Expression::Binary {
                left,
                right,
                operator,
            } => self.evaluate_binary_expression(left, operator, right)?,
            Expression::Logical {
                left,
                right,
                operator,
            } => self.evaluate_logical_expression(left, operator, right)?,
            Expression::Unary {
                expression,
                operator,
            } => {
                let value = self.evaluate(expression)?;

                match operator.kind {
                    UnaryOperatorKind::Minus => match value {
                        Value::Number(number) => Value::Number(-number),
                        x => {
                            return Err(Error {
                                line: operator.line,
                                column: operator.column,
                                source: RuntimeError::TypeError {
                                    expected: "number",
                                    found: x.type_name(),
                                },
                            })
                        }
                    },
                    UnaryOperatorKind::Bang => Value::Boolean(!value.is_truthy()),
                }
            }
            Expression::GroupingExpression(expression) => self.evaluate(expression)?,
            Expression::Literal(literal) => literal.clone().into(),
            Expression::Variable {
                line,
                column,
                ref identifier,
            } => self
                .environment
                .borrow()
                .lookup(identifier, *line, *column)?,
            Expression::Assignment {
                line,
                column,
                identifier,
                value,
            } => {
                let value = self.evaluate(value)?;
                self.environment
                    .borrow_mut()
                    .assign(identifier, value.clone(), *line, *column)?;

                value
            }
            Expression::Call {
                callee,
                args,
                line,
                column,
            } => {
                let line = *line;
                let column = *column;

                let callee = self.evaluate(callee)?;
                let mut arg_values = vec![];

                for arg in args.iter() {
                    arg_values.push(self.evaluate(arg)?);
                }

                match callee {
                    Value::Callable(function) if args.len() == function.arity => {
                        self.call(function, &arg_values)?
                    }
                    Value::Callable(Callable { arity, .. }) => {
                        return Err(Error {
                            line,
                            column,
                            source: RuntimeError::ImcorrectNumberOfArguments {
                                expected: arity,
                                found: args.len(),
                            },
                        })
                    }
                    x => {
                        return Err(Error {
                            line,
                            column,
                            source: RuntimeError::TypeIsNotCallable(x.type_name()),
                        })
                    }
                }
            }
            Expression::AnonymousFunction { parameters, body } => Value::Callable(Callable {
                arity: parameters.len(),
                kind: CallableKind::LoxFunction {
                    identifier: None,
                    parameters: Rc::clone(parameters),
                    body: Rc::clone(body),
                    closure: Rc::clone(&self.environment),
                },
            }),
        })
    }

    fn evaluate_ternary_expression(
        &mut self,
        condition: &Expression,
        truthy: &Expression,
        falsey: &Expression,
    ) -> Result<Value, RuntimeError> {
        Ok(if self.evaluate(condition)?.is_truthy() {
            self.evaluate(truthy)?
        } else {
            self.evaluate(falsey)?
        })
    }

    fn evaluate_binary_expression(
        &mut self,
        left: &Expression,
        operator: &BinaryOperator,
        right: &Expression,
    ) -> Result<Value, RuntimeError> {
        let left = self.evaluate(left)?;
        let right = self.evaluate(right)?;

        Ok(match operator.kind {
            BinaryOperatorKind::Comma => right,
            BinaryOperatorKind::BangEqual => Value::Boolean(left != right),
            BinaryOperatorKind::DoubleEquals => Value::Boolean(left == right),
            BinaryOperatorKind::GreaterThan
            | BinaryOperatorKind::GreaterEqual
            | BinaryOperatorKind::LessThan
            | BinaryOperatorKind::LessEqual => Self::evaluate_comparison(left, operator, right)?,
            BinaryOperatorKind::Plus => Self::evaluate_plus_operation(left, operator, right)?,
            BinaryOperatorKind::Minus => match (left, right) {
                (Value::Number(a), Value::Number(b)) => Value::Number(a - b),
                (Value::Number(_), x) | (x, _) => {
                    return Err(Error {
                        line: operator.line,
                        column: operator.column,
                        source: RuntimeError::TypeError {
                            expected: "number",
                            found: x.type_name(),
                        },
                    })
                }
            },
            BinaryOperatorKind::Star => match (left, right) {
                (Value::Number(a), Value::Number(b)) => Value::Number(a * b),
                (Value::Number(_), x) | (x, _) => {
                    return Err(Error {
                        line: operator.line,
                        column: operator.column,
                        source: RuntimeError::TypeError {
                            expected: "number",
                            found: x.type_name(),
                        },
                    })
                }
            },
            BinaryOperatorKind::Slash => match (left, right) {
                (Value::Number(a), Value::Number(b)) => Value::Number(a / b),
                (Value::Number(_), x) | (x, _) => {
                    return Err(Error {
                        line: operator.line,
                        column: operator.column,
                        source: RuntimeError::TypeError {
                            expected: "number",
                            found: x.type_name(),
                        },
                    })
                }
            },
        })
    }

    fn evaluate_comparison(
        left: Value,
        operator: &BinaryOperator,
        right: Value,
    ) -> Result<Value, RuntimeError> {
        use Value as L;

        Ok(L::Boolean(match (left, right) {
            (L::String(a), L::String(b)) => match operator.kind {
                BinaryOperatorKind::LessThan => a < b,
                BinaryOperatorKind::LessEqual => a <= b,
                BinaryOperatorKind::GreaterThan => a > b,
                BinaryOperatorKind::GreaterEqual => a >= b,
                _ => unreachable!(),
            },
            (L::Number(a), L::Number(b)) => match operator.kind {
                BinaryOperatorKind::LessThan => a < b,
                BinaryOperatorKind::LessEqual => a <= b,
                BinaryOperatorKind::GreaterThan => a > b,
                BinaryOperatorKind::GreaterEqual => a >= b,
                _ => unreachable!(),
            },
            (L::Boolean(a), L::Boolean(b)) => match operator.kind {
                BinaryOperatorKind::LessThan => !a && b,
                BinaryOperatorKind::LessEqual => a <= b,
                BinaryOperatorKind::GreaterThan => a && !b,
                BinaryOperatorKind::GreaterEqual => a >= b,
                _ => unreachable!(),
            },
            (L::Nil, L::Nil) => match operator.kind {
                BinaryOperatorKind::LessThan | BinaryOperatorKind::GreaterThan => true,
                BinaryOperatorKind::LessEqual | BinaryOperatorKind::GreaterEqual => false,
                _ => unreachable!(),
            },
            (a, b) => {
                return Err(Error {
                    line: operator.line,
                    column: operator.column,
                    source: RuntimeError::TypeError {
                        expected: a.type_name(),
                        found: b.type_name(),
                    },
                })
            }
        }))
    }

    fn evaluate_plus_operation(
        left: Value,
        operator: &BinaryOperator,
        right: Value,
    ) -> Result<Value, RuntimeError> {
        Ok(match (left, right) {
            (Value::Number(a), Value::Number(b)) => Value::Number(a + b),
            (a @ Value::String(_), b) | (a, b @ Value::String(_)) => {
                Self::concatenate_strings(&a, &b)
            }
            (Value::Number(_), x) => {
                return Err(Error {
                    line: operator.line,
                    column: operator.column,
                    source: RuntimeError::TypeError {
                        expected: "number",
                        found: x.type_name(),
                    },
                })
            }
            (x, _) => {
                return Err(Error {
                    line: operator.line,
                    column: operator.column,
                    source: RuntimeError::TypeError {
                        // The error will read
                        // Expected expression of type "number" or
                        // "string" found "type"
                        expected: r#"number" or "string"#,
                        found: x.type_name(),
                    },
                });
            }
        })
    }

    fn concatenate_strings(left: &Value, right: &Value) -> Value {
        let a = match left {
            Value::Number(value) => &value.to_string(),
            Value::Boolean(true) => "true",
            Value::Boolean(false) => "false",
            Value::Nil => "nil",
            Value::String(ref x) => x.as_ref(),
            Value::Callable(Callable { kind, .. }) => &kind.to_string(),
        };

        let b = match right {
            Value::Number(value) => &value.to_string(),
            Value::Boolean(true) => "true",
            Value::Boolean(false) => "false",
            Value::Nil => "nil",
            Value::String(ref x) => x.as_ref(),
            Value::Callable(Callable { kind, .. }) => &kind.to_string(),
        };

        let mut string = String::with_capacity(a.len() + b.len());
        string.push_str(a);
        string.push_str(b);

        Value::String(string.into())
    }

    fn evaluate_logical_expression(
        &mut self,
        left: &Expression,
        operator: &LogicalOperator,
        right: &Expression,
    ) -> Result<Value, RuntimeError> {
        let left = self.evaluate(left)?;

        Ok(match operator.kind {
            LogicalOperatorKind::And => {
                if left.is_truthy() {
                    self.evaluate(right)?
                } else {
                    left
                }
            }
            LogicalOperatorKind::Or => {
                if left.is_truthy() {
                    left
                } else {
                    self.evaluate(right)?
                }
            }
        })
    }

    fn call(&mut self, function: Callable, args: &[Value]) -> Result<Value, RuntimeError> {
        Ok(match function.kind {
            CallableKind::NativeFunction(function) => function(args),
            CallableKind::LoxFunction {
                parameters,
                body,
                closure,
                ..
            } => {
                let current = Rc::clone(&self.environment);

                self.environment = Environment::spawn_child(&closure);

                for (param, arg) in parameters.iter().zip(args) {
                    self.environment
                        .borrow_mut()
                        .define(param, Some(arg.clone()));
                }

                for statement in body.iter() {
                    match self.execute(statement) {
                        Ok(()) => (),
                        Err(error) => {
                            self.environment = current;

                            match error.source {
                                RuntimeError::UnexpectedReturnStatement(value) => return Ok(value),
                                _ => return Err(error),
                            }
                        }
                    }
                }

                self.environment = current;

                Value::Nil
            }
        })
    }
}
