# Slabbale Validation

Cross-validate varioous Slabbable implementations.

See [blog](https://github.com/pinkforest/pinkforest/blob/main/2025-01-25-slabbable.md).

## Check memory usage

Default impl (slabbable-hash which uses hashbrown-nohash_hasher as default)
```ignore
$ cargo run
```

slab impl
```ignore
$ env RUSTFLAGS='--cfg slabbable_impl="slab"' cargo run
```ignore

stablevec impl
```ignore
$ env RUSTFLAGS='--cfg slabbable_impl="stablevec"' cargo run
```ignore

## Benchmark

Use cargo bench instead of cargo run as above.

## Validate

Use cargo test instead of cargo run as above.
