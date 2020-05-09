use crate::{
    action::{Action, Event},
    id_map::Id,
};

pub trait Status: StatusClone + std::fmt::Debug {
    fn name(&self) -> &'static str;
    fn kind(&self) -> StatusKind;
    fn order(&self) -> StatusOrder { StatusOrder::Misc }
    fn alter(&mut self, _action: &Action) -> Option<Action> { None }
    fn trigger(&mut self, _this: StatusId, _events: &Event) -> Vec<Action> { vec![] }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum StatusKind {
    Buff,
    Debuff,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum StatusOrder {
    Misc,
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