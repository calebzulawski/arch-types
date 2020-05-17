#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod x86 {
    fn sse(_: arch_types::requires_features! { "sse" }) {}
    fn sse2(_: arch_types::requires_features! { "sse2" }) {}
    fn sse_avx(_: arch_types::requires_features! { "sse", "avx" }) {}
    fn avx2(_: arch_types::requires_features! { "avx2" }) {}

    arch_types::new_features_type! { ArchSseSse2Avx => "sse", "sse2", "avx" }
    arch_types::new_features_type! { ArchSseAvxAvx2 => "sse", "avx", "avx2" }

    #[test]
    fn requires_features() {
        use arch_types::Features;
        if let Some(tag) = ArchSseSse2Avx::detect() {
            sse(tag);
            sse_avx(tag);
            sse2(tag);
        }
        if let Some(tag) = ArchSseAvxAvx2::detect() {
            sse(tag);
            sse_avx(tag);
            avx2(tag);
        }
    }
}
