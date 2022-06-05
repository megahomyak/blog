use std::{sync::Mutex, vec};

use abstractions::sorted_set::{SortedSet, WeightedKey};
use redis::{Commands, RedisResult, ToRedisArgs};

pub struct RedisSortedSet<'conn> {
    name: String,
    conn: Mutex<&'conn mut redis::Connection>,
}

impl<'conn> RedisSortedSet<'conn> {
    pub fn new(name: String, conn: Mutex<&'conn mut redis::Connection>) -> Self {
        Self { name, conn }
    }
}

impl<'conn, Key, Weight> SortedSet<Key, Weight> for RedisSortedSet<'conn>
where
    Key: ToRedisArgs + From<Vec<u8>>,
    Weight: ToRedisArgs + From<Vec<u8>>,
{
    type Iter = vec::IntoIter<WeightedKey<Key, Weight>>;

    fn set(&mut self, key: Key, weight: Weight) {
        let _addition_result: RedisResult<()> =
            self.conn.get_mut().unwrap().zadd(&self.name, key, weight);
    }

    fn remove(&mut self, key: Key) {
        let _deletion_result: RedisResult<()> = self.conn.get_mut().unwrap().del(key);
    }

    fn iter(&mut self, chunk_size: usize, chunk_index: usize) -> Self::Iter {
        let offset = chunk_index * chunk_size;
        let range_beginning = offset;
        let range_end = offset + chunk_size - 1;
        let keys_and_weights: RedisResult<Vec<Vec<u8>>> =
            self.conn.get_mut().unwrap().zrange_withscores(
                &self.name,
                range_beginning.try_into().unwrap(),
                range_end.try_into().unwrap(),
            );
        if let Ok(keys_and_weights) = keys_and_weights {
            let mut keys_and_weights = keys_and_weights.into_iter();
            let mut results = Vec::new();
            while let Some(key) = keys_and_weights.next() {
                let weight = keys_and_weights.next().expect(
                    "Redis should have returned a weight with a key, \
                    because WITHSCORES is specified",
                );
                results.push(WeightedKey { key: Key::from(key), weight: Weight::from(weight) });
            }
            results.into_iter()
        } else {
            Vec::with_capacity(0).into_iter()
        }
    }
}
