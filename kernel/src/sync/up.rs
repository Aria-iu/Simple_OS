use core::cell::{RefCell,RefMut};

pub struct UPSafeCell<T>{
    inner: RefCell<T>,
}
// shared between threads
unsafe impl<T> Sync for UPSafeCell<T> {}

impl<T> UPSafeCell<T>{
    pub unsafe fn new(val:T) -> Self{
        Self{
            inner:RefCell::new(val),
        }
    }

    pub fn exclusive_access(&self)->RefMut<'_,T>{
        self.inner.borrow_mut()
    }
}