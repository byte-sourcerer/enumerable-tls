pub struct LazyBlocksVec<T, const ELEMENTS_PER_BLOCK: usize> {
    blocks: Vec<Option<Box<LazyBlock<T, ELEMENTS_PER_BLOCK>>>>,
}

impl<T, const ELEMENTS_PER_BLOCK: usize> Default for LazyBlocksVec<T, ELEMENTS_PER_BLOCK> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const ELEMENTS_PER_BLOCK: usize> LazyBlocksVec<T, ELEMENTS_PER_BLOCK> {
    pub const fn new() -> Self {
        Self { blocks: Vec::new() }
    }
}

// todo: allow non-default construction
impl<T: Default, const ELEMENTS_PER_BLOCK: usize> LazyBlocksVec<T, ELEMENTS_PER_BLOCK> {
    pub fn get_or_create_default(&mut self, index: usize) -> &mut T {
        let block_index = index / ELEMENTS_PER_BLOCK;
        let blocks_len = self.blocks.len();

        if block_index >= blocks_len {
            // The 32ul avoid pointless small resizes.
            self.blocks
                .resize_with((block_index + 1).max(32), Option::default)
        }

        let block = self.blocks[block_index].get_or_insert_default();
        block.data[index - block_index * ELEMENTS_PER_BLOCK].get_or_insert_default()
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

#[cfg(test)]
mod tests {
    use crate::LazyBlocksVec;

    #[test]
    fn test() {
        let mut vec = LazyBlocksVec::<i32, 8>::default();

        {
            let elem = vec.get_or_create_default(1);
            *elem = 1;
        }
        assert_eq!(vec.blocks.len(), 32);
        assert_eq!(vec.get_or_create_default(1), &1);
        assert_eq!(vec.get_or_create_default(2), &0);
    }
}
