//! Rolling time-series buffer for accumulating feed data across poll cycles.
//!
//! Each feed domain (weather, price, seismic) pushes new values into a named
//! buffer.  The buffer is capped at `max_len` so memory stays bounded while
//! the analysis functions always receive the full rolling window.

use std::collections::{HashMap, VecDeque};

/// A thread-safe rolling buffer for multiple named time-series.
///
/// Wrap in `Arc<RwLock<FeedBuffer>>` for shared access from polling tasks.
pub struct FeedBuffer {
    buffers: HashMap<String, VecDeque<f64>>,
    max_len: usize,
}

impl FeedBuffer {
    /// Create a new `FeedBuffer` with the given maximum per-series length.
    pub fn new(max_len: usize) -> Self {
        Self {
            buffers: HashMap::new(),
            max_len,
        }
    }

    /// Append values to the named buffer, trimming the front if it exceeds `max_len`.
    pub fn push(&mut self, feed_name: &str, values: &[f64]) {
        let buf = self
            .buffers
            .entry(feed_name.to_string())
            .or_insert_with(|| VecDeque::with_capacity(self.max_len));

        for &v in values {
            buf.push_back(v);
        }

        // Trim to max_len from the front (oldest data evicted first)
        while buf.len() > self.max_len {
            buf.pop_front();
        }
    }

    /// Read access to the named buffer.
    pub fn get(&self, feed_name: &str) -> Option<&VecDeque<f64>> {
        self.buffers.get(feed_name)
    }

    /// Number of values currently stored for a given feed.
    pub fn len(&self, feed_name: &str) -> usize {
        self.buffers.get(feed_name).map_or(0, |b| b.len())
    }

    /// Snapshot the buffer as a `Vec<f64>` (useful for passing to detectors).
    pub fn snapshot(&self, feed_name: &str) -> Option<Vec<f64>> {
        self.buffers
            .get(feed_name)
            .map(|b| b.iter().copied().collect())
    }
}

impl Default for FeedBuffer {
    fn default() -> Self {
        Self::new(1000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_and_trim() {
        let mut buf = FeedBuffer::new(5);
        buf.push("test", &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0]);
        assert_eq!(buf.len("test"), 5);
        let snap = buf.snapshot("test").unwrap();
        assert_eq!(snap, vec![3.0, 4.0, 5.0, 6.0, 7.0]);
    }

    #[test]
    fn incremental_push() {
        let mut buf = FeedBuffer::new(4);
        buf.push("a", &[1.0, 2.0]);
        buf.push("a", &[3.0, 4.0]);
        assert_eq!(buf.len("a"), 4);
        buf.push("a", &[5.0]);
        assert_eq!(buf.len("a"), 4);
        assert_eq!(buf.snapshot("a").unwrap(), vec![2.0, 3.0, 4.0, 5.0]);
    }

    #[test]
    fn unknown_feed_returns_none() {
        let buf = FeedBuffer::new(10);
        assert!(buf.get("nope").is_none());
        assert_eq!(buf.len("nope"), 0);
    }
}
