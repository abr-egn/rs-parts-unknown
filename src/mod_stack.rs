use crate::{
    id_map::{Id, IdMap},
};

#[derive(Debug, Clone)]
pub struct ModStack<T> {
    mods: IdMap<Mod<T>>,
    mod_order: Vec<Id<Mod<T>>>,
}

#[derive(Clone)]
pub struct Mod<T>(fn(&mut T));

impl<T> ModStack<T> {
    pub fn new() -> Self {
        ModStack {
            mods: IdMap::new(),
            mod_order: vec![],
        }
    }

    pub fn eval(&self, base: T) -> T {
        let mut value = base;
        for id in &self.mod_order {
            let m = self.mods.get(*id).unwrap();
            m.0(&mut value);
        }
        value
    }

    pub fn mods(&self) -> &IdMap<Mod<T>> { &self.mods }

    pub fn add(&mut self, m: Mod<T>) -> Id<Mod<T>> {
        let id = self.mods.add(m);
        self.mod_order.push(id);
        id
    }

    pub fn remove(&mut self, id: Id<Mod<T>>) -> bool {
        if self.mods.remove(&id).is_none() { return false; }
        let ix = self.mod_order.iter().position(|i| *i == id).unwrap();
        self.mod_order.remove(ix);
        true
    }
}

impl<T> std::fmt::Debug for Mod<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Mod")
            .field(&(self.0 as * const ()))
            .finish()
    }
}

impl<T> Default for Mod<T> {
    fn default() -> Self { Mod(|_| { }) }
}