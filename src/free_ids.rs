use std::{collections::LinkedList, marker::PhantomData};

use crate::global::GlobalProvider;

pub struct RecycledTLSId<P: GlobalProvider<FreeIds>>(TLSId, PhantomData<P>);

impl<P: GlobalProvider<FreeIds>> RecycledTLSId<P> {
    pub fn allocate() -> Self {
        P::global().lock().unwrap().create()
    }

    // todo: remove this method
    pub fn inner(&self) -> usize {
        self.0.0
    }
}

impl<P: GlobalProvider<FreeIds>> Drop for RecycledTLSId<P> {
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

impl FreeIds {
    const fn new() -> Self {
        Self {
            free_ids: LinkedList::new(),
            id: TLSId(0),
        }
    }

    fn create<P: GlobalProvider<FreeIds>>(&mut self) -> RecycledTLSId<P> {
        let id = if let Some(id) = self.free_ids.pop_back() {
            id
        } else {
            self.id = self.id.increment();
            self.id
        };

        RecycledTLSId(id, PhantomData)
    }

    fn recycle(&mut self, key: TLSId) {
        self.free_ids.push_back(key);
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use crate::{
        free_ids::{FreeIds, RecycledTLSId},
        global::GlobalProvider,
    };

    static FREE_IDS: Mutex<FreeIds> = Mutex::new(FreeIds::new());

    struct FreeIdsProvider;

    impl GlobalProvider<FreeIds> for FreeIdsProvider {
        fn global() -> &'static Mutex<FreeIds> {
            &FREE_IDS
        }
    }

    #[test]
    fn test() {
        let _id = RecycledTLSId::<FreeIdsProvider>::allocate();
    }
}
