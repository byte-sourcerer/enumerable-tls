use std::{collections::LinkedList, marker::PhantomData, sync::Mutex};

use crate::global::GlobalProvider;

pub struct RecycledTLSId<P: GlobalProvider<Mutex<FreeIds>>>(TLSId, PhantomData<P>);

impl<P: GlobalProvider<Mutex<FreeIds>>> RecycledTLSId<P> {
    pub fn allocate() -> Self {
        P::global().lock().unwrap().create()
    }

    pub fn inner(&self) -> usize {
        self.0.0
    }
}

impl<P: GlobalProvider<Mutex<FreeIds>>> Drop for RecycledTLSId<P> {
    fn drop(&mut self) {
        P::global().lock().unwrap().recycle(self.0);
    }
}

#[derive(Clone, Copy)]
struct TLSId(usize);

impl TLSId {
    fn increment(self) -> Self {
        Self(self.0 + 1)
    }
}

pub struct FreeIds {
    free_ids: LinkedList<TLSId>,
    id: TLSId,
}

impl Default for FreeIds {
    fn default() -> Self {
        Self::new()
    }
}

impl FreeIds {
    pub const fn new() -> Self {
        Self {
            free_ids: LinkedList::new(),
            id: TLSId(0),
        }
    }

    fn create<P: GlobalProvider<Mutex<FreeIds>>>(&mut self) -> RecycledTLSId<P> {
        let id = if let Some(id) = self.free_ids.pop_back() {
            id
        } else {
            let id = self.id;
            self.id = self.id.increment();
            id
        };

        RecycledTLSId(id, PhantomData)
    }

    fn recycle(&mut self, key: TLSId) {
        self.free_ids.push_back(key);
    }
}

#[macro_export]
macro_rules! declare_free_ids {
    ($TlsType:ident) => {
        $crate::paste::paste! {
            static [<$TlsType:snake:upper _ FREE_IDS>]: ::std::sync::Mutex<$crate::free_ids::FreeIds> =
                ::std::sync::Mutex::new($crate::free_ids::FreeIds::new());

            struct [<$TlsType FreeIdsProvider>];

            impl $crate::GlobalProvider<::std::sync::Mutex<$crate::FreeIds>> for [<$TlsType FreeIdsProvider>] {
                fn global() -> &'static ::std::sync::Mutex<$crate::free_ids::FreeIds> {
                    &[<$TlsType:snake:upper _ FREE_IDS>]
                }
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use crate::free_ids::RecycledTLSId;

    declare_free_ids!(My);

    #[test]
    fn test() {
        let id0 = RecycledTLSId::<MyFreeIdsProvider>::allocate();
        assert_eq!(id0.inner(), 0);

        let id1 = RecycledTLSId::<MyFreeIdsProvider>::allocate();
        assert_eq!(id1.inner(), 1);

        let id2 = RecycledTLSId::<MyFreeIdsProvider>::allocate();
        assert_eq!(id2.inner(), 2);

        drop(id1);

        let id1 = RecycledTLSId::<MyFreeIdsProvider>::allocate();
        assert_eq!(id1.inner(), 1);
    }
}
