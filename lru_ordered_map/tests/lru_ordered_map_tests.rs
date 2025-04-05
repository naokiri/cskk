use lru_ordered_map::LruOrderedMap;
use std::rc::Rc;

#[test]
fn test_basic_operations() {
    let mut map: LruOrderedMap<String, i32> = LruOrderedMap::new();

    // Test push operation
    map.push("a".to_string(), 1);
    map.push("b".to_string(), 2);
    map.push("c".to_string(), 3);

    // Test peek operation
    assert_eq!(map.peek(&"a".to_string()), Some(&1));
    assert_eq!(map.peek(&"b".to_string()), Some(&2));
    assert_eq!(map.peek(&"c".to_string()), Some(&3));
    assert_eq!(map.peek(&"d".to_string()), None);

    // Test get operation (should update LRU order)
    assert_eq!(map.get(&"b".to_string()), Some(&2));

    // Check LRU order: b should be first (most recently used)
    let items: Vec<_> = map
        .iter_lru()
        .filter_map(|(key, value)| match (key, value) {
            (Some(k), Some(v)) => Some((k.to_string(), *v)),
            _ => None,
        })
        .collect();

    assert_eq!(items.len(), 3);
    assert_eq!(items[0].0, "b");
    assert_eq!(items[1].0, "c");
    assert_eq!(items[2].0, "a");

    // Test sorted iteration
    let sorted_items: Vec<_> = map
        .iter_sorted()
        .filter_map(|(key, value)| match (key, value) {
            (Some(k), Some(v)) => Some((k.to_string(), *v)),
            _ => None,
        })
        .collect();

    assert_eq!(sorted_items.len(), 3);
    assert_eq!(sorted_items[0].0, "a");
    assert_eq!(sorted_items[1].0, "b");
    assert_eq!(sorted_items[2].0, "c");

    // Test remove operation
    let removed = map.remove("b".to_string());
    assert_eq!(removed, Some(2));

    assert_eq!(map.peek(&"b".to_string()), None);

    // Check remaining items
    let remaining: Vec<_> = map
        .iter_sorted()
        .filter_map(|(key, value)| match (key, value) {
            (Some(k), Some(v)) => Some((k.to_string(), *v)),
            _ => None,
        })
        .collect();

    assert_eq!(remaining.len(), 2);
    assert_eq!(remaining[0].0, "a");
    assert_eq!(remaining[1].0, "c");
}

#[test]
fn test_updating_values() {
    let mut map: LruOrderedMap<String, String> = LruOrderedMap::new();

    // Add initial values
    map.push("key1".to_string(), "value1".to_string());
    map.push("key2".to_string(), "value2".to_string());

    // Update an existing key
    map.push("key1".to_string(), "updated1".to_string());

    // Check that the value was updated
    assert_eq!(map.peek(&"key1".to_string()), Some(&"updated1".to_string()));

    // Check LRU order
    let items: Vec<_> = map
        .iter_lru()
        .filter_map(|(key, value)| match (key, value) {
            (Some(k), Some(v)) => Some((k.to_string(), v.to_string())),
            _ => None,
        })
        .collect();

    // key1 should be most recently used due to the update
    assert_eq!(items[0].0, "key1");
    assert_eq!(items[0].1, "updated1");
}

#[test]
fn test_get_mut() {
    let mut map: LruOrderedMap<String, Vec<i32>> = LruOrderedMap::new();

    // Add initial value
    map.push("numbers".to_string(), vec![1, 2, 3]);

    // Modify value in-place using get_mut
    if let Some(vec) = map.get_mut(&"numbers".to_string()) {
        vec.push(4);
    }

    // Check that the value was modified
    assert_eq!(map.peek(&"numbers".to_string()), Some(&vec![1, 2, 3, 4]));
}

#[test]
fn test_large_map() {
    let mut map: LruOrderedMap<i32, String> = LruOrderedMap::new();

    // Add a large number of items
    for i in 0..1000 {
        map.push(i, format!("value{}", i));
    }

    // Access items in reverse order to change LRU order
    for i in (0..1000).rev() {
        assert_eq!(map.get(&i), Some(&format!("value{}", i)));
    }

    // Check that item 0 is now most recently used
    let first_item = map.iter_lru().next().unwrap();
    assert_eq!(first_item.0.map(|rc| **rc), Some(0));

    // Check that all items are still accessible in sorted order
    let sorted_count = map.iter_sorted().count();
    assert_eq!(sorted_count, 1000);
}

#[test]
fn test_memory_management() {
    // This test checks for proper cleanup of items
    // by creating and removing Rc references

    let mut map: LruOrderedMap<String, Rc<String>> = LruOrderedMap::new();

    // Create an Rc string and store its reference count
    let value1 = Rc::new("test1".to_string());
    let value2 = Rc::new("test2".to_string());
    let value3 = Rc::new("test3".to_string());

    let rc_count1 = Rc::strong_count(&value1);
    let rc_count2 = Rc::strong_count(&value2);
    let rc_count3 = Rc::strong_count(&value3);

    // Add to map
    map.push("key1".to_string(), Rc::clone(&value1));
    map.push("key2".to_string(), Rc::clone(&value2));
    map.push("key3".to_string(), Rc::clone(&value3));

    // Reference count should increase by 1
    assert_eq!(Rc::strong_count(&value1), rc_count1 + 1);
    assert_eq!(Rc::strong_count(&value2), rc_count2 + 1);
    assert_eq!(Rc::strong_count(&value3), rc_count3 + 1);

    // Remove one item and store the returned value
    let removed = map.remove("key2".to_string());

    // The removed value should exist
    assert!(removed.is_some());

    // We now have two references to value2: our original one and the one returned by remove
    // So the count should be rc_count2 + 1 (original + returned value)
    assert_eq!(Rc::strong_count(&value2), rc_count2 + 1);

    // Drop the returned reference
    drop(removed);

    // Now the count should be back to the original
    assert_eq!(Rc::strong_count(&value2), rc_count2);
}
