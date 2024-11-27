#![allow(internal_features)]
#![feature(test)] // enable benchmarks

use rand::prelude::*;
use rayon::prelude::*;

macro_rules! assembly_algorithm {
    ($i:ident, $sum:ident, $value:ident) => {
        #[allow(unused_assignments, reason = "used in asm block")]
        unsafe {
            std::arch::asm!(
                // b = a >> 32;
                "mov {b}, {a}",
                "shr {b}, 32",

                // a &= b;
                "and {a}, {b}",

                // if i == 0 {
                //     a &= 0b1111111
                // }
                "test {i}, {i}",
                "je 2f",
                "and {a}, 0x7f",
                "2:",

                // sum += a.count_ones()
                "popcnt {a}, {a}",
                "add {sum}, {a}",

                a = inout(reg) $value,
                b = out(reg) _,
                i = inout(reg) $i,
                sum = inout(reg) $sum,
                //options(nomem, pure, nostack),
            )
        }
    };
}

/// calculates the number of turns lost out of `tries` attempts
fn calculate_attempt(rng: &mut ThreadRng) -> u64 {
    let mut turns_lost: u64 = 0;
    for mut i in 0u64..8 {
        let mut n: u64 = rng.gen();
        // Assembly is faster for some reason
        assembly_algorithm!(i, n, turns_lost);
    }
    turns_lost
}

#[inline(never)]
/// Uses rayon to calculate iterations in parallel
fn calculate_odds(iterations: usize) -> u64 {
    if iterations == 0 {
        return 0;
    }
    (0..iterations)
        .into_par_iter()
        .map_init(rand::thread_rng, |rand, _| calculate_attempt(rand))
        .max()
        .unwrap_or(0)
}

/// run with `cargo run --release` for optimizations
fn main() {
    println!("{}", calculate_odds(1_000_000));
}

#[cfg(test)]
mod tests {
    extern crate test;

    use test::Bencher;

    use super::*;

    #[bench]
    /// Results on my machine:
    /// Ryzen 9 7900
    /// 12 cores
    /// 24 threads
    /// base clock 3.7 Ghz
    /// boost clock 5.4 Ghz
    /// 1,283,788.75 ns/iter (+/- 38,464.50)
    fn bench(bencher: &mut Bencher) {
        bencher.iter(|| calculate_odds(1_000_000))
    }

    #[test]
    fn assembly_is_zero() {
        let a = 0;
        let b = 1;
        let mut a_is_zero = 0;
        let mut b_is_zero = 0;

        unsafe {
            std::arch::asm!(
                "test {a:e}, {a:e}",
                "jne 2f",
                "mov {a_is_zero:e}, 1",
                "2:",
                "test {b:e}, {b:e}",
                "jne 3f",
                "mov {b_is_zero:e}, 1",
                "3:",
                a = in(reg) a,
                b = in(reg) b,
                a_is_zero = inout(reg) a_is_zero,
                b_is_zero = inout(reg) b_is_zero,
            )
        }

        assert!(a_is_zero == 1);
        assert!(b_is_zero == 0);
    }

    #[test]
    fn verify_assembly() {
        let values = [
            0x00000000_00000000u64,
            0xffffffff_ffffffff,
            0x0000007f_0000007f,
            0xffffffff_ffffff7f,
            0xFFFF_FF80_FFFF_FF80,
            0xffff_ffff_0000_0000,
            0x0000_0000_ffff_ffff,
        ];
        let expected = [
            (0u64, 0u64),
            (32, 7),
            (7, 7),
            (31, 7),
            (25, 0),
            (0, 0),
            (0, 0),
        ];
        for (v, (high, low)) in values.into_iter().zip(expected) {
            let mut sum = 0;
            let mut zero = 0u64;
            let mut one = 1u64;
            let mut o = v;
            assembly_algorithm!(one, sum, o);
            assert_eq!(sum, low, "{v:0X}");
            sum = 0;
            o = v;
            assembly_algorithm!(zero, sum, o);
            assert_eq!(sum, high, "{v:0X}");
        }
    }
}
