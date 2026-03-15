use core::marker::PhantomData;
use core::ops::Deref;
use core::ptr::NonNull;

pub struct MyUnique<T: ?Sized> {
    ptr: NonNull<T>,
    _marker: PhantomData<T>, 
}

impl<T> MyUnique<T> {
    pub fn new(data: T) -> Self {
        let box_ptr = Box::into_raw(Box::new(data));
        Self { 
            ptr: box_ptr, 
            _marker: PhantomData 
        }
    }
    pub unsafe fn from() {
        
    }

    pub unsafe fn as_ref(&self) -> &T {
        unsafe {
            &*self.ptr
        }
    }
}

impl<T> Deref for MyUnique<T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { self.as_ref() }
    }
}

impl<T:?Sized> Drop for MyUnique<T> {
    fn drop(&mut self) {
        unsafe {
            let _ = Box::from_raw(self.ptr); 
        }
    }
}