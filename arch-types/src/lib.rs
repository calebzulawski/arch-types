#![allow(non_camel_case_types)]
#![cfg_attr(not(feature = "std"), no_std)]
//! This crate provides type-level CPU feature detection using a tag dispatch model.
//!
//! Tag types implement the [`Features`] trait, which indicates which CPU features are supported.
//! The [`new_features_type`] macro creates tag types and [`require_features`] and [`has_features`]
//! ensure CPU features are supported at compile time and run time, respectively.
//!
//! [`Features`]: trait.Features.html
//! [`new_features_type`]: macro.new_features_type.html
//! [`require_features`]: macro.require_features.html
//! [`has_features`]: macro.has_features.html

#[doc(hidden)]
#[derive(Copy, Clone)]
pub struct UnsafeConstructible(());

impl UnsafeConstructible {
    pub unsafe fn new() -> Self {
        Self(())
    }
}

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
                type $ident: $crate::logic::Bool;
            )*

            /// Detect the existence of these features, returning `None` if it isn't supported by the
            /// CPU.
            ///
            /// Requires the `std` feature.
            #[cfg(feature = "std")]
            fn detect() -> Option<Self> {
                use $crate::logic::Bool;
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

        features! { @with_dollar ($) => $([$ident, $feature_lit])* }
    };

    {
        @with_dollar ($dollar:tt) => $([$ident:ident, $feature_lit:tt])*
    } => {
        /// Creates a new type with the specified features.
        ///
        /// The generated type implements `Copy`, `Clone`, `Debug`, and [`Features`].  The only way
        /// to construct the type is via one of the methods in [`Features`].
        ///
        /// The following creates a type `SseAvxType` that indicates support for SSE and AVX:
        /// ```
        /// #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        /// arch_types::new_features_type! { SseAvxType => "sse", "avx" }
        ///
        /// # #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        /// # fn main() {
        /// #     use arch_types::{has_features, Features};
        /// #     if let Some(handle) = SseAvxType::detect() {
        /// #         assert!(has_features!(handle => "sse", "avx"));
        /// #     }
        /// # }
        /// ```
        ///
        /// [`Features`]: trait.Features.html
        #[macro_export]
        macro_rules! new_features_type {
            { $vis:vis $name:ident => $dollar($feature:tt),* } => {
                #[derive(Copy, Clone)]
                $vis struct $name($crate::UnsafeConstructible);

                impl core::fmt::Debug for $name {
                    fn fmt(&self, f: &mut core::fmt::Formatter) -> Result<(), core::fmt::Error> {
                        write!(f, stringify!($name))
                    }
                }

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
                        Self(unsafe { $crate::UnsafeConstructible::new() })
                    }
                }
            }
        }

        /// Evaluates to an `impl Features` requiring particular features.
        ///
        /// For example, `require_features!{ "sse", "avx" }` evaluates to `impl Features<sse =
        /// True, avx = True>`.
        ///
        /// This is useful for making unsafe functions safe to call:
        /// ```
        /// use arch_types::{require_features, new_features_type, Features};
        ///
        /// #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        /// #[target_feature(enable = "avx")]
        /// unsafe fn foo_unsafe() {
        ///     println!("hello from AVX!");
        /// }
        ///
        /// #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        /// fn foo_safe(_: require_features!("avx")) {
        ///     unsafe { foo_unsafe() } // the trait bound ensures we support AVX
        /// }
        ///
        /// #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        /// fn main() {
        ///     new_features_type! { Avx => "avx" }
        ///     if let Some(handle) = Avx::detect() {
        ///         foo_safe(handle)
        ///     }
        /// }
        /// ```
        ///
        /// The following example fails to compile due to the incorrect feature being provided,
        /// demonstrating that `foo_safe` is safe:
        /// ```compile_fail
        /// # use arch_types::{require_features, new_features_type, has_features, Features};
        /// #
        /// # #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        /// # #[target_feature(enable = "avx")]
        /// # unsafe fn foo_unsafe() {
        /// #     println!("hello from AVX!");
        /// # }
        /// #
        /// # #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        /// # fn foo_safe(_: require_features!("avx")) {
        /// #     unsafe { foo_unsafe() } // the trait bound ensures we support AVX
        /// # }
        /// #
        /// #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        /// fn main() {
        ///     new_features_type! { NotAvx => "sse" }
        ///     if let Some(handle) = NotAvx::detect() {
        ///         foo_safe(handle)
        ///     }
        /// }
        ///
        /// ```
        #[macro_export]
        macro_rules! require_features {
            { $dollar($feature:tt),* } => {
                $crate::require_features!{ @impl [$dollar($feature)*] => [] }
            };

            { @impl [] => [$dollar($output:tt)*] } => {
                impl $crate::Features<$dollar($output)*>
            };

            $(
                { @impl [$feature_lit $dollar($rest:tt)*] => [$dollar($output:tt)*] } => {
                    $crate::require_features!{ @impl [$dollar($rest)*] => [ $ident = $crate::logic::True, $dollar($output)* ] }
                };
            )*

            { @impl [$dollar($all:tt)*] => [$dollar($output:tt)*] } => {
                compile_error!("unknown feature")
            };
        }

        /// Reports the presence of features.
        ///
        /// This macro evaluates to `true` if all of the features are present, and `false`
        /// otherwise:
        ///
        /// ```
        /// use arch_types::{new_features_type, has_features, Features};
        ///
        /// #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        /// new_features_type! { SseAvxType => "sse", "avx" }
        ///
        /// #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        /// fn main() {
        ///     if let Some(handle) = SseAvxType::detect() {
        ///         assert!(has_features!(handle => "sse", "avx"));
        ///     }
        /// }
        /// ```
        #[macro_export]
        macro_rules! has_features {
            { $name:ident => $dollar($feature:tt),+ } => {
                { $dollar($crate::has_features!( @impl $name => $feature ) &&)* true }
            };

            $(
                { @impl $name:ident => $feature_lit } => {
                    {
                    fn __value<F>(_: F) -> bool
                    where
                        F: $crate::Features,
                    {
                        <F::$ident as $crate::logic::Bool>::VALUE
                    }
                    __value($name)
                    }
                };
            )*

            { @impl $name:ident => $unknown:tt } => {
                compile_error!("unknown feature")
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
