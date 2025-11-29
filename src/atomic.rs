/*

Atomic types provide primitive shared-memory communication between threads,
and are the building blocks of other concurrent types.

Atomic variables are safe to share between threads (they implement Sync)
but they do not themselves provide the mechanism for sharing and follow
the threading model of Rust. The most common way to share an atomic variable
is to put it into an Arc (an atomically-reference-counted shared pointer).

Atomic types may be stored in static variables, initialized using the constant
initializers like AtomicBool::new. Atomic statics are often used for lazy global initialization.

 */
use std::cell::UnsafeCell;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::spawn;

const LOCKED: bool = true;
const UNLOCKED: bool = false;

pub struct Mutex<T> {
    value: UnsafeCell<T>,
    locked: AtomicBool,
}

unsafe impl<T: Send> Sync for Mutex<T> {}

impl<T> Mutex<T> {
    pub fn new(value: T) -> Self {
        Self {
            value: UnsafeCell::new(value),
            locked: AtomicBool::new(UNLOCKED),
        }
    }

    pub fn with_lock<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        // Approach 1: Lock and store
        // while self.locked.load(Ordering::Relaxed) != UNLOCKED {};
        // // maybe other thread runs here.
        // self.locked.store(LOCKED, Ordering::Relaxed);

        // Approach 2: compare and exchange
        while self
            .locked
            .compare_exchange(UNLOCKED, LOCKED, Ordering::Relaxed, Ordering::Relaxed)
            .is_err()
        {
            // MESI Protocol
            // All cores are trying to get exclusive access which is expensive hence just read
            while self.locked.load(Ordering::Relaxed) == LOCKED {}

            // compare_exchange and compare_exchange_weak
            // compare_exchange will fail only if value is not same as current
            // compare_exchange_weak is allowed to spuriously fail even when the comparison succeeds

            // x86: CAS

            // Arm: LDREX and STREX
            // compare_exchange: implemented using loop of LDREX and STREX
            // compare_exchange_weak: tried one time
            // if you are using loop with compare_exchange it becomes a nested loop on arm
            // Nested loop does not perform well so prefer compare_exchange_weak
        }

        // Safety: We hold the lock, so we can create a mutable reference
        let ret = f(unsafe { &mut *self.value.get() });
        self.locked.store(UNLOCKED, Ordering::Relaxed);
        ret
    }
}

#[test]
fn test_mutex() {
    use std::thread::{JoinHandle, spawn};
    let m: &'static _ = Box::leak(Box::new(Mutex::new(0)));

    let handles = (0..100)
        .map(|_| {
            spawn(move || {
                for _ in 0..10000 {
                    m.with_lock(|l| *l += 1);
                }
            })
        })
        .collect::<Vec<JoinHandle<_>>>();

    for handle in handles {
        handle.join().unwrap()
    }

    assert_eq!(unsafe { *m.value.get() }, 100 * 10000);
}
