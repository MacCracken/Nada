//! Lock-free audio metering — stereo peak levels via atomic operations.
//!
//! Designed for real-time audio: the RT thread writes peak levels via
//! `AtomicU32` (f32 bit patterns), and the UI thread reads them without
//! mutexes or blocking.

use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};
use std::sync::Arc;

/// A single stereo peak level stored as two atomic u32 (f32 bit patterns).
///
/// Lock-free: safe to write from the RT thread and read from the UI thread.
#[derive(Debug)]
pub struct PeakMeter {
    left: AtomicU32,
    right: AtomicU32,
}

impl PeakMeter {
    /// Create a meter reading zero.
    pub fn new() -> Self {
        Self {
            left: AtomicU32::new(0.0f32.to_bits()),
            right: AtomicU32::new(0.0f32.to_bits()),
        }
    }

    /// Store a stereo peak level (call from RT thread).
    pub fn store(&self, left: f32, right: f32) {
        self.left.store(left.to_bits(), Ordering::Relaxed);
        self.right.store(right.to_bits(), Ordering::Relaxed);
    }

    /// Load the current stereo peak level (call from UI thread).
    pub fn load(&self) -> [f32; 2] {
        let l = f32::from_bits(self.left.load(Ordering::Relaxed));
        let r = f32::from_bits(self.right.load(Ordering::Relaxed));
        [l, r]
    }
}

impl Default for PeakMeter {
    fn default() -> Self {
        Self::new()
    }
}

/// A bank of stereo peak meters with dynamic slot activation.
///
/// Pre-allocates capacity at creation time. Slots can be activated
/// up to capacity without reallocation.
pub struct MeterBank {
    slots: Vec<PeakMeter>,
    active: AtomicUsize,
}

impl MeterBank {
    /// Create a meter bank with the given capacity.
    pub fn new(capacity: usize) -> Self {
        Self {
            slots: (0..capacity).map(|_| PeakMeter::new()).collect(),
            active: AtomicUsize::new(capacity),
        }
    }

    /// Number of active slots.
    pub fn len(&self) -> usize {
        self.active.load(Ordering::Relaxed)
    }

    /// Whether the bank has no active slots.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Total allocated capacity.
    pub fn capacity(&self) -> usize {
        self.slots.len()
    }

    /// Store a stereo peak level at the given slot index.
    ///
    /// No-op if index is out of bounds.
    pub fn store(&self, index: usize, left: f32, right: f32) {
        if let Some(slot) = self.slots.get(index) {
            slot.store(left, right);
        }
    }

    /// Load the stereo peak level at the given slot index.
    ///
    /// Returns `[0.0, 0.0]` if index is out of bounds.
    pub fn load(&self, index: usize) -> [f32; 2] {
        self.slots
            .get(index)
            .map(|s| s.load())
            .unwrap_or([0.0, 0.0])
    }

    /// Read all active slots into a Vec.
    pub fn read_all(&self) -> Vec<[f32; 2]> {
        let active = self.len();
        (0..active).map(|i| self.load(i)).collect()
    }

    /// Set the number of active slots (clamped to capacity).
    pub fn set_active(&self, count: usize) {
        self.active
            .store(count.min(self.slots.len()), Ordering::Relaxed);
    }
}

// SAFETY: PeakMeter uses only atomics, no interior mutability beyond that.
unsafe impl Sync for MeterBank {}
unsafe impl Send for MeterBank {}

impl std::fmt::Debug for MeterBank {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MeterBank")
            .field("capacity", &self.capacity())
            .field("active", &self.len())
            .finish()
    }
}

/// Shared meter bank — `Arc`-wrapped for multi-thread sharing.
pub type SharedMeterBank = Arc<MeterBank>;

/// Create a shared meter bank with the given capacity.
pub fn shared_meter_bank(capacity: usize) -> SharedMeterBank {
    Arc::new(MeterBank::new(capacity))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn peak_meter_store_load() {
        let meter = PeakMeter::new();
        assert_eq!(meter.load(), [0.0, 0.0]);
        meter.store(0.8, 0.6);
        assert_eq!(meter.load(), [0.8, 0.6]);
    }

    #[test]
    fn meter_bank_basic() {
        let bank = MeterBank::new(4);
        assert_eq!(bank.len(), 4);
        assert_eq!(bank.capacity(), 4);

        bank.store(0, 0.9, 0.7);
        bank.store(1, 0.5, 0.3);
        assert_eq!(bank.load(0), [0.9, 0.7]);
        assert_eq!(bank.load(1), [0.5, 0.3]);
        assert_eq!(bank.load(99), [0.0, 0.0]); // out of bounds
    }

    #[test]
    fn meter_bank_read_all() {
        let bank = MeterBank::new(3);
        bank.store(0, 0.1, 0.2);
        bank.store(1, 0.3, 0.4);
        bank.store(2, 0.5, 0.6);
        let all = bank.read_all();
        assert_eq!(all.len(), 3);
        assert_eq!(all[0], [0.1, 0.2]);
        assert_eq!(all[2], [0.5, 0.6]);
    }

    #[test]
    fn meter_bank_set_active() {
        let bank = MeterBank::new(8);
        assert_eq!(bank.len(), 8);
        bank.set_active(3);
        assert_eq!(bank.len(), 3);
        let all = bank.read_all();
        assert_eq!(all.len(), 3);
    }

    #[test]
    fn shared_meter_bank_threaded() {
        let bank = shared_meter_bank(4);
        let bank2 = bank.clone();

        let handle = std::thread::spawn(move || {
            bank2.store(0, 0.99, 0.88);
        });
        handle.join().unwrap();

        let levels = bank.load(0);
        assert_eq!(levels, [0.99, 0.88]);
    }

    #[test]
    fn meter_bank_out_of_bounds_store_noop() {
        let bank = MeterBank::new(2);
        bank.store(999, 1.0, 1.0); // should not panic
        assert_eq!(bank.load(999), [0.0, 0.0]);
    }
}
