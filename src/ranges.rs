use core::marker::PhantomData;
use num;

use typenum::{Unsigned, Cmp, B1, Same};
// Although this says private, it's needed to write generic inequality
// constraints for typenums
use typenum::private::IsLessOrEqualPrivate;

/// Indicates that a type-level number may be converted to a runtime-level
/// number of the type T
pub trait ReifyTo<T> {
    fn reify() -> T;
}

impl<T: Unsigned> ReifyTo<u8> for T {
    fn reify() -> u8 {
        <T as Unsigned>::to_u8()
    }
}

impl<T: Unsigned> ReifyTo<u16> for T {
    fn reify() -> u16 {
        <T as Unsigned>::to_u16()
    }
}

#[derive(Debug)]
pub struct Bounded<T,L,U> {
    pub val: T,
    _lower_inclusive: PhantomData<L>,
    _upper_inclusive: PhantomData<U>,
}

impl<T: PartialOrd, L: ReifyTo<T>, U: ReifyTo<T>> Bounded<T,L,U> {
    pub fn clamp(val: T) -> Bounded<T,L,U> {
        Bounded {
            val: num::clamp(val, L::reify(), U::reify()),
            _lower_inclusive: PhantomData,
            _upper_inclusive: PhantomData
        }
    }
}

pub fn coerce<T, Lower1, Upper1, Lower2, Upper2>(b: Bounded<T, Lower1, Upper1>) -> Bounded<T, Lower2, Upper2>
where
    T: PartialOrd,
    Lower1: ReifyTo<T>,
    Upper1: ReifyTo<T>,
    Lower2: ReifyTo<T>,
    Upper2: ReifyTo<T>,

    // Lower2 <= Upper2
    Lower2 : Cmp<Upper2>,
    Lower2: IsLessOrEqualPrivate<Upper2, <Lower2 as Cmp<Upper2>>::Output>,
    <Lower2 as IsLessOrEqualPrivate<Upper2, <Lower2 as Cmp<Upper2>>::Output>>::Output : Same<B1>,

    // Lower1 <= Lower2
    Lower2 : Cmp<Lower1>,
    Lower2: IsLessOrEqualPrivate<Lower1, <Lower2 as Cmp<Lower1>>::Output>,
    <Lower2 as IsLessOrEqualPrivate<Lower1, <Lower2 as Cmp<Lower1>>::Output>>::Output : Same<B1>,

    // Lower2 <= Upper1
    Lower2 : Cmp<Upper1>,
    Lower2: IsLessOrEqualPrivate<Upper1, <Lower2 as Cmp<Upper1>>::Output>,
    <Lower2 as IsLessOrEqualPrivate<Upper1, <Lower2 as Cmp<Upper1>>::Output>>::Output : Same<B1>,

    // Upper1 <= Upper2
    Upper1 : Cmp<Upper2>,
    Upper1: IsLessOrEqualPrivate<Upper2, <Upper1 as Cmp<Upper2>>::Output>,
    <Upper1 as IsLessOrEqualPrivate<Upper2, <Upper1 as Cmp<Upper2>>::Output>>::Output : Same<B1>,
{
    Bounded {
        val: b.val,
        _lower_inclusive: PhantomData,
        _upper_inclusive: PhantomData,
    }
}
