# MuseMap

Fast DoS-resistant hashmap based on [MuseAir] hash algorithm.

Note that due to MuseAir is non-crypto, you should not use this crate for cryptographic purpose.

#### Quick usage

```
use musemap::{HashMap, HashMapExt};

let mut map = HashMap::new();
map.insert(1, 2);
map.insert(3, 4);

assert_eq!(map.get(&1), Some(&2));
assert_eq!(map.get(&3), Some(&4));
```


[MuseAir]: https://github.com/eternal-io/museair
