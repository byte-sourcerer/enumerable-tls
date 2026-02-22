use std::{
    cell::RefCell,
    marker::PhantomData,
    sync::{Arc, Mutex, Weak},
    thread::LocalKey,
};

use crate::{
    free_ids::{FreeIds, RecycledTLSId},
    global::GlobalProvider,
    lazy_blocks_vec::LazyBlocksVec,
};

pub type TlsVec<T> = LazyBlocksVec<Arc<T>, 32>;

pub type TlsRegistry<T> = LocalKey<RefCell<TlsVec<T>>>;

pub struct EnumerableTls<
    T: 'static,
    IdProvider: GlobalProvider<Mutex<FreeIds>>,
    TlsProvider: GlobalProvider<TlsRegistry<T>>,
> {
    tls_id: RecycledTLSId<IdProvider>,
    all_tls: Mutex<Vec<Weak<T>>>,
    _marker: PhantomData<TlsProvider>,
}

impl<
    T: 'static,
    IdProvider: GlobalProvider<Mutex<FreeIds>>,
    TlsProvider: GlobalProvider<TlsRegistry<T>>,
> Default for EnumerableTls<T, IdProvider, TlsProvider>
{
    fn default() -> Self {
        Self::new()
    }
}

impl<
    T: 'static,
    IdProvider: GlobalProvider<Mutex<FreeIds>>,
    TlsProvider: GlobalProvider<TlsRegistry<T>>,
> EnumerableTls<T, IdProvider, TlsProvider>
{
    pub fn new() -> Self {
        Self {
            tls_id: RecycledTLSId::allocate(),
            all_tls: Mutex::new(Vec::with_capacity(64)),
            _marker: PhantomData,
        }
    }

    pub fn for_each(&self, mut f: impl FnMut(Arc<T>)) {
        let mut wrappers_guard = self.all_tls.lock().unwrap();
        wrappers_guard.retain(|wrapper| {
            if let Some(strong) = wrapper.upgrade() {
                f(strong);
                true
            } else {
                false
            }
        });
    }
}

impl<
    T: Default + 'static,
    IdProvider: GlobalProvider<Mutex<FreeIds>>,
    TlsProvider: GlobalProvider<TlsRegistry<T>>,
> EnumerableTls<T, IdProvider, TlsProvider>
{
    pub fn get_or_create(&self) -> Arc<T> {
        self.modify(|wrapper| wrapper.clone())
    }

    pub fn modify<U>(&self, mut f: impl FnMut(&Arc<T>) -> U) -> U {
        TlsProvider::global().with_borrow_mut(|blocks| {
            let wrapper = blocks.get_mut(self.tls_id.inner());
            let register = wrapper.is_none();
            let wrapper = wrapper.get_or_insert_default();

            if register {
                let mut guard = self.all_tls.lock().unwrap();
                guard.push(Arc::downgrade(wrapper));
                guard.retain(|wrapper| wrapper.upgrade().is_some());
            }

            f(wrapper)
        })
    }
}

#[macro_export]
macro_rules! declare_enumerable_tls {
    ($TlsType:ident, $Data:ty) => {
        $crate::paste::paste! {
            thread_local! {
                static [<$TlsType:snake:upper _ TLS_BLOCKS>]: ::std::cell::RefCell<
                    $crate::TlsVec<$Data>
                > = const { ::std::cell::RefCell::new($crate::LazyBlocksVec::new()) };
            }

            struct [<$TlsType TlsProvider>];

            impl $crate::GlobalProvider<$crate::TlsRegistry<$Data>> for [<$TlsType TlsProvider>] {
                fn global() -> &'static $crate::TlsRegistry<$Data> {
                    &[<$TlsType:snake:upper _ TLS_BLOCKS>]
                }
            }

            $crate::declare_free_ids!($TlsType);

            type $TlsType = $crate::EnumerableTls<$Data, [<$TlsType FreeIdsProvider>], [<$TlsType TlsProvider>]>;
        }
    };
}

#[cfg(test)]
mod tests {
    use std::{
        sync::atomic::{AtomicI32, Ordering},
        thread::{self, sleep},
        time::Duration,
    };

    use static_assertions::assert_impl_all;

    #[test]
    fn test() {
        declare_enumerable_tls!(TestEnumerableTls, AtomicI32);

        assert_impl_all!(TestEnumerableTls: Send, Sync);

        let tls = TestEnumerableTls::new();

        let sum_data = || {
            let mut sum = 0;
            tls.for_each(|data| {
                sum += data.load(Ordering::SeqCst);
            });
            sum
        };

        {
            let data = tls.get_or_create();
            assert_eq!(data.load(Ordering::SeqCst), 0);
            data.fetch_add(1, Ordering::SeqCst);
            assert_eq!(sum_data(), 1);
        }

        thread::scope(|s| {
            thread::Builder::new()
                .name("another thread".to_string())
                .spawn_scoped(s, || {
                    let data = tls.get_or_create();
                    assert_eq!(data.load(Ordering::SeqCst), 0);
                    data.fetch_add(2, Ordering::SeqCst);

                    assert_eq!(sum_data(), 3);
                })
                .unwrap();
        });

        // see: https://github.com/rust-lang/rust/issues/116237
        sleep(Duration::from_millis(20));
        assert_eq!(sum_data(), 1);
    }

    #[test]
    fn get_twice() {
        declare_enumerable_tls!(GetTwiceEnumerableTls, AtomicI32);

        let tls = GetTwiceEnumerableTls::new();

        let data0 = tls.get_or_create();
        data0.fetch_add(1, Ordering::SeqCst);

        let data1 = tls.get_or_create();
        data1.fetch_add(1, Ordering::SeqCst);

        let mut sum = 0;
        tls.for_each(|data| {
            sum += data.load(Ordering::SeqCst);
        });

        assert_eq!(sum, 2);
    }
}
