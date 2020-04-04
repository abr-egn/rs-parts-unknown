use std::{
    cell::{Cell, UnsafeCell},
    rc::Rc,
    ops::{Deref, DerefMut},
};

#[derive(Debug)]
pub struct RcCell<T> {
    rc: Rc<Inner<T>>,
}

#[derive(Debug)]
struct Inner<T> {
    value: UnsafeCell<T>,
    borrow: Cell<Borrow>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Borrow {
    Ref { count: usize },
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
    pub fn borrow(&self) -> Ref<T> {
        Ref {
            track: RefTrack::new(&self.rc).unwrap(),
            ptr: self.rc.value.get(),
        }
    }
    pub fn borrow_mut(&self) -> RefMut<T> {
        RefMut {
            track: RefMutTrack::new(&self).unwrap(),
            ptr: self.rc.value.get(),
        }
    }
    pub fn rc_count(&self) -> usize { Rc::strong_count(&self.rc) }
    pub fn borrow_count(&self) -> Option<usize> {
        match self.rc.borrow.get() {
            Borrow::Ref { count } => Some(count),
            Borrow::RefMut => None,
        }
    }
}

pub struct Ref<T, U=T> {
    track: RefTrack<T>,
    ptr: *const U,
}

impl<T, U> Ref<T, U> {
    pub fn clone(this: &Ref<T, U>) -> Ref<T, U> {
        Ref {
            track: RefTrack::new(&this.track.rc).unwrap(),
            ptr: this.ptr,
        }
    }
    pub fn map<V, F>(this: Ref<T, U>, f: F) -> Ref<T, V>
        where F: FnOnce(&U) -> &V,
    {
        Ref {
            track: this.track,
            ptr: f(unsafe { &*this.ptr }),
        }
    }
    pub fn map_opt<V, F>(this: Ref<T, U>, f: F) -> Option<Ref<T, V>>
        where F: FnOnce(&U) -> Option<&V>,
    {
        f(unsafe { &*this.ptr }).map(|u| Ref {
            track: this.track,
            ptr: u,
        })
    }
}

struct RefTrack<T> {
    rc: Rc<Inner<T>>
}

impl<T> RefTrack<T> {
    fn new(rc: &Rc<Inner<T>>) -> Option<RefTrack<T>> {
        let b = rc.borrow.get();
        match b {
            Borrow::Ref { count } => {
                rc.borrow.set(Borrow::Ref { count: count+1 });
                Some(RefTrack { rc: Rc::clone(rc) })
            }
            _ => None,
        }
    }
}

impl<T> Drop for RefTrack<T> {
    fn drop(&mut self) {
        let b = self.rc.borrow.get();
        match b {
            Borrow::Ref { count } if count > 0 => {
                self.rc.borrow.set(Borrow::Ref { count: count-1 });
            }
            _ => panic!("Invalid borrow on Ref drop: {:?}", b),
        }
    }
}

impl<T, U> Deref for Ref<T, U> {
    type Target = U;
    fn deref(&self) -> &U { unsafe { &*self.ptr } }
}

pub struct RefMut<T, U=T> {
    track: RefMutTrack<T>,
    ptr: *mut U,
}

impl<T, U> RefMut<T, U> {
    pub fn map<V, F>(this: RefMut<T, U>, f: F) -> RefMut<T, V>
        where F: FnOnce(&mut U) -> &mut V,
    {
        RefMut {
            track: this.track,
            ptr: f(unsafe { &mut *this.ptr }),
        }
    }
    pub fn map_opt<V, F>(this: RefMut<T, U>, f: F) -> Option<RefMut<T, V>>
        where F: FnOnce(&mut U) -> Option<&mut V>,
    {
        f(unsafe { &mut *this.ptr }).map(|u| RefMut {
            track: this.track,
            ptr: u,
        })
    }
}

impl<T, U> Deref for RefMut<T, U> {
    type Target = U;
    fn deref(&self) -> &U { unsafe { &*self.ptr } }
}

impl<T, U> DerefMut for RefMut<T, U> {
    fn deref_mut(&mut self) -> &mut U { unsafe { &mut *self.ptr } }
}

struct RefMutTrack<T> {
    rc: Rc<Inner<T>>
}

impl<T> RefMutTrack<T> {
    fn new(cell: &RcCell<T>) -> Option<RefMutTrack<T>> {
        let b = cell.rc.borrow.get();
        match b {
            Borrow::Ref { count: 0 } => {
                cell.rc.borrow.set(Borrow::RefMut);
                Some(RefMutTrack { rc: Rc::clone(&cell.rc) })
            }
            _ => None,
        }
    }
}

impl<T> Drop for RefMutTrack<T> {
    fn drop(&mut self) {
        let b = self.rc.borrow.get();
        match b {
            Borrow::RefMut => {
                self.rc.borrow.set(Borrow::Ref { count: 0 });
            }
            _ => panic!("Invalid borrow on RefMut drop: {:?}", b),
        }
    }
}