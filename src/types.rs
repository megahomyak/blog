use std::{future::Future, pin::Pin};

pub type TraitFuture<'future, ReturnType> = Pin<Box<dyn Future<Output = ReturnType> + 'future + Send>>;
