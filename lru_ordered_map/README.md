# LruOrderedMap

A Rust data structure that maintains elements in both LRU (Least Recently Used) order and sorted order.

This crate was extracted from the [cskk](https://github.com/naokiri/cskk) project.

## Features

- Maintains elements in both LRU order and sorted order
- Uses a double-linked list for LRU ordering and a sorted vector for ordered access
- Provides iterators for both LRU and sorted order traversal
- Supports operations like push, get, peek, and remove with LRU tracking

## Usage

```rust
use lru_ordered_map::LruOrderedMap;

let mut map = LruOrderedMap::new();
map.push("key1", "value1");
map.push("key2", "value2");

// Get updates LRU order
if let Some(value) = map.get(&"key1") {
    println!("Found value: {}", value);
}

// Iterate in LRU order (most recently used first)
for (key, value) in map.iter_lru() {
    if let (Some(k), Some(v)) = (key, value) {
        println!("Key: {}, Value: {}", k, v);
    }
}

// Iterate in sorted order (by key)
for (key, value) in map.iter_sorted() {
    if let (Some(k), Some(v)) = (key, value) {
        println!("Key: {}, Value: {}", k, v);
    }
}
```

## Test
For Rust unit tests, just run cargo test.

For memory leak test, Run ./run_valgrind_test.sh
The script will run valgrind on the test binary and check for memory leaks. Note that the "possibly lost" are likely false positives if the output contains the code about `std::thread::Thread::new` and only occurs occasionally based on timing of the valgrind.

## License

GPL-3.0-or-later