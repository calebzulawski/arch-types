#![allow(non_camel_case_types)]
#![cfg_attr(not(feature = "std"), no_std)]
//! This crate provides type-level CPU feature detection using a tag dispatch model.
//!
//! Tag types implement the [`Features`] trait, which proves either statically or dynamically
//! that a particular set of features is supported by the CPU.
//!
//! The [`new_features_type`] macro creates tag types and [`impl_features`] and [`has_features`]
//! ensure CPU features are supported statically and dynamically, respectively.
//!
//! # Cargo features
//! This crate provides the following cargo features:
//!   * `std` (enabled by default) - Use the `std` crate for feature detection.  Disable this
//!     feature for `#[no_std]` support.
//!   * `nightly` - Enable nightly features.  This includes run-time feature detection for some
//!     architectures, as well as detection of some particular features.
//!
//! If feature detection cannot be performed (either not using `std` or not using a nightly
//! compiler for a particular feature or architecture), feature detection is performed at compile
//! time using `#[cfg(target_feature)]`.
//!
//! [`Features`]: trait.Features.html
//! [`new_features_type`]: macro.new_features_type.html
//! [`impl_features`]: macro.impl_features.html
//! [`has_features`]: macro.has_features.html

// Cannot be (safely) constructed in other crates.
#[doc(hidden)]
#[derive(Copy, Clone)]
pub struct UnsafeConstructible(());

impl UnsafeConstructible {
    #[doc(hidden)]
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

/// Constructs a feature set from another feature set.
///
/// You should not implement this trait.  It is automatically implemented by [`new_features_type`].
///
/// [`new_features_type`]: macro.new_features_type.html
pub trait FromFeatures<T>: Features
where
    T: Features,
{
    /// Construct this from another feature set.
    fn from_features(features: T) -> Self;
}

#[allow(unused_macros)]
macro_rules! features {
    {
        @detect_macro $detect_macro:ident
        $(
            @feature $ident:ident
            @detect $feature_lit:tt
            @version #$attr:tt $version_string:literal
        )*
    } => {
        /// Indicates the presence of available CPU features.
        ///
        /// An instance of a type implementing `Features` serves as a proof that the specified CPU
        /// features are supported by the CPU.
        pub unsafe trait Features: Copy {
            $(
                #[doc = "Indicates presence of the `"]
                #[doc = $feature_lit]
                #[doc = "` feature. Requires Rust "]
                #[doc = $version_string]
                #[doc = "."]
                type $ident: $crate::logic::Bool;
            )*

            /// Detect the existence of these features, returning `None` if it isn't supported by the
            /// CPU.
            ///
            /// When the `std` feature is enabled, this function performs feature detection.
            /// Otherwise, available features are determined with `target_arch`.
            #[inline(always)]
            fn new() -> Option<Self> {
                use $crate::logic::Bool;

                $(
                    if Self::$ident::VALUE && !detect::$ident() {
                        return None;
                    }
                )*
                Some(unsafe { Self::new_unchecked() })
            }

            /// Create a new architecture type handle.
            ///
            /// # Safety
            /// Undefined behavior if the feature set is not supported by the CPU.
            unsafe fn new_unchecked() -> Self;

            /// Convert this into a subset of this feature set, if possible.
            #[inline(always)]
            fn shrink<T>(self) -> Option<T>
            where
                T: Features
            {
                $(
                    if <<T as $crate::Features>::$ident as $crate::logic::Bool>::VALUE && !<Self::$ident as $crate::logic::Bool>::VALUE {
                        return None;
                    }
                )*
                unsafe { Some(T::new_unchecked()) }
            }

            /// Convert this into another feature set, performing additional feature detection if
            /// necessary.
            #[inline(always)]
            fn expand<T>(self) -> Option<T>
            where
                T: Features
            {
                $(
                    if <<T as $crate::Features>::$ident as $crate::logic::Bool>::VALUE && !<Self::$ident as $crate::logic::Bool>::VALUE && !detect::$ident() {
                        return None;
                    }
                )*
                unsafe { Some(T::new_unchecked()) }
            }
        }

        features! { @with_dollar ($), $detect_macro => $([$attr, $ident, $feature_lit])* }
    };

    {
        @with_dollar ($dollar:tt), $detect_macro:ident => $([$attr:tt, $ident:ident, $feature_lit:tt])*
    } => {
        #[macro_export]
        #[doc(hidden)]
        macro_rules! new_features_type_internal {
            $(
                {
                    [$dollar($docs:literal)*] $vis:vis $name:ident => [$feature_lit $dollar($feature:tt)*] => [$dollar($feature_ident:tt)*]
                } => {
                    $crate::new_features_type_internal! { [$dollar($docs)*] $vis $name => [$dollar($feature)*] => [$dollar($feature_ident)* $ident] }
                };
            )*

            {
                [$dollar($docs:literal)*] $vis:vis $name:ident => [] => [$dollar($feature:ident)*]
            } => {
                $dollar(#[doc = $docs])*
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
                    { $other:ident } => { $crate::logic::False };
                }
                unsafe impl $crate::Features for $name {
                    $(
                        type $ident = __associated_type!{ $ident };
                    )*

                    unsafe fn new_unchecked() -> Self {
                        Self(unsafe { $crate::UnsafeConstructible::new() })
                    }
                }

                impl<T> $crate::FromFeatures<T> for $name
                where
                    T: $crate::Features<$dollar($feature = $crate::logic::True),*>,
                {
                    #[inline(always)]
                    fn from_features(features: T) -> $name {
                        features.shrink().unwrap()
                    }
                }
            }
        }

        #[macro_export]
        #[doc(hidden)]
        macro_rules! impl_features_internal {
            { [] => [$dollar($output:tt)*] } => {
                impl $crate::Features<$dollar($output)*>
            };

            $(
                { [$feature_lit $dollar($rest:tt)*] => [$dollar($output:tt)*] } => {
                    $crate::impl_features_internal!{ [$dollar($rest)*] => [ $ident = $crate::logic::True, $dollar($output)* ] }
                };
            )*

            { [$dollar($all:tt)*] => [$dollar($output:tt)*] } => {
                compile_error!("unknown feature")
            };
        }

        #[macro_export]
        #[doc(hidden)]
        macro_rules! has_features_internal {
            $(
                { $type:ty => $feature_lit } => {
                    <<$type as $crate::Features>::$ident as $crate::logic::Bool>::VALUE
                };
            )*

            { $name:ident => $unknown:tt } => {
                compile_error!("unknown feature")
            }
        }

        mod detect {
            macro_rules! implement_detector {
                {
                    [nightly], $impl_feature_lit:tt, $impl_ident:ident
                } => {
                    // If supported, detect the feature
                    #[cfg(feature = "nightly")]
                    #[rustversion::nightly]
                    #[inline(always)]
                    pub(crate) fn $impl_ident() -> bool {
                        #[cfg(target_feature = $impl_feature_lit)]
                        {
                            true
                        }
                        #[cfg(not(target_feature = $impl_feature_lit))]
                        {
                            #[cfg(feature = "std")]
                            {
                                $detect_macro!($impl_feature_lit)
                            }
                            #[cfg(not(feature = "std"))]
                            {
                                false
                            }
                        }
                    }

                    // If not supported, we don't detect it
                    #[cfg(feature = "nightly")]
                    #[rustversion::not(nightly)]
                    #[inline(always)]
                    pub(crate) fn $impl_ident() -> bool {
                        false
                    }

                    // If not nightly, we also don't detect it
                    #[cfg(not(feature = "nightly"))]
                    #[inline(always)]
                    pub(crate) fn $impl_ident() -> bool {
                        false
                    }
                };

                {
                    [$dollar($impl_attr:tt)*], $impl_feature_lit:tt, $impl_ident:ident
                } => {
                    // If supported, detect the feature
                    #[rustversion::$dollar($impl_attr)*]
                    #[inline(always)]
                    pub(crate) fn $impl_ident() -> bool {
                        #[cfg(target_feature = $impl_feature_lit)]
                        {
                            true
                        }
                        #[cfg(not(target_feature = $impl_feature_lit))]
                        {
                            #[cfg(feature = "std")]
                            {
                                $detect_macro!($impl_feature_lit)
                            }
                            #[cfg(not(feature = "std"))]
                            {
                                false
                            }
                        }
                    }

                    // If not supported, we don't detect it
                    #[rustversion::not($dollar($impl_attr)*)]
                    #[inline(always)]
                    pub(crate) fn $impl_ident() -> bool {
                        false
                    }
                }
            }

            $(
                implement_detector!{$attr, $feature_lit, $ident}
            )*
        }
    }
}

/// Evaluates to an `impl Features` requiring particular CPU features.
///
/// For example, `impl_features!{ "sse", "avx" }` evaluates to `impl Features<sse =
/// True, avx = True>`.
///
/// This is useful for making unsafe functions safe to call:
/// ```
/// use arch_types::{impl_features, new_features_type, Features};
///
/// #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
/// #[target_feature(enable = "avx")]
/// unsafe fn foo_unsafe() {
///     println!("hello from AVX!");
/// }
///
/// #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
/// fn foo_safe(_: impl_features!("avx")) {
///     unsafe { foo_unsafe() } // the trait bound ensures we support AVX
/// }
///
/// #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
/// fn main() {
///     new_features_type! { Avx => "avx" }
///     if let Some(handle) = Avx::new() {
///         foo_safe(handle)
///     }
/// }
/// ```
///
/// The following example fails to compile due to the incorrect feature being provided,
/// demonstrating that `foo_safe` is safe:
/// ```compile_fail
/// # use arch_types::{impl_features, new_features_type, has_features, Features};
/// #
/// # #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
/// # #[target_feature(enable = "avx")]
/// # unsafe fn foo_unsafe() {
/// #     println!("hello from AVX!");
/// # }
/// #
/// # #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
/// # fn foo_safe(_: impl_features!("avx")) {
/// #     unsafe { foo_unsafe() } // the trait bound ensures we support AVX
/// # }
/// #
/// #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
/// fn main() {
///     new_features_type! { NotAvx => "sse" }
///     if let Some(handle) = NotAvx::new() {
///         foo_safe(handle)
///     }
/// }
///
/// ```
#[macro_export]
macro_rules! impl_features {
    { $($feature:tt),* } => {
        $crate::impl_features_internal!{ [$($feature)*] => [] }
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
///     assert!(has_features!(type SseAvxType => "sse", "avx"));
///     if let Some(handle) = SseAvxType::new() {
///         assert!(has_features!(handle => "sse", "avx"));
///     }
/// }
/// ```
#[macro_export]
macro_rules! has_features {
    { type $features_type:ty => $($feature:tt),+ } => {
        { $($crate::has_features_internal!( $features_type => $feature ) &&)* true }
    };

    { $features_expr:expr => $($feature:tt),+ } => {
        {
            #[inline(always)]
            fn __value<F>(_: F) -> bool
            where
                F: $crate::Features,
            {
                $crate::has_features!(type F => $($feature),*)
            }
            __value($features_expr)
        }
    };
}

/// Creates a new type that proves support of the specified CPU features.
///
/// The generated type implements `Copy`, `Clone`, `Debug`, [`Features`], and [`FromFeatures`].
/// The only way to construct the type is via one of the methods in [`Features`].
///
/// The following creates a type `SseAvxType` that indicates support for SSE and AVX:
/// ```
/// #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
/// arch_types::new_features_type! { SseAvxType => "sse", "avx" }
///
/// # #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
/// # fn main() {
/// #     use arch_types::{has_features, Features};
/// #     if let Some(handle) = SseAvxType::new() {
/// #         assert!(has_features!(handle => "sse", "avx"));
/// #     }
/// # }
/// ```
///
/// Optionally, the type can be documented:
/// ```
/// #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
/// arch_types::new_features_type! { #[doc = "A type supporting SSE and AVX."] SseAvxType => "sse", "avx" }
/// ```
/// [`Features`]: trait.Features.html
/// [`FromFeatures`]: trait.FromFeatures.html
#[macro_export]
macro_rules! new_features_type {
    { $vis:vis $name:ident => $($feature:tt),* } => { $crate::new_features_type_internal!{ [] $vis $name => [$($feature)*] => [] } };
    { $(#[doc = $docs:literal])* $vis:vis $name:ident => $($feature:tt),* } => { $crate::new_features_type_internal!{ [$($docs)*] $vis $name => [$($feature)*] => [] } }
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
features! {
    @detect_macro is_x86_feature_detected

    @feature aes
    @detect "aes"
    @version #[since(1.33)] "1.33"

    @feature pclmulqdq
    @detect "pclmulqdq"
    @version #[since(1.33)] "1.33"

    @feature rdrand
    @detect "rdrand"
    @version #[since(1.33)] "1.33"

    @feature rdseed
    @detect "rdseed"
    @version #[since(1.33)] "1.33"

    @feature tsc
    @detect "tsc"
    @version #[since(1.33)] "1.33"

    @feature mmx
    @detect "mmx"
    @version #[since(1.33)] "1.33"

    @feature sse
    @detect "sse"
    @version #[since(1.33)] "1.33"

    @feature sse2
    @detect "sse2"
    @version #[since(1.33)] "1.33"

    @feature sse3
    @detect "sse3"
    @version #[since(1.33)] "1.33"

    @feature ssse3
    @detect "ssse3"
    @version #[since(1.33)] "1.33"

    @feature sse41
    @detect "sse4.1"
    @version #[since(1.33)] "1.33"

    @feature sse42
    @detect "sse4.2"
    @version #[since(1.33)] "1.33"

    @feature sse4a
    @detect "sse4a"
    @version #[since(1.33)] "1.33"

    @feature sha
    @detect "sha"
    @version #[since(1.33)] "1.33"

    @feature avx
    @detect "avx"
    @version #[since(1.33)] "1.33"

    @feature avx2
    @detect "avx2"
    @version #[since(1.33)] "1.33"

    @feature avx512f
    @detect "avx512f"
    @version #[since(1.33)] "1.33"

    @feature avx512cd
    @detect "avx512cd"
    @version #[since(1.33)] "1.33"

    @feature avx512er
    @detect "avx512er"
    @version #[since(1.33)] "1.33"

    @feature avx512pf
    @detect "avx512pf"
    @version #[since(1.33)] "1.33"

    @feature avx512bw
    @detect "avx512bw"
    @version #[since(1.33)] "1.33"

    @feature avx512dq
    @detect "avx512dq"
    @version #[since(1.33)] "1.33"

    @feature avx512vl
    @detect "avx512vl"
    @version #[since(1.33)] "1.33"

    @feature avx512ifma
    @detect "avx512ifma"
    @version #[since(1.33)] "1.33"

    @feature avx512vbmi
    @detect "avx512vbmi"
    @version #[since(1.33)] "1.33"

    @feature avx512vpopcntdq
    @detect "avx512vpopcntdq"
    @version #[since(1.33)] "1.33"

    @feature avx512vbmi2
    @detect "avx512vbmi2"
    @version #[since(1.43.1)] "1.43.1"

    @feature avx512gfni
    @detect "avx512gfni"
    @version #[since(1.43.1)] "1.43.1"

    @feature avx512vaes
    @detect "avx512vaes"
    @version #[since(1.43.1)] "1.43.1"

    @feature avx512vpclmulqdq
    @detect "avx512vpclmulqdq"
    @version #[since(1.43.1)] "1.43.1"

    @feature avx512vnni
    @detect "avx512vnni"
    @version #[since(1.43.1)] "1.43.1"

    @feature avx512bitalg
    @detect "avx512bitalg"
    @version #[since(1.43.1)] "1.43.1"

    @feature avx512bf16
    @detect "avx512bf16"
    @version #[since(1.43.1)] "1.43.1"

    @feature avx512vp2intersect
    @detect "avx512vp2intersect"
    @version #[since(1.43.1)] "1.43.1"

    @feature f16c
    @detect "f16c"
    @version #[since(1.38)] "1.38"

    @feature fma
    @detect "fma"
    @version #[since(1.33)] "1.33"

    @feature bmi1
    @detect "bmi1"
    @version #[since(1.33)] "1.33"

    @feature bmi2
    @detect "bmi2"
    @version #[since(1.33)] "1.33"

    @feature abm
    @detect "abm"
    @version #[since(1.33)] "1.33"

    @feature lzcnt
    @detect "lzcnt"
    @version #[since(1.33)] "1.33"

    @feature tbm
    @detect "tbm"
    @version #[since(1.33)] "1.33"

    @feature popcnt
    @detect "popcnt"
    @version #[since(1.33)] "1.33"

    @feature fxsr
    @detect "fxsr"
    @version #[since(1.33)] "1.33"

    @feature xsave
    @detect "xsave"
    @version #[since(1.33)] "1.33"

    @feature xsaveopt
    @detect "xsaveopt"
    @version #[since(1.33)] "1.33"

    @feature xsaves
    @detect "xsaves"
    @version #[since(1.33)] "1.33"

    @feature xsavec
    @detect "xsavec"
    @version #[since(1.33)] "1.33"

    @feature cmpxchg16b
    @detect "cmpxchg16b"
    @version #[since(1.33)] "1.33"

    @feature adx
    @detect "adx"
    @version #[since(1.33)] "1.33"

    @feature rtm
    @detect "rtm"
    @version #[since(1.38)] "1.38"
}

#[cfg(all(target_arch = "arm"))]
features! {
    @detect_macro is_arm_feature_detected

    @feature neon
    @detect "neon"
    @version #[nightly] "nightly"

    @feature pmull
    @detect "pmull"
    @version #[nightly] "nightly"

    @feature crc
    @detect "crc"
    @version #[nightly] "nightly"

    @feature crypto
    @detect "crypto"
    @version #[nightly] "nightly"
}

#[cfg(all(target_arch = "aarch64"))]
features! {
    @detect_macro is_aarch64_feature_detected

    @feature neon
    @detect "neon"
    @version #[nightly] "nightly"

    @feature pmull
    @detect "pmull"
    @version #[nightly] "nightly"

    @feature fp
    @detect "fp"
    @version #[nightly] "nightly"

    @feature fp16
    @detect "fp16"
    @version #[nightly] "nightly"

    @feature sve
    @detect "sve"
    @version #[nightly] "nightly"

    @feature crc
    @detect "crc"
    @version #[nightly] "nightly"

    @feature crypto
    @detect "crypto"
    @version #[nightly] "nightly"

    @feature lse
    @detect "lse"
    @version #[nightly] "nightly"

    @feature rdm
    @detect "rdm"
    @version #[nightly] "nightly"

    @feature rcpc
    @detect "rcpc"
    @version #[nightly] "nightly"

    @feature dotprod
    @detect "dotprod"
    @version #[nightly] "nightly"
}

#[cfg(all(target_arch = "mips"))]
features! {
    @detect_macro is_mips_feature_detected

    @feature msa
    @detect "msa"
    @version #[nightly] "nightly"
}

#[cfg(all(target_arch = "mips64"))]
features! {
    @detect_macro is_mips64_feature_detected

    @feature msa
    @detect "msa"
    @version #[nightly] "nightly"
}

#[cfg(all(target_arch = "powerpc"))]
features! {
    @detect_macro is_powerpc_feature_detected

    @feature altivec
    @detect "altivec"
    @version #[nightly] "nightly"

    @feature vsx
    @detect "vsx"
    @version #[nightly] "nightly"

    @feature power8
    @detect "power8"
    @version #[nightly] "nightly"
}

#[cfg(all(target_arch = "powerpc64"))]
features! {
    @detect_macro is_powerpc64_feature_detected

    @feature altivec
    @detect "altivec"
    @version #[nightly] "nightly"

    @feature vsx
    @detect "vsx"
    @version #[nightly] "nightly"

    @feature power8
    @detect "power8"
    @version #[nightly] "nightly"
}
