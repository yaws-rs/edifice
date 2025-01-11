# Slabbable Impl Selector

Proxy for conveniently pulling and selecting Slabbable impl.

In your crate that pulls slabbable-impl-selector as dependency:
```bash
env RUSTFLAGS='--cfg slabbable_impl="impl"' cargo ..
```

And at code level:
```ignore
use slabbable_impl_selector::SelectedSlab;
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
