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

//! Slabbable is the trait for storage slab-slotmaps for the purpose of
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
    /// Reserve the next free slot, ideally with least re-used ID and return it's key ID
    fn reserve_next(&mut self) -> Result<ReservedSlot, Self::Error>;
    /// Take the previously reserved slot
    fn take_reserved_with(&mut self, _: ReservedSlot, _: T) -> Result<usize, Self::Error>;
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

/// Reserved marked for any slot that can be taken later.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ReservedSlot {
    taken: usize,
    _priv: (),
}

impl ReservedSlot {
    /// Provide a new ReservedSlot.
    ///
    /// # Warning
    ///
    /// This is not intended to be used from code that uses one of the implementations.
    /// This is solely used when implementing Slabbable trait for reservation functionality.
    pub fn issue(id: usize) -> Self {
        Self {
            taken: id,
            _priv: (),
        }
    }
    /// Id of the resered slot in case this is needed before taking the actual slot.
    pub fn id(&self) -> usize {
        self.taken
    }
}

mod error;
/// All implementations should use the harmonized error type.
#[doc(inline)]
pub use error::SlabbableError;

#[cfg(test)]
mod testable;

#[cfg(test)]
mod test {
    use super::testable::*;
    use super::Slabbable;
    use rstest::rstest;

    #[repr(packed, C)]
    #[derive(Debug, Clone)]
    struct SomeCStruct {
        forever: u8,
        whatever: u16,
        yet_another: u32,
    }

    #[rstest]
    #[case(TestableSlab::<SomeCStruct>::with_fixed_capacity(10).unwrap())]
    #[case(TestableSlab::<SomeCStruct>::with_fixed_capacity(100).unwrap())]
    fn test_1_impl_stable_memory_init<ImplT, Slabber>(#[case] impl_ut_t: ImplT)
    where
        ImplT: core::fmt::Debug + Slabbable<Slabber, SomeCStruct>,
        Slabber: core::fmt::Debug,
    {
        let mut impl_ut = impl_ut_t;
        let cap = impl_ut.capacity();

        let mut ptrs_chk = Vec::with_capacity(cap);
        for _z in 0..cap {
            let slot = impl_ut.take_next_with(SomeCStruct {
                forever: 0,
                whatever: 0,
                yet_another: 0,
            });
            let slot = match slot {
                Ok(slot) => slot,
                _ => panic!("Could not take slot"),
            };
            let g = impl_ut.slot_get_ref(slot);
            let g = match g {
                Ok(g) => g,
                _ => panic!("Error with slot_get_ref(slot)"),
            };
            let g = if let Some(g) = g {
                g
            } else {
                panic!("Error finding reference back with slot_get_ref(slot)");
            };
            let ptr = std::ptr::addr_of!(*g);
            ptrs_chk.push((slot, ptr));
        }
        for (slot, ptr) in ptrs_chk {
            let chk = match impl_ut.slot_get_ref(slot) {
                Ok(Some(chk)) => chk,
                _ => panic!("Error with slot_get_ref(slot)"),
            };
            let chk_ptr = std::ptr::addr_of!(*chk);
            assert_eq!(ptr, chk_ptr);
        }
    }
}
