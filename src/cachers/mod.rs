use std::{marker::PhantomData, sync::{Arc, Mutex}};

use crate::types::TraitFuture;

pub mod redis_cacher;

trait CacheMakerFunction<Input, Output>: Sync + Send + Fn(Input) -> Output {}
impl<Input, Output, T: Sync + Send + Fn(Input) -> Output> CacheMakerFunction<Input, Output> for T {}

pub struct CacheMakerGuard<Input, Output, CacheMaker: CacheMakerFunction<Input, Output>> {
    cache_maker: CacheMaker,
    phantom_data: PhantomData<(Input, Output)>,
}

impl<Input: Sync + Send, Output: Send + Sync + 'static, CacheMaker: Fn(Input) -> Output + Send + Sync> CacheMakerGuard<Input, Output, CacheMaker> {
    fn new(cache_maker: CacheMaker) -> Self {
        Self { cache_maker, phantom_data: PhantomData }
    }

    async fn make(&self, input: Input) -> Output {
        let calculate = Arc::new(|| (self.cache_maker)(input));
        tokio::task::spawn_blocking(move || calculate.clone()()).await.unwrap()
    }
}

pub trait Cacher<'cacher, CacheMaker: Sync + Send + Fn(Self::Input) -> Self::Output> {
    type Input: 'cacher + Sync;
    type Output: Send + 'cacher;

    fn get_or_set(
        &'cacher mut self,
        input: Self::Input,
        cache_maker: CacheMakerGuard<Self::Input, Self::Output, CacheMaker>,
    ) -> TraitFuture<'cacher, Self::Output>;
    fn remove(&'cacher mut self, input: Self::Input) -> TraitFuture<'cacher, ()>;
    fn set(&'cacher mut self, input: Self::Input, cache_maker: CacheMaker) -> TraitFuture<'cacher, ()>;
}
