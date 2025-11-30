use std::sync::atomic::{AtomicUsize, Ordering};

/*
#[non_exhaustive]
pub enum Ordering {
    Relaxed,
    Release,
    Acquire,
    AcqRel,
    SeqCst,
}
 */

/// Relaxed: No ordering constraints, only operations is atomic.
/// Relaxed is too weak.
#[test]
fn too_relaxed() {
    use std::thread::spawn;
    let x: &'static _ = Box::leak(Box::new(AtomicUsize::new(0)));
    let y: &'static _ = Box::leak(Box::new(AtomicUsize::new(0)));

    let t1 = spawn(move || {
        let r1 = y.load(Ordering::Relaxed);
        x.store(r1, Ordering::Relaxed);
        r1
    });

    let t2 = spawn(move || {
        let r2 = x.load(Ordering::Relaxed);
        y.store(42, Ordering::Relaxed);
        r2
    });

    let r1 = t1.join().unwrap();
    let r2 = t2.join().unwrap();
    // there is a possibility that r1 = r2 = 42;

    // Modification order for x = 0, 42
    // Modification order for y = 0, 42

    // For optimization purposes cpu/compiler can execute out of order,
    // if there is no dependency, so that happens in t2 we can see
    // r1 = r2 = 42

    assert_eq!(r1, 42);
    //assert_eq!(r2, 42);
}

// Release
// When coupled with a store, all previous operations become ordered before
// any load of this value with Acquire (or stronger) ordering. In particular,
// all previous writes become visible to all threads that perform an Acquire
// (or stronger)load of this value.
// No reads or writes in the current thread can be reordered after this store.
// All writes in the current thread are visible in other threads that acquire
// the same atomic variable.

// Acquire
// When coupled with a load, if the loaded value was written by a store operation
// with Release (or stronger) ordering, then all subsequent operations become
// ordered after that store. In particular, all subsequent loads will see data
// written before the store.
//
// Notice that using this ordering for an operation that combines loads and stores
// leads to a Relaxed store operation!.
// This ordering is only applicable for operations that can perform a load.
//
// no reads or writes in the current thread can be reordered after this store

// So basically Acquire and Release are used as locking
// You load with Acquire and store with Release
// All store is visible to load if these two ordering used.
// No reads and write is reordered in the current between store and load.

// AcqRel - Acquire Release
// Has the effects of both Acquire and Release together:
// For loads it uses Acquire ordering. For stores it uses the Release ordering.
// This ordering is only applicable for operations that combine both loads and stores.
// Used when you are doing one operation like fetch_add and want to synchronize with
// other threads.

// SeqCst
// Like Acquire/Release/AcqRel (for load, store, and load-with-store operations, respectively)
// with the additional guarantee that all threads see all sequentially consistent operations
// in the same order.
// A load operation with this memory order performs an acquire operation, a store performs
// a release operation, and read-modify-write performs both an acquire operation and a
// release operation, plus a single total order exists in which all threads observe all
// modifications in the same order

// Acquire/Release provide local ordering only whereas SeqCst provide global ordering
