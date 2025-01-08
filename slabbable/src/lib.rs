#![warn(
    clippy::unwrap_used,
    missing_docs,
    rust_2018_idioms,
    unused_lifetimes,
    unused_qualifications
)]
#![allow(clippy::single_match, rustdoc::bare_urls)]
#![cfg_attr(all(not(feature = "std"), not(test)), no_std)]
#![doc = include_str!("../README.md")]

//! Slottable is the trait for storage slab-slotmaps for the purpose of
//! keeping the underlying request data alive when kernel is either modifying
//! or keeping a reference of the said data through raw pointers.
//!
//! This trait is opinionated into purpose of network server proramming that may
//! have high and low load periods with ramp-up and downs.
//!
//! The underlying implementation does not need to be Sync or Send and typically
//! should live within one thread only requiring no synchronisation / atomics.
//!
//! The user of the slab-slotmap must guarantee that the owned slotmap itself is
//! not dropped whilst guaranteeing the items inside are not moving or dropped.
//!
//! # Required Guarantees
//!
//! The slab-slotmap implementing this trait must:
//!
//! 1. Keep the memory addresses stable as-in self-referential structs
//! 2. Provide free slot and upon freeing the slot must be re-usable
//! 3. Lookable key by usize that can be copy-referenced without pointer access
//! 4. Must not leak memory beyond the fixed capacity max.
//! 5. Must be tested for 1-4 and documented for A-E and perhaps benchmarked.
//!
//! # Desired Properties
//!
//! A. Fast random access via key that is usize
//! B. Occupying takes a sequential usize id
//! C. Not re-using sequential usize id until it is recycled at usize::MAX
//! D; Ability to free-up memory e.g. in acses of ramp-up / down high/low loads.
//! E. Minimal memory usage for free slots
/// See module documentation of guarantees needed.
pub trait Slabbable<Slabber, T> {
    /// Error
    type Error;
    /// Provided with capacity the impl must keep the underlying T addresses stable.
    /// The capacity must be fixed and must not change.
    fn with_fixed_capacity(_: usize) -> Result<Slabber, Self::Error>;
    /// Take the next free slot, ideally with least re-used ID and return it's key ID
    fn take_next_with(&mut self, _: T) -> Result<usize, Self::Error>;
    /// Mark a given slot for re-use
    fn mark_for_reuse(&mut self, _: usize) -> Result<T, Self::Error>;
    /// Get reference of slot
    fn slot_get_ref(&self, _: usize) -> Result<Option<&T>, Self::Error>;
    /// The capacity of the slab-slotmap
    fn capacity(&self) -> usize;
    /// Remaining capacity of teh slab-slotmap
    fn remaining(&self) -> Option<usize>;
    /// Reap memory that can be freed opportunistically-optionally but keep the capacity intanct.
    /// This may mean wiping out entries at the tail and re-allocating at the end etc.
    /// Implementations that don't provide this should return None and the ones providing it
    /// must return the number of slots affected denoting the expected effetiveness of it.
    /// This is an opportunity to reap the freelist or gc in the periods that may afford slowness
    /// traded for opportunity to free up operating memory.
    fn reap(&mut self) -> Option<usize>;
}

#[cfg(test)]
mod testable;

#[cfg(test)]
mod test {
    use super::testable::*;
    use super::Slabbable;
    use rstest::rstest;

    #[repr(packed, C)]
    #[derive(Debug, Clone)]
    struct EvilCStruct {
        forever: u8,
        whatever: u16,
        yet_another: u32,
    }

    #[rstest]
    #[case(100)]
    fn test_1_impl_stable_memory_init(#[case] cap: usize) {
        let mut impl_ut = TestableSlab::<EvilCStruct>::with_fixed_capacity(cap).unwrap();
        let cap = impl_ut.capacity();
        assert!(cap > 0);
        let mut ptrs_chk = Vec::with_capacity(cap);
        for z in 0..cap {
            let slot = impl_ut
                .take_next_with(EvilCStruct {
                    forever: 0,
                    whatever: 0,
                    yet_another: 0,
                })
                .expect("oof");
            let g = impl_ut
                .slot_get_ref(slot)
                .unwrap()
                .expect(format!("slot {} was not inserted", z).as_str());
            let ptr = std::ptr::addr_of!(*g);
            ptrs_chk.push((slot, ptr));
        }
        for (slot, ptr) in ptrs_chk {
            let chk = impl_ut.slot_get_ref(slot).unwrap().unwrap();
            let chk_ptr = std::ptr::addr_of!(*chk);
            assert_eq!(ptr, chk_ptr);
        }
    }
}
