use once_cell::sync::OnceCell;
use rand::prelude::SliceRandom;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// A stat represents an attribute of a character, such as strength or speed.
/// This struct contains a stat starting value and the amount that should be
/// applied when the level increases.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Stat(pub i32, pub i32);

impl Stat {
    pub fn base(&self) -> i32 {
        self.0
    }

    pub fn increase(&self) -> i32 {
        self.1
    }

    pub fn at(&self, level: i32) -> i32 {
        self.0 + (level - 1) * self.increase()
    }
}

/// Classes are archetypes for characters.
/// The struct contains a specific stat configuration such that all instances of
/// the class have a similar combat behavior.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Class {
    pub name: String,

    pub hp: Stat,
    pub mp: Option<Stat>,
    pub strength: Stat,
    pub speed: Stat,

    pub category: Category,

    pub inflicts: Option<(super::StatusEffect, u32)>,
}

/// Determines whether the class is intended for a Player or, if it's for an enemy,
/// How rare it is (how frequently it should appear).
/// Enables easier customization of the classes via an external file.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, std::hash::Hash)]
#[serde(rename_all = "snake_case")]
pub enum Category {
    Player,
    Common,
    Rare,
    Legendary,
}

static CLASSES: OnceCell<HashMap<Category, Vec<Class>>> = OnceCell::new();

impl Class {
    /// Returns whether this is a magic class, i.e. it can inflict
    /// magic damage.
    pub fn is_magic(&self) -> bool {
        self.mp.is_some()
    }

    /// Customize the classes definitions based on an input yaml byte array.
    pub fn load(bytes: &[u8]) {
        CLASSES.set(from_bytes(bytes)).unwrap();
    }

    /// The default player class, exposed for initialization and parameterization of
    /// items and equipment.
    pub fn player_first() -> &'static Self {
        Self::of(Category::Player).first().unwrap()
    }

    pub fn player_by_name(name: &str) -> Option<&'static Self> {
        Self::of(Category::Player)
            .iter()
            .filter(|class| class.name == name)
            .collect::<Vec<&Class>>()
            .first()
            .copied()
    }

    pub fn random(category: Category) -> &'static Self {
        let mut rng = rand::thread_rng();
        Self::of(category).choose(&mut rng).unwrap()
    }

    pub fn names(category: Category) -> HashSet<String> {
        Self::of(category)
            .iter()
            .map(|class| class.name.clone())
            .collect()
    }

    fn of(category: Category) -> &'static Vec<Class> {
        CLASSES.get_or_init(default_classes).get(&category).unwrap()
    }
}

fn default_classes() -> HashMap<Category, Vec<Class>> {
    from_bytes(include_bytes!("classes.yaml"))
}

fn from_bytes(bytes: &[u8]) -> HashMap<Category, Vec<Class>> {
    // it would arguably be better for these module not to deal with deserialization
    // and yaml, but at this stage it's easier allow it to pick up defaults from
    // the local file when it hasn't been customized (especially for tests)
    let mut classes: Vec<Class> = serde_yaml::from_slice(bytes).unwrap();

    let mut class_groups = HashMap::new();
    for class in classes.drain(..) {
        let entry = class_groups
            .entry(class.category.clone())
            .or_insert_with(Vec::new);
        entry.push(class);
    }
    class_groups
}
