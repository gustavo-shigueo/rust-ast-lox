use crate::ResolverError;

pub struct Resolver {
    _x: std::marker::PhantomData<()>,
}

impl Resolver {
    #[must_use]
    pub fn new() -> Self {
        Self {
            _x: std::marker::PhantomData,
        }
    }

    pub fn resolve() -> Result<(), ResolverError> {
        Ok(())
    }
}
