use crate::{
    id_map::{Id, IdMap},
};

pub struct ModStack<T> {
    pub base: T,
    mods: IdMap<Mod<T>>,
    mod_order: Vec<Id<Mod<T>>>,
}

pub type Mod<T> = fn(&mut T);

impl<T: Clone> ModStack<T> {
    pub fn new(base: T) -> Self {
        ModStack {
            base,
            mods: IdMap::new(),
            mod_order: vec![],
        }
    }

    pub fn eval(&self) -> T {
        let mut value = self.base.clone();
        for id in &self.mod_order {
            let m = self.mods.get(*id).unwrap();
            m(&mut value);
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