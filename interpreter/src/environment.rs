use crate::{RuntimeError, Value};
use lox_core::{Error, Result};
use parser::Reference;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

#[derive(Debug, Default)]
pub struct Environment {
    parent: Option<Rc<RefCell<Environment>>>,
    values: HashMap<Rc<str>, State>,
}

#[derive(Debug, Clone)]
enum State {
    Undeclared,
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
    pub fn assign(&mut self, reference: &Reference, value: Value) -> Result<(), RuntimeError> {
        if self.values.contains_key(&reference.identifier) {
            self.values
                .insert(Rc::clone(&reference.identifier), State::Assigned(value));
            return Ok(());
        }

        Err(Error {
            line: reference.line,
            column: reference.column,
            source: RuntimeError::UndeclaredVariable(Rc::clone(&reference.identifier)),
        })
    }

    /// Returns the value of an existing variable
    ///
    /// # Errors
    /// This function will error if no variable is found with the given `name`
    pub fn lookup(&self, reference: &Reference) -> Result<Value, RuntimeError> {
        let state = self
            .values
            .get(&reference.identifier)
            .cloned()
            .unwrap_or(State::Undeclared);

        match state {
            State::Assigned(value) => Ok(value),
            State::Unassigned => Err(Error {
                line: reference.line,
                column: reference.column,
                source: RuntimeError::UnassignedVariable(Rc::clone(&reference.identifier)),
            }),
            State::Undeclared => Err(Error {
                line: reference.line,
                column: reference.column,
                source: RuntimeError::UndeclaredVariable(Rc::clone(&reference.identifier)),
            }),
        }
    }

    /// Returns the value of an existing variable at a specific enclosing scope
    ///
    /// # Errors
    /// This function will error if no variable is found with the given `name`
    pub fn lookup_at(&self, distance: usize, reference: &Reference) -> Result<Value, RuntimeError> {
        let state = match distance {
            0 => self.values[&reference.identifier].clone(),
            _ => self.ancestor(distance).borrow().values[&reference.identifier].clone(),
        };

        match state {
            State::Assigned(value) => Ok(value),
            State::Unassigned => Err(Error {
                line: reference.line,
                column: reference.column,
                source: RuntimeError::UnassignedVariable(Rc::clone(&reference.identifier)),
            }),
            State::Undeclared => unreachable!(),
        }
    }

    /// Overrides the value of an existing variable at a specific enclosing scope
    ///
    /// # Errors
    /// This function will error if no variable is found with the given `name`
    pub fn assign_at(
        &mut self,
        distance: usize,
        reference: &Reference,
        value: Value,
    ) -> Result<(), RuntimeError> {
        if distance == 0 {
            let values = &mut self.values;

            if values.contains_key(&reference.identifier) {
                values.insert(Rc::clone(&reference.identifier), State::Assigned(value));

                Ok(())
            } else {
                Err(Error {
                    line: reference.line,
                    column: reference.column,
                    source: RuntimeError::UndeclaredVariable(Rc::clone(&reference.identifier)),
                })
            }
        } else {
            let ancestor = self.ancestor(distance);

            let values = &mut ancestor.borrow_mut().values;
            if values.contains_key(&reference.identifier) {
                values.insert(Rc::clone(&reference.identifier), State::Assigned(value));

                Ok(())
            } else {
                Err(Error {
                    line: reference.line,
                    column: reference.column,
                    source: RuntimeError::UndeclaredVariable(Rc::clone(&reference.identifier)),
                })
            }
        }
    }

    fn ancestor(&self, distance: usize) -> Rc<RefCell<Self>> {
        assert_ne!(distance, 0);
        let mut current = self.parent.clone();

        for _ in 1..distance {
            current = current.map_or_else(
                // This method will only be called with guaranteed certainty
                // that a valid environment will be found
                || unreachable!(),
                |curr| curr.borrow().parent.clone(),
            );
        }

        current.unwrap()
    }
}
