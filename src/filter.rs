use crate::{
    creature::{Creature, CreatureAction},
    event::{Action},
    id_map::Id,
    part::{Part, PartAction},
};

pub trait Filter: FilterClone + std::fmt::Debug {
    fn name(&self) -> &'static str;
    fn kind(&self) -> FilterKind;
    // PartAction
    fn apply_hit(&mut self, _cid: Id<Creature>, _pid: Id<Part>, _damage: &mut i32) {}
}

impl dyn Filter {
    fn apply(&mut self, action: &mut Action) {
        match action {
            Action::ToCreature { id: cid, action: creature_action } => {
                match creature_action {
                    CreatureAction::ToPart { id: pid, action: part_action } => {
                        match part_action {
                            PartAction::Hit { damage } => self.apply_hit(*cid, *pid, damage),
                            _ => (),
                        }
                    }
                    _ => (),
                }
            }
            _ => (),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum FilterKind {
    DamageMul,
    DamageAdd,
    Misc,
}

pub type FilterId = Id<Box<dyn Filter>>;

pub trait FilterClone {
    fn clone_box(&self) -> Box<dyn Filter>;
}

impl<T> FilterClone for T
where
    T: 'static + Filter + Clone,
{
    fn clone_box(&self) -> Box<dyn Filter> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn Filter> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}