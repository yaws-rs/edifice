use criterion::{black_box, criterion_group, criterion_main, Criterion};

#[repr(packed, C)]
#[derive(Debug, Clone)]
struct SomeCStruct {
    forever: u8,
    whatever: u16,
    yet_another: u32,
}

use slabbable::Slabbable;

fn criterion_benchmark(c: &mut Criterion) {
    //    #[cfg(feature = "slabbable-slab")]
    //    let mut imp = SlabSlab::<SomeCStruct>::with_fixed_capacity(1024).unwrap();

    //    #[cfg(feature = "slabbable-stablevec")]
    //    let mut imp = StableVecSlab::<SomeCStruct>::with_fixed_capacity(1024).unwrap();

    #[cfg(feature = "slabbable-nohash-hasher")]
    c.bench_function("nohash-hasher 1,024,000 insert", |b| {
        b.iter(|| {
            let mut imp =
                slabbable_nohash_hasher::NoHashSlab::<SomeCStruct>::with_fixed_capacity(1_024_000)
                    .unwrap();
            for _z in 0..1_024_000 {
                let _slot = imp
                    .take_next_with(black_box(SomeCStruct {
                        forever: 0,
                        whatever: 0,
                        yet_another: 0,
                    }))
                    .unwrap();
            }
        })
    });

    #[cfg(feature = "slabbable-nohash-hasher")]
    c.bench_function("nohash-hasher get the 512,000 th of 1,024,000", |b| {
        let mut imp =
            slabbable_nohash_hasher::NoHashSlab::<SomeCStruct>::with_fixed_capacity(1_024_000)
                .unwrap();
        for _z in 0..1_024_000 {
            let _slot = imp
                .take_next_with(black_box(SomeCStruct {
                    forever: 0,
                    whatever: 0,
                    yet_another: 0,
                }))
                .unwrap();
        }
        b.iter(|| {
            black_box(imp.slot_get_ref(512_000).unwrap());
        })
    });

    /*
    c.bench_function(
        "nohash-hasher 1024x get-1024th",
        |b| b.iter(|| {

        })
    );

    c.bench_function(
        "nohash-hasher 1024x remove-1024th",
        |b| b.iter(|| {

        })
    );
    */
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
