//! Volatile, fixed-capacity audio ring buffer.

/// A circular buffer for ephemeral PCM samples.
///
/// The backing allocation is made once and never grows. Pushing samples
/// overwrites the oldest samples when the ring is full.
#[derive(Debug)]
pub struct AudioRing {
    buffer: Vec<i16>,
    next_write: usize,
    len: usize,
}

impl AudioRing {
    /// Create a ring with capacity for `capacity_samples` samples.
    pub fn new(capacity_samples: usize) -> Self {
        Self {
            buffer: vec![0; capacity_samples],
            next_write: 0,
            len: 0,
        }
    }

    /// Append samples, retaining only the newest samples that fit.
    pub fn push_samples(&mut self, samples: &[i16]) {
        if self.buffer.is_empty() {
            return;
        }
        let capacity = self.buffer.len();
        let source = if samples.len() > capacity {
            &samples[samples.len() - capacity..]
        } else {
            samples
        };
        for &sample in source {
            self.buffer[self.next_write] = sample;
            self.next_write = (self.next_write + 1) % capacity;
            self.len = (self.len + 1).min(capacity);
        }
    }

    /// Number of samples currently held.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Whether this ring contains no samples.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Maximum number of samples held by this ring.
    pub fn capacity(&self) -> usize {
        self.buffer.len()
    }

    /// Whether the ring has received at least `capacity()` samples.
    pub fn is_full(&self) -> bool {
        self.len == self.capacity()
    }

    /// Copy the newest `n` samples in oldest-to-newest order.
    pub fn copy_latest(&self, out: &mut Vec<i16>, n: usize) {
        out.clear();
        if self.len == 0 || self.buffer.is_empty() {
            return;
        }
        let count = n.min(self.len);
        let start = (self.next_write + self.capacity() - self.len) % self.capacity();
        let offset = self.len - count;
        out.reserve(count);
        for index in 0..count {
            let position = (start + offset + index) % self.capacity();
            out.push(self.buffer[position]);
        }
    }

    /// Zero all storage and forget all samples.
    pub fn clear(&mut self) {
        self.buffer.fill(0);
        self.next_write = 0;
        self.len = 0;
    }

    #[cfg(test)]
    pub(crate) fn backing_store(&self) -> &[i16] {
        &self.buffer
    }
}

impl Default for AudioRing {
    fn default() -> Self {
        Self::new(160_000)
    }
}

#[cfg(test)]
mod tests {
    use super::AudioRing;

    #[test]
    fn wraparound_preserves_order() {
        let mut ring = AudioRing::new(4);
        ring.push_samples(&[1, 2, 3]);
        ring.push_samples(&[4, 5]);
        let mut out = Vec::new();
        ring.copy_latest(&mut out, 4);
        assert_eq!(out, [2, 3, 4, 5]);
    }

    #[test]
    fn over_capacity_push_keeps_newest() {
        let mut ring = AudioRing::new(3);
        ring.push_samples(&[1, 2, 3, 4, 5]);
        let mut out = Vec::new();
        ring.copy_latest(&mut out, 3);
        assert_eq!(out, [3, 4, 5]);
        assert!(ring.is_full());
    }

    #[test]
    fn copy_latest_clamps_and_clears_output() {
        let mut ring = AudioRing::new(4);
        ring.push_samples(&[10, 11, 12]);
        let mut out = vec![99, 100];
        ring.copy_latest(&mut out, 20);
        assert_eq!(out, [10, 11, 12]);
        ring.copy_latest(&mut out, 0);
        assert!(out.is_empty());
    }

    #[test]
    fn clear_zeroes_backing_store() {
        let mut ring = AudioRing::new(4);
        ring.push_samples(&[1, -2, 3, 4]);
        ring.clear();
        assert_eq!(ring.len(), 0);
        assert!(ring.backing_store().iter().all(|sample| *sample == 0));
    }

    #[test]
    fn capacity_never_grows() {
        let mut ring = AudioRing::new(4);
        let capacity = ring.capacity();
        ring.push_samples(&[1; 100]);
        assert_eq!(ring.capacity(), capacity);
        assert_eq!(ring.len(), capacity);
    }
}
