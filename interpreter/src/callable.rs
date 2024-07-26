use std::{cell::RefCell, rc::Rc};

use crate::{Environment, Value};
use parser::Statement;

#[derive(Debug, Clone)]
pub struct Callable {
    pub arity: usize,
    pub kind: CallableKind,
}

impl PartialEq for Callable {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
    }
}

impl std::fmt::Display for Callable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.kind)
    }
}

pub type NativeFunction = Rc<dyn Fn(&[Value]) -> Value>;

#[derive(Clone)]
pub enum CallableKind {
    NativeFunction(NativeFunction),
    LoxFunction {
        identifier: Option<Rc<str>>,
        parameters: Rc<[Rc<str>]>,
        body: Rc<[Statement]>,
        closure: Rc<RefCell<Environment>>,
    },
}

impl std::fmt::Debug for CallableKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NativeFunction(_) => write!(f, "<native fn>"),
            Self::LoxFunction {
                identifier: Some(identifier),
                ..
            } => write!(f, "<fn {identifier}>"),
            Self::LoxFunction {
                identifier: None, ..
            } => write!(f, "<anonymous fn>"),
        }
    }
}

impl std::fmt::Display for CallableKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as std::fmt::Debug>::fmt(self, f)
    }
}

impl PartialEq for CallableKind {
    #[allow(ambiguous_wide_pointer_comparisons)]
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::NativeFunction(a), Self::NativeFunction(b)) => {
                let a = a.as_ref() as *const _;
                let b = b.as_ref() as *const _;

                a == b
            }
            (Self::LoxFunction { body: a, .. }, Self::LoxFunction { body: b, .. }) => {
                let a = a.as_ref() as *const _;
                let b = b.as_ref() as *const _;

                a == b
            }
            _ => false,
        }
    }
}
