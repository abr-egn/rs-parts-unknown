use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct IdMap<T> {
    map: HashMap<Id<T>, T>,
    next_id: Id<T>,
}

#[allow(dead_code)]
impl<T> IdMap<T> {
    pub fn new() -> IdMap<T> {
        IdMap {
            map: HashMap::new(),
            next_id: Id::new(0),
        }
    }

    pub fn map(&self) -> &HashMap<Id<T>, T> { &self.map }

    pub fn add(&mut self, value: T) -> Id<T> {
        let id = self.next_id.inc();
        self.map.insert(id, value);
        id
    }
    pub fn get_mut(&mut self, id: &Id<T>) -> Option<&mut T> { self.map.get_mut(id) }
    pub fn values_mut(&mut self) -> impl Iterator<Item=&mut T> { self.map.values_mut() }
    pub fn iter_mut(&mut self) -> impl Iterator<Item=(&Id<T>, &mut T)> { self.map.iter_mut() }
    pub fn remove(&mut self, id: &Id<T>) -> Option<T> { self.map.remove(id) }
}

#[derive(Serialize, Deserialize)]
pub struct Id<T> {
    value: u32,
    #[serde(skip)]
    phantom: PhantomData<*const T>,
}

/*
impl<T> Serialize for Id<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u32(self.value)
    }
}

impl<'de, T> Deserialize<'de> for Id<T> {
    
}
*/

impl<T> Id<T> {
    fn new(value: u32) -> Id<T> { Id { value: value, phantom: PhantomData } }
    fn inc(&mut self) -> Id<T> {
        let out = *self;
        self.value += 1;
        out
    }
    /*
    pub fn value(&self) -> u32 { self.value }
    pub fn synthesize(value: u32) -> Id<T> { Id::new(value) }
    */
}

impl<T> fmt::Debug for Id<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Id({:?})", self.value)
    }
}

impl<T> Copy for Id<T> { }

impl<T> Clone for Id<T> {
    fn clone(&self) -> Id<T> { Id::new(self.value) }
    fn clone_from(&mut self, source: &Id<T>) { self.value = source.value }
}

impl<T> PartialEq for Id<T> {
    fn eq(&self, other: &Id<T>) -> bool { self.value == other.value }
}

impl<T> Eq for Id<T> { }

impl<T> PartialOrd for Id<T>{
    fn partial_cmp(&self, other: &Id<T>) -> Option<Ordering> { Some(self.cmp(other)) }
}

impl<T> Ord for Id<T> {
    fn cmp(&self, other: &Id<T>) -> Ordering { self.value.cmp(&other.value) }
}

impl<T> Hash for Id<T> {
    fn hash<H: Hasher>(&self, state: &mut H) { self.value.hash(state) }
}

unsafe impl<T> Send for Id<T> { }
unsafe impl<T> Sync for Id<T> { }