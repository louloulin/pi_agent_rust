//! Bounded tail buffers for server stderr and similar streaming logs.

/// Bounded tail buffer for server stderr.
#[derive(Debug, Default)]
pub struct TailBuffer {
    pub(crate) data: String,
    pub(crate) cap: usize,
}

impl TailBuffer {
    /// Create a new tail buffer with the given cap (bytes).
    #[must_use]
    pub const fn new(cap: usize) -> Self {
        Self {
            data: String::new(),
            cap,
        }
    }

    /// Append, discarding the oldest content past the cap.
    pub fn push(&mut self, chunk: &str) {
        self.data.push_str(chunk);
        if self.data.len() > self.cap {
            let keep_from = self.data.len() - self.cap;
            let boundary = self.data.ceil_char_boundary(keep_from);
            self.data.drain(..boundary);
        }
    }

    /// The retained tail.
    #[must_use]
    pub fn tail(&self) -> String {
        self.data.clone()
    }

    /// Borrow the retained tail without cloning.
    #[must_use]
    pub fn tail_str(&self) -> &str {
        &self.data
    }

    /// Current size of the retained tail in bytes.
    #[must_use]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Whether the buffer currently holds no data.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

/// Crate-shared bounded tail buffer (the DAP transport reuses it for
/// adapter stderr and process output).
#[derive(Debug)]
pub struct PublicTailBuffer {
    inner: TailBuffer,
}

impl PublicTailBuffer {
    /// A 32 KiB tail buffer (matches the legacy default).
    #[must_use]
    pub const fn new() -> Self {
        Self {
            inner: TailBuffer {
                data: String::new(),
                cap: 32 * 1024,
            },
        }
    }

    /// Create a `PublicTailBuffer` with a custom cap.
    #[must_use]
    pub const fn with_cap(cap: usize) -> Self {
        Self {
            inner: TailBuffer {
                data: String::new(),
                cap,
            },
        }
    }

    /// Append, discarding the oldest content past the cap.
    pub fn push(&mut self, chunk: &str) {
        self.inner.push(chunk);
    }

    /// The retained tail.
    #[must_use]
    pub fn tail(&self) -> String {
        self.inner.tail()
    }
}

impl Default for PublicTailBuffer {
    fn default() -> Self {
        Self::new()
    }
}
