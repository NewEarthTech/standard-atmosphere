# Contributing to standard-atmosphere

Thanks for your interest in improving `standard-atmosphere`. This is a small,
focused, dependency-free crate for International Standard Atmosphere (ISA / U.S.
Standard Atmosphere 1976) conversions, and contributions are genuinely welcome:
bug reports, fixes, tests, documentation, and correctness reviews. More eyes on
physical-model code is exactly the point of keeping it open.

## Ground rules

- **Correctness first.** This crate returns physical quantities other people
  build on. A change that alters a computed value must cite its source (the 1976
  standard, or another authoritative reference) and come with a known-value test.
- **Zero dependencies, by design.** The crate depends on nothing but the standard
  library. Please do not add dependencies.
- **Keep it teachable.** The code reads like a reference implementation:
  unit-suffixed names, explicit constants, and doc comments that explain the
  physics. New code should match that.

## Getting started

```bash
git clone https://github.com/NewEarthTech/standard-atmosphere
cd standard-atmosphere
cargo test
```

## Before you open a pull request

Everything CI checks, you can run locally:

```bash
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test
```

- Add a test that fails before your change and passes after. Known-value tests
  are ideal: an input with a published, expected output.
- Update the doc comments if behavior or units change.
- Keep each pull request to one focused change.

## Reporting issues

Open an issue with a minimal reproduction: the inputs, the value you got, and the
value you expected. For a correctness question, include the source that defines
the expected value.

## Releases and versioning

Maintainers cut releases from `main` by tagging `vX.Y.Z`; CI then publishes to
crates.io via Trusted Publishing. The crate follows semantic versioning, and its
minimum supported Rust version is declared in `Cargo.toml` and checked in CI.

## Provenance

`standard-atmosphere` is developed here as an independent open-source crate. It is
also used in production by [Orrery](https://orreryhq.com), a weather and energy
data platform, but the crate is generic and stands on its own: nothing
application-specific belongs in it.

## License

By contributing, you agree that your contributions are dual-licensed under
`MIT OR Apache-2.0`, the same terms as the crate.
