use std::{cell::RefCell, collections::HashMap, rc::Rc};

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
        is_initializer: bool,
    },
    LoxClass(LoxClass),
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
            Self::LoxClass(LoxClass { identifier, .. }) => write!(f, "<class {identifier}>"),
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

#[derive(Clone)]
pub struct LoxClass {
    pub identifier: Rc<str>,
    pub methods: HashMap<Rc<str>, Callable>,
    pub super_class: Option<Rc<LoxClass>>,
}

impl LoxClass {
    #[must_use]
    pub fn find_method(&self, identifier: &Rc<str>) -> Option<Callable> {
        if let Some(method) = self.methods.get(identifier) {
            return Some(method.clone());
        }

        if let Some(ref super_class) = self.super_class {
            return super_class.find_method(identifier);
        }

        None
    }
}
