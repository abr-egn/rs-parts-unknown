use crate::{
    action::{Action, Event},
    id_map::Id,
};

pub trait Status: StatusClone + std::fmt::Debug {
    fn name(&self) -> &'static str;
    fn kind(&self) -> StatusKind;
    fn alter_order(&self) -> AlterOrder { AlterOrder::Misc }
    fn trigger_order(&self) -> TriggerOrder { TriggerOrder::Misc }
    fn alter(&mut self, _action: &Action) -> Option<Action> { None }
    fn trigger(&mut self, _event: &Event) -> (Vec<Action>, StatusDone) { (vec![], StatusDone::Continue) }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum StatusKind {
    Buff,
    Debuff,
    Hidden,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum AlterOrder {
    Mul,
    Add,
    Misc,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum TriggerOrder {
    Misc,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum StatusDone {
    Continue,
    Expire,
}

pub type StatusId = Id<Box<dyn Status>>;

pub trait StatusClone {
    fn clone_box(&self) -> Box<dyn Status>;
}

impl<T> StatusClone for T
where
    T: 'static + Status + Clone,
{
    fn clone_box(&self) -> Box<dyn Status> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn Status> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}