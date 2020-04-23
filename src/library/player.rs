use std::iter::FromIterator;

use crate::{
    creature::{Creature, Part, PartTag,},
    id_map::IdMap,
    library,
};

pub fn player() -> Creature {
    let head = Part {
        thought: 3,
        memory: 5,
        ..Part::new(
            "Head",
            &[PartTag::Head, PartTag::Flesh, PartTag::Vital, PartTag::Open],
            20)
    };
    let torso = Part::new(
        "Torso",
        &[PartTag::Torso, PartTag::Flesh, PartTag::Vital, PartTag::Open],
        50);
    let arm_l = Part {
        cards: IdMap::from_iter(vec![
            library::card::throw_debris(),
            library::card::punch(),
            library::card::guard(),
        ]),
        ..Part::new(
            "Arm",
            &[PartTag::Limb, PartTag::Flesh, PartTag::Arm, PartTag::Open],
            30)
    };
    let arm_r = arm_l.clone();
    let leg_l = Part {
        cards: IdMap::from_iter(vec![library::card::stagger()]),
        mp: 1,
        ..Part::new(
            "Leg",
            &[PartTag::Limb, PartTag::Flesh, PartTag::Leg, PartTag::Open],
            30)
    };
    let leg_r = leg_l.clone();
    Creature::new("Player", &[head, torso, arm_l, arm_r, leg_l, leg_r], None)
}