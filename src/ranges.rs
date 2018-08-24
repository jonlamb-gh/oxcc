use core::marker::PhantomData;

use core::ops::{AddAssign, Mul, Sub, Div};
use num;
use typenum::{Cmp, Same, Unsigned, B1};
use typenum::consts::*;

// Although these say private, they're needed to write generic inequality
// constraints for typenums
use typenum::private::{IsGreaterOrEqualPrivate, IsLessOrEqualPrivate};

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

impl<T: Unsigned> ReifyTo<u32> for T {
    fn reify() -> u32 {
        <T as Unsigned>::to_u32()
    }
}

impl<T: Unsigned> ReifyTo<usize> for T {
    fn reify() -> usize {
        <T as Unsigned>::to_usize()
    }
}

pub trait PrimitiveBounds {
    type Min;
    type Max;
}

macro_rules! impl_primitive_bounds {
    ($T:ty, $TMin:ty, $TMax:ty) => {
        impl PrimitiveBounds for $T {
            type Min = $TMin;
            type Max = $TMax;
        }
    };
}

impl_primitive_bounds!(u8, U0, op!{U256 - U1});
impl_primitive_bounds!(u16, U0, op!{U65536 - U1});
impl_primitive_bounds!(u32, U0, op!{U4294967296 - U1});

#[derive(Debug)]
pub struct Bounded<T, L, U>
where
    // U <= T::Max
    T: PrimitiveBounds,
    U: Cmp<<T as PrimitiveBounds>::Max>,
    U: IsLessOrEqualPrivate<<T as PrimitiveBounds>::Max, <U as Cmp<<T as PrimitiveBounds>::Max>>::Output>,
    <U as IsLessOrEqualPrivate<<T as PrimitiveBounds>::Max, <U as Cmp<<T as PrimitiveBounds>::Max>>::Output>>::Output: Same<B1>,

    // L >= T::Min
    T: PrimitiveBounds,
    L: Cmp<<T as PrimitiveBounds>::Min>,
    L: IsGreaterOrEqualPrivate<<T as PrimitiveBounds>::Min, <L as Cmp<<T as PrimitiveBounds>::Min>>::Output>,
    <L as IsGreaterOrEqualPrivate<<T as PrimitiveBounds>::Min, <L as Cmp<<T as PrimitiveBounds>::Min>>::Output>>::Output: Same<B1>,
{
    val: T,
    _lower_inclusive: PhantomData<L>,
    _upper_inclusive: PhantomData<U>,
}

#[derive(Debug)]
pub enum BoundedError {
    LTLower,
    GTUpper,
}

impl<T, L, U> Bounded<T, L, U>
where
    T: PartialOrd,
    L: ReifyTo<T>,
    U: ReifyTo<T>,

    // U <= T::Max
    T: PrimitiveBounds,
    U: Cmp<<T as PrimitiveBounds>::Max>,
    U: IsLessOrEqualPrivate<<T as PrimitiveBounds>::Max, <U as Cmp<<T as PrimitiveBounds>::Max>>::Output>,
    <U as IsLessOrEqualPrivate<<T as PrimitiveBounds>::Max, <U as Cmp<<T as PrimitiveBounds>::Max>>::Output>>::Output: Same<B1>,

    // L >= T::Min
    T: PrimitiveBounds,
    L: Cmp<<T as PrimitiveBounds>::Min>,
    L: IsGreaterOrEqualPrivate<<T as PrimitiveBounds>::Min, <L as Cmp<<T as PrimitiveBounds>::Min>>::Output>,
    <L as IsGreaterOrEqualPrivate<<T as PrimitiveBounds>::Min, <L as Cmp<<T as PrimitiveBounds>::Min>>::Output>>::Output: Same<B1>,
{
    pub fn new(val: T) -> Result<Bounded<T, L, U>, BoundedError> {
        // TODO: check that upper >= lower
        if val < L::reify() {
            Err(BoundedError::LTLower)
        } else if val > U::reify() {
            Err(BoundedError::GTUpper)
        } else {
            Ok(Bounded {
                val,
                _lower_inclusive: PhantomData,
                _upper_inclusive: PhantomData,
            })
        }
    }

    pub fn val(&self) -> &T {
        &self.val
    }

    pub fn move_val(self) -> T {
        self.val
    }

    pub fn upper_bound(&self) -> T {
        U::reify()
    }

    pub fn lower_bound(&self) -> T {
        L::reify()
    }

    pub fn clamp(val: T) -> Bounded<T, L, U> {
        Bounded {
            val: num::clamp(val, L::reify(), U::reify()),
            _lower_inclusive: PhantomData,
            _upper_inclusive: PhantomData,
        }
    }
}

pub fn coerce<T, Lower1, Upper1, Lower2, Upper2>(
    b: Bounded<T, Lower1, Upper1>,
) -> Bounded<T, Lower2, Upper2>
where
    T: PartialOrd,
    Lower1: ReifyTo<T>,
    Upper1: ReifyTo<T>,
    Lower2: ReifyTo<T>,
    Upper2: ReifyTo<T>,

    // Lower2 <= Upper2
    Lower2: Cmp<Upper2>,
    Lower2: IsLessOrEqualPrivate<Upper2, <Lower2 as Cmp<Upper2>>::Output>,
    <Lower2 as IsLessOrEqualPrivate<Upper2, <Lower2 as Cmp<Upper2>>::Output>>::Output: Same<B1>,

    // Lower1 <= Lower2
    Lower2: Cmp<Lower1>,
    Lower2: IsLessOrEqualPrivate<Lower1, <Lower2 as Cmp<Lower1>>::Output>,
    <Lower2 as IsLessOrEqualPrivate<Lower1, <Lower2 as Cmp<Lower1>>::Output>>::Output: Same<B1>,

    // Lower2 <= Upper1
    Lower2: Cmp<Upper1>,
    Lower2: IsLessOrEqualPrivate<Upper1, <Lower2 as Cmp<Upper1>>::Output>,
    <Lower2 as IsLessOrEqualPrivate<Upper1, <Lower2 as Cmp<Upper1>>::Output>>::Output: Same<B1>,

    // Upper1 <= Upper2
    Upper1: Cmp<Upper2>,
    Upper1: IsLessOrEqualPrivate<Upper2, <Upper1 as Cmp<Upper2>>::Output>,
    <Upper1 as IsLessOrEqualPrivate<Upper2, <Upper1 as Cmp<Upper2>>::Output>>::Output: Same<B1>,

    // Upper1 <= T::Max
    T: PrimitiveBounds,
    Upper1: Cmp<<T as PrimitiveBounds>::Max>,
    Upper1: IsLessOrEqualPrivate<<T as PrimitiveBounds>::Max, <Upper1 as Cmp<<T as PrimitiveBounds>::Max>>::Output>,
    <Upper1 as IsLessOrEqualPrivate<<T as PrimitiveBounds>::Max, <Upper1 as Cmp<<T as PrimitiveBounds>::Max>>::Output>>::Output: Same<B1>,

    // Upper2 <= T::Max
    T: PrimitiveBounds,
    Upper2: Cmp<<T as PrimitiveBounds>::Max>,
    Upper2: IsLessOrEqualPrivate<<T as PrimitiveBounds>::Max, <Upper2 as Cmp<<T as PrimitiveBounds>::Max>>::Output>,
    <Upper2 as IsLessOrEqualPrivate<<T as PrimitiveBounds>::Max, <Upper2 as Cmp<<T as PrimitiveBounds>::Max>>::Output>>::Output: Same<B1>,

    // Lower1 >= T::Min
    T: PrimitiveBounds,
    Lower1: Cmp<<T as PrimitiveBounds>::Min>,
    Lower1: IsGreaterOrEqualPrivate<<T as PrimitiveBounds>::Min, <Lower1 as Cmp<<T as PrimitiveBounds>::Min>>::Output>,
    <Lower1 as IsGreaterOrEqualPrivate<<T as PrimitiveBounds>::Min, <Lower1 as Cmp<<T as PrimitiveBounds>::Min>>::Output>>::Output: Same<B1>,

    // Lower2 >= T::Min
    T: PrimitiveBounds,
    Lower2: Cmp<<T as PrimitiveBounds>::Min>,
    Lower2: IsGreaterOrEqualPrivate<<T as PrimitiveBounds>::Min, <Lower2 as Cmp<<T as PrimitiveBounds>::Min>>::Output>,
    <Lower2 as IsGreaterOrEqualPrivate<<T as PrimitiveBounds>::Min, <Lower2 as Cmp<<T as PrimitiveBounds>::Min>>::Output>>::Output: Same<B1>,
{
    Bounded {
        val: b.val,
        _lower_inclusive: PhantomData,
        _upper_inclusive: PhantomData,
    }
}

pub struct Summation<SumT, SumL: ReifyTo<usize>, SumU: ReifyTo<usize>> {
    _sum: PhantomData<SumT>,
    _sum_lower: PhantomData<SumL>,
    _sum_upper: PhantomData<SumU>,
}

pub trait BoundedSummation<F, T, L, U>
where
    F: Fn(usize) -> Bounded<T, L, U>,

    // U <= T::Max
    T: PrimitiveBounds,
    U: Cmp<<T as PrimitiveBounds>::Max>,
    U: IsLessOrEqualPrivate<<T as PrimitiveBounds>::Max, <U as Cmp<<T as PrimitiveBounds>::Max>>::Output>,
    <U as IsLessOrEqualPrivate<<T as PrimitiveBounds>::Max, <U as Cmp<<T as PrimitiveBounds>::Max>>::Output>>::Output: Same<B1>,

    // L >= T::Min
    T: PrimitiveBounds,
    L: Cmp<<T as PrimitiveBounds>::Min>,
    L: IsGreaterOrEqualPrivate<<T as PrimitiveBounds>::Min, <L as Cmp<<T as PrimitiveBounds>::Min>>::Output>,
    <L as IsGreaterOrEqualPrivate<<T as PrimitiveBounds>::Min, <L as Cmp<<T as PrimitiveBounds>::Min>>::Output>>::Output: Same<B1>,
{
    type Output;
    fn eval(f: F) -> Self::Output;
}

impl<SumT, SumL, SumU, F, T, L, U> BoundedSummation<F, T, L, U> for Summation<SumT, SumL, SumU>
where
    SumT: Default + AddAssign + From<T>,
    SumL: ReifyTo<usize>,
    SumU: ReifyTo<usize> + Sub<SumL>,
    F: Fn(usize) -> Bounded<T, L, U>,
    T: PartialOrd,
    L: ReifyTo<T> + Mul<op!{SumU - SumL}>,
    U: ReifyTo<T> + Mul<op!{SumU - SumL}>,

    // U <= T::Max
    T: PrimitiveBounds,
    U: Cmp<<T as PrimitiveBounds>::Max>,
    U: IsLessOrEqualPrivate<<T as PrimitiveBounds>::Max, <U as Cmp<<T as PrimitiveBounds>::Max>>::Output>,
    <U as IsLessOrEqualPrivate<<T as PrimitiveBounds>::Max, <U as Cmp<<T as PrimitiveBounds>::Max>>::Output>>::Output: Same<B1>,

    // L >= T::Min
    T: PrimitiveBounds,
    L: Cmp<<T as PrimitiveBounds>::Min>,
    L: IsGreaterOrEqualPrivate<<T as PrimitiveBounds>::Min, <L as Cmp<<T as PrimitiveBounds>::Min>>::Output>,
    <L as IsGreaterOrEqualPrivate<<T as PrimitiveBounds>::Min, <L as Cmp<<T as PrimitiveBounds>::Min>>::Output>>::Output: Same<B1>,

    // SumU <= SumT::Max
    SumT: PrimitiveBounds,
    SumU: Cmp<<SumT as PrimitiveBounds>::Max>,
    SumU: IsLessOrEqualPrivate<<SumT as PrimitiveBounds>::Max, <SumU as Cmp<<SumT as PrimitiveBounds>::Max>>::Output>,
    <SumU as IsLessOrEqualPrivate<<SumT as PrimitiveBounds>::Max, <SumU as Cmp<<SumT as PrimitiveBounds>::Max>>::Output>>::Output: Same<B1>,

    // L >= T::Min
    SumT: PrimitiveBounds,
    SumL: Cmp<<SumT as PrimitiveBounds>::Min>,
    SumL: IsGreaterOrEqualPrivate<<SumT as PrimitiveBounds>::Min, <SumL as Cmp<<SumT as PrimitiveBounds>::Min>>::Output>,
    <SumL as IsGreaterOrEqualPrivate<<SumT as PrimitiveBounds>::Min, <SumL as Cmp<<SumT as PrimitiveBounds>::Min>>::Output>>::Output: Same<B1>,

    // U * (SumU - SumL) <= SumT::Max
    <U as Mul<<SumU as Sub<SumL>>::Output>>::Output: Cmp<<SumT as PrimitiveBounds>::Max>,
    <U as Mul<<SumU as Sub<SumL>>::Output>>::Output: IsLessOrEqualPrivate<<SumT as PrimitiveBounds>::Max, <<U as Mul<<SumU as Sub<SumL>>::Output>>::Output as Cmp<<SumT as PrimitiveBounds>::Max>>::Output>,
    <<U as Mul<<SumU as Sub<SumL>>::Output>>::Output as IsLessOrEqualPrivate<<SumT as PrimitiveBounds>::Max, <<U as Mul<<SumU as Sub<SumL>>::Output>>::Output as Cmp<<SumT as PrimitiveBounds>::Max>>::Output>>::Output: Same<B1>,

    // L * (SumU - SumL) >= SumT::Min
    <L as Mul<<SumU as Sub<SumL>>::Output>>::Output: Cmp<<SumT as PrimitiveBounds>::Min>,
    <L as Mul<<SumU as Sub<SumL>>::Output>>::Output: IsGreaterOrEqualPrivate<<SumT as PrimitiveBounds>::Min, <<L as Mul<<SumU as Sub<SumL>>::Output>>::Output as Cmp<<SumT as PrimitiveBounds>::Min>>::Output>,
    <<L as Mul<<SumU as Sub<SumL>>::Output>>::Output as IsGreaterOrEqualPrivate<<SumT as PrimitiveBounds>::Min, <<L as Mul<<SumU as Sub<SumL>>::Output>>::Output as Cmp<<SumT as PrimitiveBounds>::Min>>::Output>>::Output: Same<B1>,

{
    type Output = Bounded<SumT, op!{L * (SumU - SumL)}, op!{U * (SumU - SumL)}>;

    fn eval(f: F) -> Self::Output {
        let mut sum: SumT = SumT::default();
        for index in SumL::reify()..SumU::reify() {
            sum += f(index).move_val().into();
        }

        Bounded {
            val: sum,
            _lower_inclusive: PhantomData,
            _upper_inclusive: PhantomData,
        }
    }
}

pub struct ConstDiv<Divisor> {
    _divisor: PhantomData<Divisor>,
}

pub trait BoundedConstDiv<T, L, U>
where
    // U <= T::Max
    T: PrimitiveBounds,
    U: Cmp<<T as PrimitiveBounds>::Max>,
    U: IsLessOrEqualPrivate<<T as PrimitiveBounds>::Max, <U as Cmp<<T as PrimitiveBounds>::Max>>::Output>,
    <U as IsLessOrEqualPrivate<<T as PrimitiveBounds>::Max, <U as Cmp<<T as PrimitiveBounds>::Max>>::Output>>::Output: Same<B1>,

    // L >= T::Min
    T: PrimitiveBounds,
    L: Cmp<<T as PrimitiveBounds>::Min>,
    L: IsGreaterOrEqualPrivate<<T as PrimitiveBounds>::Min, <L as Cmp<<T as PrimitiveBounds>::Min>>::Output>,
    <L as IsGreaterOrEqualPrivate<<T as PrimitiveBounds>::Min, <L as Cmp<<T as PrimitiveBounds>::Min>>::Output>>::Output: Same<B1>,
{
    type Output;
    fn eval(Bounded<T, L, U>) -> Self::Output;
}

impl<T, L, U, Divisor> BoundedConstDiv<T, L, U> for ConstDiv<Divisor>
where
    T: PartialOrd + Div + From<<T as Div>::Output>,
    L: ReifyTo<T> + Div<Divisor>,
    U: ReifyTo<T> + Div<Divisor>,
    Divisor: ReifyTo<T>,

    // U <= T::Max
    T: PrimitiveBounds,
    U: Cmp<<T as PrimitiveBounds>::Max>,
    U: IsLessOrEqualPrivate<<T as PrimitiveBounds>::Max, <U as Cmp<<T as PrimitiveBounds>::Max>>::Output>,
    <U as IsLessOrEqualPrivate<<T as PrimitiveBounds>::Max, <U as Cmp<<T as PrimitiveBounds>::Max>>::Output>>::Output: Same<B1>,

    // L >= T::Min
    T: PrimitiveBounds,
    L: Cmp<<T as PrimitiveBounds>::Min>,
    L: IsGreaterOrEqualPrivate<<T as PrimitiveBounds>::Min, <L as Cmp<<T as PrimitiveBounds>::Min>>::Output>,
    <L as IsGreaterOrEqualPrivate<<T as PrimitiveBounds>::Min, <L as Cmp<<T as PrimitiveBounds>::Min>>::Output>>::Output: Same<B1>,

    // U / Divisor <= T::Max
    <U as Div<Divisor>>::Output: Cmp<<T as PrimitiveBounds>::Max>,
    <U as Div<Divisor>>::Output: IsLessOrEqualPrivate<<T as PrimitiveBounds>::Max, <<U as Div<Divisor>>::Output as Cmp<<T as PrimitiveBounds>::Max>>::Output>,
    <<U as Div<Divisor>>::Output as IsLessOrEqualPrivate<<T as PrimitiveBounds>::Max, <<U as Div<Divisor>>::Output as Cmp<<T as PrimitiveBounds>::Max>>::Output>>::Output: Same<B1>,

    // L / Divisor >= T::Min
    <L as Div<Divisor>>::Output: Cmp<<T as PrimitiveBounds>::Min>,
    <L as Div<Divisor>>::Output: IsGreaterOrEqualPrivate<<T as PrimitiveBounds>::Min, <<L as Div<Divisor>>::Output as Cmp<<T as PrimitiveBounds>::Min>>::Output>,
    <<L as Div<Divisor>>::Output as IsGreaterOrEqualPrivate<<T as PrimitiveBounds>::Min, <<L as Div<Divisor>>::Output as Cmp<<T as PrimitiveBounds>::Min>>::Output>>::Output: Same<B1>,
{
    type Output = Bounded<T, op!{ L / Divisor }, op!{ U / Divisor }>;

    fn eval(b: Bounded<T, L, U>) -> Self::Output {
        Bounded {
            val: (b.move_val() / Divisor::reify()).into(),
            _lower_inclusive: PhantomData,
            _upper_inclusive: PhantomData,
        }
    }
}

pub trait LossyCoerceNum<T> {
    fn lossy_coerce_num(self) -> T;
}

macro_rules! impl_lossy_coerce_num {
    ($TSelf:ty, $TOut:ty) => {
        impl LossyCoerceNum<$TOut> for $TSelf {
            #[inline]
            fn lossy_coerce_num(self) -> $TOut {
                self as $TOut
            }
        }
    };
}

impl_lossy_coerce_num!(u8, u8);
impl_lossy_coerce_num!(u8, u16);
impl_lossy_coerce_num!(u8, u32);

impl_lossy_coerce_num!(u16, u8);
impl_lossy_coerce_num!(u16, u16);
impl_lossy_coerce_num!(u16, u32);

impl_lossy_coerce_num!(u32, u8);
impl_lossy_coerce_num!(u32, u16);
impl_lossy_coerce_num!(u32, u32);

pub fn retype<T1, T2, L, U>(b: Bounded<T1, L, U>) -> Bounded<T2, L, U>
where
    T1: PartialOrd + LossyCoerceNum<T2>,
    T2: PartialOrd,
    L: ReifyTo<T1> + ReifyTo<T2>,
    U: ReifyTo<T1> + ReifyTo<T2>,

    // U <= T1::Max
    T1: PrimitiveBounds,
    U: Cmp<<T1 as PrimitiveBounds>::Max>,
    U: IsLessOrEqualPrivate<<T1 as PrimitiveBounds>::Max, <U as Cmp<<T1 as PrimitiveBounds>::Max>>::Output>,
    <U as IsLessOrEqualPrivate<<T1 as PrimitiveBounds>::Max, <U as Cmp<<T1 as PrimitiveBounds>::Max>>::Output>>::Output: Same<B1>,

    // L >= T1::Min
    T1: PrimitiveBounds,
    L: Cmp<<T1 as PrimitiveBounds>::Min>,
    L: IsGreaterOrEqualPrivate<<T1 as PrimitiveBounds>::Min, <L as Cmp<<T1 as PrimitiveBounds>::Min>>::Output>,
    <L as IsGreaterOrEqualPrivate<<T1 as PrimitiveBounds>::Min, <L as Cmp<<T1 as PrimitiveBounds>::Min>>::Output>>::Output: Same<B1>,

    // U <= T2::Max
    T2: PrimitiveBounds,
    U: Cmp<<T2 as PrimitiveBounds>::Max>,
    U: IsLessOrEqualPrivate<<T2 as PrimitiveBounds>::Max, <U as Cmp<<T2 as PrimitiveBounds>::Max>>::Output>,
    <U as IsLessOrEqualPrivate<<T2 as PrimitiveBounds>::Max, <U as Cmp<<T2 as PrimitiveBounds>::Max>>::Output>>::Output: Same<B1>,

    // L >= T2::Min
    T2: PrimitiveBounds,
    L: Cmp<<T2 as PrimitiveBounds>::Min>,
    L: IsGreaterOrEqualPrivate<<T2 as PrimitiveBounds>::Min, <L as Cmp<<T2 as PrimitiveBounds>::Min>>::Output>,
    <L as IsGreaterOrEqualPrivate<<T2 as PrimitiveBounds>::Min, <L as Cmp<<T2 as PrimitiveBounds>::Min>>::Output>>::Output: Same<B1>,
{
    Bounded {
        val: b.move_val().lossy_coerce_num(),
        _lower_inclusive: PhantomData,
        _upper_inclusive: PhantomData,
    }
}
