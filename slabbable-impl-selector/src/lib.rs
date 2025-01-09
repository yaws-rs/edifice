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

#[cfg(all(slabbable_impl = "stablevec", any(slabbable_impl = "slab", slabbable_impl = "hash")))]
compile_error!("slabbable-impl_selector: must not choose stablevec with anything else");

#[cfg(all(slabbable_impl = "slab", any(slabbable_impl = "stablevec", slabbable_impl = "hash")))]
compile_error!("slabbable-impl_selector: must not choose slab with anything else");

#[cfg(all(slabbable_impl = "hash", any(slabbable_impl = "slab", slabbable_impl = "stablevec")))]
compile_error!("slabbable-impl_selector: must not choose hash with anything else");

cfg_if::cfg_if! {

    if #[cfg(slabbable_impl = "stablevec")] {
        /// Selected impl is StableVec
        pub type SelectedSlab<Item> = slabbable_stablevec::StableVecSlab<Item>;
    } else if #[cfg(slabbable_impl = "slab")] {
        /// Selected impl is Slab
        pub type SelectedSlab<Item> = slabbable_slab::SlabSlab<Item>;
    } else if  #[cfg(slabbable_impl = "hash")] {
        /// Selected impl is Hash
        pub type SelectedSlab<Item> = slabbable_hash::HashSlab<Item>;
    }
}
