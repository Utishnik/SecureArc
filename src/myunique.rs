use core::marker::PhantomData;
use core::ops::Deref;
use core::ptr::NonNull;

pub struct MyUnique<T: ?Sized> {
    ptr: NonNull<T>,
    _marker: PhantomData<T>, 
}

impl<T> MyUnique<T> {
    pub fn new(data: T) -> Self {
        unsafe {
            let box_ptr: NonNull<T> = NonNull::new( Box::into_raw(Box::new(data))).unwrap_unchecked();
            Self { 
                ptr: box_ptr, 
                _marker: PhantomData 
            }
        }
    }
    pub unsafe fn from(ptr: &mut T) -> Self{
        unsafe {
            let nn: NonNull<T> = NonNull::new_unchecked(ptr);
            Self { ptr: nn, _marker: PhantomData}
        }
    }

    pub unsafe fn as_ref(&self) -> &T {
        unsafe {
            &*self.ptr.as_ptr()
        }
    }
}

impl<T> From<MyUnique<T>> for NonNull<T>{
    fn from(value: MyUnique<T>) -> Self {
            value.ptr
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
            let _ = Box::from_raw(self.ptr.as_ptr()); 
        }
    }
}