use std::{collections::HashMap, rc::Rc};

use lox_core::{report, Error, Result};
use parser::{Expression, Function, Reference, Statement};

use crate::ResolverError;

#[derive(Debug)]
pub struct Resolver<'a> {
    pub source: &'a str,
    pub scopes: Vec<HashMap<Rc<str>, bool>>,
    pub locals: HashMap<Reference, usize>,
    pub had_error: bool,
    pub is_in_loop: bool,
    pub function_kind: FunctionKind,
    pub class_kind: ClassKind,
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum FunctionKind {
    None,
    Function,
    Initializer,
    Method,
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum ClassKind {
    None,
    Class,
    Subclass,
}

impl<'a> Resolver<'a> {
    #[must_use]
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            scopes: Vec::new(),
            locals: HashMap::new(),
            had_error: false,
            is_in_loop: false,
            function_kind: FunctionKind::None,
            class_kind: ClassKind::None,
        }
    }

    pub fn resolve(&mut self, statements: &[Statement]) {
        for statement in statements {
            match self.resolve_statement(statement) {
                Ok(()) => (),
                Err(error) => {
                    report(self.source, &error);
                    self.had_error = true
                }
            }
        }
    }

    fn resolve_statement(&mut self, statement: &Statement) -> Result<(), ResolverError> {
        match statement {
            Statement::Expression(expression) => self.resolve_expression(expression)?,
            Statement::Declaration {
                identifier,
                initializer,
                line,
                column,
            } => {
                self.declare(identifier, *line, *column)?;

                if let Some(initializer) = initializer {
                    self.resolve_expression(initializer)?;
                }

                self.define(identifier);
            }
            Statement::Block(statements) => {
                self.begin_scope();
                for statement in statements.iter() {
                    self.resolve_statement(statement)?;
                }
                self.end_scope();
            }
            Statement::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.resolve_expression(condition)?;
                self.resolve_statement(then_branch)?;

                if let Some(else_branch) = else_branch {
                    self.resolve_statement(else_branch)?;
                }
            }
            Statement::For {
                condition,
                body,
                increment,
            } => {
                let is_in_loop = self.is_in_loop;
                self.is_in_loop = true;

                self.resolve_expression(condition)?;
                self.resolve_statement(body)?;

                if let Some(ref increment) = increment {
                    self.resolve_expression(increment)?;
                }

                self.is_in_loop = is_in_loop;
            }
            Statement::While { condition, body } => {
                let is_in_loop = self.is_in_loop;
                self.is_in_loop = true;

                self.resolve_expression(condition)?;
                self.resolve_statement(body)?;

                self.is_in_loop = is_in_loop;
            }
            Statement::Break { line, column } => {
                if !self.is_in_loop {
                    return Err(Error {
                        line: *line,
                        column: *column,
                        source: ResolverError::UnexpectedBreakStatement,
                    });
                }
            }
            Statement::Continue { line, column } => {
                if !self.is_in_loop {
                    return Err(Error {
                        line: *line,
                        column: *column,
                        source: ResolverError::UnexpectedContinueStatement,
                    });
                }
            }
            Statement::Function(Function {
                identifier,
                parameters,
                body,
                line,
                column,
            }) => {
                self.declare(identifier, *line, *column)?;
                self.define(identifier);
                self.resolve_function(parameters, body, FunctionKind::Function)?;
            }
            Statement::Return {
                expression,
                line,
                column,
            } => {
                let is_in_function = self.function_kind != FunctionKind::None;

                if !is_in_function {
                    return Err(Error {
                        line: *line,
                        column: *column,
                        source: ResolverError::UnexpectedReturnStatement,
                    });
                }

                if let Some(expression) = expression {
                    if self.function_kind == FunctionKind::Initializer {
                        return Err(Error {
                            line: *line,
                            column: *column,
                            source: ResolverError::CannotReturnFromInitializer,
                        });
                    }

                    self.resolve_expression(expression)?
                }
            }
            Statement::Class {
                line,
                column,
                identifier,
                super_class,
                methods,
            } => {
                let class_kind = self.class_kind;

                self.class_kind = ClassKind::Class;
                self.declare(identifier, *line, *column)?;
                self.define(identifier);

                if let Some(super_class) = super_class {
                    self.class_kind = ClassKind::Subclass;
                    let Expression::Variable(reference) = super_class else {
                        unreachable!()
                    };

                    if reference.identifier.as_ref() == identifier.as_ref() {
                        return Err(Error {
                            line: reference.line,
                            column: reference.column,
                            source: ResolverError::ClassCannotInheritFromItself,
                        });
                    }

                    self.begin_scope();
                    self.declare(&"super".into(), *line, *column)?;
                    self.define(&"super".into());
                    self.resolve_expression(super_class)?;
                }

                self.begin_scope();

                self.declare(&"this".into(), *line, *column)?;
                self.define(&"this".into());

                for method in methods.iter() {
                    let method_type = if method.identifier.as_ref() == "init" {
                        FunctionKind::Initializer
                    } else {
                        FunctionKind::Method
                    };

                    self.resolve_function(&method.parameters, &method.body, method_type)?
                }

                if super_class.is_some() {
                    self.end_scope();
                }

                self.end_scope();
                self.class_kind = class_kind;
            }
        }

        Ok(())
    }

    fn resolve_expression(&mut self, expression: &Expression) -> Result<(), ResolverError> {
        match expression {
            Expression::Ternary {
                condition,
                truthy,
                falsey,
            } => {
                self.resolve_expression(condition)?;
                self.resolve_expression(truthy)?;
                self.resolve_expression(falsey)?;
            }
            Expression::Logical { left, right, .. } | Expression::Binary { left, right, .. } => {
                self.resolve_expression(left)?;
                self.resolve_expression(right)?;
            }
            Expression::GroupingExpression(expression) | Expression::Unary { expression, .. } => {
                self.resolve_expression(expression)?
            }
            Expression::Literal(_) => (),
            Expression::Variable(reference) => {
                if let Some(false) = self
                    .scopes
                    .last()
                    .and_then(|x| x.get(&reference.identifier))
                {
                    return Err(Error {
                        line: reference.line,
                        column: reference.column,
                        source: ResolverError::AttemptedToAccessVariableInItsOwnInitializer,
                    });
                }

                self.resolve_local(reference);
            }
            Expression::Assignment { reference, value } => {
                self.resolve_expression(value)?;
                self.resolve_local(reference);
            }
            Expression::AnonymousFunction { body, parameters } => {
                self.resolve_function(parameters, body, FunctionKind::Function)?;
            }
            Expression::Call { callee, args, .. } => {
                self.resolve_expression(callee)?;

                for arg in args.iter() {
                    self.resolve_expression(arg)?;
                }
            }
            Expression::Get { object, .. } => self.resolve_expression(object)?,
            Expression::Set { object, value, .. } => {
                self.resolve_expression(object)?;
                self.resolve_expression(value)?;
            }
            Expression::This { line, column } => {
                if self.class_kind == ClassKind::None {
                    return Err(Error {
                        line: *line,
                        column: *column,
                        source: ResolverError::UnexpectedThisKeyword,
                    });
                }

                let reference = Reference {
                    line: *line,
                    column: *column,
                    identifier: "this".into(),
                };
                self.resolve_local(&reference)
            }
            Expression::Super { line, column, .. } => {
                if self.class_kind != ClassKind::Subclass {
                    return Err(Error {
                        line: *line,
                        column: *column,
                        source: ResolverError::UnexpectedSuperKeyword,
                    });
                }

                let reference = Reference {
                    line: *line,
                    column: *column,
                    identifier: "super".into(),
                };
                self.resolve_local(&reference)
            }
        }

        Ok(())
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare(
        &mut self,
        identifier: &Rc<str>,
        line: usize,
        column: usize,
    ) -> Result<(), ResolverError> {
        if let Some(scope) = self.scopes.last_mut() {
            if scope.contains_key(identifier) {
                return Err(Error {
                    line,
                    column,
                    source: ResolverError::AttemptedToRedeclareVariable(Rc::clone(identifier)),
                });
            }

            scope.insert(Rc::clone(identifier), false);
        }

        Ok(())
    }

    fn define(&mut self, identifier: &Rc<str>) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(Rc::clone(identifier), true);
        }
    }

    fn resolve_local(&mut self, reference: &Reference) {
        for i in (0..self.scopes.len()).rev() {
            if self.scopes[i].contains_key(&reference.identifier) {
                self.locals
                    .insert(reference.clone(), self.scopes.len() - 1 - i);
            }
        }
    }

    fn resolve_function(
        &mut self,
        parameters: &[Rc<str>],
        body: &[Statement],
        function_kind: FunctionKind,
    ) -> Result<(), ResolverError> {
        let prev_function_kind = self.function_kind;
        let is_in_loop = self.is_in_loop;
        self.is_in_loop = false;
        self.function_kind = function_kind;
        self.begin_scope();

        for parameter in parameters {
            // Paramenters are imune to declaration errors
            self.declare(parameter, 0, 0)?;
            self.define(parameter);
        }

        for statement in body {
            self.resolve_statement(statement)?;
        }

        self.end_scope();
        self.function_kind = prev_function_kind;
        self.is_in_loop = is_in_loop;

        Ok(())
    }
}
