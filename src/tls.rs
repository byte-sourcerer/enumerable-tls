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

// todo: allow T = Arc<XXX>
type TlsRegistry<T> = LocalKey<RefCell<LazyBlocksVec<Arc<T>, 32>>>;

pub struct EnumerableTls<
    T: Default + 'static,
    IdProvider: GlobalProvider<Mutex<FreeIds>>,
    TlsProvider: GlobalProvider<TlsRegistry<T>>,
> {
    tls_id: RecycledTLSId<IdProvider>,
    all_tls: Mutex<Vec<Weak<T>>>,
    _marker: PhantomData<TlsProvider>,
}

impl<
    T: Default + 'static,
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

    pub fn get_or_create(&self) -> Arc<T> {
        let wrapper = TlsProvider::global()
            .with_borrow_mut(|blocks| blocks.get_or_create_default(&self.tls_id.inner()));

        {
            let mut guard = self.all_tls.lock().unwrap();
            guard.push(Arc::downgrade(&wrapper));
            guard.retain(|wrapper| wrapper.upgrade().is_some());
        }

        wrapper
    }
}

#[cfg(test)]
mod tests {
    use std::{
        cell::RefCell,
        sync::{Arc, Mutex},
    };

    use static_assertions::assert_impl_all;

    use crate::{
        free_ids::FreeIds,
        global::GlobalProvider,
        lazy_blocks_vec::LazyBlocksVec,
        tls::{EnumerableTls, TlsRegistry},
    };

    type Data = ();

    thread_local! {
        static TLS_BLOCKS: RefCell<LazyBlocksVec<Arc<Data>, 32>> = const { RefCell::new(LazyBlocksVec::new()) };
    }

    struct TlsProvider;

    impl GlobalProvider<TlsRegistry<Data>> for TlsProvider {
        fn global() -> &'static TlsRegistry<Data> {
            &TLS_BLOCKS
        }
    }

    static FREE_IDS: Mutex<FreeIds> = Mutex::new(FreeIds::new());

    struct FreeIdsProvider;

    impl GlobalProvider<Mutex<FreeIds>> for FreeIdsProvider {
        fn global() -> &'static Mutex<FreeIds> {
            &FREE_IDS
        }
    }

    type MyEnumerableTls = EnumerableTls<Data, FreeIdsProvider, TlsProvider>;

    assert_impl_all!(MyEnumerableTls: Send, Sync);

    #[test]
    fn test() {
        let tls = MyEnumerableTls::new();
        let _data = tls.get_or_create();
    }
}
