/// INVARIANT: No Raw node construction in production code.
///
/// Raw is deprecated. This test proves it's not used in the
/// generation pipeline. Any new code that constructs Raw will
/// fail this test.

#[test]
fn no_raw_in_production_code() {
    let sources = [
        ("node.rs (outside tests)", include_str!("../src/node.rs")),
        ("emitter.rs", include_str!("../src/emitter.rs")),
        ("builders.rs", include_str!("../src/builders.rs")),
        ("syn_gen.rs", include_str!("../src/syn_gen.rs")),
        // validators.rs excluded — it's test infrastructure
    ];

    for (name, source) in &sources {
        // Split at #[cfg(test)] to only check non-test code
        let production_code = source.split("#[cfg(test)]").next().unwrap_or(source);

        // Count Raw constructor calls (not variant definitions or emit match arms)
        let raw_constructors = production_code
            .lines()
            .filter(|line| {
                let trimmed = line.trim();
                // Skip variant definition, emit match arm, deprecated attribute, allow attribute
                !trimmed.starts_with("///")
                    && !trimmed.starts_with("//")
                    && !trimmed.starts_with("#[deprecated")
                    && !trimmed.starts_with("#[allow")
                    && !trimmed.starts_with("Self::Raw")
                    && !trimmed.contains("Raw(String)")
                    && (trimmed.contains("::Raw(") || trimmed.contains("::raw("))
            })
            .count();

        assert_eq!(
            raw_constructors, 0,
            "INVARIANT VIOLATION: {name} has {raw_constructors} Raw constructor(s) in production code"
        );
    }
}
