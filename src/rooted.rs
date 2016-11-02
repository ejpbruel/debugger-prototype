use js::jsapi::{Handle, MutableHandle};

pub trait Rooted<T> {
    fn get(&self) -> T;
    fn handle(&self) -> Handle<T>;
    fn handle_mut(&mut self) -> MutableHandle<T>;
}
