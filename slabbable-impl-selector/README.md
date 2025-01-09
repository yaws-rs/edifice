# Slabbable Impl Selector

Proxy for conveniently pulling and selecting Slabbable impl.

## cfg(slabbable_impl = "..")

| value       | description               |
| :---        | :---                      |
| [stablevec] | StableVec                 |
| [slab]      | Slab                      |
| [hash]      [ Hash                      |

We do not select any default and this raises a compilation error if not selected.

The choice of the implementation selection is solely by the top-level binary.
