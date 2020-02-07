use super::float::ExtendedFloat;
use super::num::*;

// INTO FLOAT

// Export extended-precision float to native float.
//
// The extended-precision float must be in native float representation,
// with overflow/underflow appropriately handled.
pub(crate) fn into_float<F>(fp: ExtendedFloat) -> F
    where F: Float
{
    // Export floating-point number.
    if fp.mant == 0 || fp.exp < F::DENORMAL_EXPONENT {
        // sub-denormal, underflow
        F::ZERO
    } else if fp.exp >= F::MAX_EXPONENT {
        // overflow
        F::from_bits(F::INFINITY_BITS)
    } else {
        // calculate the exp and fraction bits, and return a float from bits.
        let exp: u64;
        if (fp.exp == F::DENORMAL_EXPONENT) && (fp.mant & F::HIDDEN_BIT_MASK.as_u64()) == 0 {
            exp = 0;
        } else {
            exp = (fp.exp + F::EXPONENT_BIAS).as_u64();
        }
        let exp = exp << F::MANTISSA_SIZE;
        let mant = fp.mant & F::MANTISSA_MASK.as_u64();
        F::from_bits(F::Unsigned::as_cast(mant | exp))
    }
}
