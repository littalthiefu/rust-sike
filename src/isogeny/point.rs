//! Points in projective coordinates

use crate::ff::FiniteField;
use std::fmt::{Debug, Formatter, Result};

/// Point defined by (X: Z) in projective coordinates
#[derive(Clone)]
pub struct Point<K: FiniteField + Clone> {
    /// X coordinate in projective space
    pub x: K,
    /// Z coordinate in projective space
    pub z: K,
}

impl<K: FiniteField + Clone + Debug> Debug for Point<K> {
    /// A point is represented as (x : z)
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "({:?}:{:?})", self.x, self.z)
    }
}

impl<K: FiniteField + Clone> Point<K> {
    /// Returns the points (x : 1)
    #[inline]
    pub fn from_x(x: K) -> Self {
        Self { x, z: K::one() }
    }
}

impl<K: FiniteField + Clone> PartialEq<Self> for Point<K> {
    /// Two points are equal if (z != 0 and x/z) match, or if z = 0 for both
    fn eq(&self, other: &Self) -> bool {
        let other_zero = other.z.is_zero();
        if self.z.is_zero() {
            other_zero
        } else if other_zero {
            false
        } else {
            // Z / Z' are not zero and operation take place on finite field, div cannot panic
            let ratio = self.x.div(&self.z).unwrap();
            let other_ratio = &other.x.div(&other.z).unwrap();
            ratio.equals(&other_ratio)
        }
    }
}
