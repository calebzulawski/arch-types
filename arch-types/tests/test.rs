#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod x86 {
    fn sse<F>(_: F)
    where
        F: arch_types::Features<sse = arch_types::logic::True>,
    {
    }

    fn sse2<F>(_: F)
    where
        F: arch_types::Features<sse2 = arch_types::logic::True>,
    {
    }

    fn sse_avx<F>(_: F)
    where
        F: arch_types::Features<sse = arch_types::logic::True, avx = arch_types::logic::True>,
    {
    }

    fn avx2<F>(_: F)
    where
        F: arch_types::Features<avx2 = arch_types::logic::True>,
    {
    }

    arch_types::new_features_type! { ArchSseSse2Avx => "sse", "sse2", "avx" }
    arch_types::new_features_type! { ArchSseAvxAvx2 => "sse", "avx", "avx2" }

    #[test]
    fn x86() {
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
