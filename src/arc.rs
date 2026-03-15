use allocator_api2::alloc::{Allocator, Global};
use allocator_api2::boxed::Box;
use core::marker::PhantomData;
use core::mem::ManuallyDrop;
use core::ptr;
use core::ptr::NonNull;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use crate::myunique::*;

#[derive(Clone)]
struct SecureArc<T: ?Sized, A: Allocator = Global> {
    ptr: NonNull<SecureArcInner<T>>,
    phantom: PhantomData<SecureArcInner<T>>,
    alloc: A,
}

impl<T> SecureArc<T> {
    pub fn new(data: T) -> SecureArc<T> {
        let x: Box<_> = Box::new(SecureArcInner {
            strong: AtomicUsize::new(1),
            weak: AtomicUsize::new(1),
            data,
        });
        unsafe { Self::from_inner(Box::leak(x).into()) }
    }
}

/* 
#[repr(transparent)]
pub struct MiniUnique<T> {
    pointer: NonNull<T>,
    // NOTE: this marker has no consequences for variance, but is necessary
    // for dropck to understand that we logically own a `T`.
    //
    // For details, see:
    // https://github.com/rust-lang/rfcs/blob/master/text/0769-sound-generic-drop.md#phantom-data
    _marker: PhantomData<T>,
}*/

#[inline]
pub fn into_unique<T, A: Allocator>(b: Box<T, A>) -> (MyUnique<T>, A) {
    let (ptr, alloc) = Box::into_raw_with_allocator(b);
    unsafe { (MyUnique::from(&mut *ptr), alloc) }
}

impl<T, A: Allocator> SecureArc<T, A> {
    #[inline]
    pub fn new_in(data: T, alloc: A) -> SecureArc<T, A> {
        let x = Box::new_in(
            SecureArcInner {
                strong: AtomicUsize::new(1),
                weak: AtomicUsize::new(1),
                data,
            },
            alloc,
        );
        let (ptr, alloc) = into_unique(x);
        unsafe { Self::from_inner_in(ptr.into(), alloc) }
    }

    #[inline]
    #[must_use]
    pub fn new_zeroed() {}
}

impl<T: ?Sized> SecureArc<T> {
    unsafe fn from_inner(ptr: NonNull<SecureArcInner<T>>) -> Self {
        unsafe { Self::from_inner_in(ptr, Global) }
    }

    unsafe fn from_ptr(ptr: *mut SecureArcInner<T>) -> Self {
        unsafe { Self::from_ptr_in(ptr, Global) }
    }
}

impl<T: ?Sized, A: Allocator> SecureArc<T, A> {
    #[inline]
    fn into_inner_with_allocator(this: Self) -> (NonNull<SecureArcInner<T>>, A) {
        let this = ManuallyDrop::new(this);
        (this.ptr, unsafe { ptr::read(&this.alloc) })
    }

    #[inline]
    unsafe fn from_inner_in(ptr: NonNull<SecureArcInner<T>>, alloc: A) -> Self {
        Self {
            ptr,
            phantom: PhantomData,
            alloc,
        }
    }

    #[inline]
    unsafe fn from_ptr_in(ptr: *mut SecureArcInner<T>, alloc: A) -> Self {
        unsafe { Self::from_inner_in(NonNull::new_unchecked(ptr), alloc) }
    }
}

pub struct Weak<T: ?Sized, A: Allocator = Global> {
    // This is a `NonNull` to allow optimizing the size of this type in enums,
    // but it is not necessarily a valid pointer.
    // `Weak::new` sets this to `usize::MAX` so that it doesn’t need
    // to allocate space on the heap. That's not a value a real pointer
    // will ever have because ArcInner has alignment at least 2.
    ptr: NonNull<SecureArcInner<T>>,
    alloc: A,
}

#[repr(C, align(2))]
struct SecureArcInner<T: ?Sized> {
    strong: AtomicUsize,
    weak: AtomicUsize,
    data: T,
}
