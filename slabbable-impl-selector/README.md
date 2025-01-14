# Slabbable Impl Selector

Proxy for conveniently pulling and selecting Slabbable impl.

In your crate that pulls slabbable-impl-selector as dependency:
```bash
env RUSTFLAGS='--cfg slabbable_impl="impl"' cargo ..
```

Cargo.toml
```toml
slabbable = { version = "0.1" }
slabbable-impl-selector = { version = "0.1" }
```

Code level:
```ignore
use slabbable::{Slabbable, SlabbableError};
use ::slabbable_impl_selector::SelectedSlab;

#[derive(Clone, Debug)]
struct Holder {
  my_stuff: SelectedSlab<usize>,
}

let mut slab = SelectedSlab::<usize>::with_fixed_capacity(1);
slab.take_next_with(1).expect("Could not take the first one");
assert!(slab.take_next_with(2), Err(SlabbableError::AtCapacity(1)));

```

## cfg(slabbable_impl = "..")

| value       | rotating usize? | description               |
| :---        | :---            | :---                      |
| [stablevec] | no              | StableVec                 |
| [slab]      | no              | Slab                      |
| [hash]      [ yes             | Hash                      |

Default impl is hash.

The choice of the implementation selection is solely by the top-level binary otherwise.

## rotating usize

Rotating usize allows avoiding immedia re-use of key index until whole usize has spinned over.

Without rotating, the re-use will pick-up slots that may have been recently free'd.

[stablevec]: https://docs.rs/slabbable-impl-stablevec
[slab]: https://docs.rs/slabbable-impl-slab
[hash]: https://docs.rs/slabbable-impl-hash