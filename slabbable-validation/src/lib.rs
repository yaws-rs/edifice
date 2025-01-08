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

#[cfg(test)]
mod test {
    use slabbable::Slabbable;

    #[repr(packed, C)]
    #[derive(Debug, Clone)]
    struct SomeCStruct {
        forever: u8,
        whatever: u16,
        yet_another: u32,
    }

    #[cfg(feature = "slabbable-stablevec")]
    mod slabbable_stablevec {
        use super::*;
        use ::slabbable_stablevec::StableVecSlab;

        #[test]
        fn stable_vec() {
            let mut imp = StableVecSlab::<SomeCStruct>::with_fixed_capacity(5).unwrap();
            _1_impl_stable_memory_init(&mut imp)
        }
    }

    fn _1_impl_stable_memory_init<ImplT, Slabber>(impl_ut: &mut ImplT)
    where
        ImplT: core::fmt::Debug + Slabbable<Slabber, SomeCStruct>,
        Slabber: core::fmt::Debug,
    {
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
