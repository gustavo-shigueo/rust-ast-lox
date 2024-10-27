use lox_core::{Error, Result};
use std::rc::Rc;
use std::{cell::RefCell, collections::HashMap};

use crate::{Callable, CallableKind, Environment, LoxClass, RuntimeError, Value};

pub struct LoxInstance {
    pub class: LoxClass,
    pub fields: HashMap<Rc<str>, Value>,
}

impl LoxInstance {
    /// # Errors
    ///
    /// This function errors if the property doesn't exist
    pub fn get(
        instance: &Rc<RefCell<Self>>,
        identifier: &Rc<str>,
        line: usize,
        column: usize,
    ) -> Result<Value, RuntimeError> {
        if let Some(value) = instance.borrow().fields.get(identifier) {
            return Ok(value.clone());
        }

        if let Some(method) = instance.borrow().class.find_method(identifier) {
            let bound_method = match method.kind {
                CallableKind::LoxFunction {
                    ref parameters,
                    ref body,
                    ref closure,
                    ref identifier,
                    is_initializer,
                } => CallableKind::LoxFunction {
                    identifier: identifier.clone(),
                    parameters: Rc::clone(parameters),
                    body: Rc::clone(body),
                    closure: {
                        let env = Environment::spawn_child(closure);
                        env.borrow_mut()
                            .define(&"this".into(), Some(Value::Instance(Rc::clone(instance))));
                        env
                    },
                    is_initializer,
                },
                _ => unreachable!(),
            };

            return Ok(Value::Callable(Callable {
                arity: method.arity,
                kind: bound_method,
            }));
        }

        Err(Error {
            line,
            column,
            source: RuntimeError::UndefinedProperty(Rc::clone(identifier)),
        })
    }

    pub fn set(&mut self, identifier: &Rc<str>, value: Value) {
        self.fields.insert(Rc::clone(identifier), value);
    }
}

impl std::fmt::Debug for LoxInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<{} instance>", self.class.identifier)
    }
}

impl std::fmt::Display for LoxInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}
