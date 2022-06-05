pub trait Cacher<Input, Output, CacheMaker>
where
    CacheMaker: Fn(Input) -> Output,
{
    fn get_or_set(&mut self, input: Input, cache_maker: CacheMaker) -> Output;
    fn set(&mut self, input: Input, cache_maker: CacheMaker);
    fn remove(&mut self, input: Input);
}
