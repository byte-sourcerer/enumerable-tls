pub struct LazyBlocksVec<T, const ELEMENTS_PER_BLOCK: usize> {
    blocks: Vec<Option<Box<LazyBlock<T, ELEMENTS_PER_BLOCK>>>>,
}

impl<T, const ELEMENTS_PER_BLOCK: usize> LazyBlocksVec<T, ELEMENTS_PER_BLOCK> {
    pub const fn new() -> Self {
        Self { blocks: Vec::new() }
    }
}

// todo: allow non-default construction
impl<T: Clone + Default, const ELEMENTS_PER_BLOCK: usize> LazyBlocksVec<T, ELEMENTS_PER_BLOCK> {
    pub fn get_or_create_default(&mut self, id: &usize) -> T {
        let block_id = id / ELEMENTS_PER_BLOCK;
        let blocks_len = self.blocks.len();

        if block_id >= blocks_len {
            // The 32ul avoid pointless small resizes.
            self.blocks
                .resize_with((block_id + 1).max(32), Option::default)
        }

        let block = &mut self.blocks[block_id];
        let block = block.get_or_insert_default();
        let ptr = &mut block.data[id - block_id * ELEMENTS_PER_BLOCK];
        ptr.get_or_insert_default().clone()
    }
}

// todo: align 64
// todo: can we use Rc?
struct LazyBlock<T, const ELEMENTS_PER_BLOCK: usize> {
    data: [Option<T>; ELEMENTS_PER_BLOCK],
}

impl<T, const ELEMENTS_PER_BLOCK: usize> Default for LazyBlock<T, ELEMENTS_PER_BLOCK> {
    fn default() -> Self {
        let data = std::array::from_fn(|_| None);
        Self { data }
    }
}
