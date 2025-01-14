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
    #[derive(Debug, Clone, PartialEq)]
    struct SomeCStruct {
        forever: u8,
        whatever: u16,
        yet_another: u32,
    }

    mod slabbable_selected {
        use super::*;
        use ::slabbable_impl_selector::SelectedSlab;

        #[inline]
        fn selected_impl<T: Clone + core::fmt::Debug>(cap: usize) -> SelectedSlab<T> {
            SelectedSlab::<T>::with_fixed_capacity(cap).unwrap()
        }

        #[test]
        fn selected_1_3() {
            let mut imp = selected_impl::<SomeCStruct>(5);
            _1_3_impl_stable_memory_init(&mut imp, 5);
        }

        #[test]
        fn selected_2_3() {
            let mut imp = selected_impl::<SomeCStruct>(5);
            _2_3_impl_reserve_re_usable(&mut imp, 5);
        }
    }

    // Fill to capacity and check the stored item did not move address
    fn _1_3_impl_stable_memory_init<ImplT, Slabber>(impl_ut: &mut ImplT, cap: usize)
    where
        ImplT: core::fmt::Debug + Slabbable<Slabber, SomeCStruct>,
        <ImplT as Slabbable<Slabber, SomeCStruct>>::Error: core::fmt::Debug,
        Slabber: core::fmt::Debug,
    {
        assert!(cap > 0);
        let mut ptrs_chk = Vec::with_capacity(cap);
        for _z in 0..cap {
            let slot = impl_ut
                .take_next_with(SomeCStruct {
                    forever: 0,
                    whatever: 0,
                    yet_another: 0,
                })
                .unwrap();
            let g = impl_ut.slot_get_ref(slot).unwrap().unwrap();
            let ptr = std::ptr::addr_of!(*g);
            ptrs_chk.push((slot, ptr));
        }

        for (slot, ptr) in ptrs_chk {
            let chk = impl_ut.slot_get_ref(slot).unwrap().unwrap();
            let chk_ptr = std::ptr::addr_of!(*chk);
            assert_eq!(ptr, chk_ptr);
        }
    }

    // Reserve to capacity and check we are able to take all
    fn _2_3_impl_reserve_re_usable<ImplT, Slabber>(impl_ut: &mut ImplT, cap: usize)
    where
        ImplT: core::fmt::Debug + Slabbable<Slabber, SomeCStruct>,
        <ImplT as Slabbable<Slabber, SomeCStruct>>::Error: core::fmt::Debug,
        Slabber: core::fmt::Debug,
    {
        assert!(cap > 0);
        let mut reserve_chk = Vec::with_capacity(cap);
        for _z in 0..cap {
            reserve_chk.push(impl_ut.reserve_next().unwrap());
        }

        let mut c = 0;
        let mut taken_chk = Vec::with_capacity(cap);
        for reserved in reserve_chk {
            c += 1;
            let st = SomeCStruct {
                forever: 0,
                whatever: 0,
                yet_another: c as u32,
            };
            taken_chk.push((reserved.id(), c));
            impl_ut.take_reserved_with(reserved, st).unwrap();
        }
        for (taken_id, counter) in taken_chk {
            let chk_r = impl_ut.slot_get_ref(taken_id).unwrap().unwrap();
            let chk_st = SomeCStruct {
                forever: 0,
                whatever: 0,
                yet_another: counter as u32,
            };
            assert_eq!(chk_st, *chk_r);
        }
    }
}
