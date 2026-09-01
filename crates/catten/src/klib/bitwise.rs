/// Obtain a bitfield from a larger word size.
#[inline(always)]
pub fn mask_shift_read<T>(val: T, mask: T, shift: u8) -> T
where
    T: core::ops::BitAnd<Output = T> + core::ops::Shr<u8, Output = T>,
{
    (val & mask) >> shift
}
/// Used to compare against bitfields embedded in larger word sizes.
#[inline(always)]
pub fn mask_shift_cmp<T>(val: T, mask: T, shift: T, cmp: T) -> bool
where
    T: core::ops::BitAnd<Output = T> + core::ops::Shr<T, Output = T> + core::cmp::PartialEq + Copy,
{
    (val & mask) >> shift == cmp
}
#[inline(always)]
pub fn mask_from_shift_len<T>(shift: u8, len: u8) -> T
where
    T: core::ops::Shl<u8, Output = T>
        + core::ops::Sub<Output = T>
        + core::ops::BitOr<Output = T>
        + From<u8>,
{
    ((T::from(1) << len) - T::from(1)) << shift
}
/// write a bitfield into a larger word size.
#[inline(always)]
pub fn splice_into<T>(dest: &mut T, val: T, mask: T, shift: u8) -> Result<T, ()>
where
    T: core::ops::Not<Output = T>
        + core::ops::Shl<u8, Output = T>
        + core::ops::BitOrAssign<T>
        + core::ops::BitAndAssign<T>
        + core::ops::BitAnd<Output = T>
        + Copy,
{
    // clear the bits in dest that are covered by the mask
    dest.bitand_assign(!mask);
    // set the bits in dest
    dest.bitor_assign((val << shift) & mask);
    Ok(*dest)
}
#[inline(always)]
pub fn mask_from_len<T>(len: u8) -> T
where
    T: core::ops::Shl<u8, Output = T> + core::ops::Sub<Output = T> + From<u8>,
{
    (T::from(1) << len) - T::from(1)
}
