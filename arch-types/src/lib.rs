#![allow(non_camel_case_types)]
use std::ops::{BitAnd, BitOr, Shl};
use typenum::{marker_traits::Bit, False, True};

macro_rules! tuple_impl {
    // Initial state
    {
        $($features:ident)*
    } => {
        tuple_impl! {
            (T12, T11, T10, T9, T8, T7, T6, T5, T4, T3, T2, T1)
        }
    };

    // Unpack the next tuple
    {
        ($first:ident, $second:ident, $($rest:ident),+)
    } => {
        tuple_impl! {
            ($first, $second, $($rest),*)
            ($second, $($rest),*)
        }
    };

    // Special case the 2-tuple
    {
        ($first:ident, $second:ident)
    } => {
        unsafe impl<$first, $second> Features for ($first, $second)
        where
            $first: Features,
            $second: Features,
            <$first as Features>::Available: BitOr<<$second as Features>::Available>
        {
            type Available = <<$first as Features>::Available as BitOr<<$second as Features>::Available>>::Output;

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
            <$first as Features>::Available: BitOr<<$next as Features>::Available>
        {
            type Available = <<$first as Features>::Available as BitOr<<$next as Features>::Available>>::Output;

            fn detect() -> Option<Self> {
                Some(($first::detect()?, $($rest::detect()?),*))
            }

            unsafe fn new_unchecked() -> Self {
                ($first::new_unchecked(), $($rest::new_unchecked()),*)
            }
        }

        tuple_impl! {
            ($($rest),*)
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
            type Available;

            fn detect() -> Option<Self>;

            unsafe fn new_unchecked() -> Self;
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

        tuple_impl! { $($ident)* }

        features!{ @impl $detect_macro => $([$ident, $detect])* }
    };

    {
        @impl $detect_macro:ident => [$ident:ident, $detect:tt]
    } => {
        unsafe impl Features for $ident {
            type Available = typenum::U1;

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
    };

    {
        @impl $detect_macro:ident => [$ident:ident, $detect:tt] [$next:ident, $next_detect:tt] $($rest:tt)*
    } => {
        unsafe impl Features for $ident {
            type Available = <<$next as Features>::Available as Shl<typenum::U1>>::Output;

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

        features!{ @impl $detect_macro => [$next, $next_detect] $($rest)* }
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

#[cfg(test)]
mod test {
    use super::*;

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    #[test]
    fn feature_requirement() {
        fn foo<F>(_: F)
        where
            F: Features<sse = True, avx = True>,
        {
            println!(
                "sse: {}, avx: {}, avx2: {}",
                F::sse::BOOL,
                F::avx::BOOL,
                F::avx2::BOOL
            );
        }
        if let Some(tag) = <(sse, avx, avx2)>::detect() {
            foo(tag);
        }
    }
}
