/// A fixed-capacity rolling buffer that keeps the most recent samples.
///
/// Pushing into a full window evicts the oldest sample and returns it, which
/// is what running-sum style indicators need in order to subtract it.
///
/// # Example
///
/// ```
/// use nova_ta_core::Window;
///
/// let mut w = Window::new(2);
/// assert_eq!(w.push(1), None);
/// assert_eq!(w.push(2), None);
/// assert_eq!(w.push(3), Some(1));
/// assert_eq!(w.iter().copied().collect::<Vec<_>>(), [2, 3]);
/// ```
#[derive(Clone, Debug)]
pub struct Window<T> {
    cap: usize,
    buf: Vec<T>,
    /// Index of the OLDEST element once full (the next overwrite target).
    next: usize,
}

impl<T> Window<T> {
    /// Creates an empty window holding at most `cap` samples.
    ///
    /// # Panics
    /// Panics if `cap == 0`.
    pub fn new(cap: usize) -> Self {
        assert!(cap >= 1, "Window capacity must be >= 1");
        Self { cap, buf: Vec::with_capacity(cap), next: 0 }
    }

    /// Appends a sample. Returns the evicted oldest sample if the window was
    /// already full, otherwise `None`.
    pub fn push(&mut self, value: T) -> Option<T> {
        if self.buf.len() < self.cap {
            self.buf.push(value);
            None
        } else {
            let old = core::mem::replace(&mut self.buf[self.next], value);
            self.next = (self.next + 1) % self.cap;
            Some(old)
        }
    }

    /// Number of samples currently held, at most [`capacity`](Window::capacity).
    pub fn len(&self) -> usize {
        self.buf.len()
    }

    /// Returns `true` if no samples are held.
    pub fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }

    /// Returns `true` once `capacity` samples are held.
    pub fn is_full(&self) -> bool {
        self.buf.len() == self.cap
    }

    /// The maximum number of samples held.
    pub fn capacity(&self) -> usize {
        self.cap
    }

    /// Returns the oldest retained sample, or `None` if empty.
    pub fn oldest(&self) -> Option<&T> {
        if self.buf.is_empty() {
            None
        } else if self.buf.len() < self.cap {
            self.buf.first()
        } else {
            Some(&self.buf[self.next])
        }
    }

    /// Iterates from oldest to newest.
    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &T> {
        let (older, newer) = if self.buf.len() < self.cap {
            (&self.buf[..], &[][..])
        } else {
            (&self.buf[self.next..], &self.buf[..self.next])
        };

        older.iter().chain(newer.iter())
    }

    /// Iterates from newest to oldest.
    pub fn rev_iter(&self) -> impl DoubleEndedIterator<Item = &T> {
        self.iter().rev()
    }

    /// Removes all samples. The capacity is unchanged.
    pub fn clear(&mut self) {
        self.buf.clear();
        self.next = 0;
    }

    /// Returns the newest retained sample.
    pub fn newest(&self) -> Option<&T> {
        self.rev_iter().next()
    }

    /// Returns the sample `lag` positions before the newest sample.
    ///
    /// `lag = 0` returns the newest sample.
    /// Returns `None` if `lag >= self.len()`.
    pub fn get_from_newest(&self, lag: usize) -> Option<&T> {
        if lag >= self.len() {
            return None;
        }
    
        let index = if self.is_full() {
            (self.next + self.cap - 1 - lag) % self.cap
        } else {
            self.buf.len() - 1 - lag
        };
    
        self.buf.get(index)
    }
    /// Returns the sample at a chronological index.
    ///
    /// Index 0 is the oldest retained sample.
    pub fn get(&self, index: usize) -> Option<&T> {
        if index >= self.len() {
            return None;
        }

        if !self.is_full() {
            Some(&self.buf[index])
        } else {
            Some(&self.buf[(self.next + index) % self.cap])
        }
    }

}
