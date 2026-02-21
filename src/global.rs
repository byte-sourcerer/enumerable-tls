pub trait GlobalProvider<T>: 'static {
    fn global() -> &'static T;
}
