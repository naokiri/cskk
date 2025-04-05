#[cfg(test)]
mod memory_leak_tests {
    use lru_ordered_map::LruOrderedMap;
    use std::cell::RefCell;
    use std::rc::Rc;

    /// This test creates and destroys a large number of LruOrderedMap instances
    /// to check for memory leaks. Run with valgrind for leak detection:
    /// ```
    /// RUSTFLAGS="-C debug-assertions=yes" cargo test -p lru_ordered_map --test memory_leak_test -- --nocapture
    /// valgrind --leak-check=full --show-leak-kinds=all ./target/debug/deps/memory_leak_test-*
    /// ```
    #[test]
    fn create_and_destroy_maps() {
        for _ in 0..100 {
            let mut map: LruOrderedMap<String, String> = LruOrderedMap::new();

            // Add some items
            for i in 0..100 {
                let key = format!("key{}", i);
                let value = format!("value{}", i);
                map.push(key, value);
            }

            // Get some items to test LRU functionality
            for i in 0..50 {
                let key = format!("key{}", i);
                let _ = map.get(&key);
            }

            // Remove some items
            for i in 0..30 {
                let key = format!("key{}", i);
                let _ = map.remove(key);
            }

            // Map is dropped here, should clean up all resources
        }
    }

    /// This test creates non-cyclic references to test proper cleanup
    #[test]
    fn test_with_references() {
        // Create a type for testing
        #[allow(dead_code)]
        #[derive(Debug)]
        struct Node {
            id: usize,
            next: Option<Rc<Node>>,
        }

        for i in 0..10 {
            let mut map: LruOrderedMap<i32, Rc<Node>> = LruOrderedMap::new();

            // Create a chain of nodes without cycles
            let node3 = Rc::new(Node {
                id: i * 3 + 2,
                next: None,
            });

            let node2 = Rc::new(Node {
                id: i * 3 + 1,
                next: Some(Rc::clone(&node3)),
            });

            let node1 = Rc::new(Node {
                id: i * 3,
                next: Some(Rc::clone(&node2)),
            });

            // Add to map
            map.push(1, Rc::clone(&node1));
            map.push(2, Rc::clone(&node2));
            map.push(3, Rc::clone(&node3));

            // Remove from map
            map.remove(1);
            map.remove(2);
            map.remove(3);

            // Map and nodes are dropped here
        }
    }

    /// This test repeatedly adds and removes items to check for incremental leaks
    #[test]
    fn repeated_operations() {
        let mut map: LruOrderedMap<String, Vec<u8>> = LruOrderedMap::new();

        for i in 0..1000 {
            let key = format!("key{}", i);
            let value = vec![0u8; 1024]; // 1KB data

            map.push(key.clone(), value);

            if i > 9 {
                let old_key = format!("key{}", i - 10);
                map.remove(old_key);
            }
        }

        assert_eq!(map.iter_lru().count(), 10);
    }

    /// This test stresses the replace functionality to ensure no memory leaks
    #[test]
    fn test_replace_operations() {
        let mut map: LruOrderedMap<String, Vec<u8>> = LruOrderedMap::new();

        // First add some items
        for i in 0..100 {
            let key = format!("key{}", i);
            let value = vec![0u8; 1024]; // 1KB data
            map.push(key, value);
        }

        // Now replace all items with new values multiple times
        for iteration in 0..10 {
            for i in 0..100 {
                let key = format!("key{}", i);
                let value = vec![iteration as u8; 1024]; // Different data each time
                map.push(key, value);
            }
        }

        // Map should still have 100 items
        assert_eq!(map.iter_lru().count(), 100);
    }

    /// Test that tracks object drops to ensure that map is properly cleaning up
    #[test]
    fn test_drop_impl() {
        // Create a counter to track drop calls
        struct DropCounter {
            counter: Rc<RefCell<usize>>,
            id: usize,
        }

        impl Drop for DropCounter {
            fn drop(&mut self) {
                *self.counter.borrow_mut() += 1;
                println!("Dropping item {}", self.id);
            }
        }

        // Create a shared counter
        let counter = Rc::new(RefCell::new(0));

        {
            let mut map: LruOrderedMap<usize, DropCounter> = LruOrderedMap::new();

            // Add items
            for i in 0..100 {
                map.push(
                    i,
                    DropCounter {
                        counter: Rc::clone(&counter),
                        id: i,
                    },
                );
            }

            // Remove some items - these should be dropped immediately
            for i in 0..50 {
                map.remove(i);
            }

            // We should have dropped exactly 50 items
            assert_eq!(
                *counter.borrow(),
                50,
                "Expected 50 drops after removing 50 items"
            );

            // Map goes out of scope here and should drop remaining 50 items
        }

        // All 100 items should have been dropped
        assert_eq!(
            *counter.borrow(),
            100,
            "Expected 100 drops after map is dropped"
        );
    }
}
