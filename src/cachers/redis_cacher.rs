use redis::{aio::Connection, AsyncCommands, RedisResult};

use crate::{cachers::Cacher, types::TraitFuture};

use super::CacheMakerGuard;

pub struct RedisCacher {
    redis_connection: Connection,
}

impl RedisCacher {
    pub const fn new(redis_connection: Connection) -> Self {
        Self { redis_connection }
    }
}

impl<'cacher, CacheMaker: Sync + Send + 'cacher + Fn(&'cacher String) -> String>
    Cacher<'cacher, CacheMaker> for RedisCacher
{
    type Input = &'cacher String;
    type Output = String;

    fn get_or_set(
        &'cacher mut self,
        input: Self::Input,
        cache_maker: CacheMakerGuard<Self::Input, Self::Output, CacheMaker>,
    ) -> TraitFuture<'cacher, Self::Output> {
        Box::pin(async move {
            let cached_value: RedisResult<String> = self.redis_connection.get(input).await;
            match cached_value {
                Ok(cached_value) => cached_value,
                Err(error) => {
                    let value = cache_maker.make(input).await;
                    self.redis_connection
                        .set::<Self::Input, Self::Output, ()>(input, value)
                        .await;
                    value
                }
            }
        })
    }

    fn remove(&'cacher mut self, input: Self::Input) -> TraitFuture<'cacher, ()> {
        Box::pin(async move {
            self.redis_connection.del::<Self::Input, ()>(input).await;
        })
    }

    fn set(&'cacher mut self, input: Self::Input, cache_maker: CacheMaker) -> TraitFuture<'cacher, ()> {
        Box::pin(async move {
            self.redis_connection.set::<Self::Input, Self::Output, ()>(input, cache_maker(input)).await;
        })
    }
}
