use std::{
    cell::{Cell, UnsafeCell},
    rc::Rc,
    ops::{Deref, DerefMut},
};

pub struct RcCell<T> {
    rc: Rc<Inner<T>>,
}

struct Inner<T> {
    value: UnsafeCell<T>,
    borrow: Cell<Borrow>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Borrow {
    Ref { count: isize },
    RefMut,
}

impl<T> Clone for RcCell<T> {
    fn clone(&self) -> Self {
        RcCell { rc: Rc::clone(&self.rc) }
    }
}

impl<T> RcCell<T> {
    pub fn new(value: T) -> Self {
        RcCell { rc: Rc::new(Inner {
            value: UnsafeCell::new(value),
            borrow: Cell::new(Borrow::Ref { count: 0 }),
        })}
    }
    pub fn borrow(&self) -> Ref<T, T> { self.borrow_as(|t| t) }
    pub fn borrow_as<U, F: FnOnce(&T) -> &U>(&self, f: F) -> Ref<T, U> {
        Ref::new(&self, f).unwrap()
    }
    pub fn borrow_mut(&self) -> RefMut<T, T> { self.borrow_mut_as(|t| t) }
    pub fn borrow_mut_as<U, F: FnOnce(&mut T) -> &mut U>(&self, f: F) -> RefMut<T, U> {
        RefMut::new(&self, f).unwrap()
    }
}

pub struct Ref<T, U> {
    rc: Rc<Inner<T>>,
    ptr: *const U,
}

impl<T, U> Ref<T, U> {
    fn new<F: FnOnce(&T) -> &U>(cell: &RcCell<T>, f: F) -> Option<Ref<T, U>> {
        let b = cell.rc.borrow.get();
        match b {
            Borrow::Ref { count } => {
                cell.rc.borrow.set(Borrow::Ref { count: count+1 });
                Some(Ref {
                    rc: Rc::clone(&cell.rc),
                    ptr: f(unsafe { &*cell.rc.value.get() }),
                })
            }
            _ => None,
        }
    }
}

impl<T, U> Drop for Ref<T, U> {
    fn drop(&mut self) {
        let b = self.rc.borrow.get();
        match b {
            Borrow::Ref { count } if count > 0 => {
                self.rc.borrow.set(Borrow::Ref { count: count-1 });
            }
            _ => panic!("Invalid borrow on ref drop: {:?}", b),
        }
    }
}

impl<T, U> Deref for Ref<T, U> {
    type Target = U;
    fn deref(&self) -> &U { unsafe { &*self.ptr } }
}

pub struct RefMut<T, U> {
    rc: Rc<Inner<T>>,
    ptr: *mut U,
}

impl<T, U> RefMut<T, U> {
    fn new<F: FnOnce(&mut T) -> &mut U>(cell: &RcCell<T>, f: F) -> Option<RefMut<T, U>> {
        let b = cell.rc.borrow.get();
        match b {
            Borrow::Ref { count: 0 } => {
                cell.rc.borrow.set(Borrow::RefMut);
                Some(RefMut {
                    rc: Rc::clone(&cell.rc),
                    ptr: f(unsafe { &mut *cell.rc.value.get() }),
                })
            }
            _ => None,
        }
    }
}

impl<T, U> Drop for RefMut<T, U> {
    fn drop(&mut self) {
        let b = self.rc.borrow.get();
        match b {
            Borrow::RefMut => {
                self.rc.borrow.set(Borrow::Ref { count: 0 });
            }
            _ => panic!("Invalid borrow on ref drop: {:?}", b),
        }
    }
}

impl<T, U> Deref for RefMut<T, U> {
    type Target = U;
    fn deref(&self) -> &U { unsafe { &*self.ptr } }
}

impl<T, U> DerefMut for RefMut<T, U> {
    fn deref_mut(&mut self) -> &mut U { unsafe { &mut *self.ptr } }
}