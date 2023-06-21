use sbbf_rs::{FilterFn, ALIGNMENT, BUCKET_SIZE};
use std::alloc::{alloc_zeroed, dealloc, Layout};

/// A split block bloom filter that handles it's own memory
pub struct Filter {
    filter_fn: FilterFn,
    buf: Buf,
    num_buckets: usize,
}

impl Filter {
    /// Create a new filter using the parameters.
    ///
    /// Calculated length will be rounded up to the nearest multiple of 64.
    pub fn new(bits_per_key: usize, num_keys: usize) -> Self {
        assert_eq!(64, BUCKET_SIZE * 2);
        let len = bits_per_key * num_keys / 8;
        let len = ((len + 64 - 1) / 64) * 64;
        let len = if len == 0 { 64 } else { len };
        Self {
            filter_fn: FilterFn::new(),
            buf: Buf::new(len),
            num_buckets: len / BUCKET_SIZE,
        }
    }

    /// Check if the filter contains the hash.
    #[inline(always)]
    pub fn contains_hash(&self, hash: u64) -> bool {
        unsafe {
            self.filter_fn
                .contains(self.buf.ptr, self.num_buckets, hash)
        }
    }

    /// Insert the hash into the filter.
    ///
    /// Returns true if the hash was already in the filter.
    #[inline(always)]
    pub fn insert_hash(&mut self, hash: u64) -> bool {
        unsafe { self.filter_fn.insert(self.buf.ptr, self.num_buckets, hash) }
    }

    /// Returns a slice the slice of bytes that represent this filter.
    ///
    /// The filter can be restored using these bytes with the `Filter::from_bytes` method.
    pub fn as_bytes(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.buf.ptr, self.buf.layout.size()) }
    }

    /// Restore a filter from the given bytes.
    ///
    /// Returns None if the bytes are invalid.
    pub fn from_bytes<B: AsRef<[u8]>>(bytes: B) -> Option<Self> {
        let bytes = bytes.as_ref();

        if bytes.len() < BUCKET_SIZE {
            return None;
        }

        let len = bytes.len();
        let len = ((len + 64 - 1) / 64) * 64;
        let len = if len == 0 { 64 } else { len };

        let buf = Buf::new(len);

        let buf_bytes = unsafe { std::slice::from_raw_parts_mut(buf.ptr, buf.layout.size()) };
        buf_bytes[..bytes.len()].copy_from_slice(bytes);

        Some(Self {
            filter_fn: FilterFn::new(),
            buf,
            num_buckets: bytes.len() / BUCKET_SIZE,
        })
    }
}

struct Buf {
    ptr: *mut u8,
    layout: Layout,
}

impl Buf {
    fn new(len: usize) -> Self {
        let layout = Layout::from_size_align(len, ALIGNMENT).unwrap();
        let ptr = unsafe { alloc_zeroed(layout) };

        Self { layout, ptr }
    }
}

impl Drop for Buf {
    fn drop(&mut self) {
        unsafe {
            dealloc(self.ptr, self.layout);
        }
    }
}

unsafe impl Send for Filter {}
unsafe impl Sync for Filter {}
