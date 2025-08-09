use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info, trace};

use crate::environment::WorldTime;
use crate::pages::{PageGraph, PageId};

/// Represents a general actor, ie NPC, in the world.
/// Stores current page/location and state, `flags` for behaviors
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Actor {
    pub id: String, // TODO: non stringly typed actor ID
    pub name: String,
    pub location: PageId, // page id
    pub state: ActorState,
    pub flags: Vec<ActorFlag>,
    // actor-specific overrides/settings for routines etc:
    //pub decision_overlays: Option<DecisionOverlay>, // combination of file loaded and inline
}

/// Decision-making for an Actor.
/// Accepts current world time, actors at the same location, and page graph.
impl Actor {
    /// Choose which action this actor will try to take this tick
    /// (pure function; dont mutate)
    pub fn decide(
        &self,
        world_time: &WorldTime,
        local_actors: &[&Actor],
        page_graph: &PageGraph,
    ) -> ActorAction {
        // fatigue-aware logic:
        let fatigue_threshold = 20; // could be per-actor/future config
        if self.state.fatigue >= fatigue_threshold {
            // Too tired! Either sleep (if awake) or continue sleeping.
            if self.state.awake {
                debug!(%self.id, fatigue=%self.state.fatigue, "Too tired, going to sleep.");
                return ActorAction::Sleep;
            } else {
                return ActorAction::Sleep; // Already sleeping
            }
        }

        let mut actions = Vec::new();

        // sleep pattern
        let is_nocturnal = self.has_flag(ActorFlag::Nocturnal);
        let is_awake = self.state.awake;
        if is_nocturnal && world_time.is_night() && !is_awake {
            actions.push(ActorAction::WakeUp);
        }
        if !is_nocturnal && world_time.is_daytime() && !is_awake {
            actions.push(ActorAction::WakeUp);
        }
        // behavior: predatory attack
        if self.has_flag(ActorFlag::Predatory) && is_awake {
            if let Some(target) = local_actors.iter().find(|a| {
                a.location == self.location && a.has_flag(ActorFlag::Organic) && a.id != self.id
            }) {
                info!(attacker=%self.id, target=%target.id, "Predator will attack");
                actions.push(ActorAction::Attack(target.id.clone()));
            }
        }
        // default: move if not tired/fatigued, else idle
        if actions.is_empty() && is_awake {
            actions.push(self.default_behavior(page_graph));
        }

        actions.into_iter().next().unwrap_or(ActorAction::Idle)
    }

    /// Return true if actor has specified flag (~component).
    pub fn has_flag(&self, flag: ActorFlag) -> bool {
        self.flags.contains(&flag)
    }

    /// Default fallback behavior: randomly move somewhere, or idle if not.
    fn default_behavior(&self, page_graph: &PageGraph) -> ActorAction {
        // For now: move very rarely (slow actors)
        // Example: ~1/100 chance to move each tick
        let move_chance = rand::random::<u8>() % 100 == 0;
        if move_chance {
            if let Some(page) = page_graph.get(&self.location) {
                if !page.connections.is_empty() {
                    // Pick a random connection
                    let mut rng = rand::rng();
                    // Specify the range and generate a random usize within it
                    let idx = rng.random_range(0..page.connections.len());
                    return ActorAction::MoveTo(page.connections[idx].target.clone());
                }
            }
        }
        ActorAction::Idle
    }

    /// Applies the decided action to mutate this actor's state.
    /// Handles fatigue, waking/sleeping, moving, etc.
    pub fn apply_action(&mut self, action: ActorAction) {
        // Modify state depending on action
        match action {
            ActorAction::Idle => {
                // Idle reduces fatigue
                if self.state.fatigue > 0 {
                    self.state.fatigue = self.state.fatigue.saturating_sub(1);
                }
                trace!(%self.id, fatigue=%self.state.fatigue, "Idling...");
            }
            ActorAction::MoveTo(page_id) => {
                // Move increases fatigue
                self.location = page_id;
                self.state.fatigue = self.state.fatigue.saturating_add(4);
                debug!(%self.id, fatigue=%self.state.fatigue, "Moved to new location.");
            }
            ActorAction::Attack(target_id) => {
                // Attack increases fatigue
                self.state.fatigue = self.state.fatigue.saturating_add(6);
                info!(%self.id, %target_id, fatigue=%self.state.fatigue, "Attacks another actor.");
            }
            ActorAction::Sleep => {
                self.state.awake = false;
                // Sleeping reduces fatigue
                self.state.fatigue = self.state.fatigue.saturating_sub(1);
                debug!(%self.id, fatigue=%self.state.fatigue, "Goes to sleep.");
            }
            ActorAction::WakeUp => {
                self.state.awake = true;
                // Waking resets fatigue a bit
                if self.state.fatigue > 2 {
                    self.state.fatigue -= 2;
                }
                debug!(%self.id, fatigue=%self.state.fatigue, "Waking up.");
            }
        }
    }
}

/// Actions an actor can perform in a single tick
#[derive(Debug)]
pub enum ActorAction {
    Idle,
    MoveTo(PageId), // page id
    Attack(String), // actor id
    Sleep,
    WakeUp,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ActorKind {
    GenericNPC,
}

/// Track actor's health, fatigue, awake state, targeting, etc
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActorState {
    pub health: i32,
    pub awake: bool,
    pub fatigue: u8,
    pub target: Option<String>, // optional id of another actor
}

/// long-term memory, tracking
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActorMemory {
    pub last_seen: HashMap<PageId, u64>, // page id -> timestamp
}

/// Map actor id -> Actor for efficient lookup
pub type ActorMap = HashMap<String, Actor>;

/// Manage all actors in the world and their tick scheduling
pub struct ActorManager {
    pub actors: ActorMap, // actor_id -> Actor
}

impl ActorManager {
    pub fn new() -> Self {
        let mut actors = HashMap::new();
        actors.insert(
            "prof".to_string(),
            Actor {
                id: "prof".to_string(),
                name: "Professor Tree".to_string(),
                location: PageId::from("small-town"),
                state: ActorState {
                    health: 10,
                    awake: true,
                    fatigue: 0,
                    target: None,
                },
                flags: vec![ActorFlag::Organic, ActorFlag::CanSpeak],
            },
        );
        actors.insert(
            "joey".to_string(),
            Actor {
                id: "joey".to_string(),
                name: "Young Joey".to_string(),
                location: PageId::from("route-1"),
                state: ActorState {
                    health: 8,
                    awake: true,
                    fatigue: 0,
                    target: None,
                },
                flags: vec![ActorFlag::Organic, ActorFlag::CanSpeak],
            },
        );
        actors.insert(
            "sneezer".to_string(),
            Actor {
                id: "sneezer".to_string(),
                name: "Sneezer".to_string(),
                location: PageId::from("route-1"),
                state: ActorState {
                    health: 2,
                    awake: true,
                    fatigue: 0,
                    target: None,
                },
                flags: vec![ActorFlag::Organic],
            },
        );
        actors.insert(
            "susan".to_string(),
            Actor {
                id: "susan".to_string(),
                name: "Susan B. Anthony".to_string(),
                location: PageId::from("green-city"),
                state: ActorState {
                    health: 99,
                    awake: true,
                    fatigue: 1,
                    target: None,
                },
                flags: vec![ActorFlag::Organic, ActorFlag::CanSpeak],
            },
        );

        ActorManager { actors }
    }

    /// Advance world, updating only 1-2 randomly selected actors
    // TODO: sequential ticking
    pub fn tick_some(&mut self, world_time: &WorldTime, page_graph: &PageGraph) {
        use rand::seq::IteratorRandom;
        let num_to_tick: usize = 1 + (self.actors.len() / 10).max(1); // customizable

        let mut rng = rand::rng();
        let chosen: Vec<String> = self
            .actors
            .keys()
            .choose_multiple(&mut rng, num_to_tick)
            .into_iter()
            .cloned()
            .collect();

        // location map for filtering
        let mut location_map: HashMap<&PageId, Vec<&str>> = HashMap::new();
        for (id, actor) in self.actors.iter() {
            location_map
                .entry(&actor.location)
                .or_default()
                .push(id.as_str());
        }

        // Gather actions just for chosen actors
        let mut events = Vec::new();
        for id in &chosen {
            if let Some(actor) = self.actors.get(id) {
                let empty = Vec::<&str>::new();
                let local_ids = location_map.get(&actor.location).unwrap_or(&empty);
                let locals: Vec<&Actor> = local_ids
                    .iter()
                    .filter_map(|oid| {
                        if *oid != id {
                            self.actors.get(*oid)
                        } else {
                            None
                        }
                    })
                    .collect();
                let action = actor.decide(world_time, &locals, page_graph);
                events.push((id.clone(), action));
            }
        }
        // Now apply their actions
        for (id, action) in events {
            if let Some(actor) = self.actors.get_mut(&id) {
                actor.apply_action(action);
            }
        }
        debug!(
            "World tick: updated {} of {} actors.",
            num_to_tick,
            self.actors.len()
        );
    }
}

// TODO: modularize as more complex components instead
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ActorFlag {
    Organic,
    CanAttack,
    CanSpeak,
    Nocturnal,
    Predatory,
}
