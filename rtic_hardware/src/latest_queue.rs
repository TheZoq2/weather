use heapless::{Vec, ArrayLength};

pub struct LatestQueue<T, N: ArrayLength<(u16, T)>> {
    pub inner: Vec<(u16, T), N>,
    next: u16,
}

impl<T, N: ArrayLength<(u16, T)>> LatestQueue<T, N> {
    pub fn new() -> Self {
        Self {inner: Vec::new(), next: 0}
    }

    pub fn push(&mut self, val: T) {
        if self.inner.len() < self.inner.capacity() {
            self.inner.push((self.next, val));
        }
        else {
            // Find the lowest index and replace it
            // NOTE: Safe unwrap because we know the vec has elements
            let (target, _) = self.inner.iter().enumerate()
                .min_by_key(|(_, (i, _))| i)
                .unwrap();

            self.inner[target] = (self.next, val)
        }

        self.next = self.next.wrapping_add(1);
    }

    pub fn clear(&mut self) {
        self.inner.clear();
        self.next = 0;
    }
}
