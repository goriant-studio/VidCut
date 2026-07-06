//! Frame cache — LRU cache for decoded video frames.
//!
//! **Phase 1 stub.** Phase 2 will implement a bounded LRU cache backed by a
//! `parking_lot::RwLock`-protected `LinkedHashMap`, keyed by
//! `(asset_id, frame_index)`.

use anyhow::Result;
use uuid::Uuid;

/// A bounded LRU cache for decoded RGBA video frames.
///
/// Frames are evicted least-recently-used once the cache reaches its capacity.
///
/// Phase 2: backed by a `parking_lot::RwLock<LinkedHashMap<FrameKey, Vec<u8>>>`.
#[allow(dead_code)]
pub struct FrameCache {
    /// Maximum number of frames to keep in memory.
    capacity: usize,
}

impl FrameCache {
    /// Create a new cache with the given `capacity` (number of frames).
    pub fn new(capacity: usize) -> Self {
        Self { capacity }
    }

    /// Look up a cached frame by asset id and frame index.
    ///
    /// Returns `None` if the frame is not in cache (a cache miss).
    ///
    /// # Phase 2
    /// Will acquire a read lock, look up the key, promote to most-recently-used.
    pub fn get(&self, _asset_id: Uuid, _frame_index: u64) -> Option<Vec<u8>> {
        todo!("Phase 2: LRU lookup with parking_lot::RwLock")
    }

    /// Insert a decoded frame into the cache, evicting the LRU entry if full.
    ///
    /// # Phase 2
    /// Will acquire a write lock, insert the entry, and evict if over capacity.
    pub fn insert(&self, _asset_id: Uuid, _frame_index: u64, _rgba: Vec<u8>) -> Result<()> {
        todo!("Phase 2: LRU insert with eviction")
    }

    /// Remove all cached frames for the given asset (e.g. after re-import).
    ///
    /// # Phase 2
    /// Will scan the cache for keys matching `asset_id` and remove them.
    pub fn invalidate(&self, _asset_id: Uuid) {
        todo!("Phase 2: invalidate all frames for a given asset")
    }
}
