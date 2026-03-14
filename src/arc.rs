use std::sync::atomic::*;
use std::sync::Arc;
use allocator_api2::alloc::{Allocator,Global};
use core::ptr::NonNull;
use core::marker::PhantomData;
use std::sync::atomic::Ordering::*;
use core::ptr;
use core::mem;

fn t(){
    let w = std::sync::Weak;;
}

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

pub struct SecureWeak<
    T: ?Sized,
    A: Allocator = Global,
> {
    // This is a `NonNull` to allow optimizing the size of this type in enums,
    // but it is not necessarily a valid pointer.
    // `Weak::new` sets this to `usize::MAX` so that it doesn’t need
    // to allocate space on the heap. That's not a value a real pointer
    // will ever have because ArcInner has alignment at least 2.
    ptr: NonNull<SecureArcInner<T>>,
    alloc: A,
}

impl<T: ?Sized, A: Allocator> SecureWeak<T, A> {
    #[inline]
    fn inner(&self) -> Option<SecureWeak<'_>> {
        let ptr = self.ptr.as_ptr();
        if is_dangling(ptr) {
            None
        } else {
            // We are careful to *not* create a reference covering the "data" field, as
            // the field may be mutated concurrently (for example, if the last `Arc`
            // is dropped, the data field will be dropped in-place).
            Some(unsafe { SecureWeak { strong: &(*ptr).strong, weak: &(*ptr).weak } })
        }
    }

}

unsafe impl<T: ?Sized + Sync + Send> Send for SecureArcInner<T> {}
unsafe impl<T: ?Sized + Sync + Send> Sync for SecureArcInner<T> {}

const MAX_REFCOUNT: usize = 9223372036854775807;
pub struct SecureArc<
    T: ?Sized,
    A: Allocator = Global,
> {
    ptr: NonNull<SecureArcInner<T>>,
    phantom: PhantomData<SecureArcInner<T>>,
    alloc: A,
}

impl<T: ?Sized, A: Allocator + Clone> Clone for SecureWeak<T, A> {
    /// Makes a clone of the `Weak` pointer that points to the same allocation.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::{Arc, Weak};
    ///
    /// let weak_five = Arc::downgrade(&Arc::new(5));
    ///
    /// let _ = Weak::clone(&weak_five);
    /// ```
    #[inline]
    fn clone(&self) -> SecureWeak<T, A> {
        if let Some(inner) = self.inner() {
            // See comments in Arc::clone() for why this is relaxed. This can use a
            // fetch_add (ignoring the lock) because the weak count is only locked
            // where are *no other* weak pointers in existence. (So we can't be
            // running this code in that case).
            let old_size = inner.weak.fetch_add(1, Relaxed);

            // See comments in Arc::clone() for why we do this (for mem::forget).
            if old_size > MAX_REFCOUNT {
                panic!();
            }
        }

        SecureWeak { ptr: self.ptr, alloc: self.alloc.clone() }
    }
}


impl<T: ?Sized, A: Allocator + Clone> Clone for SecureArc<T, A> {

    #[inline]
    fn clone(&self) -> SecureArc<T, A> {

        let old_size = self.inner().strong.fetch_add(1, Relaxed);


        if old_size > MAX_REFCOUNT {
            panic!("");
        }

        unsafe { Self::from_inner_in(self.ptr, self.alloc.clone()) }
    }
}

impl<T: ?Sized, A: Allocator> SecureArc<T, A> {
    #[inline]
    fn inner(&self) -> &SecureArcInner<T> {
        unsafe { self.ptr.as_ref() }
    }
}

impl<T: ?Sized, A: Allocator> SecureArc<T, A> {
    #[inline]
    fn into_inner_with_allocator(this: Self) -> (NonNull<SecureArcInner<T>>, A) {
        let this: mem::ManuallyDrop<SecureArc<T, A>> = mem::ManuallyDrop::new(this);
        (this.ptr, unsafe { ptr::read(&this.alloc) })
    }

    #[inline]
    unsafe fn from_inner_in(ptr: NonNull<SecureArcInner<T>>, alloc: A) -> Self {
        Self { ptr, phantom: PhantomData, alloc }
    }

    #[inline]
    unsafe fn from_ptr_in(ptr: *mut SecureArcInner<T>, alloc: A) -> Self {
        unsafe { Self::from_inner_in(NonNull::new_unchecked(ptr), alloc) }
    }
}
