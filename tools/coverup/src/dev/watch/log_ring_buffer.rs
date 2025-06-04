use std::collections::VecDeque;

#[derive(Debug)]
pub struct LogRingBuffer {
    inner: VecDeque<String>,
}

impl LogRingBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            inner: VecDeque::with_capacity(capacity),
        }
    }

    pub fn push(&mut self, item: impl Into<String>) {
        let item = item.into();

        if self.inner.len() == self.inner.capacity() {
            self.inner.pop_front();
            self.inner.push_back(item);
            debug_assert!(self.inner.len() == self.inner.capacity());
        } else {
            self.inner.push_back(item);
        }
    }

    pub fn clear(&mut self) {
        self.inner.clear();
    }

    pub fn iter(&self) -> impl Iterator<Item = &String> {
        self.inner.iter()
    }
}

impl Default for LogRingBuffer {
    fn default() -> Self {
        Self::new(1000)
    }
}

#[cfg(test)]
mod tests {
    use super::LogRingBuffer;

    #[test]
    fn iter_iters_as_expected() {
        let mut buf = LogRingBuffer::new(3);
        buf.push("foo");
        buf.push("bar");
        buf.push("baz");

        assert_eq!(buf.iter().collect::<Vec<_>>(), vec!["foo", "bar", "baz"]);
        buf.push("qux");
        assert_eq!(buf.iter().collect::<Vec<_>>(), vec!["bar", "baz", "qux"]);
    }
}
