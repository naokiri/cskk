/// Invariant checking tests for LruOrderedMap
///
/// These tests verify that the internal data structure invariants are maintained
/// across all operations. LruOrderedMap maintains two synchronized data structures:
/// - keys: Vec<Rc<K>> (sorted keys)
/// - value_map: HashMap<Rc<K>, Box<LruEntry<K, V>>>
///
/// Critical invariants:
/// 1. keys.len() == value_map.len() (same number of entries)
/// 2. Every key in keys exists in value_map
/// 3. Every key in value_map exists in keys
/// 4. keys vector is always sorted
///
/// These tests help catch bugs like:
/// - remove() failing to update keys vector (Issue: inconsistent state)
/// - push() inserting at wrong position (Issue: broken sort order)
/// - Operations leaving orphaned entries in one structure but not the other
use lru_ordered_map::LruOrderedMap;

#[test]
fn test_remove_maintains_invariants() {
    // BAD OPERATION: remove() with binary_search failure
    //
    // This test catches the bug where remove() successfully removes from value_map
    // but fails to remove from keys vector (due to using if let Ok instead of expect).
    //
    // Without the fix:
    // - Entry removed from value_map
    // - binary_search mysteriously fails
    // - keys vector still has the key (orphaned reference)
    // - Invariant: keys.len() > value_map.len()
    // - iter_sorted() will panic when trying to access the removed entry

    let mut map = LruOrderedMap::new();

    // Add many items to increase chance of edge cases
    for i in 0..50 {
        map.push(i, format!("val_{}", i));
    }

    // Remove every other item
    for i in (0..50).step_by(2) {
        map.remove(i);

        // Verify the removed item is actually gone
        assert_eq!(map.peek(&i), None, "Item {} should be removed", i);
    }

    // Verify we can iterate without panics
    let count = map.iter_sorted().count();
    assert_eq!(count, 25, "Should have 25 items after removing 25");

    // Verify every item in iterator is actually accessible
    for (key_opt, _) in map.iter_sorted() {
        if let Some(key) = key_opt {
            assert!(
                map.peek(&**key).is_some(),
                "Iterator returned key {:?} but it's not accessible via peek",
                key
            );
        }
    }
}

#[test]
fn test_push_maintains_sorted_order() {
    // BAD OPERATIONS this catches:
    // 1. push() inserting at wrong position in keys vector (not maintaining sort order)
    // 2. push() failing to add to keys vector while adding to value_map
    // 3. push() adding duplicate keys to keys vector

    let mut map = LruOrderedMap::new();

    // Insert in random order to stress-test sorted insertion
    let keys = vec!["m", "a", "z", "b", "y", "c", "x", "d"];

    for key in keys {
        map.push(key, key);
    }

    // Collect keys via sorted iterator
    let sorted_keys: Vec<String> = map
        .iter_sorted()
        .filter_map(|(k, _)| k.map(|rc| (**rc).to_string()))
        .collect();

    // Verify they're actually sorted
    let expected = vec!["a", "b", "c", "d", "m", "x", "y", "z"];
    assert_eq!(sorted_keys, expected, "Keys should be sorted");
}

#[test]
fn test_push_update_no_duplicates() {
    // BAD OPERATIONS this catches:
    // 1. push() on existing key creating duplicate entries in keys vector
    // 2. push() on existing key removing old entry from keys

    let mut map: LruOrderedMap<String, String> = LruOrderedMap::new();

    map.push("key".to_string(), "value1".to_string());
    let count1 = map.iter_sorted().count();
    assert_eq!(count1, 1);

    // Update same key multiple times
    for i in 2..20 {
        map.push("key".to_string(), format!("value{}", i));

        // Should still have exactly 1 entry
        let count = map.iter_sorted().count();
        assert_eq!(
            count, 1,
            "After {} updates, should still have 1 entry, got {}",
            i, count
        );
    }

    // Verify final value is the latest
    assert_eq!(map.peek(&"key".to_string()), Some(&"value19".to_string()));
}

#[test]
fn test_mixed_operations_consistency() {
    // BAD CASES this catches:
    // 1. Cumulative corruption from multiple operations
    // 2. Edge cases when alternating between push/remove
    // 3. State corruption after get/get_mut operations

    let mut map = LruOrderedMap::new();

    // Complex sequence of operations
    map.push(5, "five");
    assert_eq!(map.iter_sorted().count(), 1);

    map.push(3, "three");
    assert_eq!(map.iter_sorted().count(), 2);

    map.push(7, "seven");
    assert_eq!(map.iter_sorted().count(), 3);

    map.get(&5); // LRU update shouldn't affect sorted iterator count
    assert_eq!(map.iter_sorted().count(), 3);

    map.push(3, "three_updated"); // Update existing
    assert_eq!(map.iter_sorted().count(), 3);

    map.remove(5);
    assert_eq!(map.iter_sorted().count(), 2);

    map.push(1, "one");
    assert_eq!(map.iter_sorted().count(), 3);

    map.push(9, "nine");
    assert_eq!(map.iter_sorted().count(), 4);

    map.remove(3);
    assert_eq!(map.iter_sorted().count(), 3);

    map.remove(1);
    assert_eq!(map.iter_sorted().count(), 2);

    // Final state should be just [7, 9]
    let final_keys: Vec<i32> = map
        .iter_sorted()
        .filter_map(|(k, _)| k.map(|rc| **rc))
        .collect();
    assert_eq!(final_keys, vec![7, 9]);
}

#[test]
fn test_iterator_count_consistency() {
    // BAD CASES this catches:
    // 1. Iterator seeing orphaned keys (in keys vector but not in value_map)
    // 2. Iterator count mismatch due to data structure inconsistency
    // 3. Panics during iteration due to missing entries

    let mut map = LruOrderedMap::new();

    for i in 0..100 {
        map.push(format!("key_{:03}", i), i);
    }

    // Remove many items
    for i in (0..100).step_by(3) {
        map.remove(format!("key_{:03}", i));
    }

    // Count items via iterator
    let iter_count = map.iter_sorted().count();

    // Expected count: 100 - 34 = 66
    // (Removed: 0, 3, 6, 9, ..., 99 = 34 items)
    assert_eq!(iter_count, 66, "Should have 66 items remaining");

    // Every item from iterator should be accessible via peek
    let mut peek_count = 0;
    for (key_opt, _) in map.iter_sorted() {
        if let Some(key) = key_opt {
            assert!(
                map.peek(&**key).is_some(),
                "Iterator returned key {:?} but peek returns None (orphaned key!)",
                key
            );
            peek_count += 1;
        }
    }

    // Peek count should match iterator count
    assert_eq!(
        peek_count, iter_count,
        "Peek count should match iterator count"
    );
}

#[test]
fn test_remove_all_items() {
    // BAD CASES this catches:
    // 1. Edge case when map becomes empty
    // 2. Removing the last item leaves orphaned entries
    // 3. Iterator behavior on empty map after removals

    let mut map = LruOrderedMap::new();

    // Add items
    for i in 0..10 {
        map.push(i, i * 10);
    }

    // Remove all items
    for i in 0..10 {
        let removed = map.remove(i);
        assert!(removed.is_some(), "Item {} should exist", i);
        assert_eq!(removed.unwrap(), i * 10);
    }

    // Verify map is empty
    assert_eq!(map.iter_sorted().count(), 0, "Map should be empty");

    // Verify we can still use the map
    map.push(100, 1000);
    assert_eq!(map.iter_sorted().count(), 1);
    assert_eq!(map.peek(&100), Some(&1000));
}

#[test]
fn test_remove_nonexistent_items() {
    // BAD CASES this catches:
    // 1. remove() on non-existent key corrupting state
    // 2. Multiple removes of same key causing issues

    let mut map = LruOrderedMap::new();

    map.push("a", 1);
    map.push("b", 2);
    map.push("c", 3);

    // Try to remove non-existent items
    assert_eq!(map.remove("x"), None);
    assert_eq!(map.remove("y"), None);
    assert_eq!(map.remove("z"), None);

    // Original items should still be there
    assert_eq!(map.iter_sorted().count(), 3);

    // Remove an existing item
    assert_eq!(map.remove("b"), Some(2));
    assert_eq!(map.iter_sorted().count(), 2);

    // Try to remove it again
    assert_eq!(map.remove("b"), None);
    assert_eq!(map.iter_sorted().count(), 2);
}
