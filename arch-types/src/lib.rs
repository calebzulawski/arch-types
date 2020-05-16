#![allow(non_camel_case_types)]
use std::ops::{BitAnd, BitOr, Not};
use typenum::{marker_traits::Bit, False, True};

macro_rules! tuple_impl {
    // Initial state
    {
        $($features:ident)*
    } => {
        tuple_impl! {
            @features [$($features)*]
            (T12, T11, T10, T9, T8, T7, T6, T5, T4, T3, T2, T1)
        }
    };

    // Unpack the next tuple
    {
        @features $features:tt
        ($first:ident, $second:ident, $($rest:ident),+)
    } => {
        tuple_impl! {
            @features $features
            ($first, $second, $($rest),*)
            ($second, $($rest),*)
        }
    };

    // Special case the 2-tuple
    {
        @features [$($feature:ident)*]
        ($first:ident, $second:ident)
    } => {
        unsafe impl<$first, $second> Features for ($first, $second)
        where
            $first: Features,
            $second: Features,
            $(
                $first::$feature: BitOr<<$second as Features>::$feature>,
                <$first::$feature as BitOr<<$second as Features>::$feature>>::Output: Bit,
            )*
        {
            $(
                type $feature = <$first::$feature as BitOr<<$second as Features>::$feature>>::Output;
            )*

            fn detect() -> Option<Self> {
                Some(($first::detect()?, $second::detect()?))
            }

            unsafe fn new_unchecked() -> Self {
                ($first::new_unchecked(), $second::new_unchecked())
            }
        }
    };

    // Implement
    {
        @features [$($feature:ident)*]
        ($first:ident, $($rest:ident),+)
        $next:tt
    } => {
        unsafe impl<$first, $($rest),*> Features for ($first, $($rest),*)
        where
            $first: Features,
            $(
                $rest: Features,
            )*
            $next: Features,
            $(
                $first::$feature: BitOr<<$next as Features>::$feature>,
                <$first::$feature as BitOr<<$next as Features>::$feature>>::Output: Bit,
            )*
        {
            $(
                type $feature = <$first::$feature as BitOr<<$next as Features>::$feature>>::Output;
            )*

            fn detect() -> Option<Self> {
                Some(($first::detect()?, $($rest::detect()?),*))
            }

            unsafe fn new_unchecked() -> Self {
                ($first::new_unchecked(), $($rest::new_unchecked()),*)
            }
        }

        tuple_impl! {
            @features [$($feature)*]
            ($($rest),*)
        }
    };
}

macro_rules! subset_impl {
    {
        $ident:ident $($rest:ident)*
    } => {
        subset_impl! {
            @impl $($rest)*
            @value [
                <
                    <<Target as Features>::$ident as Not>::Output
                    as BitOr<<Arch as Features>::$ident>
                >::Output
            ]
            @bounds [
                <Target as Features>::$ident: Not,
                <<Target as Features>::$ident as Not>::Output: BitOr<<Arch as Features>::$ident>,
            ]
        }
    };

    {
        @impl
        @value [$($value:tt)*]
        @bounds [$($bounds:tt)*]
    } => {
        impl<Target, Arch> IsSubset<Target> for Arch
        where
            Target: Features,
            Arch: Features,
            $($value)*: Bit,
            $($bounds)*
        {
            type Value = $($value)*;
        }
    };

    {
        @impl $ident:ident $($rest:ident)*
        @value [$($value:tt)*]
        @bounds [$($bounds:tt)*]
    } => {
        subset_impl! {
            @impl $($rest)*
            @value [
                <
                    <
                        <<Target as Features>::$ident as Not>::Output
                        as BitOr<<Arch as Features>::$ident>
                    >::Output
                    as BitAnd<$($value)*>
                >::Output
            ]
            @bounds [
                $($bounds)*
                <Target as Features>::$ident: Not,
                <<Target as Features>::$ident as Not>::Output: BitOr<<Arch as Features>::$ident>,
                <
                    <<Target as Features>::$ident as Not>::Output
                    as BitOr<<Arch as Features>::$ident>
                >::Output: BitAnd<$($value)*>,
            ]
        }
    };
}

macro_rules! features {
    {
        @detect_macro $detect_macro:ident
        $(
            @feature $ident:ident
            @detect $detect:tt
        )*
    } => {
        /// Indicates the presence of available features.
        pub unsafe trait Features: Copy {
            $(
                type $ident: Bit;
            )*

            /// Detect the existence of this feature, returning `None` if it isn't supported by the
            /// CPU.
            fn detect() -> Option<Self>;

            /// Create a new feature type.
            ///
            /// # Safety
            /// Undefined behavior if the feature is not supported by the CPU.
            unsafe fn new_unchecked() -> Self;

            /// Determine if this feature is a subset of another feature.
            fn is_subset_of<Arch>(self) -> bool
            where
                Self: IsSubset<Arch>,
            {
                <Self as IsSubset<Arch>>::Value::BOOL
            }

            /// Determine if this feature is a superset of another feature.
            fn is_superset_of<Arch>(self) -> bool
            where
                Arch: IsSubset<Self>,
            {
                <Arch as IsSubset<Self>>::Value::BOOL
            }
        }

        $(
            #[doc = "The `"]
            #[doc = $detect]
            #[doc = "` feature."]
            #[derive(Copy, Clone, Debug)]
            pub struct $ident(());
        )*

        macro_rules! feature_as_type {
            $(
                {
                    $detect
                } => {
                    $ident
                };
            )*
        }

        subset_impl! { $($ident)* }

        tuple_impl! { $($ident)* }

        features! { @pack $detect_macro, $([$detect, $ident])* => [$($ident)*] }
    };

    // This rule packs the list of feature idents into a token tree, so they can be iterated later
    {
        @pack $detect_macro:ident, $([$detect:tt, $ident:ident])* => $all:tt
    } => {
        $(
            features! { @unpack $detect_macro, $detect, $ident => $all }
        )*
    };

    // This rule unpacks the token tree to implement all of the traits
    {
        @unpack $detect_macro:ident, $detect:tt, $ident:ident => [$($all:ident)*]
    } => {
        // This macro generates the Supports trait for just $ident
        macro_rules! generate_associated_type {
            // This is the target type!
            {
                $ident
            } => {
                type $ident = True;
            };

            // This is another type
            {
                $other:ident
            } => {
                type $other = False;
            }
        }

        unsafe impl Features for $ident {
            $(
                generate_associated_type! { $all }
            )*

            fn detect() -> Option<Self> {
                if $detect_macro!($detect) {
                    Some(Self(()))
                } else {
                    None
                }
            }

            unsafe fn new_unchecked() -> Self {
                Self(())
            }
        }
    }
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
features! {
    @detect_macro is_x86_feature_detected

    @feature sse
    @detect "sse"

    @feature avx
    @detect "avx"

    @feature avx2
    @detect "avx2"
}

/// Indicates if an architecture contains all of the features of another architecture.
pub trait IsSubset<Target> {
    /// `True` if `Self` is a subset of `Target`, `False` otherwise.
    type Value: Bit;
}

/// Ensures an architecture contains all of the features of another architecture.
pub trait Subset<Target>: IsSubset<Target, Value = True> {}
impl<Target, Arch> Subset<Target> for Arch
where
    Target: Features,
    Arch: IsSubset<Target, Value = True>,
{
}

#[cfg(test)]
mod test {
    use super::*;

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    #[test]
    fn feature_requirement() {
        type Required = (sse, avx);
        fn foo<F>(f: F)
        where
            F: Features + Subset<Required, Value = True>,
            Required: IsSubset<F>,
        {
            println!(
                "sse: {}, avx: {}, avx2: {}",
                F::sse::BOOL,
                F::avx::BOOL,
                F::avx2::BOOL
            );
            println!(
                "is superset: {}, is subset: {}",
                f.is_superset_of::<Required>(),
                f.is_subset_of::<Required>()
            );
        }
        if let Some(tag) = <(sse, avx, avx2)>::detect() {
            foo(tag);
        }
    }
}
