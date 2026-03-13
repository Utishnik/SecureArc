use std::sync::atomic::*;
use allocator_api2::alloc::{Allocator,Global};
use core::ptr::NonNull;
use core::marker::PhantomData;

#[repr(C, align(2))]
struct SecureArcInner<T: ?Sized> {
    #[cfg(not(feature = "nightly"))]
    strong: AtomicUsize,
    #[cfg(feature = "nightly")]
    strong: Atomic<usize>,

    // the value usize::MAX acts as a sentinel for temporarily "locking" the
    // ability to upgrade weak pointers or downgrade strong ones; this is used
    // to avoid races in `make_mut` and `get_mut`.
    #[cfg(not(feature = "nightly"))]
    weak: AtomicUsize,
    #[cfg(feature = "nightly")]
    weak: Atomic<usize>,

    data: T,
}


unsafe impl<T: ?Sized + Sync + Send> Send for SecureArcInner<T> {}
unsafe impl<T: ?Sized + Sync + Send> Sync for SecureArcInner<T> {}

pub struct SecureArc<
    T: ?Sized,
    A: Allocator = Global,
> {
    ptr: NonNull<SecureArcInner<T>>,
    phantom: PhantomData<SecureArcInner<T>>,
    alloc: A,
}
