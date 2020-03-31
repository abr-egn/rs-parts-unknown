#![allow(unused)]

pub struct Card {
    name: String,
    behavior: Box<dyn Behavior>,
}

impl Card {
    pub fn name(&self) -> &str { &self.name }
    pub fn behavior(&self) -> &dyn Behavior { &*self.behavior }
}

pub trait Behavior: BehaviorClone {
    
}

pub trait BehaviorClone {
    fn clone_box(&self) -> Box<dyn Behavior>;
}

impl<T: 'static + Behavior + Clone> BehaviorClone for T {
    fn clone_box(&self) -> Box<dyn Behavior> {
        Box::new(self.clone())
    }
}