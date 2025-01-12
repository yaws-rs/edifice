#![allow(dead_code)]

use memory_stats::memory_stats;

#[repr(packed, C)]
#[derive(Debug, Clone)]
struct SomeCStruct {
    forever: u8,
    whatever: u16,
    yet_another: u32,
}

use slabbable::Slabbable;

fn fill_1m_basic<S, I: core::fmt::Debug>(slab: &mut S)
where
    S: Slabbable<I, SomeCStruct>,
    <S as Slabbable<I, SomeCStruct>>::Error: std::fmt::Debug,
{
    for _z in 0..10_024_000 {
        let _slot = slab
            .take_next_with(SomeCStruct {
                forever: 0,
                whatever: 0,
                yet_another: 0,
            })
            .unwrap();
    }
}

struct MemSnapshot {
    phys: usize,
    virt: usize,
}

fn mem_take_snapshot() -> MemSnapshot {
    if let Some(stats) = memory_stats() {
        return MemSnapshot {
            phys: stats.physical_mem,
            virt: stats.virtual_mem,
        };
    }
    panic!("infallible: Could not get memory statistics");
}

#[derive(Debug)]
enum Direction {
    Reduced(usize),
    Increased(usize),
    Same(usize),
}

#[derive(Debug)]
struct MemReport {
    phys: Direction,
    virt: Direction,
}

fn mem_cmp_to(was: &MemSnapshot) -> MemReport {
    let cur = mem_take_snapshot();

    let phys = if cur.phys > was.phys {
        Direction::Increased(cur.phys - was.phys)
    } else if cur.phys == was.phys {
        Direction::Same(cur.phys)
    } else {
        Direction::Reduced(was.phys - cur.phys)
    };

    let virt = if cur.virt > was.virt {
        Direction::Increased(cur.virt - was.virt)
    } else if cur.virt == was.virt {
        Direction::Same(cur.virt)
    } else {
        Direction::Reduced(was.virt - cur.virt)
    };

    MemReport { phys, virt }
}

use humansize::{format_size, DECIMAL};

fn fmt_direction(d: &Direction) -> String {
    match d {
        Direction::Reduced(v) => format!("-{}", format_size(*v as u64, DECIMAL)),
        Direction::Increased(v) => format!("+{}", format_size(*v as u64, DECIMAL)),
        Direction::Same(v) => format!("={}", format_size(*v as u64, DECIMAL)),
    }
}

fn print_mem_report(stage: String, rep: &MemReport) {
    let phys = fmt_direction(&rep.phys);
    let virt = fmt_direction(&rep.virt);
    println!("== {stage}\n - phys {phys} virt {virt}");
}

fn run_errand<S, I: core::fmt::Debug>(info: &'static str, slab: &mut S)
where
    S: Slabbable<I, SomeCStruct>,
    <S as Slabbable<I, SomeCStruct>>::Error: std::fmt::Debug,
{
    let baseline = mem_take_snapshot();
    print_mem_report(
        format!("{} / initialized (baseline)", info),
        &mem_cmp_to(&baseline),
    );
    fill_1m_basic(slab);
    print_mem_report(
        format!("{} / flled (over baseline)", info),
        &mem_cmp_to(&baseline),
    );
}

#[cfg(feature = "slabbable-nohash-hasher")]
fn nohash_hasher() {
    let mut slab =
        slabbable_nohash_hasher::NoHashSlab::<SomeCStruct>::with_fixed_capacity(10_024_000)
            .unwrap();
    run_errand("nohash-hasher", &mut slab);
}

#[cfg(feature = "slabbable-slab")]
fn slab() {
    let mut slab =
        slabbable_slab::SlabSlab::<SomeCStruct>::with_fixed_capacity(10_024_000).unwrap();
    run_errand("slab", &mut slab);
}

#[cfg(feature = "slabbable-stablevec")]
fn stablevec() {
    let mut slab =
        slabbable_stablevec::StableVecSlab::<SomeCStruct>::with_fixed_capacity(10_024_000).unwrap();
    run_errand("StableVec", &mut slab);
}

fn main() {
    #[cfg(feature = "slabbable-nohash-hasher")]
    nohash_hasher();

    #[cfg(feature = "slabbable-slab")]
    slab();

    #[cfg(feature = "slabbable-stablevec")]
    stablevec();
}
