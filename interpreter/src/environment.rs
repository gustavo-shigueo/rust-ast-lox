use crate::{RuntimeError, Value};
use lox_core::{Error, Result};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

#[derive(Debug, Default)]
pub struct Environment {
    parent: Option<Rc<RefCell<Environment>>>,
    values: HashMap<Rc<str>, State>,
}

#[derive(Debug, Clone)]
enum State {
    Undefined,
    Unassigned,
    Assigned(Value),
}

impl Environment {
    #[must_use]
    pub fn new() -> Self {
        Self {
            parent: None,
            values: HashMap::new(),
        }
    }

    #[must_use]
    pub fn spawn_child(parent: &Rc<RefCell<Self>>) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            parent: Some(Rc::clone(parent)),
            values: HashMap::new(),
        }))
    }

    /// Creates a new variable in the environment or overrides its
    /// value if it already exists
    pub fn define(&mut self, name: &Rc<str>, value: Option<Value>) {
        self.values.insert(
            Rc::clone(name),
            value.map_or(State::Unassigned, State::Assigned),
        );
    }

    /// Overrides the value of an existing variable
    ///
    /// # Errors
    /// This function will error if no variable is found with the given `name`
    pub fn assign(
        &mut self,
        name: &Rc<str>,
        value: Value,
        line: usize,
        column: usize,
    ) -> Result<(), RuntimeError> {
        if self.values.contains_key(name) {
            self.values.insert(Rc::clone(name), State::Assigned(value));
            return Ok(());
        }

        if let Some(ref parent) = self.parent {
            return parent.borrow_mut().assign(name, value, line, column);
        }

        Err(Error {
            line,
            column,
            source: RuntimeError::UndefinedVariable(name.to_string()),
        })
    }

    /// Returns the value of an existing variable
    ///
    /// # Errors
    /// This function will error if no variable is found with the given `name`
    pub fn lookup(
        &self,
        name: &Rc<str>,
        line: usize,
        column: usize,
    ) -> Result<Value, RuntimeError> {
        let state = self.values.get(name).cloned().unwrap_or(State::Undefined);

        match state {
            State::Assigned(value) => Ok(value),
            State::Unassigned => Err(Error {
                line,
                column,
                source: RuntimeError::UnassignedVariable(name.to_string()),
            }),
            State::Undefined => self.parent.as_ref().map_or_else(
                || {
                    Err(Error {
                        line,
                        column,
                        source: RuntimeError::UndefinedVariable(name.to_string()),
                    })
                },
                |x| x.borrow().lookup(name, line, column),
            ),
        }
    }
}
