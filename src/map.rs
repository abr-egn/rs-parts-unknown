use std::collections::{
    hash_map::{Entry, VacantEntry},
    HashMap, HashSet, VecDeque,
};

use fnv::FnvHashSet;
use hex::Hex;
use serde::Serialize;

use crate::creature::Creature;
use crate::error::{Error, Result};
use crate::id_map::Id;

#[derive(Debug, Clone)]
pub struct Map {
    tiles: HashMap<Hex, Tile>,
    creatures: HashMap<Id<Creature>, Hex>,
}

impl Map {
    pub fn new() -> Self {
        let mut tiles = HashMap::new();
        for coord in hex::ORIGIN.area(5) {
            let space = match coord.distance_to(hex::ORIGIN) {
                5 => Space::Wall,
                2 if coord.x % 2 == 0 && coord.y % 2 == 0 => Space::Wall,
                _ => Space::Empty,
            };
            tiles.insert(coord, Tile { space, creature: None });
        }
        Map { tiles, creatures: HashMap::new() }
    }

    // Accessors

    pub fn tiles(&self) -> &HashMap<Hex, Tile> {
        &self.tiles
    }

    pub fn creatures(&self) -> &HashMap<Id<Creature>, Hex> {
        &self.creatures
    }

    pub fn range_from(&self, start: Hex, range: i32) -> HashSet<Hex> {
        let mut out = HashSet::new();
        let mut pending: VecDeque<(Hex, i32)> = VecDeque::new();
        pending.push_back((start, range));
        while !pending.is_empty() {
            let (current, remaining_range) = pending.pop_front().unwrap();
            if !out.insert(current) { continue }
            if remaining_range > 0 {
                for hex in current.neighbors() {
                    if out.contains(&hex) { continue }
                    if let Some(Tile { space: Space::Empty, .. }) = self.tiles.get(&hex) {
                        pending.push_back((hex, remaining_range-1));
                    }
                }
            }
        }
        out
    }

    pub fn path_to(&self, from: Hex, to: Hex) -> Result<Vec<Hex>> {
        let to_tile = self.tiles.get(&to).ok_or(Error::OutOfBounds)?;
        if to_tile.space != Space::Empty || to_tile.creature.is_some() {
            return Err(Error::Obstructed);
        }
        let tiles = &self.tiles;
        let neighbors = |hex: Hex| -> Vec<Hex> {
            let mut out = vec![];
            for neighbor in hex.neighbors() {
                match tiles.get(&neighbor) {
                    Some(t) if is_open(t) => (),
                    _ => continue,
                }
                out.push(neighbor)
            }
            out
        };
        let path: Vec<_> = match a_star(from, to, neighbors) {
            Some(p) => p,
            None => return Err(Error::Obstructed),
        };
        for coord in path.iter().skip(1) {
            let tile = self.tiles.get(coord).ok_or(Error::OutOfBounds)?;
            if !is_open(tile) {
                return Err(Error::Obstructed);
            }
        }
        Ok(path)
    }

    // Mutators

    pub fn place_at(&mut self, creature_id: Id<Creature>, at: Hex) -> Result<()> {
        let tile = self.tiles.get_mut(&at).ok_or(Error::OutOfBounds)?;
        if tile.creature.is_some() { return Err(Error::Obstructed) }
        let c_ent = vacant_or(self.creatures.entry(creature_id), Error::Obstructed)?;
        tile.creature = Some(creature_id);
        c_ent.insert(at);
        Ok(())
    }

    pub fn move_to(&mut self, creature_id: Id<Creature>, to: Hex) -> Result<()> {
        let from: &Hex = self.creatures.get(&creature_id).ok_or(Error::NoSuchCreature)?;
        self.tiles.get_mut(&to).ok_or(Error::OutOfBounds)?.creature = Some(creature_id);
        self.tiles.get_mut(from).unwrap().creature = None;
        self.creatures.insert(creature_id, to);
        Ok(())
    }
}

fn is_open(tile: &Tile) -> bool {
    match tile {
        Tile { space: Space::Empty, creature: None } => true,
        _ => false,
    }
}

fn a_star<F>(start: Hex, goal: Hex, neighbors: F) -> Option<Vec<Hex>>
    where F: Fn(Hex) -> Vec<Hex>
{
    let mut open_set = FnvHashSet::default();
    open_set.insert(start);

    let mut came_from: HashMap<Hex, Hex> = HashMap::new();

    let mut g_score: HashMap<Hex, i32> = HashMap::new();
    g_score.insert(start, 0);

    let mut f_score: HashMap<Hex, i32> = HashMap::new();
    f_score.insert(start, start.distance_to(goal));

    while !open_set.is_empty() {
        let mut current = None;
        for &coord in &open_set {
            let f = f_score.get(&coord).unwrap_or(&std::i32::MAX);
            match current {
                None => {
                    current = Some((coord, f));
                }
                Some((_, other_f)) if f < other_f => {
                    current = Some((coord, f));
                }
                _ => (),
            }
        }
        let (current, _) = current.unwrap();
        if current == goal {
            return Some(reconstruct_path(came_from, current))
        }
        open_set.remove(&current);
        for neighbor in neighbors(current) {
            let tentative_g_score = g_score.get(&current).unwrap() + 1;
            if tentative_g_score < *g_score.get(&neighbor).unwrap_or(&std::i32::MAX) {
                came_from.insert(neighbor, current);
                g_score.insert(neighbor, tentative_g_score);
                f_score.insert(neighbor, tentative_g_score + neighbor.distance_to(goal));
                open_set.insert(neighbor);
            }
        }
    }

    None
}

fn reconstruct_path(came_from: HashMap<Hex, Hex>, candidate: Hex) -> Vec<Hex> {
    let mut out = vec![];
    out.push(candidate);
    let mut candidate = candidate;
    while let Some(prev) = came_from.get(&candidate) {
        candidate = *prev;
        out.push(candidate);
    }
    out.reverse();
    out
}

fn vacant_or<K, V, E>(e: Entry<K, V>, err: E) -> std::result::Result<VacantEntry<K, V>, E> {
    match e {
        Entry::Occupied(_) => Err(err),
        Entry::Vacant(v) => Ok(v),
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Tile {
    pub space: Space,
    pub creature: Option<Id<Creature>>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize)]
pub enum Space {
    Empty,
    Wall,
}