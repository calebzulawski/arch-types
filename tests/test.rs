#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod x86 {
    fn sse(tag: arch_types::impl_features! { "sse" }) {
        assert!(arch_types::has_features!(tag => "sse"));
    }

    fn sse2(tag: arch_types::impl_features! { "sse2" }) {
        assert!(arch_types::has_features!(tag => "sse2"));
    }

    fn sse_avx(tag: arch_types::impl_features! { "sse", "avx" }) {
        assert!(arch_types::has_features!(tag => "sse", "avx"));
    }

    fn avx2(tag: arch_types::impl_features! { "avx2" }) {
        assert!(arch_types::has_features!(tag => "avx2"));
    }

    arch_types::new_features_type! { ArchSseSse2Avx => "sse", "sse2", "avx" }
    arch_types::new_features_type! { ArchSseAvxAvx2 => "sse", "avx", "avx2" }
    arch_types::new_features_type! { ArchSseAvx2 => "sse", "avx2" }

    #[test]
    fn requires_features() {
        use arch_types::Features;
        if let Some(tag) = ArchSseSse2Avx::new() {
            sse(tag);
            sse_avx(tag);
            sse2(tag);
        }
        if let Some(tag) = ArchSseAvxAvx2::new() {
            sse(tag);
            sse_avx(tag);
            avx2(tag);
        }
    }

    #[test]
    fn shrink() {
        use arch_types::Features;
        if let Some(tag) = ArchSseSse2Avx::new() {
            assert!(tag.shrink::<ArchSseAvx2>().is_none());
            assert!(tag.shrink::<ArchSseAvxAvx2>().is_none());
        }
        if let Some(tag) = ArchSseAvxAvx2::new() {
            assert!(tag.shrink::<ArchSseAvx2>().is_some());
            assert!(tag.shrink::<ArchSseSse2Avx>().is_none());
        }
    }
}
