#![allow(non_camel_case_types)]
use std::ops::{BitAnd, BitOr};
use typenum::{False, True};

macro_rules! features {
    {
        @detect_macro $detect_macro:ident
        $(
            @feature $ident:ident
            @detect $detect:tt
        )*
    } => {
        $(
            #[doc = "The `\""]
            #[doc = $detect]
            #[doc = " \"` feature."]
            pub struct $ident;

            unsafe impl Detect for Arch<$ident> {
                fn detect() -> Option<Self> {
                    if $detect_macro!($detect) {
                        Some(Self(std::marker::PhantomData))
                    } else {
                        None
                    }
                }

                unsafe fn new_unchecked() -> Self {
                    Self(std::marker::PhantomData)
                }
            }
        )*
        features! { @pack $($ident)* => [$($ident)*] }
    };

    // This rule packs the list of feature idents into a token tree, so they can be iterated later
    {
        @pack $($ident:ident)* => $all:tt
    } => {
        $(
            features! { @unpack $ident => $all }
        )*
    };

    // This rule unpacks the token tree to implement all of the traits
    {
        @unpack $ident:ident => [$($all:ident)*]
    } => {
        // This macro generates the Supports trait for just $ident
        macro_rules! generate_trait {
            // This is the target type!
            {
                $ident
            } => {
                unsafe impl Supports<Arch<$ident>> for Arch<$ident> {
                    type Value = True;
                }
            };

            // This is another type
            {
                $other:ident
            } => {
                unsafe impl Supports<Arch<$ident>> for Arch<$other> {
                    type Value = False;
                }
            }
        }

        // Generate the Supports trait for each feature
        $(
            generate_trait! { $all }
        )*
    }
}

features! {
    @detect_macro is_x86_feature_detected

    @feature sse
    @detect "sse"

    @feature avx
    @detect "avx"

    @feature avx2
    @detect "avx2"
}

pub unsafe trait Supports<T> {
    type Value;
}

struct Arch<T>(std::marker::PhantomData<T>);
struct And<T, U>((std::marker::PhantomData<T>, std::marker::PhantomData<U>));

// A pair of architectures support the target if either one supports it
unsafe impl<Target, Arch1, Arch2> Supports<Arch<Target>> for And<Arch1, Arch2>
where
    Arch1: Supports<Arch<Target>>,
    Arch2: Supports<Arch<Target>>,
    Arch1::Value: BitOr<Arch2::Value>,
{
    type Value = <Arch1::Value as BitOr<Arch2::Value>>::Output;
}

// A single architecture supports a pair of targets if it supports both individually
// The architecture must be wrapped in a 1-tuple to deconflict with the previous impl
unsafe impl<Target1, Target2, Arch> Supports<And<Target1, Target2>> for Arch
where
    Arch: Supports<Target1> + Supports<Target2>,
    <Arch as Supports<Target1>>::Value: BitAnd<<Arch as Supports<Target2>>::Value>,
{
    type Value =
        <<Arch as Supports<Target1>>::Value as BitAnd<<Arch as Supports<Target2>>::Value>>::Output;
}

pub unsafe trait Detect: Sized {
    fn detect() -> Option<Self>;

    unsafe fn new_unchecked() -> Self;
}

unsafe impl<T, U> Detect for And<T, U>
where
    T: Detect,
    U: Detect,
{
    fn detect() -> Option<Self> {
        if let (Some(_), Some(_)) = (T::detect(), U::detect()) {
            Some(Self(Default::default()))
        } else {
            None
        }
    }

    unsafe fn new_unchecked() -> Self {
        Self(Default::default())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn provide_one_feature() {
        fn foo(_: impl Supports<Arch<sse>, Value = True>) {}
        if let Some(tag) = Arch::<sse>::detect() {
            foo(tag);
        }
    }

    #[test]
    fn provide_two_features() {
        fn foo(_: impl Supports<Arch<sse>, Value = True>) {}
        if let Some(tag) = <And<Arch<sse>, Arch<avx>>>::detect() {
            foo(tag);
        }
    }

    #[test]
    fn provide_three_features() {
        fn foo(_: impl Supports<Arch<sse>, Value = True>) {}
        if let Some(tag) = <And<And<Arch<sse>, Arch<avx>>, Arch<avx>>>::detect() {
            foo(tag);
        }
    }

    #[test]
    fn require_two_features() {
        fn foo(_: impl Supports<And<Arch<sse>, Arch<avx>>, Value = True>) {}
        if let Some(tag) = And::<Arch<sse>, Arch<avx>>::detect() {
            foo(tag);
        }
    }

    #[test]
    fn require_three_features() {
        fn foo(_: impl Supports<And<And<Arch<sse>, Arch<avx>>, Arch<avx2>>, Value = True>) {}
        if let Some(tag) = And::<Arch<sse>, And<Arch<avx>, Arch<avx2>>>::detect() {
            foo(tag);
        }
    }
}
