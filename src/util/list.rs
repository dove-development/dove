use {
    super::revert, crate::traits::Pod, std::ops::{Index, IndexMut}
};

#[repr(C)]
#[derive(Clone, Copy)]
pub struct List<T: Copy, const C: usize> {
    len: usize,
    data: [T; C],
}

impl<T: Copy, const C: usize> List<T, C> {
    pub const fn new() -> Self {
        // Safety: `data` is not accessed because `len` is zero.
        unsafe { std::mem::zeroed() }
    }

    pub fn push(&mut self, item: T) {
        if self.len < C {
            self.data[self.len] = item;
            self.len += 1;
        } else {
            revert("List is full")
        }
    }
    pub fn len(&self) -> usize {
        self.len
    }
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.data[..self.len].iter()
    }
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.data[..self.len].iter_mut()
    }
    pub fn swap_remove(&mut self, index: usize) -> T {
        let last_index = self.len - 1;
        self.data.swap(index, last_index);
        self.len -= 1;
        self.data[self.len]
    }
    #[cfg(feature = "wasm")]
    pub fn to_vec(&self) -> Vec<T> {
        self.data[..self.len].to_vec()
    }
}

impl<T: Copy, const C: usize> Index<usize> for List<T, C> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

impl<T: Copy, const C: usize> IndexMut<usize> for List<T, C> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.data[index]
    }
}

unsafe impl<T: Pod, const C: usize> Pod for List<T, C> {
    const NAME: &'static str = "List";
}
