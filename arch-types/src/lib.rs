#![allow(non_camel_case_types)]

/// Type-level logic.
pub mod logic {
    /// A type-level `bool` type.
    pub trait Bool {
        const VALUE: bool;
    }

    /// A type-level `true`.
    #[derive(Copy, Clone, Debug)]
    pub struct True;

    /// A type-level `false`.
    #[derive(Copy, Clone, Debug)]
    pub struct False;

    impl Bool for True {
        const VALUE: bool = true;
    }

    impl Bool for False {
        const VALUE: bool = false;
    }
}

use logic::*;

macro_rules! features {
    {
        @detect_macro $detect_macro:ident
        $(
            @feature $ident:ident
            @detect $feature_lit:tt
        )*
    } => {
        /// Indicates the presence of available features.
        pub unsafe trait Features: Copy {
            $(
                #[doc = "Indicates presence of the `"]
                #[doc = $feature_lit]
                #[doc = "` feature."]
                type $ident: Bool;
            )*

            /// Detect the existence of these features, returning `None` if it isn't supported by the
            /// CPU.
            fn detect() -> Option<Self> {
                if $((!Self::$ident::VALUE || $detect_macro!($feature_lit)) && )* true {
                    Some(unsafe { Self::new_unchecked() })
                } else {
                    None
                }
            }

            /// Create a new architecture type handle.
            ///
            /// # Safety
            /// Undefined behavior if the feature is not supported by the CPU.
            unsafe fn new_unchecked() -> Self;
        }

        #[macro_export]
        macro_rules! feature_ident {
            $( { $feature_lit } => { $ident }; )*
            { $other:tt } => { compile_error!("unknown feature") }
        }

        features! { @with_dollar ($) => $([$ident, $feature_lit])* }
    };

    {
        @with_dollar ($dollar:tt) => $([$ident:ident, $feature_lit:tt])*
    } => {
        #[macro_export]
        macro_rules! new_features_type {
            { $vis:vis $name:ident => $dollar($feature:tt),* } => {
                #[derive(Copy, Clone, Debug)]
                $vis struct $name(());

                macro_rules! __associated_type {
                    $dollar(
                        { $feature } => { $crate::logic::True };
                    )*
                    { $other:tt } => { $crate::logic::False };
                }
                unsafe impl $crate::Features for $name {
                    $(
                        type $ident = __associated_type!{ $feature_lit };
                    )*

                    unsafe fn new_unchecked() -> Self {
                        Self(())
                    }
                }
            }
        }
    }
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
features! {
    @detect_macro is_x86_feature_detected

    @feature aes
    @detect "aes"

    @feature pclmulqdq
    @detect "pclmulqdq"

    @feature rdrand
    @detect "rdrand"

    @feature rdseed
    @detect "rdseed"

    @feature tsc
    @detect "tsc"

    @feature mmx
    @detect "mmx"

    @feature sse
    @detect "sse"

    @feature sse2
    @detect "sse2"

    @feature sse3
    @detect "sse3"

    @feature ssse3
    @detect "ssse3"

    @feature sse41
    @detect "sse4.1"

    @feature sse42
    @detect "sse4.2"

    @feature sse4a
    @detect "sse4a"

    @feature sha
    @detect "sha"

    @feature avx
    @detect "avx"

    @feature avx2
    @detect "avx2"

    @feature avx512f
    @detect "avx512f"

    @feature avx512cd
    @detect "avx512cd"

    @feature avx512er
    @detect "avx512er"

    @feature avx512pf
    @detect "avx512pf"

    @feature avx512bw
    @detect "avx512bw"

    @feature avx512dq
    @detect "avx512dq"

    @feature avx512vl
    @detect "avx512vl"

    @feature avx512ifma
    @detect "avx512ifma"

    @feature avx512vbmi
    @detect "avx512vbmi"

    @feature avx512vpopcntdq
    @detect "avx512vpopcntdq"

    @feature avx512vbmi2
    @detect "avx512vbmi2"

    @feature avx512gfni
    @detect "avx512gfni"

    @feature avx512vaes
    @detect "avx512vaes"

    @feature avx512vpclmulqdq
    @detect "avx512vpclmulqdq"

    @feature avx512vnni
    @detect "avx512vnni"

    @feature avx512bitalg
    @detect "avx512bitalg"

    @feature avx512bf16
    @detect "avx512bf16"

    @feature avx512vp2intersect
    @detect "avx512vp2intersect"

    @feature f16c
    @detect "f16c"

    @feature fma
    @detect "fma"

    @feature bmi1
    @detect "bmi1"

    @feature bmi2
    @detect "bmi2"

    @feature abm
    @detect "abm"

    @feature lzcnt
    @detect "lzcnt"

    @feature tbm
    @detect "tbm"

    @feature popcnt
    @detect "popcnt"

    @feature fxsr
    @detect "fxsr"

    @feature xsave
    @detect "xsave"

    @feature xsaveopt
    @detect "xsaveopt"

    @feature xsaves
    @detect "xsaves"

    @feature xsavec
    @detect "xsavec"

    @feature cmpxchg16b
    @detect "cmpxchg16b"

    @feature adx
    @detect "adx"

    @feature rtm
    @detect "rtm"
}
