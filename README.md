# Pre-computed hashing

[![No Maintenance Intended](http://unmaintained.tech/badge.svg)](http://unmaintained.tech/)

A proof-of-concept Rust library for:

* Values that intrusively store their own hash
* A hash map that uses these pre-computed hashes

This aims to improve performance of hashmap-heavy applications
where key construction is much less frequent than map operations.

Criterion (`cargo bench`) output on commit `af06d8dd46bc`,
calling `get` on a map containing 100 entries with a key that is present.

```
hashmap get/prehash     time:   [3.2359 ns 3.2447 ns 3.2544 ns]
hashmap get/hashbrown   time:   [6.0660 ns 6.0845 ns 6.1064 ns]
```
