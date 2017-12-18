use integer::Integer;
use {BigDigit, BigUint, ToBigUint, big_digit};
use {BigInt, RandBigInt, ToBigInt};
use Sign::Plus;

use std::cmp::Ordering::{Less, Equal, Greater};
use std::{f32, f64};
use std::i64;
use std::iter::repeat;
use std::str::FromStr;
use std::{u8, u16, u32, u64, usize};

use rand::thread_rng;
use traits::{Num, Zero, One, CheckedAdd, CheckedSub, CheckedMul, CheckedDiv, ToPrimitive,
             FromPrimitive, Float};


/// Assert that an op works for all val/ref combinations
macro_rules! assert_op {
    ($left:ident $op:tt $right:ident == $expected:expr) => {
        assert_eq!((&$left) $op (&$right), $expected);
        assert_eq!((&$left) $op $right.clone(), $expected);
        assert_eq!($left.clone() $op (&$right), $expected);
        assert_eq!($left.clone() $op $right.clone(), $expected);
    };
}
/// Assert that an assign-op works for all val/ref combinations
macro_rules! assert_assign_op {
    ($left:ident $op:tt $right:ident == $expected:expr) => {
        {
            let mut tmp12384 = $left.clone();
            assert_eq!({ tmp12384 $op &$right; tmp12384}, $expected);

            let mut tmp12384 = $left.clone();
            assert_eq!({ tmp12384 $op $right.clone(); tmp12384}, $expected);
        }
    };
}

/// Assert that an op works for scalar left or right
macro_rules! assert_scalar_op {
    (($($to:ident),*) $left:ident $op:tt $right:ident == $expected:expr) => {
        $(
            if let Some(left) = $left.$to() {
                assert_op!(left $op $right == $expected);
            }
            if let Some(right) = $right.$to() {
                assert_op!($left $op right == $expected);
            }
        )*
    };
    ($left:ident $op:tt $right:ident == $expected:expr) => {
        assert_scalar_op!((to_u8, to_u16, to_u32, to_u64, to_usize)
                          $left $op $right == $expected);
    };
}

#[test]
fn test_from_slice() {
    fn check(slice: &[BigDigit], data: &[BigDigit]) {
        assert!(BigUint::from_slice(slice).data == data);
    }
    check(&[1], &[1]);
    check(&[0, 0, 0], &[]);
    check(&[1, 2, 0, 0], &[1, 2]);
    check(&[0, 0, 1, 2], &[0, 0, 1, 2]);
    check(&[0, 0, 1, 2, 0, 0], &[0, 0, 1, 2]);
    check(&[-1i32 as BigDigit], &[-1i32 as BigDigit]);
}

#[test]
fn test_assign_from_slice() {
    fn check(slice: &[BigDigit], data: &[BigDigit]) {
        let mut p = BigUint::from_slice(&[2627_u32, 0_u32, 9182_u32, 42_u32]);
        p.assign_from_slice(slice);
        assert!(p.data == data);
    }
    check(&[1], &[1]);
    check(&[0, 0, 0], &[]);
    check(&[1, 2, 0, 0], &[1, 2]);
    check(&[0, 0, 1, 2], &[0, 0, 1, 2]);
    check(&[0, 0, 1, 2, 0, 0], &[0, 0, 1, 2]);
    check(&[-1i32 as BigDigit], &[-1i32 as BigDigit]);
}

#[test]
fn test_from_bytes_be() {
    fn check(s: &str, result: &str) {
        assert_eq!(BigUint::from_bytes_be(s.as_bytes()),
                   BigUint::parse_bytes(result.as_bytes(), 10).unwrap());
    }
    check("A", "65");
    check("AA", "16705");
    check("AB", "16706");
    check("Hello world!", "22405534230753963835153736737");
    assert_eq!(BigUint::from_bytes_be(&[]), Zero::zero());
}

#[test]
fn test_to_bytes_be() {
    fn check(s: &str, result: &str) {
        let b = BigUint::parse_bytes(result.as_bytes(), 10).unwrap();
        assert_eq!(b.to_bytes_be(), s.as_bytes());
    }
    check("A", "65");
    check("AA", "16705");
    check("AB", "16706");
    check("Hello world!", "22405534230753963835153736737");
    let b: BigUint = Zero::zero();
    assert_eq!(b.to_bytes_be(), [0]);

    // Test with leading/trailing zero bytes and a full BigDigit of value 0
    let b = BigUint::from_str_radix("00010000000000000200", 16).unwrap();
    assert_eq!(b.to_bytes_be(), [1, 0, 0, 0, 0, 0, 0, 2, 0]);
}

#[test]
fn test_from_bytes_le() {
    fn check(s: &str, result: &str) {
        assert_eq!(BigUint::from_bytes_le(s.as_bytes()),
                   BigUint::parse_bytes(result.as_bytes(), 10).unwrap());
    }
    check("A", "65");
    check("AA", "16705");
    check("BA", "16706");
    check("!dlrow olleH", "22405534230753963835153736737");
    assert_eq!(BigUint::from_bytes_le(&[]), Zero::zero());
}

#[test]
fn test_to_bytes_le() {
    fn check(s: &str, result: &str) {
        let b = BigUint::parse_bytes(result.as_bytes(), 10).unwrap();
        assert_eq!(b.to_bytes_le(), s.as_bytes());
    }
    check("A", "65");
    check("AA", "16705");
    check("BA", "16706");
    check("!dlrow olleH", "22405534230753963835153736737");
    let b: BigUint = Zero::zero();
    assert_eq!(b.to_bytes_le(), [0]);

    // Test with leading/trailing zero bytes and a full BigDigit of value 0
    let b = BigUint::from_str_radix("00010000000000000200", 16).unwrap();
    assert_eq!(b.to_bytes_le(), [0, 2, 0, 0, 0, 0, 0, 0, 1]);
}

#[test]
fn test_cmp() {
    let data: [&[_]; 7] = [&[], &[1], &[2], &[!0], &[0, 1], &[2, 1], &[1, 1, 1]];
    let data: Vec<BigUint> = data.iter().map(|v| BigUint::from_slice(*v)).collect();
    for (i, ni) in data.iter().enumerate() {
        for (j0, nj) in data[i..].iter().enumerate() {
            let j = j0 + i;
            if i == j {
                assert_eq!(ni.cmp(nj), Equal);
                assert_eq!(nj.cmp(ni), Equal);
                assert_eq!(ni, nj);
                assert!(!(ni != nj));
                assert!(ni <= nj);
                assert!(ni >= nj);
                assert!(!(ni < nj));
                assert!(!(ni > nj));
            } else {
                assert_eq!(ni.cmp(nj), Less);
                assert_eq!(nj.cmp(ni), Greater);

                assert!(!(ni == nj));
                assert!(ni != nj);

                assert!(ni <= nj);
                assert!(!(ni >= nj));
                assert!(ni < nj);
                assert!(!(ni > nj));

                assert!(!(nj <= ni));
                assert!(nj >= ni);
                assert!(!(nj < ni));
                assert!(nj > ni);
            }
        }
    }
}

#[test]
fn test_hash() {
    use hash;

    let a = BigUint::new(vec![]);
    let b = BigUint::new(vec![0]);
    let c = BigUint::new(vec![1]);
    let d = BigUint::new(vec![1, 0, 0, 0, 0, 0]);
    let e = BigUint::new(vec![0, 0, 0, 0, 0, 1]);
    assert!(hash(&a) == hash(&b));
    assert!(hash(&b) != hash(&c));
    assert!(hash(&c) == hash(&d));
    assert!(hash(&d) != hash(&e));
}

const BIT_TESTS: &'static [(&'static [BigDigit],
           &'static [BigDigit],
           &'static [BigDigit],
           &'static [BigDigit],
           &'static [BigDigit])] = &[// LEFT              RIGHT        AND          OR                XOR
                                     (&[], &[], &[], &[], &[]),
                                     (&[1, 0, 1], &[1, 1], &[1], &[1, 1, 1], &[0, 1, 1]),
                                     (&[1, 0, 1], &[0, 1, 1], &[0, 0, 1], &[1, 1, 1], &[1, 1]),
                                     (&[268, 482, 17],
                                      &[964, 54],
                                      &[260, 34],
                                      &[972, 502, 17],
                                      &[712, 468, 17])];

#[test]
fn test_bitand() {
    for elm in BIT_TESTS {
        let (a_vec, b_vec, c_vec, _, _) = *elm;
        let a = BigUint::from_slice(a_vec);
        let b = BigUint::from_slice(b_vec);
        let c = BigUint::from_slice(c_vec);

        assert_op!(a & b == c);
        assert_op!(b & a == c);
        assert_assign_op!(a &= b == c);
        assert_assign_op!(b &= a == c);
    }
}

#[test]
fn test_bitor() {
    for elm in BIT_TESTS {
        let (a_vec, b_vec, _, c_vec, _) = *elm;
        let a = BigUint::from_slice(a_vec);
        let b = BigUint::from_slice(b_vec);
        let c = BigUint::from_slice(c_vec);

        assert_op!(a | b == c);
        assert_op!(b | a == c);
        assert_assign_op!(a |= b == c);
        assert_assign_op!(b |= a == c);
    }
}

#[test]
fn test_bitxor() {
    for elm in BIT_TESTS {
        let (a_vec, b_vec, _, _, c_vec) = *elm;
        let a = BigUint::from_slice(a_vec);
        let b = BigUint::from_slice(b_vec);
        let c = BigUint::from_slice(c_vec);

        assert_op!(a ^ b == c);
        assert_op!(b ^ a == c);
        assert_op!(a ^ c == b);
        assert_op!(c ^ a == b);
        assert_op!(b ^ c == a);
        assert_op!(c ^ b == a);
        assert_assign_op!(a ^= b == c);
        assert_assign_op!(b ^= a == c);
        assert_assign_op!(a ^= c == b);
        assert_assign_op!(c ^= a == b);
        assert_assign_op!(b ^= c == a);
        assert_assign_op!(c ^= b == a);
    }
}

#[test]
fn test_shl() {
    fn check(s: &str, shift: usize, ans: &str) {
        let opt_biguint = BigUint::from_str_radix(s, 16).ok();
        let mut bu_assign = opt_biguint.unwrap();
        let bu = (bu_assign.clone() << shift).to_str_radix(16);
        assert_eq!(bu, ans);
        bu_assign <<= shift;
        assert_eq!(bu_assign.to_str_radix(16), ans);
    }

    check("0", 3, "0");
    check("1", 3, "8");

    check("1\
           0000\
           0000\
           0000\
           0001\
           0000\
           0000\
           0000\
           0001",
          3,
          "8\
           0000\
           0000\
           0000\
           0008\
           0000\
           0000\
           0000\
           0008");
    check("1\
           0000\
           0001\
           0000\
           0001",
          2,
          "4\
           0000\
           0004\
           0000\
           0004");
    check("1\
           0001\
           0001",
          1,
          "2\
           0002\
           0002");

    check("\
          4000\
          0000\
          0000\
          0000",
          3,
          "2\
          0000\
          0000\
          0000\
          0000");
    check("4000\
          0000",
          2,
          "1\
          0000\
          0000");
    check("4000",
          2,
          "1\
          0000");

    check("4000\
          0000\
          0000\
          0000",
          67,
          "2\
          0000\
          0000\
          0000\
          0000\
          0000\
          0000\
          0000\
          0000");
    check("4000\
          0000",
          35,
          "2\
          0000\
          0000\
          0000\
          0000");
    check("4000",
          19,
          "2\
          0000\
          0000");

    check("fedc\
          ba98\
          7654\
          3210\
          fedc\
          ba98\
          7654\
          3210",
          4,
          "f\
          edcb\
          a987\
          6543\
          210f\
          edcb\
          a987\
          6543\
          2100");
    check("88887777666655554444333322221111",
          16,
          "888877776666555544443333222211110000");
}

#[test]
fn test_shr() {
    fn check(s: &str, shift: usize, ans: &str) {
        let opt_biguint = BigUint::from_str_radix(s, 16).ok();
        let mut bu_assign = opt_biguint.unwrap();
        let bu = (bu_assign.clone() >> shift).to_str_radix(16);
        assert_eq!(bu, ans);
        bu_assign >>= shift;
        assert_eq!(bu_assign.to_str_radix(16), ans);
    }

    check("0", 3, "0");
    check("f", 3, "1");

    check("1\
          0000\
          0000\
          0000\
          0001\
          0000\
          0000\
          0000\
          0001",
          3,
          "2000\
          0000\
          0000\
          0000\
          2000\
          0000\
          0000\
          0000");
    check("1\
          0000\
          0001\
          0000\
          0001",
          2,
          "4000\
          0000\
          4000\
          0000");
    check("1\
          0001\
          0001",
          1,
          "8000\
          8000");

    check("2\
          0000\
          0000\
          0000\
          0001\
          0000\
          0000\
          0000\
          0001",
          67,
          "4000\
          0000\
          0000\
          0000");
    check("2\
          0000\
          0001\
          0000\
          0001",
          35,
          "4000\
          0000");
    check("2\
          0001\
          0001",
          19,
          "4000");

    check("1\
          0000\
          0000\
          0000\
          0000",
          1,
          "8000\
          0000\
          0000\
          0000");
    check("1\
          0000\
          0000",
          1,
          "8000\
          0000");
    check("1\
          0000",
          1,
          "8000");
    check("f\
          edcb\
          a987\
          6543\
          210f\
          edcb\
          a987\
          6543\
          2100",
          4,
          "fedc\
          ba98\
          7654\
          3210\
          fedc\
          ba98\
          7654\
          3210");

    check("888877776666555544443333222211110000",
          16,
          "88887777666655554444333322221111");
}

const N1: BigDigit = -1i32 as BigDigit;
const N2: BigDigit = -2i32 as BigDigit;

// `DoubleBigDigit` size dependent
#[test]
fn test_convert_i64() {
    fn check(b1: BigUint, i: i64) {
        let b2: BigUint = FromPrimitive::from_i64(i).unwrap();
        assert_eq!(b1, b2);
        assert_eq!(b1.to_i64().unwrap(), i);
    }

    check(Zero::zero(), 0);
    check(One::one(), 1);
    check(i64::MAX.to_biguint().unwrap(), i64::MAX);

    check(BigUint::new(vec![]), 0);
    check(BigUint::new(vec![1]), (1 << (0 * big_digit::BITS)));
    check(BigUint::new(vec![N1]), (1 << (1 * big_digit::BITS)) - 1);
    check(BigUint::new(vec![0, 1]), (1 << (1 * big_digit::BITS)));
    check(BigUint::new(vec![N1, N1 >> 1]), i64::MAX);

    assert_eq!(i64::MIN.to_biguint(), None);
    assert_eq!(BigUint::new(vec![N1, N1]).to_i64(), None);
    assert_eq!(BigUint::new(vec![0, 0, 1]).to_i64(), None);
    assert_eq!(BigUint::new(vec![N1, N1, N1]).to_i64(), None);
}

// `DoubleBigDigit` size dependent
#[test]
fn test_convert_u64() {
    fn check(b1: BigUint, u: u64) {
        let b2: BigUint = FromPrimitive::from_u64(u).unwrap();
        assert_eq!(b1, b2);
        assert_eq!(b1.to_u64().unwrap(), u);
    }

    check(Zero::zero(), 0);
    check(One::one(), 1);
    check(u64::MIN.to_biguint().unwrap(), u64::MIN);
    check(u64::MAX.to_biguint().unwrap(), u64::MAX);

    check(BigUint::new(vec![]), 0);
    check(BigUint::new(vec![1]), (1 << (0 * big_digit::BITS)));
    check(BigUint::new(vec![N1]), (1 << (1 * big_digit::BITS)) - 1);
    check(BigUint::new(vec![0, 1]), (1 << (1 * big_digit::BITS)));
    check(BigUint::new(vec![N1, N1]), u64::MAX);

    assert_eq!(BigUint::new(vec![0, 0, 1]).to_u64(), None);
    assert_eq!(BigUint::new(vec![N1, N1, N1]).to_u64(), None);
}

#[test]
fn test_convert_f32() {
    fn check(b1: &BigUint, f: f32) {
        let b2 = BigUint::from_f32(f).unwrap();
        assert_eq!(b1, &b2);
        assert_eq!(b1.to_f32().unwrap(), f);
    }

    check(&BigUint::zero(), 0.0);
    check(&BigUint::one(), 1.0);
    check(&BigUint::from(u16::MAX), 2.0.powi(16) - 1.0);
    check(&BigUint::from(1u64 << 32), 2.0.powi(32));
    check(&BigUint::from_slice(&[0, 0, 1]), 2.0.powi(64));
    check(&((BigUint::one() << 100) + (BigUint::one() << 123)),
          2.0.powi(100) + 2.0.powi(123));
    check(&(BigUint::one() << 127), 2.0.powi(127));
    check(&(BigUint::from((1u64 << 24) - 1) << (128 - 24)), f32::MAX);

    // keeping all 24 digits with the bits at different offsets to the BigDigits
    let x: u32 = 0b00000000101111011111011011011101;
    let mut f = x as f32;
    let mut b = BigUint::from(x);
    for _ in 0..64 {
        check(&b, f);
        f *= 2.0;
        b = b << 1;
    }

    // this number when rounded to f64 then f32 isn't the same as when rounded straight to f32
    let n: u64 = 0b0000000000111111111111111111111111011111111111111111111111111111;
    assert!((n as f64) as f32 != n as f32);
    assert_eq!(BigUint::from(n).to_f32(), Some(n as f32));

    // test rounding up with the bits at different offsets to the BigDigits
    let mut f = ((1u64 << 25) - 1) as f32;
    let mut b = BigUint::from(1u64 << 25);
    for _ in 0..64 {
        assert_eq!(b.to_f32(), Some(f));
        f *= 2.0;
        b = b << 1;
    }

    // rounding
    assert_eq!(BigUint::from_f32(-1.0), None);
    assert_eq!(BigUint::from_f32(-0.99999), Some(BigUint::zero()));
    assert_eq!(BigUint::from_f32(-0.5), Some(BigUint::zero()));
    assert_eq!(BigUint::from_f32(-0.0), Some(BigUint::zero()));
    assert_eq!(BigUint::from_f32(f32::MIN_POSITIVE / 2.0),
               Some(BigUint::zero()));
    assert_eq!(BigUint::from_f32(f32::MIN_POSITIVE), Some(BigUint::zero()));
    assert_eq!(BigUint::from_f32(0.5), Some(BigUint::zero()));
    assert_eq!(BigUint::from_f32(0.99999), Some(BigUint::zero()));
    assert_eq!(BigUint::from_f32(f32::consts::E), Some(BigUint::from(2u32)));
    assert_eq!(BigUint::from_f32(f32::consts::PI),
               Some(BigUint::from(3u32)));

    // special float values
    assert_eq!(BigUint::from_f32(f32::NAN), None);
    assert_eq!(BigUint::from_f32(f32::INFINITY), None);
    assert_eq!(BigUint::from_f32(f32::NEG_INFINITY), None);
    assert_eq!(BigUint::from_f32(f32::MIN), None);

    // largest BigUint that will round to a finite f32 value
    let big_num = (BigUint::one() << 128) - BigUint::one() - (BigUint::one() << (128 - 25));
    assert_eq!(big_num.to_f32(), Some(f32::MAX));
    assert_eq!((big_num + BigUint::one()).to_f32(), None);

    assert_eq!(((BigUint::one() << 128) - BigUint::one()).to_f32(), None);
    assert_eq!((BigUint::one() << 128).to_f32(), None);
}

#[test]
fn test_convert_f64() {
    fn check(b1: &BigUint, f: f64) {
        let b2 = BigUint::from_f64(f).unwrap();
        assert_eq!(b1, &b2);
        assert_eq!(b1.to_f64().unwrap(), f);
    }

    check(&BigUint::zero(), 0.0);
    check(&BigUint::one(), 1.0);
    check(&BigUint::from(u32::MAX), 2.0.powi(32) - 1.0);
    check(&BigUint::from(1u64 << 32), 2.0.powi(32));
    check(&BigUint::from_slice(&[0, 0, 1]), 2.0.powi(64));
    check(&((BigUint::one() << 100) + (BigUint::one() << 152)),
          2.0.powi(100) + 2.0.powi(152));
    check(&(BigUint::one() << 1023), 2.0.powi(1023));
    check(&(BigUint::from((1u64 << 53) - 1) << (1024 - 53)), f64::MAX);

    // keeping all 53 digits with the bits at different offsets to the BigDigits
    let x: u64 = 0b0000000000011110111110110111111101110111101111011111011011011101;
    let mut f = x as f64;
    let mut b = BigUint::from(x);
    for _ in 0..128 {
        check(&b, f);
        f *= 2.0;
        b = b << 1;
    }

    // test rounding up with the bits at different offsets to the BigDigits
    let mut f = ((1u64 << 54) - 1) as f64;
    let mut b = BigUint::from(1u64 << 54);
    for _ in 0..128 {
        assert_eq!(b.to_f64(), Some(f));
        f *= 2.0;
        b = b << 1;
    }

    // rounding
    assert_eq!(BigUint::from_f64(-1.0), None);
    assert_eq!(BigUint::from_f64(-0.99999), Some(BigUint::zero()));
    assert_eq!(BigUint::from_f64(-0.5), Some(BigUint::zero()));
    assert_eq!(BigUint::from_f64(-0.0), Some(BigUint::zero()));
    assert_eq!(BigUint::from_f64(f64::MIN_POSITIVE / 2.0),
               Some(BigUint::zero()));
    assert_eq!(BigUint::from_f64(f64::MIN_POSITIVE), Some(BigUint::zero()));
    assert_eq!(BigUint::from_f64(0.5), Some(BigUint::zero()));
    assert_eq!(BigUint::from_f64(0.99999), Some(BigUint::zero()));
    assert_eq!(BigUint::from_f64(f64::consts::E), Some(BigUint::from(2u32)));
    assert_eq!(BigUint::from_f64(f64::consts::PI),
               Some(BigUint::from(3u32)));

    // special float values
    assert_eq!(BigUint::from_f64(f64::NAN), None);
    assert_eq!(BigUint::from_f64(f64::INFINITY), None);
    assert_eq!(BigUint::from_f64(f64::NEG_INFINITY), None);
    assert_eq!(BigUint::from_f64(f64::MIN), None);

    // largest BigUint that will round to a finite f64 value
    let big_num = (BigUint::one() << 1024) - BigUint::one() - (BigUint::one() << (1024 - 54));
    assert_eq!(big_num.to_f64(), Some(f64::MAX));
    assert_eq!((big_num + BigUint::one()).to_f64(), None);

    assert_eq!(((BigInt::one() << 1024) - BigInt::one()).to_f64(), None);
    assert_eq!((BigUint::one() << 1024).to_f64(), None);
}

#[test]
fn test_convert_to_bigint() {
    fn check(n: BigUint, ans: BigInt) {
        assert_eq!(n.to_bigint().unwrap(), ans);
        assert_eq!(n.to_bigint().unwrap().to_biguint().unwrap(), n);
    }
    check(Zero::zero(), Zero::zero());
    check(BigUint::new(vec![1, 2, 3]),
          BigInt::from_biguint(Plus, BigUint::new(vec![1, 2, 3])));
}

#[test]
fn test_convert_from_uint() {
    macro_rules! check {
        ($ty:ident, $max:expr) => {
            assert_eq!(BigUint::from($ty::zero()), BigUint::zero());
            assert_eq!(BigUint::from($ty::one()), BigUint::one());
            assert_eq!(BigUint::from($ty::MAX - $ty::one()), $max - BigUint::one());
            assert_eq!(BigUint::from($ty::MAX), $max);
        }
    }

    check!(u8, BigUint::from_slice(&[u8::MAX as BigDigit]));
    check!(u16, BigUint::from_slice(&[u16::MAX as BigDigit]));
    check!(u32, BigUint::from_slice(&[u32::MAX]));
    check!(u64, BigUint::from_slice(&[u32::MAX, u32::MAX]));
    check!(usize, BigUint::from(usize::MAX as u64));
}

const SUM_TRIPLES: &'static [(&'static [BigDigit],
           &'static [BigDigit],
           &'static [BigDigit])] = &[(&[], &[], &[]),
                                     (&[], &[1], &[1]),
                                     (&[1], &[1], &[2]),
                                     (&[1], &[1, 1], &[2, 1]),
                                     (&[1], &[N1], &[0, 1]),
                                     (&[1], &[N1, N1], &[0, 0, 1]),
                                     (&[N1, N1], &[N1, N1], &[N2, N1, 1]),
                                     (&[1, 1, 1], &[N1, N1], &[0, 1, 2]),
                                     (&[2, 2, 1], &[N1, N2], &[1, 1, 2])];

#[test]
fn test_add() {
    for elm in SUM_TRIPLES.iter() {
        let (a_vec, b_vec, c_vec) = *elm;
        let a = BigUint::from_slice(a_vec);
        let b = BigUint::from_slice(b_vec);
        let c = BigUint::from_slice(c_vec);

        assert_op!(a + b == c);
        assert_op!(b + a == c);
        assert_assign_op!(a += b == c);
        assert_assign_op!(b += a == c);
    }
}

#[test]
fn test_scalar_add() {
    for elm in SUM_TRIPLES.iter() {
        let (a_vec, b_vec, c_vec) = *elm;
        let a = BigUint::from_slice(a_vec);
        let b = BigUint::from_slice(b_vec);
        let c = BigUint::from_slice(c_vec);

        assert_scalar_op!(a + b == c);
        assert_scalar_op!(b + a == c);
    }
}

#[test]
fn test_sub() {
    for elm in SUM_TRIPLES.iter() {
        let (a_vec, b_vec, c_vec) = *elm;
        let a = BigUint::from_slice(a_vec);
        let b = BigUint::from_slice(b_vec);
        let c = BigUint::from_slice(c_vec);

        assert_op!(c - a == b);
        assert_op!(c - b == a);
        assert_assign_op!(c -= a == b);
        assert_assign_op!(c -= b == a);
    }
}

#[test]
fn test_scalar_sub() {
    for elm in SUM_TRIPLES.iter() {
        let (a_vec, b_vec, c_vec) = *elm;
        let a = BigUint::from_slice(a_vec);
        let b = BigUint::from_slice(b_vec);
        let c = BigUint::from_slice(c_vec);

        assert_scalar_op!(c - a == b);
        assert_scalar_op!(c - b == a);
    }
}

#[test]
#[should_panic]
fn test_sub_fail_on_underflow() {
    let (a, b): (BigUint, BigUint) = (Zero::zero(), One::one());
    a - b;
}

const M: u32 = ::std::u32::MAX;
const MUL_TRIPLES: &'static [(&'static [BigDigit],
           &'static [BigDigit],
           &'static [BigDigit])] = &[(&[], &[], &[]),
                                     (&[], &[1], &[]),
                                     (&[2], &[], &[]),
                                     (&[1], &[1], &[1]),
                                     (&[2], &[3], &[6]),
                                     (&[1], &[1, 1, 1], &[1, 1, 1]),
                                     (&[1, 2, 3], &[3], &[3, 6, 9]),
                                     (&[1, 1, 1], &[N1], &[N1, N1, N1]),
                                     (&[1, 2, 3], &[N1], &[N1, N2, N2, 2]),
                                     (&[1, 2, 3, 4], &[N1], &[N1, N2, N2, N2, 3]),
                                     (&[N1], &[N1], &[1, N2]),
                                     (&[N1, N1], &[N1], &[1, N1, N2]),
                                     (&[N1, N1, N1], &[N1], &[1, N1, N1, N2]),
                                     (&[N1, N1, N1, N1], &[N1], &[1, N1, N1, N1, N2]),
                                     (&[M / 2 + 1], &[2], &[0, 1]),
                                     (&[0, M / 2 + 1], &[2], &[0, 0, 1]),
                                     (&[1, 2], &[1, 2, 3], &[1, 4, 7, 6]),
                                     (&[N1, N1], &[N1, N1, N1], &[1, 0, N1, N2, N1]),
                                     (&[N1, N1, N1],
                                      &[N1, N1, N1, N1],
                                      &[1, 0, 0, N1, N2, N1, N1]),
                                     (&[0, 0, 1], &[1, 2, 3], &[0, 0, 1, 2, 3]),
                                     (&[0, 0, 1], &[0, 0, 0, 1], &[0, 0, 0, 0, 0, 1])];

const DIV_REM_QUADRUPLES: &'static [(&'static [BigDigit],
           &'static [BigDigit],
           &'static [BigDigit],
           &'static [BigDigit])] = &[(&[1], &[2], &[], &[1]),
                                     (&[3], &[2], &[1], &[1]),
                                     (&[1, 1], &[2], &[M / 2 + 1], &[1]),
                                     (&[1, 1, 1], &[2], &[M / 2 + 1, M / 2 + 1], &[1]),
                                     (&[0, 1], &[N1], &[1], &[1]),
                                     (&[N1, N1], &[N2], &[2, 1], &[3])];

#[test]
fn test_mul() {
    for elm in MUL_TRIPLES.iter() {
        let (a_vec, b_vec, c_vec) = *elm;
        let a = BigUint::from_slice(a_vec);
        let b = BigUint::from_slice(b_vec);
        let c = BigUint::from_slice(c_vec);

        assert_op!(a * b == c);
        assert_op!(b * a == c);
        assert_assign_op!(a *= b == c);
        assert_assign_op!(b *= a == c);
    }

    for elm in DIV_REM_QUADRUPLES.iter() {
        let (a_vec, b_vec, c_vec, d_vec) = *elm;
        let a = BigUint::from_slice(a_vec);
        let b = BigUint::from_slice(b_vec);
        let c = BigUint::from_slice(c_vec);
        let d = BigUint::from_slice(d_vec);

        assert!(a == &b * &c + &d);
        assert!(a == &c * &b + &d);
    }
}

#[test]
fn test_scalar_mul() {
    for elm in MUL_TRIPLES.iter() {
        let (a_vec, b_vec, c_vec) = *elm;
        let a = BigUint::from_slice(a_vec);
        let b = BigUint::from_slice(b_vec);
        let c = BigUint::from_slice(c_vec);

        assert_scalar_op!(a * b == c);
        assert_scalar_op!(b * a == c);
    }
}

#[test]
fn test_div_rem() {
    for elm in MUL_TRIPLES.iter() {
        let (a_vec, b_vec, c_vec) = *elm;
        let a = BigUint::from_slice(a_vec);
        let b = BigUint::from_slice(b_vec);
        let c = BigUint::from_slice(c_vec);

        if !a.is_zero() {
            assert_op!(c / a == b);
            assert_op!(c % a == Zero::zero());
            assert_assign_op!(c /= a == b);
            assert_assign_op!(c %= a == Zero::zero());
            assert_eq!(c.div_rem(&a), (b.clone(), Zero::zero()));
        }
        if !b.is_zero() {
            assert_op!(c / b == a);
            assert_op!(c % b == Zero::zero());
            assert_assign_op!(c /= b == a);
            assert_assign_op!(c %= b == Zero::zero());
            assert_eq!(c.div_rem(&b), (a.clone(), Zero::zero()));
        }
    }

    for elm in DIV_REM_QUADRUPLES.iter() {
        let (a_vec, b_vec, c_vec, d_vec) = *elm;
        let a = BigUint::from_slice(a_vec);
        let b = BigUint::from_slice(b_vec);
        let c = BigUint::from_slice(c_vec);
        let d = BigUint::from_slice(d_vec);

        if !b.is_zero() {
            assert_op!(a / b == c);
            assert_op!(a % b == d);
            assert_assign_op!(a /= b == c);
            assert_assign_op!(a %= b == d);
            assert!(a.div_rem(&b) == (c, d));
        }
    }
}

#[test]
fn test_scalar_div_rem() {
    for elm in MUL_TRIPLES.iter() {
        let (a_vec, b_vec, c_vec) = *elm;
        let a = BigUint::from_slice(a_vec);
        let b = BigUint::from_slice(b_vec);
        let c = BigUint::from_slice(c_vec);

        if !a.is_zero() {
            assert_scalar_op!(c / a == b);
            assert_scalar_op!(c % a == Zero::zero());
        }

        if !b.is_zero() {
            assert_scalar_op!(c / b == a);
            assert_scalar_op!(c % b == Zero::zero());
        }
    }

    for elm in DIV_REM_QUADRUPLES.iter() {
        let (a_vec, b_vec, c_vec, d_vec) = *elm;
        let a = BigUint::from_slice(a_vec);
        let b = BigUint::from_slice(b_vec);
        let c = BigUint::from_slice(c_vec);
        let d = BigUint::from_slice(d_vec);

        if !b.is_zero() {
            assert_scalar_op!(a / b == c);
            assert_scalar_op!(a % b == d);
        }
    }
}

#[test]
fn test_checked_add() {
    for elm in SUM_TRIPLES.iter() {
        let (a_vec, b_vec, c_vec) = *elm;
        let a = BigUint::from_slice(a_vec);
        let b = BigUint::from_slice(b_vec);
        let c = BigUint::from_slice(c_vec);

        assert!(a.checked_add(&b).unwrap() == c);
        assert!(b.checked_add(&a).unwrap() == c);
    }
}

#[test]
fn test_checked_sub() {
    for elm in SUM_TRIPLES.iter() {
        let (a_vec, b_vec, c_vec) = *elm;
        let a = BigUint::from_slice(a_vec);
        let b = BigUint::from_slice(b_vec);
        let c = BigUint::from_slice(c_vec);

        assert!(c.checked_sub(&a).unwrap() == b);
        assert!(c.checked_sub(&b).unwrap() == a);

        if a > c {
            assert!(a.checked_sub(&c).is_none());
        }
        if b > c {
            assert!(b.checked_sub(&c).is_none());
        }
    }
}

#[test]
fn test_checked_mul() {
    for elm in MUL_TRIPLES.iter() {
        let (a_vec, b_vec, c_vec) = *elm;
        let a = BigUint::from_slice(a_vec);
        let b = BigUint::from_slice(b_vec);
        let c = BigUint::from_slice(c_vec);

        assert!(a.checked_mul(&b).unwrap() == c);
        assert!(b.checked_mul(&a).unwrap() == c);
    }

    for elm in DIV_REM_QUADRUPLES.iter() {
        let (a_vec, b_vec, c_vec, d_vec) = *elm;
        let a = BigUint::from_slice(a_vec);
        let b = BigUint::from_slice(b_vec);
        let c = BigUint::from_slice(c_vec);
        let d = BigUint::from_slice(d_vec);

        assert!(a == b.checked_mul(&c).unwrap() + &d);
        assert!(a == c.checked_mul(&b).unwrap() + &d);
    }
}

#[test]
fn test_mul_overflow() {
    /* Test for issue #187 - overflow due to mac3 incorrectly sizing temporary */
    let s = "531137992816767098689588206552468627329593117727031923199444138200403559860852242739162502232636710047537552105951370000796528760829212940754539968588340162273730474622005920097370111";
    let a: BigUint = s.parse().unwrap();
    let b = a.clone();
    let _ = a.checked_mul(&b);
}

#[test]
fn test_checked_div() {
    for elm in MUL_TRIPLES.iter() {
        let (a_vec, b_vec, c_vec) = *elm;
        let a = BigUint::from_slice(a_vec);
        let b = BigUint::from_slice(b_vec);
        let c = BigUint::from_slice(c_vec);

        if !a.is_zero() {
            assert!(c.checked_div(&a).unwrap() == b);
        }
        if !b.is_zero() {
            assert!(c.checked_div(&b).unwrap() == a);
        }

        assert!(c.checked_div(&Zero::zero()).is_none());
    }
}

#[test]
fn test_gcd() {
    fn check(a: usize, b: usize, c: usize) {
        let big_a: BigUint = FromPrimitive::from_usize(a).unwrap();
        let big_b: BigUint = FromPrimitive::from_usize(b).unwrap();
        let big_c: BigUint = FromPrimitive::from_usize(c).unwrap();

        assert_eq!(big_a.gcd(&big_b), big_c);
    }

    check(10, 2, 2);
    check(10, 3, 1);
    check(0, 3, 3);
    check(3, 3, 3);
    check(56, 42, 14);
}

#[test]
fn test_lcm() {
    fn check(a: usize, b: usize, c: usize) {
        let big_a: BigUint = FromPrimitive::from_usize(a).unwrap();
        let big_b: BigUint = FromPrimitive::from_usize(b).unwrap();
        let big_c: BigUint = FromPrimitive::from_usize(c).unwrap();

        assert_eq!(big_a.lcm(&big_b), big_c);
    }

    check(1, 0, 0);
    check(0, 1, 0);
    check(1, 1, 1);
    check(8, 9, 72);
    check(11, 5, 55);
    check(99, 17, 1683);
}

#[test]
fn test_is_even() {
    let one: BigUint = FromStr::from_str("1").unwrap();
    let two: BigUint = FromStr::from_str("2").unwrap();
    let thousand: BigUint = FromStr::from_str("1000").unwrap();
    let big: BigUint = FromStr::from_str("1000000000000000000000").unwrap();
    let bigger: BigUint = FromStr::from_str("1000000000000000000001").unwrap();
    assert!(one.is_odd());
    assert!(two.is_even());
    assert!(thousand.is_even());
    assert!(big.is_even());
    assert!(bigger.is_odd());
    assert!((&one << 64).is_even());
    assert!(((&one << 64) + one).is_odd());
}

#[test]
fn test_modpow() {
    fn check(b: usize, e: usize, m: usize, r: usize) {
        let big_b = BigUint::from(b);
        let big_e = BigUint::from(e);
        let big_m = BigUint::from(m);
        let big_r = BigUint::from(r);

        assert_eq!(big_b.modpow(&big_e, &big_m), big_r);

        let even_m = &big_m << 1;
        let even_modpow = big_b.modpow(&big_e, &even_m);
        assert!(even_modpow < even_m);
        assert_eq!(even_modpow % big_m, big_r);
    }

    check(1, 0, 11, 1);
    check(0, 15, 11, 0);
    check(3, 7, 11, 9);
    check(5, 117, 19, 1);
}

#[test]
fn test_modpow_big() {
    let b = BigUint::from_str_radix("\
        efac3c0a_0de55551_fee0bfe4_67fa017a_1a898fa1_6ca57cb1\
        ca9e3248_cacc09a9_b99d6abc_38418d0f_82ae4238_d9a68832\
        aadec7c1_ac5fed48_7a56a71b_67ac59d5_afb28022_20d9592d\
        247c4efc_abbd9b75_586088ee_1dc00dc4_232a8e15_6e8191dd\
        675b6ae0_c80f5164_752940bc_284b7cee_885c1e10_e495345b\
        8fbe9cfd_e5233fe1_19459d0b_d64be53c_27de5a02_a829976b\
        33096862_82dad291_bd38b6a9_be396646_ddaf8039_a2573c39\
        1b14e8bc_2cb53e48_298c047e_d9879e9c_5a521076_f0e27df3\
        990e1659_d3d8205b_6443ebc0_9918ebee_6764f668_9f2b2be3\
        b59cbc76_d76d0dfc_d737c3ec_0ccf9c00_ad0554bf_17e776ad\
        b4edf9cc_6ce540be_76229093_5c53893b", 16).unwrap();
    let e = BigUint::from_str_radix("\
        be0e6ea6_08746133_e0fbc1bf_82dba91e_e2b56231_a81888d2\
        a833a1fc_f7ff002a_3c486a13_4f420bf3_a5435be9_1a5c8391\
        774d6e6c_085d8357_b0c97d4d_2bb33f7c_34c68059_f78d2541\
        eacc8832_426f1816_d3be001e_b69f9242_51c7708e_e10efe98\
        449c9a4a_b55a0f23_9d797410_515da00d_3ea07970_4478a2ca\
        c3d5043c_bd9be1b4_6dce479d_4302d344_84a939e6_0ab5ada7\
        12ae34b2_30cc473c_9f8ee69d_2cac5970_29f5bf18_bc8203e4\
        f3e895a2_13c94f1e_24c73d77_e517e801_53661fdd_a2ce9e47\
        a73dd7f8_2f2adb1e_3f136bf7_8ae5f3b8_08730de1_a4eff678\
        e77a06d0_19a522eb_cbefba2a_9caf7736_b157c5c6_2d192591\
        17946850_2ddb1822_117b68a0_32f7db88", 16).unwrap();
    // This modulus is the prime from the 2048-bit MODP DH group:
    // https://tools.ietf.org/html/rfc3526#section-3
    let m = BigUint::from_str_radix("\
        FFFFFFFF_FFFFFFFF_C90FDAA2_2168C234_C4C6628B_80DC1CD1\
        29024E08_8A67CC74_020BBEA6_3B139B22_514A0879_8E3404DD\
        EF9519B3_CD3A431B_302B0A6D_F25F1437_4FE1356D_6D51C245\
        E485B576_625E7EC6_F44C42E9_A637ED6B_0BFF5CB6_F406B7ED\
        EE386BFB_5A899FA5_AE9F2411_7C4B1FE6_49286651_ECE45B3D\
        C2007CB8_A163BF05_98DA4836_1C55D39A_69163FA8_FD24CF5F\
        83655D23_DCA3AD96_1C62F356_208552BB_9ED52907_7096966D\
        670C354E_4ABC9804_F1746C08_CA18217C_32905E46_2E36CE3B\
        E39E772C_180E8603_9B2783A2_EC07A28F_B5C55DF0_6F4C52C9\
        DE2BCBF6_95581718_3995497C_EA956AE5_15D22618_98FA0510\
        15728E5A_8AACAA68_FFFFFFFF_FFFFFFFF", 16).unwrap();
    let r = BigUint::from_str_radix("\
        a1468311_6e56edc9_7a98228b_5e924776_0dd7836e_caabac13\
        eda5373b_4752aa65_a1454850_40dc770e_30aa8675_6be7d3a8\
        9d3085e4_da5155cf_b451ef62_54d0da61_cf2b2c87_f495e096\
        055309f7_77802bbb_37271ba8_1313f1b5_075c75d1_024b6c77\
        fdb56f17_b05bce61_e527ebfd_2ee86860_e9907066_edd526e7\
        93d289bf_6726b293_41b0de24_eff82424_8dfd374b_4ec59542\
        35ced2b2_6b195c90_10042ffb_8f58ce21_bc10ec42_64fda779\
        d352d234_3d4eaea6_a86111ad_a37e9555_43ca78ce_2885bed7\
        5a30d182_f1cf6834_dc5b6e27_1a41ac34_a2e91e11_33363ff0\
        f88a7b04_900227c9_f6e6d06b_7856b4bb_4e354d61_060db6c8\
        109c4735_6e7db425_7b5d74c7_0b709508", 16).unwrap();

    assert_eq!(b.modpow(&e, &m), r);

    let even_m = &m << 1;
    let even_modpow = b.modpow(&e, &even_m);
    assert!(even_modpow < even_m);
    assert_eq!(even_modpow % m, r);
}

fn to_str_pairs() -> Vec<(BigUint, Vec<(u32, String)>)> {
    let bits = big_digit::BITS;
    vec![(Zero::zero(),
          vec![(2, "0".to_string()), (3, "0".to_string())]),
         (BigUint::from_slice(&[0xff]),
          vec![(2, "11111111".to_string()),
               (3, "100110".to_string()),
               (4, "3333".to_string()),
               (5, "2010".to_string()),
               (6, "1103".to_string()),
               (7, "513".to_string()),
               (8, "377".to_string()),
               (9, "313".to_string()),
               (10, "255".to_string()),
               (11, "212".to_string()),
               (12, "193".to_string()),
               (13, "168".to_string()),
               (14, "143".to_string()),
               (15, "120".to_string()),
               (16, "ff".to_string())]),
         (BigUint::from_slice(&[0xfff]),
          vec![(2, "111111111111".to_string()),
               (4, "333333".to_string()),
               (16, "fff".to_string())]),
         (BigUint::from_slice(&[1, 2]),
          vec![(2,
                format!("10{}1", repeat("0").take(bits - 1).collect::<String>())),
               (4,
                format!("2{}1", repeat("0").take(bits / 2 - 1).collect::<String>())),
               (10,
                match bits {
                   64 => "36893488147419103233".to_string(),
                   32 => "8589934593".to_string(),
                   16 => "131073".to_string(),
                   _ => panic!(),
               }),
               (16,
                format!("2{}1", repeat("0").take(bits / 4 - 1).collect::<String>()))]),
         (BigUint::from_slice(&[1, 2, 3]),
          vec![(2,
                format!("11{}10{}1",
                        repeat("0").take(bits - 2).collect::<String>(),
                        repeat("0").take(bits - 1).collect::<String>())),
               (4,
                format!("3{}2{}1",
                        repeat("0").take(bits / 2 - 1).collect::<String>(),
                        repeat("0").take(bits / 2 - 1).collect::<String>())),
               (8,
                match bits {
                   64 => "14000000000000000000004000000000000000000001".to_string(),
                   32 => "6000000000100000000001".to_string(),
                   16 => "140000400001".to_string(),
                   _ => panic!(),
               }),
               (10,
                match bits {
                   64 => "1020847100762815390427017310442723737601".to_string(),
                   32 => "55340232229718589441".to_string(),
                   16 => "12885032961".to_string(),
                   _ => panic!(),
               }),
               (16,
                format!("3{}2{}1",
                        repeat("0").take(bits / 4 - 1).collect::<String>(),
                        repeat("0").take(bits / 4 - 1).collect::<String>()))])]
}

#[test]
fn test_to_str_radix() {
    let r = to_str_pairs();
    for num_pair in r.iter() {
        let &(ref n, ref rs) = num_pair;
        for str_pair in rs.iter() {
            let &(ref radix, ref str) = str_pair;
            assert_eq!(n.to_str_radix(*radix), *str);
        }
    }
}

#[test]
fn test_from_and_to_radix() {
    const GROUND_TRUTH : &'static[(&'static[u8], u32, &'static[u8])] = &[
        (b"0",          42, &[0]),
        (b"ffffeeffbb", 2, &[1, 1, 0, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1,
                             1, 0, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                             1, 1, 1, 1, 1, 1, 1, 1, 1, 1]),
        (b"ffffeeffbb", 3, &[2, 2, 1, 1, 2, 1, 1, 2, 0, 0, 0, 0, 0, 1, 2,
                             0, 0, 0, 0, 1, 0, 0, 2, 2, 0, 1]),
        (b"ffffeeffbb", 4, &[3, 2, 3, 2, 3, 3, 3, 3, 2, 3, 2, 3, 3, 3, 3,
                             3, 3, 3, 3, 3]),
        (b"ffffeeffbb", 5, &[0, 4, 3, 3, 1, 4, 2, 4, 1, 4, 4, 2, 3, 0, 0,
                             1, 2, 1]),
        (b"ffffeeffbb", 6, &[5, 5, 4, 5, 5, 0, 0, 1, 2, 5, 3, 0, 1, 0, 2, 2]),
        (b"ffffeeffbb", 7, &[4, 2, 3, 6, 0, 1, 6, 1, 6, 2, 0, 3, 2, 4, 1]),
        (b"ffffeeffbb", 8, &[3, 7, 6, 7, 7, 5, 3, 7, 7, 7, 7, 7, 7, 1]),
        (b"ffffeeffbb", 9, &[8, 4, 5, 7, 0, 0, 3, 2, 0, 3, 0, 8, 3]),
        (b"ffffeeffbb", 10, &[5, 9, 5, 3, 1, 5, 0, 1, 5, 9, 9, 0, 1]),
        (b"ffffeeffbb", 11, &[10, 7, 6, 5, 2, 0, 3, 3, 3, 4, 9, 3]),
        (b"ffffeeffbb", 12, &[11, 8, 5, 10, 1, 10, 3, 1, 1, 9, 5, 1]),
        (b"ffffeeffbb", 13, &[0, 5, 7, 4, 6, 5, 6, 11, 8, 12, 7]),
        (b"ffffeeffbb", 14, &[11, 4, 4, 11, 8, 4, 6, 0, 3, 11, 3]),
        (b"ffffeeffbb", 15, &[5, 11, 13, 2, 1, 10, 2, 0, 9, 13, 1]),
        (b"ffffeeffbb", 16, &[11, 11, 15, 15, 14, 14, 15, 15, 15, 15]),
        (b"ffffeeffbb", 17, &[0, 2, 14, 12, 2, 14, 8, 10, 4, 9]),
        (b"ffffeeffbb", 18, &[17, 15, 5, 13, 10, 16, 16, 13, 9, 5]),
        (b"ffffeeffbb", 19, &[14, 13, 2, 8, 9, 0, 1, 14, 7, 3]),
        (b"ffffeeffbb", 20, &[15, 19, 3, 14, 0, 17, 19, 18, 2, 2]),
        (b"ffffeeffbb", 21, &[11, 5, 4, 13, 5, 18, 9, 1, 8, 1]),
        (b"ffffeeffbb", 22, &[21, 3, 7, 21, 15, 12, 17, 0, 20]),
        (b"ffffeeffbb", 23, &[21, 21, 6, 9, 10, 7, 21, 0, 14]),
        (b"ffffeeffbb", 24, &[11, 10, 19, 14, 22, 11, 17, 23, 9]),
        (b"ffffeeffbb", 25, &[20, 18, 21, 22, 21, 14, 3, 5, 7]),
        (b"ffffeeffbb", 26, &[13, 15, 24, 11, 17, 6, 23, 6, 5]),
        (b"ffffeeffbb", 27, &[17, 16, 7, 0, 21, 0, 3, 24, 3]),
        (b"ffffeeffbb", 28, &[11, 16, 11, 15, 14, 18, 13, 25, 2]),
        (b"ffffeeffbb", 29, &[6, 8, 7, 19, 14, 13, 21, 5, 2]),
        (b"ffffeeffbb", 30, &[5, 13, 18, 11, 10, 7, 8, 20, 1]),
        (b"ffffeeffbb", 31, &[22, 26, 15, 19, 8, 27, 29, 8, 1]),
        (b"ffffeeffbb", 32, &[27, 29, 31, 29, 30, 31, 31, 31]),
        (b"ffffeeffbb", 33, &[32, 20, 27, 12, 1, 12, 26, 25]),
        (b"ffffeeffbb", 34, &[17, 9, 16, 33, 13, 25, 31, 20]),
        (b"ffffeeffbb", 35, &[25, 32, 2, 25, 11, 4, 3, 17]),
        (b"ffffeeffbb", 36, &[35, 34, 5, 6, 32, 3, 1, 14]),
        (b"ffffeeffbb", 37, &[16, 21, 18, 4, 33, 19, 21, 11]),
        (b"ffffeeffbb", 38, &[33, 25, 19, 29, 20, 6, 23, 9]),
        (b"ffffeeffbb", 39, &[26, 27, 29, 23, 16, 18, 0, 8]),
        (b"ffffeeffbb", 40, &[35, 39, 30, 11, 16, 17, 28, 6]),
        (b"ffffeeffbb", 41, &[36, 30, 9, 18, 12, 19, 26, 5]),
        (b"ffffeeffbb", 42, &[11, 34, 37, 27, 1, 13, 32, 4]),
        (b"ffffeeffbb", 43, &[3, 24, 11, 2, 10, 40, 1, 4]),
        (b"ffffeeffbb", 44, &[43, 12, 40, 32, 3, 23, 19, 3]),
        (b"ffffeeffbb", 45, &[35, 38, 44, 18, 22, 18, 42, 2]),
        (b"ffffeeffbb", 46, &[21, 45, 18, 41, 17, 2, 24, 2]),
        (b"ffffeeffbb", 47, &[37, 37, 11, 12, 6, 0, 8, 2]),
        (b"ffffeeffbb", 48, &[11, 41, 40, 43, 5, 43, 41, 1]),
        (b"ffffeeffbb", 49, &[18, 45, 7, 13, 20, 21, 30, 1]),
        (b"ffffeeffbb", 50, &[45, 21, 5, 34, 21, 18, 20, 1]),
        (b"ffffeeffbb", 51, &[17, 6, 26, 22, 38, 24, 11, 1]),
        (b"ffffeeffbb", 52, &[39, 33, 38, 30, 46, 31, 3, 1]),
        (b"ffffeeffbb", 53, &[31, 7, 44, 23, 9, 32, 49]),
        (b"ffffeeffbb", 54, &[17, 35, 8, 37, 31, 18, 44]),
        (b"ffffeeffbb", 55, &[10, 52, 9, 48, 36, 39, 39]),
        (b"ffffeeffbb", 56, &[11, 50, 51, 22, 25, 36, 35]),
        (b"ffffeeffbb", 57, &[14, 55, 12, 43, 20, 3, 32]),
        (b"ffffeeffbb", 58, &[35, 18, 45, 56, 9, 51, 28]),
        (b"ffffeeffbb", 59, &[51, 28, 20, 26, 55, 3, 26]),
        (b"ffffeeffbb", 60, &[35, 6, 27, 46, 58, 33, 23]),
        (b"ffffeeffbb", 61, &[58, 7, 6, 54, 49, 20, 21]),
        (b"ffffeeffbb", 62, &[53, 59, 3, 14, 10, 22, 19]),
        (b"ffffeeffbb", 63, &[53, 50, 23, 4, 56, 36, 17]),
        (b"ffffeeffbb", 64, &[59, 62, 47, 59, 63, 63, 15]),
        (b"ffffeeffbb", 65, &[0, 53, 39, 4, 40, 37, 14]),
        (b"ffffeeffbb", 66, &[65, 59, 39, 1, 64, 19, 13]),
        (b"ffffeeffbb", 67, &[35, 14, 19, 16, 25, 10, 12]),
        (b"ffffeeffbb", 68, &[51, 38, 63, 50, 15, 8, 11]),
        (b"ffffeeffbb", 69, &[44, 45, 18, 58, 68, 12, 10]),
        (b"ffffeeffbb", 70, &[25, 51, 0, 60, 13, 24, 9]),
        (b"ffffeeffbb", 71, &[54, 30, 9, 65, 28, 41, 8]),
        (b"ffffeeffbb", 72, &[35, 35, 55, 54, 17, 64, 7]),
        (b"ffffeeffbb", 73, &[34, 4, 48, 40, 27, 19, 7]),
        (b"ffffeeffbb", 74, &[53, 47, 4, 56, 36, 51, 6]),
        (b"ffffeeffbb", 75, &[20, 56, 10, 72, 24, 13, 6]),
        (b"ffffeeffbb", 76, &[71, 31, 52, 60, 48, 53, 5]),
        (b"ffffeeffbb", 77, &[32, 73, 14, 63, 15, 21, 5]),
        (b"ffffeeffbb", 78, &[65, 13, 17, 32, 64, 68, 4]),
        (b"ffffeeffbb", 79, &[37, 56, 2, 56, 25, 41, 4]),
        (b"ffffeeffbb", 80, &[75, 59, 37, 41, 43, 15, 4]),
        (b"ffffeeffbb", 81, &[44, 68, 0, 21, 27, 72, 3]),
        (b"ffffeeffbb", 82, &[77, 35, 2, 74, 46, 50, 3]),
        (b"ffffeeffbb", 83, &[52, 51, 19, 76, 10, 30, 3]),
        (b"ffffeeffbb", 84, &[11, 80, 19, 19, 76, 10, 3]),
        (b"ffffeeffbb", 85, &[0, 82, 20, 14, 68, 77, 2]),
        (b"ffffeeffbb", 86, &[3, 12, 78, 37, 62, 61, 2]),
        (b"ffffeeffbb", 87, &[35, 12, 20, 8, 52, 46, 2]),
        (b"ffffeeffbb", 88, &[43, 6, 54, 42, 30, 32, 2]),
        (b"ffffeeffbb", 89, &[49, 52, 85, 21, 80, 18, 2]),
        (b"ffffeeffbb", 90, &[35, 64, 78, 24, 18, 6, 2]),
        (b"ffffeeffbb", 91, &[39, 17, 83, 63, 17, 85, 1]),
        (b"ffffeeffbb", 92, &[67, 22, 85, 79, 75, 74, 1]),
        (b"ffffeeffbb", 93, &[53, 60, 39, 29, 4, 65, 1]),
        (b"ffffeeffbb", 94, &[37, 89, 2, 72, 76, 55, 1]),
        (b"ffffeeffbb", 95, &[90, 74, 89, 9, 9, 47, 1]),
        (b"ffffeeffbb", 96, &[59, 20, 46, 35, 81, 38, 1]),
        (b"ffffeeffbb", 97, &[94, 87, 60, 71, 3, 31, 1]),
        (b"ffffeeffbb", 98, &[67, 22, 63, 50, 62, 23, 1]),
        (b"ffffeeffbb", 99, &[98, 6, 69, 12, 61, 16, 1]),
        (b"ffffeeffbb", 100, &[95, 35, 51, 10, 95, 9, 1]),
        (b"ffffeeffbb", 101, &[87, 27, 7, 8, 62, 3, 1]),
        (b"ffffeeffbb", 102, &[17, 3, 32, 79, 59, 99]),
        (b"ffffeeffbb", 103, &[30, 22, 90, 0, 87, 94]),
        (b"ffffeeffbb", 104, &[91, 68, 87, 68, 38, 90]),
        (b"ffffeeffbb", 105, &[95, 80, 54, 73, 15, 86]),
        (b"ffffeeffbb", 106, &[31, 30, 24, 16, 17, 82]),
        (b"ffffeeffbb", 107, &[51, 50, 10, 12, 42, 78]),
        (b"ffffeeffbb", 108, &[71, 71, 96, 78, 89, 74]),
        (b"ffffeeffbb", 109, &[33, 18, 93, 22, 50, 71]),
        (b"ffffeeffbb", 110, &[65, 53, 57, 88, 29, 68]),
        (b"ffffeeffbb", 111, &[53, 93, 67, 90, 27, 65]),
        (b"ffffeeffbb", 112, &[11, 109, 96, 65, 43, 62]),
        (b"ffffeeffbb", 113, &[27, 23, 106, 56, 76, 59]),
        (b"ffffeeffbb", 114, &[71, 84, 31, 112, 11, 57]),
        (b"ffffeeffbb", 115, &[90, 22, 1, 56, 76, 54]),
        (b"ffffeeffbb", 116, &[35, 38, 98, 57, 40, 52]),
        (b"ffffeeffbb", 117, &[26, 113, 115, 62, 17, 50]),
        (b"ffffeeffbb", 118, &[51, 14, 5, 18, 7, 48]),
        (b"ffffeeffbb", 119, &[102, 31, 110, 108, 8, 46]),
        (b"ffffeeffbb", 120, &[35, 93, 96, 50, 22, 44]),
        (b"ffffeeffbb", 121, &[87, 61, 2, 36, 47, 42]),
        (b"ffffeeffbb", 122, &[119, 64, 1, 22, 83, 40]),
        (b"ffffeeffbb", 123, &[77, 119, 32, 90, 6, 39]),
        (b"ffffeeffbb", 124, &[115, 122, 31, 79, 62, 37]),
        (b"ffffeeffbb", 125, &[95, 108, 47, 74, 3, 36]),
        (b"ffffeeffbb", 126, &[53, 25, 116, 39, 78, 34]),
        (b"ffffeeffbb", 127, &[22, 23, 125, 67, 35, 33]),
        (b"ffffeeffbb", 128, &[59, 127, 59, 127, 127, 31]),
        (b"ffffeeffbb", 129, &[89, 36, 1, 59, 100, 30]),
        (b"ffffeeffbb", 130, &[65, 91, 123, 89, 79, 29]),
        (b"ffffeeffbb", 131, &[58, 72, 39, 63, 65, 28]),
        (b"ffffeeffbb", 132, &[131, 62, 92, 82, 57, 27]),
        (b"ffffeeffbb", 133, &[109, 31, 51, 123, 55, 26]),
        (b"ffffeeffbb", 134, &[35, 74, 21, 27, 60, 25]),
        (b"ffffeeffbb", 135, &[125, 132, 49, 37, 70, 24]),
        (b"ffffeeffbb", 136, &[51, 121, 117, 133, 85, 23]),
        (b"ffffeeffbb", 137, &[113, 60, 135, 22, 107, 22]),
        (b"ffffeeffbb", 138, &[113, 91, 73, 93, 133, 21]),
        (b"ffffeeffbb", 139, &[114, 75, 102, 51, 26, 21]),
        (b"ffffeeffbb", 140, &[95, 25, 35, 16, 62, 20]),
        (b"ffffeeffbb", 141, &[131, 137, 16, 110, 102, 19]),
        (b"ffffeeffbb", 142, &[125, 121, 108, 34, 6, 19]),
        (b"ffffeeffbb", 143, &[65, 78, 138, 55, 55, 18]),
        (b"ffffeeffbb", 144, &[107, 125, 121, 15, 109, 17]),
        (b"ffffeeffbb", 145, &[35, 13, 122, 42, 22, 17]),
        (b"ffffeeffbb", 146, &[107, 38, 103, 123, 83, 16]),
        (b"ffffeeffbb", 147, &[116, 96, 71, 98, 2, 16]),
        (b"ffffeeffbb", 148, &[127, 23, 75, 99, 71, 15]),
        (b"ffffeeffbb", 149, &[136, 110, 53, 114, 144, 14]),
        (b"ffffeeffbb", 150, &[95, 140, 133, 130, 71, 14]),
        (b"ffffeeffbb", 151, &[15, 50, 29, 137, 0, 14]),
        (b"ffffeeffbb", 152, &[147, 15, 89, 121, 83, 13]),
        (b"ffffeeffbb", 153, &[17, 87, 93, 72, 17, 13]),
        (b"ffffeeffbb", 154, &[109, 113, 3, 133, 106, 12]),
        (b"ffffeeffbb", 155, &[115, 141, 120, 139, 44, 12]),
        (b"ffffeeffbb", 156, &[143, 45, 4, 82, 140, 11]),
        (b"ffffeeffbb", 157, &[149, 92, 15, 106, 82, 11]),
        (b"ffffeeffbb", 158, &[37, 107, 79, 46, 26, 11]),
        (b"ffffeeffbb", 159, &[137, 37, 146, 51, 130, 10]),
        (b"ffffeeffbb", 160, &[155, 69, 29, 115, 77, 10]),
        (b"ffffeeffbb", 161, &[67, 98, 46, 68, 26, 10]),
        (b"ffffeeffbb", 162, &[125, 155, 60, 63, 138, 9]),
        (b"ffffeeffbb", 163, &[96, 43, 118, 93, 90, 9]),
        (b"ffffeeffbb", 164, &[159, 99, 123, 152, 43, 9]),
        (b"ffffeeffbb", 165, &[65, 17, 1, 69, 163, 8]),
        (b"ffffeeffbb", 166, &[135, 108, 25, 165, 119, 8]),
        (b"ffffeeffbb", 167, &[165, 116, 164, 103, 77, 8]),
        (b"ffffeeffbb", 168, &[11, 166, 67, 44, 36, 8]),
        (b"ffffeeffbb", 169, &[65, 59, 71, 149, 164, 7]),
        (b"ffffeeffbb", 170, &[85, 83, 26, 76, 126, 7]),
        (b"ffffeeffbb", 171, &[71, 132, 140, 157, 88, 7]),
        (b"ffffeeffbb", 172, &[3, 6, 127, 47, 52, 7]),
        (b"ffffeeffbb", 173, &[122, 66, 53, 83, 16, 7]),
        (b"ffffeeffbb", 174, &[35, 6, 5, 88, 155, 6]),
        (b"ffffeeffbb", 175, &[95, 20, 84, 56, 122, 6]),
        (b"ffffeeffbb", 176, &[43, 91, 57, 159, 89, 6]),
        (b"ffffeeffbb", 177, &[110, 127, 54, 40, 58, 6]),
        (b"ffffeeffbb", 178, &[49, 115, 43, 47, 27, 6]),
        (b"ffffeeffbb", 179, &[130, 91, 4, 178, 175, 5]),
        (b"ffffeeffbb", 180, &[35, 122, 109, 70, 147, 5]),
        (b"ffffeeffbb", 181, &[94, 94, 4, 79, 119, 5]),
        (b"ffffeeffbb", 182, &[39, 54, 66, 19, 92, 5]),
        (b"ffffeeffbb", 183, &[119, 2, 143, 69, 65, 5]),
        (b"ffffeeffbb", 184, &[67, 57, 90, 44, 39, 5]),
        (b"ffffeeffbb", 185, &[90, 63, 141, 123, 13, 5]),
        (b"ffffeeffbb", 186, &[53, 123, 172, 119, 174, 4]),
        (b"ffffeeffbb", 187, &[153, 21, 68, 28, 151, 4]),
        (b"ffffeeffbb", 188, &[131, 138, 94, 32, 128, 4]),
        (b"ffffeeffbb", 189, &[179, 121, 156, 130, 105, 4]),
        (b"ffffeeffbb", 190, &[185, 179, 164, 131, 83, 4]),
        (b"ffffeeffbb", 191, &[118, 123, 37, 31, 62, 4]),
        (b"ffffeeffbb", 192, &[59, 106, 83, 16, 41, 4]),
        (b"ffffeeffbb", 193, &[57, 37, 47, 86, 20, 4]),
        (b"ffffeeffbb", 194, &[191, 140, 63, 45, 0, 4]),
        (b"ffffeeffbb", 195, &[65, 169, 83, 84, 175, 3]),
        (b"ffffeeffbb", 196, &[67, 158, 64, 6, 157, 3]),
        (b"ffffeeffbb", 197, &[121, 26, 167, 3, 139, 3]),
        (b"ffffeeffbb", 198, &[197, 151, 165, 75, 121, 3]),
        (b"ffffeeffbb", 199, &[55, 175, 36, 22, 104, 3]),
        (b"ffffeeffbb", 200, &[195, 167, 162, 38, 87, 3]),
        (b"ffffeeffbb", 201, &[35, 27, 136, 124, 70, 3]),
        (b"ffffeeffbb", 202, &[87, 64, 153, 76, 54, 3]),
        (b"ffffeeffbb", 203, &[151, 191, 14, 94, 38, 3]),
        (b"ffffeeffbb", 204, &[119, 103, 135, 175, 22, 3]),
        (b"ffffeeffbb", 205, &[200, 79, 123, 115, 7, 3]),
        (b"ffffeeffbb", 206, &[133, 165, 202, 115, 198, 2]),
        (b"ffffeeffbb", 207, &[44, 153, 193, 175, 184, 2]),
        (b"ffffeeffbb", 208, &[91, 190, 125, 86, 171, 2]),
        (b"ffffeeffbb", 209, &[109, 151, 34, 53, 158, 2]),
        (b"ffffeeffbb", 210, &[95, 40, 171, 74, 145, 2]),
        (b"ffffeeffbb", 211, &[84, 195, 162, 150, 132, 2]),
        (b"ffffeeffbb", 212, &[31, 15, 59, 68, 120, 2]),
        (b"ffffeeffbb", 213, &[125, 57, 127, 36, 108, 2]),
        (b"ffffeeffbb", 214, &[51, 132, 2, 55, 96, 2]),
        (b"ffffeeffbb", 215, &[175, 133, 177, 122, 84, 2]),
        (b"ffffeeffbb", 216, &[179, 35, 78, 23, 73, 2]),
        (b"ffffeeffbb", 217, &[53, 101, 208, 186, 61, 2]),
        (b"ffffeeffbb", 218, &[33, 9, 214, 179, 50, 2]),
        (b"ffffeeffbb", 219, &[107, 147, 175, 217, 39, 2]),
        (b"ffffeeffbb", 220, &[175, 81, 179, 79, 29, 2]),
        (b"ffffeeffbb", 221, &[0, 76, 95, 204, 18, 2]),
        (b"ffffeeffbb", 222, &[53, 213, 16, 150, 8, 2]),
        (b"ffffeeffbb", 223, &[158, 161, 42, 136, 221, 1]),
        (b"ffffeeffbb", 224, &[123, 54, 52, 162, 212, 1]),
        (b"ffffeeffbb", 225, &[170, 43, 151, 2, 204, 1]),
        (b"ffffeeffbb", 226, &[27, 68, 224, 105, 195, 1]),
        (b"ffffeeffbb", 227, &[45, 69, 157, 20, 187, 1]),
        (b"ffffeeffbb", 228, &[71, 213, 64, 199, 178, 1]),
        (b"ffffeeffbb", 229, &[129, 203, 66, 186, 170, 1]),
        (b"ffffeeffbb", 230, &[205, 183, 57, 208, 162, 1]),
        (b"ffffeeffbb", 231, &[32, 50, 164, 33, 155, 1]),
        (b"ffffeeffbb", 232, &[35, 135, 53, 123, 147, 1]),
        (b"ffffeeffbb", 233, &[209, 47, 89, 13, 140, 1]),
        (b"ffffeeffbb", 234, &[143, 56, 175, 168, 132, 1]),
        (b"ffffeeffbb", 235, &[225, 157, 216, 121, 125, 1]),
        (b"ffffeeffbb", 236, &[51, 66, 119, 105, 118, 1]),
        (b"ffffeeffbb", 237, &[116, 150, 26, 119, 111, 1]),
        (b"ffffeeffbb", 238, &[221, 15, 87, 162, 104, 1]),
        (b"ffffeeffbb", 239, &[234, 155, 214, 234, 97, 1]),
        (b"ffffeeffbb", 240, &[155, 46, 84, 96, 91, 1]),
        (b"ffffeeffbb", 241, &[187, 48, 90, 225, 84, 1]),
        (b"ffffeeffbb", 242, &[87, 212, 151, 140, 78, 1]),
        (b"ffffeeffbb", 243, &[206, 22, 189, 81, 72, 1]),
        (b"ffffeeffbb", 244, &[119, 93, 122, 48, 66, 1]),
        (b"ffffeeffbb", 245, &[165, 224, 117, 40, 60, 1]),
        (b"ffffeeffbb", 246, &[77, 121, 100, 57, 54, 1]),
        (b"ffffeeffbb", 247, &[52, 128, 242, 98, 48, 1]),
        (b"ffffeeffbb", 248, &[115, 247, 224, 164, 42, 1]),
        (b"ffffeeffbb", 249, &[218, 127, 223, 5, 37, 1]),
        (b"ffffeeffbb", 250, &[95, 54, 168, 118, 31, 1]),
        (b"ffffeeffbb", 251, &[121, 204, 240, 3, 26, 1]),
        (b"ffffeeffbb", 252, &[179, 138, 123, 162, 20, 1]),
        (b"ffffeeffbb", 253, &[21, 50, 1, 91, 15, 1]),
        (b"ffffeeffbb", 254, &[149, 11, 63, 40, 10, 1]),
        (b"ffffeeffbb", 255, &[170, 225, 247, 9, 5, 1]),
        (b"ffffeeffbb", 256, &[187, 255, 238, 255, 255]),
    ];

    for &(bigint, radix, inbaseradix_le) in GROUND_TRUTH.iter() {
        let bigint = BigUint::parse_bytes(bigint, 16).unwrap();
        // to_radix_le
        assert_eq!(bigint.to_radix_le(radix), inbaseradix_le);
        // to_radix_be
        let mut inbase_be = bigint.to_radix_be(radix);
        inbase_be.reverse(); // now le
        assert_eq!(inbase_be, inbaseradix_le);
        // from_radix_le
        assert_eq!(BigUint::from_radix_le(inbaseradix_le, radix).unwrap(), bigint);
        // from_radix_be
        let mut inbaseradix_be = Vec::from(inbaseradix_le);
        inbaseradix_be.reverse();
        assert_eq!(BigUint::from_radix_be(&inbaseradix_be, radix).unwrap(), bigint);
    }

    assert!(BigUint::from_radix_le(&[10,100,10], 50).is_none());
}

#[test]
fn test_from_str_radix() {
    let r = to_str_pairs();
    for num_pair in r.iter() {
        let &(ref n, ref rs) = num_pair;
        for str_pair in rs.iter() {
            let &(ref radix, ref str) = str_pair;
            assert_eq!(n, &BigUint::from_str_radix(str, *radix).unwrap());
        }
    }

    let zed = BigUint::from_str_radix("Z", 10).ok();
    assert_eq!(zed, None);
    let blank = BigUint::from_str_radix("_", 2).ok();
    assert_eq!(blank, None);
    let blank_one = BigUint::from_str_radix("_1", 2).ok();
    assert_eq!(blank_one, None);
    let plus_one = BigUint::from_str_radix("+1", 10).ok();
    assert_eq!(plus_one, Some(BigUint::from_slice(&[1])));
    let plus_plus_one = BigUint::from_str_radix("++1", 10).ok();
    assert_eq!(plus_plus_one, None);
    let minus_one = BigUint::from_str_radix("-1", 10).ok();
    assert_eq!(minus_one, None);
    let zero_plus_two = BigUint::from_str_radix("0+2", 10).ok();
    assert_eq!(zero_plus_two, None);
    let three = BigUint::from_str_radix("1_1", 2).ok();
    assert_eq!(three, Some(BigUint::from_slice(&[3])));
    let ff = BigUint::from_str_radix("1111_1111", 2).ok();
    assert_eq!(ff, Some(BigUint::from_slice(&[0xff])));
}

#[test]
fn test_all_str_radix() {
    use std::ascii::AsciiExt;

    let n = BigUint::new((0..10).collect());
    for radix in 2..37 {
        let s = n.to_str_radix(radix);
        let x = BigUint::from_str_radix(&s, radix);
        assert_eq!(x.unwrap(), n);

        let s = s.to_ascii_uppercase();
        let x = BigUint::from_str_radix(&s, radix);
        assert_eq!(x.unwrap(), n);
    }
}

#[test]
fn test_lower_hex() {
    let a = BigUint::parse_bytes(b"A", 16).unwrap();
    let hello = BigUint::parse_bytes("22405534230753963835153736737".as_bytes(), 10).unwrap();

    assert_eq!(format!("{:x}", a), "a");
    assert_eq!(format!("{:x}", hello), "48656c6c6f20776f726c6421");
    assert_eq!(format!("{:♥>+#8x}", a), "♥♥♥♥+0xa");
}

#[test]
fn test_upper_hex() {
    let a = BigUint::parse_bytes(b"A", 16).unwrap();
    let hello = BigUint::parse_bytes("22405534230753963835153736737".as_bytes(), 10).unwrap();

    assert_eq!(format!("{:X}", a), "A");
    assert_eq!(format!("{:X}", hello), "48656C6C6F20776F726C6421");
    assert_eq!(format!("{:♥>+#8X}", a), "♥♥♥♥+0xA");
}

#[test]
fn test_binary() {
    let a = BigUint::parse_bytes(b"A", 16).unwrap();
    let hello = BigUint::parse_bytes("224055342307539".as_bytes(), 10).unwrap();

    assert_eq!(format!("{:b}", a), "1010");
    assert_eq!(format!("{:b}", hello),
               "110010111100011011110011000101101001100011010011");
    assert_eq!(format!("{:♥>+#8b}", a), "♥+0b1010");
}

#[test]
fn test_octal() {
    let a = BigUint::parse_bytes(b"A", 16).unwrap();
    let hello = BigUint::parse_bytes("22405534230753963835153736737".as_bytes(), 10).unwrap();

    assert_eq!(format!("{:o}", a), "12");
    assert_eq!(format!("{:o}", hello), "22062554330674403566756233062041");
    assert_eq!(format!("{:♥>+#8o}", a), "♥♥♥+0o12");
}

#[test]
fn test_display() {
    let a = BigUint::parse_bytes(b"A", 16).unwrap();
    let hello = BigUint::parse_bytes("22405534230753963835153736737".as_bytes(), 10).unwrap();

    assert_eq!(format!("{}", a), "10");
    assert_eq!(format!("{}", hello), "22405534230753963835153736737");
    assert_eq!(format!("{:♥>+#8}", a), "♥♥♥♥♥+10");
}

#[test]
fn test_factor() {
    fn factor(n: usize) -> BigUint {
        let mut f: BigUint = One::one();
        for i in 2..n + 1 {
            // FIXME(#5992): assignment operator overloads
            // f *= FromPrimitive::from_usize(i);
            let bu: BigUint = FromPrimitive::from_usize(i).unwrap();
            f = f * bu;
        }
        return f;
    }

    fn check(n: usize, s: &str) {
        let n = factor(n);
        let ans = match BigUint::from_str_radix(s, 10) {
            Ok(x) => x,
            Err(_) => panic!(),
        };
        assert_eq!(n, ans);
    }

    check(3, "6");
    check(10, "3628800");
    check(20, "2432902008176640000");
    check(30, "265252859812191058636308480000000");
}

#[test]
fn test_bits() {
    assert_eq!(BigUint::new(vec![0, 0, 0, 0]).bits(), 0);
    let n: BigUint = FromPrimitive::from_usize(0).unwrap();
    assert_eq!(n.bits(), 0);
    let n: BigUint = FromPrimitive::from_usize(1).unwrap();
    assert_eq!(n.bits(), 1);
    let n: BigUint = FromPrimitive::from_usize(3).unwrap();
    assert_eq!(n.bits(), 2);
    let n: BigUint = BigUint::from_str_radix("4000000000", 16).unwrap();
    assert_eq!(n.bits(), 39);
    let one: BigUint = One::one();
    assert_eq!((one << 426).bits(), 427);
}

#[test]
fn test_rand() {
    let mut rng = thread_rng();
    let _n: BigUint = rng.gen_biguint(137);
    assert!(rng.gen_biguint(0).is_zero());
}

#[test]
fn test_rand_range() {
    let mut rng = thread_rng();

    for _ in 0..10 {
        assert_eq!(rng.gen_bigint_range(&FromPrimitive::from_usize(236).unwrap(),
                                        &FromPrimitive::from_usize(237).unwrap()),
                   FromPrimitive::from_usize(236).unwrap());
    }

    let l = FromPrimitive::from_usize(403469000 + 2352).unwrap();
    let u = FromPrimitive::from_usize(403469000 + 3513).unwrap();
    for _ in 0..1000 {
        let n: BigUint = rng.gen_biguint_below(&u);
        assert!(n < u);

        let n: BigUint = rng.gen_biguint_range(&l, &u);
        assert!(n >= l);
        assert!(n < u);
    }
}

#[test]
#[should_panic]
fn test_zero_rand_range() {
    thread_rng().gen_biguint_range(&FromPrimitive::from_usize(54).unwrap(),
                                   &FromPrimitive::from_usize(54).unwrap());
}

#[test]
#[should_panic]
fn test_negative_rand_range() {
    let mut rng = thread_rng();
    let l = FromPrimitive::from_usize(2352).unwrap();
    let u = FromPrimitive::from_usize(3513).unwrap();
    // Switching u and l should fail:
    let _n: BigUint = rng.gen_biguint_range(&u, &l);
}

fn test_mul_divide_torture_count(count: usize) {
    use rand::{SeedableRng, StdRng, Rng};

    let bits_max = 1 << 12;
    let seed: &[_] = &[1, 2, 3, 4];
    let mut rng: StdRng = SeedableRng::from_seed(seed);

    for _ in 0..count {
        // Test with numbers of random sizes:
        let xbits = rng.gen_range(0, bits_max);
        let ybits = rng.gen_range(0, bits_max);

        let x = rng.gen_biguint(xbits);
        let y = rng.gen_biguint(ybits);

        if x.is_zero() || y.is_zero() {
            continue;
        }

        let prod = &x * &y;
        assert_eq!(&prod / &x, y);
        assert_eq!(&prod / &y, x);
    }
}

#[test]
fn test_mul_divide_torture() {
    test_mul_divide_torture_count(1000);
}

#[test]
#[ignore]
fn test_mul_divide_torture_long() {
    test_mul_divide_torture_count(1000000);
}