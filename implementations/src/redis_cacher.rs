use std::sync::Mutex;

use abstractions::cacher::Cacher;
use redis::{Commands, RedisResult, FromRedisValue, ToRedisArgs};

pub struct RedisCacher<'cacher> {
    namespace: &'cacher [u8],
    conn: Mutex<&'cacher mut redis::Connection>,
}

impl<'cacher> RedisCacher<'cacher> {
    pub fn new(namespace: &'cacher [u8], conn: Mutex<&'cacher mut redis::Connection>) -> Self {
        Self { namespace, conn }
    }
}

pub trait RedisPrefixable<Prefix> {
    type Output: ToRedisArgs;

    fn prefix(&self, prefix: Prefix) -> Self::Output;
}

impl<'cacher, Input, Output, CacheMaker> Cacher<Input, Output, CacheMaker> for RedisCacher<'cacher>
where
    CacheMaker: Fn(Input) -> Output,
    Input: RedisPrefixable<&'cacher [u8]>,
    Output: ToRedisArgs + FromRedisValue,
{
    fn get_or_set(&mut self, input: Input, cache_maker: CacheMaker) -> Output {
        if let Ok(result) = self.conn.get_mut().unwrap().get(input.prefix(self.namespace)) {
            result
        } else {
            let key = input.prefix(self.namespace);
            let result = cache_maker(input);
            let _setting_result: RedisResult<()> = self
                .conn
                .get_mut()
                .unwrap()
                .set(key, &result);
            result
        }
    }

    fn set(&mut self, input: Input, cache_maker: CacheMaker) {
        let _setting_result: RedisResult<()> = self
            .conn
            .get_mut()
            .unwrap()
            .set(input.prefix(self.namespace), cache_maker(input));
    }

    fn remove(&mut self, input: Input) {
        let _deletion_result: RedisResult<()> =
            self.conn.get_mut().unwrap().del(input.prefix(self.namespace));
    }
}
