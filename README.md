arch-types
==========
[![Build Status](https://github.com/calebzulawski/arch-types/workflows/Build/badge.svg?branch=master)](https://github.com/calebzulawski/arch-types/actions)
![Rustc Version 1.34+](https://img.shields.io/badge/rustc-1.34+-lightgray.svg)

Type-level CPU feature detection using a tag dispatch model.

The following example uses a type that proves AVX support to make an AVX function safe to call:
```rust
use arch_types::{impl_features, new_features_type, Features};

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[target_feature(enable = "avx")]
unsafe fn foo_unsafe() {
    println!("hello from AVX!");
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn foo_safe(_: impl_features!("avx")) {
    unsafe { foo_unsafe() } // the trait bound ensures we support AVX
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn main() {
    new_features_type! { Avx => "avx" }
    if let Some(handle) = Avx::new() {
        foo_safe(handle)
    }
}
```

The following fails to compile due to missing AVX support:
```rust
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn main() {
    new_features_type! { NotAvx => "sse" }
    if let Some(handle) = NotAvx::new() {
        foo_safe(handle)
    }
}
```
