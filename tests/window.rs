//! The ring buffer under every windowed indicator: fill → evict FIFO →
//! chronological iteration across the wrap point.

use nova_ta_core::Window;

#[test]
fn fills_then_evicts_oldest_first() {
    let mut w = Window::new(3);
    assert_eq!(w.push(1), None);
    assert_eq!(w.push(2), None);
    assert_eq!(w.push(3), None);
    assert!(w.is_full());
    // FIFO from here on.
    assert_eq!(w.push(4), Some(1));
    assert_eq!(w.push(5), Some(2));
    assert_eq!(w.push(6), Some(3));
    assert_eq!(w.push(7), Some(4));
}

#[test]
fn iterates_chronologically_across_the_wrap() {
    let mut w = Window::new(3);
    for v in [1, 2, 3, 4, 5] {
        w.push(v);
    }
    // Storage has wrapped; iteration must still be oldest → newest.
    assert_eq!(w.iter().copied().collect::<Vec<_>>(), vec![3, 4, 5]);
    assert_eq!(w.oldest(), Some(&3));
}

#[test]
fn partial_window_iterates_in_insertion_order() {
    let mut w = Window::new(4);
    w.push(10);
    w.push(20);
    assert_eq!(w.len(), 2);
    assert!(!w.is_full());
    assert_eq!(w.iter().copied().collect::<Vec<_>>(), vec![10, 20]);
    assert_eq!(w.oldest(), Some(&10));
}

#[test]
fn capacity_one_always_replaces() {
    let mut w = Window::new(1);
    assert_eq!(w.push('a'), None);
    assert_eq!(w.push('b'), Some('a'));
    assert_eq!(w.push('c'), Some('b'));
    assert_eq!(w.iter().copied().collect::<Vec<_>>(), vec!['c']);
}

#[test]
fn clear_restarts_from_empty() {
    let mut w = Window::new(2);
    w.push(1);
    w.push(2);
    w.push(3);
    w.clear();
    assert!(w.is_empty());
    assert_eq!(w.oldest(), None::<&i32>);
    // Refilling behaves like a fresh window.
    assert_eq!(w.push(9), None);
    assert_eq!(w.push(8), None);
    assert_eq!(w.push(7), Some(9));
}

#[test]
#[should_panic(expected = "capacity must be >= 1")]
fn zero_capacity_is_a_construction_bug() {
    let _ = Window::<i32>::new(0);
}

#[test]
fn iterators_preserve_order_after_wraparound() {
    let mut window = Window::new(3);

    window.push(10);
    window.push(20);
    window.push(30);

    assert_eq!(
        window.iter().copied().collect::<Vec<_>>(),
        vec![10, 20, 30]
    );

    assert_eq!(
        window.rev_iter().copied().collect::<Vec<_>>(),
        vec![30, 20, 10]
    );

    assert_eq!(window.push(40), Some(10));

    assert_eq!(
        window.iter().copied().collect::<Vec<_>>(),
        vec![20, 30, 40]
    );

    assert_eq!(
        window.rev_iter().copied().collect::<Vec<_>>(),
        vec![40, 30, 20]
    );
}