//! Finite fields
//!
//! Provides the standard structure for finite fields and their quadratic extensions.
//! It also includes specific finite fields implementation used for SIKE

use std::fmt::Debug;

pub mod ff_p434;
pub mod ff_p503;
pub mod ff_p610;
pub mod ff_p751;

pub use crate::ff::{
    ff_p434::PrimeFieldP434, ff_p503::PrimeFieldP503, ff_p610::PrimeFieldP610,
    ff_p751::PrimeFieldP751,
};

/// Finite field element
pub trait FiniteField: Sized {
    /// Check if the element is the additive identity of the field
    fn is_zero(&self) -> bool;

    /// Returns the dimension of the finite field
    fn dimension() -> usize;

    /// Returns the additive identity of the field
    fn zero() -> Self;

    /// Returns the multiplicative identity of the field
    fn one() -> Self;

    /// Returns the additive inverse of the element
    fn neg(&self) -> Self;

    /// Returns the multiplicative inverse of the element
    fn inv(&self) -> Result<Self, String>;

    /// Defines the addition of two elements
    fn add(&self, other: &Self) -> Self;

    /// Defines the substraction of two elements
    fn sub(&self, other: &Self) -> Self;

    /// Defines the multiplication of two elements
    fn mul(&self, other: &Self) -> Self;

    /// Defines the divison of two elements
    fn div(&self, other: &Self) -> Result<Self, String>;

    /// Checks if two elements are equal
    fn equals(&self, other: &Self) -> bool;

    /// Converts the element to a bytes representation
    fn into_bytes(self) -> Vec<u8>;

    /// Converts a bytes representation to an element of the finite field
    fn from_bytes(bytes: &[u8]) -> Result<Self, String>;
}

/// Given a specific finite field 𝔽ₚ, represents an element of
/// its quadratic extension 𝔽ₚ(i) as `x = a + ib`, (`i² = -1`)
#[derive(Clone, Copy, PartialEq)]
pub struct QuadraticExtension<F: FiniteField> {
    a: F,
    b: F,
}

impl<F: FiniteField + Debug> Debug for QuadraticExtension<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} + i {:?}", self.a, self.b)
    }
}

impl<F: FiniteField> QuadraticExtension<F> {
    /// Generates an element of the quadratic extension given two elements of the base field: `z = a + i b`.
    pub fn from(a: F, b: F) -> Self {
        Self { a, b }
    }
}

impl<F: FiniteField + Debug> FiniteField for QuadraticExtension<F> {
    fn is_zero(&self) -> bool {
        self.a.is_zero() && self.b.is_zero()
    }

    fn dimension() -> usize {
        2 * F::dimension()
    }

    fn zero() -> Self {
        Self {
            a: F::zero(),
            b: F::zero(),
        }
    }

    fn one() -> Self {
        Self {
            a: F::one(),
            b: F::zero(),
        }
    }

    fn neg(&self) -> Self {
        Self {
            a: self.a.neg(),
            b: self.b.neg(),
        }
    }

    fn add(&self, other: &Self) -> Self {
        Self {
            a: self.a.add(&other.a),
            b: self.b.add(&other.b),
        }
    }

    fn sub(&self, other: &Self) -> Self {
        self.add(&other.neg())
    }

    fn div(&self, other: &Self) -> Result<Self, String> {
        Ok(self.mul(&other.inv()?))
    }

    fn mul(&self, other: &Self) -> Self {
        let m1 = self.a.mul(&other.a);
        let m2 = self.b.mul(&other.b);

        let m3 = self.a.mul(&other.b);
        let m4 = other.a.mul(&self.b);

        Self {
            a: m1.sub(&m2),
            b: m3.add(&m4),
        }
    }

    fn inv(&self) -> Result<Self, String> {
        let asq = self.a.mul(&self.a);
        let bsq = self.b.mul(&self.b);
        let inv_norm = asq.add(&bsq).inv()?;

        Ok(Self {
            a: inv_norm.mul(&self.a),
            b: inv_norm.mul(&self.b.neg()),
        })
    }

    fn equals(&self, other: &Self) -> bool {
        self.a.equals(&other.a) && self.b.equals(&other.b)
    }

    fn into_bytes(self) -> Vec<u8> {
        use crate::utils::conversion::concatenate;

        let part1 = self.a.into_bytes();
        let part2 = self.b.into_bytes();

        // Left padding to the nearest power of 2
        let p21 = part1.len().next_power_of_two();
        let p22 = part2.len().next_power_of_two();
        let len = std::cmp::max(p21, p22);

        let pad1 = vec![0; len - part1.len()];
        let pad2 = vec![0; len - part2.len()];

        concatenate(&[&pad1, &part1, &pad2, &part2])
    }

    /// Element from byte representation (ref `ostofp2` Algorithm 1.2.4.)
    fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        let n = bytes.len() / 2;
        let a = F::from_bytes(&bytes[..n])?;
        let b = F::from_bytes(&bytes[n..])?;
        Ok(Self::from(a, b))
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::constants::cs_p434::{SIKE_P434_XP20, SIKE_P434_XP21};

    #[test]
    fn test_conversion_ff434_bytes() {
        let num = PrimeFieldP434::from_string(SIKE_P434_XP20).unwrap();

        let b = num.clone().into_bytes();
        let num_recovered = PrimeFieldP434::from_bytes(&b).unwrap();

        println!("{:?}", num);
        println!("{:?}", num_recovered);

        assert!(num.equals(&num_recovered));
    }

    #[test]
    fn test_conversion_quadratic_bytes() {
        let num1 = PrimeFieldP434::from_string(SIKE_P434_XP20).unwrap();
        let num2 = PrimeFieldP434::from_string(SIKE_P434_XP21).unwrap();

        let q = QuadraticExtension::from(num1, num2);
        let b = q.clone().into_bytes();
        let q_recovered = QuadraticExtension::from_bytes(&b).unwrap();

        println!("{:?}", q);
        println!("{:?}", q_recovered);

        assert!(q.equals(&q_recovered));
    }

    #[test]
    fn test_ff() {
        let one = PrimeFieldP434::one();
        let two = one.add(&one);
        let three = two.add(&one);
        let four1 = two.add(&two);
        let four2 = two.mul(&two);
        let zero = one.sub(&one);

        println!("zero = {:?}", zero);
        println!("one = {:?}", one);
        println!("two = {:?}", two);
        println!("three = {:?}", three);
        println!("four1 = {:?}", four1);
        println!("four2 = {:?}", four2);
    }

    #[test]
    fn test_qff() {
        let one = PrimeFieldP434::one();
        let two = one.add(&one);
        let x = QuadraticExtension::from(two.clone(), two.clone());

        let eight_i = x.mul(&x);

        println!("eight_i = {:?}", eight_i);

        let two_plus_two_i = eight_i.div(&x).unwrap();

        println!("two_plus_two_i = {:?}", two_plus_two_i);

        assert_eq!(two_plus_two_i, x)
    }
}
