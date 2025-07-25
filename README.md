# MuseMap

[![](https://img.shields.io/crates/v/musemap)](https://crates.io/crates/musemap)
[![](https://img.shields.io/crates/d/musemap)](https://crates.io/crates/musemap)
[![](https://img.shields.io/crates/l/musemap)](#)
[![](https://img.shields.io/docsrs/musemap)](https://docs.rs/musemap)
[![](https://img.shields.io/github/stars/eternal-io/musemap?style=social)](https://github.com/eternal-io/musemap)

Fast DoS-resistant hashmap based on [MuseAir] hash algorithm.

The output of the Hasher in this crate may vary depending on the version or the platform. It should only be used for in-memory maps.

Due to MuseAir is non-crypto, this crate should NOT be used for cryptographic purpose.

#### Usage

```rust
use musemap::{HashMap, HashMapExt};

let mut map = HashMap::new();
map.insert("hello", "world");
```


[MuseAir]: https://github.com/eternal-io/museair
