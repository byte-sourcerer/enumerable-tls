use std::sync::Mutex;

pub trait GlobalProvider<T>: 'static {
    fn global() -> &'static Mutex<T>;
}
