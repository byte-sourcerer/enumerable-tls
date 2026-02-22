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

impl<T, const ELEMENTS_PER_BLOCK: usize> LazyBlocksVec<T, ELEMENTS_PER_BLOCK> {
    pub fn get_mut(&mut self, index: usize) -> &mut Option<T> {
        let block_index = index / ELEMENTS_PER_BLOCK;
        let blocks_len = self.blocks.len();

        if block_index >= blocks_len {
            // The 32ul avoid pointless small resizes.
            self.blocks
                .resize_with((block_index + 1).max(32), Option::default)
        }

        let block = self.blocks[block_index].get_or_insert_default();
        &mut block.data[index - block_index * ELEMENTS_PER_BLOCK]
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
            let elem = vec.get_mut(1);
            *elem = Some(1);
        }
        assert_eq!(vec.blocks.len(), 32);
        assert_eq!(vec.get_mut(1), &Some(1));
        assert_eq!(vec.get_mut(2), &None);
    }
}
