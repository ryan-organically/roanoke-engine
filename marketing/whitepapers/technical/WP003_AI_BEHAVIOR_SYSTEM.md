# WHITEPAPER WP-003
## AI Behavior System Architecture
### Emergent Intelligence for Living Worlds

---

<!--
@document-metadata
doc_id: WP-003
title: AI Behavior System
version: 1.0.0
status: ACTIVE
owner: CTO
created: 2025-12-05
updated: 2025-12-05
review_date: 2026-06-05
classification: Public Technical Documentation
changelog: See /marketing/CHANGELOG.md
-->

| Field | Value |
|-------|-------|
| **Document ID** | WP-003 |
| **Version** | 1.0.0 |
| **Status** | ACTIVE |
| **Owner** | CTO / Engine Team |
| **Last Updated** | 2025-12-05 |
| **Classification** | Public Technical Documentation |

**Abstract:** This whitepaper describes the Roanoke Engine's AI behavior system, which enables thousands of autonomous agents (wildlife, NPCs) to exhibit believable, emergent behaviors. We present our utility-based decision architecture, hierarchical goal system, and optimization strategies that create living ecosystems without scripted interactions.

---

## 1. Introduction

### 1.1 The Believability Challenge

Traditional game AI relies on state machines or behavior trees that produce predictable, repetitive actions. For Roanoke's vision of a living world, we require AI that:

- **Adapts** to changing environmental conditions
- **Remembers** past experiences and player interactions
- **Emerges** complex behaviors from simple rules
- **Scales** to thousands of concurrent agents
- **Surprises** players without feeling random

### 1.2 Design Principles

1. **Utility over States:** Actions are selected based on continuous utility scores, not discrete state transitions
2. **Goals over Scripts:** Agents pursue dynamic goals rather than following predetermined paths
3. **Emergence over Authoring:** Complex behaviors emerge from simple, composable components
4. **Efficiency over Accuracy:** Approximate solutions at scale beat perfect solutions for few agents

---

## 2. System Architecture

### 2.1 Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│                    AI BEHAVIOR ARCHITECTURE                          │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   PERCEPTION LAYER                                                   │
│   ┌────────────────────────────────────────────────────────────┐    │
│   │  Sensory System  │  Spatial Queries  │  Memory System      │    │
│   └────────────────────────────────────────────────────────────┘    │
│                              │                                       │
│   ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ┼ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─   │
│                              ▼                                       │
│   DECISION LAYER                                                     │
│   ┌────────────────────────────────────────────────────────────┐    │
│   │  Goal Manager  │  Utility Evaluator  │  Action Selector    │    │
│   └────────────────────────────────────────────────────────────┘    │
│                              │                                       │
│   ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ┼ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─   │
│                              ▼                                       │
│   EXECUTION LAYER                                                    │
│   ┌────────────────────────────────────────────────────────────┐    │
│   │  Navigation  │  Animation  │  Interaction  │  Physics      │    │
│   └────────────────────────────────────────────────────────────┘    │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

### 2.2 Agent Definition

```rust
pub struct AIAgent {
    // Identity
    pub id: AgentId,
    pub species: SpeciesType,
    pub archetype: BehaviorArchetype,

    // Physical State
    pub transform: Transform,
    pub velocity: Vec3,
    pub physical_stats: PhysicalStats,

    // Mental State
    pub needs: NeedsState,
    pub emotions: EmotionalState,
    pub memory: AgentMemory,
    pub goals: GoalStack,

    // Perception
    pub senses: SensorySystem,
    pub known_entities: KnowledgeBase,

    // Execution
    pub current_action: Option<ActionInstance>,
    pub navigation: NavigationState,
}

#[derive(Clone)]
pub struct PhysicalStats {
    pub health: f32,           // 0.0 - 1.0
    pub stamina: f32,          // 0.0 - 1.0
    pub hunger: f32,           // 0.0 - 1.0 (1.0 = starving)
    pub age: f32,              // 0.0 - 1.0 (1.0 = elderly)
    pub size: f32,             // Relative to species average
}

#[derive(Clone)]
pub struct NeedsState {
    pub survival: f32,         // Immediate danger response
    pub sustenance: f32,       // Hunger, thirst
    pub rest: f32,             // Energy recovery
    pub social: f32,           // Pack/herd interaction
    pub reproduction: f32,     // Seasonal breeding
    pub territory: f32,        // Space defense
}
```

---

## 3. Perception System

### 3.1 Sensory Model

Each species has unique sensory capabilities:

```rust
pub struct SensorySystem {
    pub vision: VisionSense,
    pub hearing: HearingSense,
    pub smell: SmellSense,
    pub touch: TouchSense,
}

pub struct VisionSense {
    pub range: f32,
    pub field_of_view: f32,      // Radians
    pub night_vision: f32,       // 0.0 - 1.0
    pub motion_sensitivity: f32,  // Detects movement more easily
    pub color_perception: bool,   // Some animals are colorblind
}

impl SensorySystem {
    pub fn perceive(&self, agent: &AIAgent, world: &World) -> PerceptionResult {
        let mut perceived = PerceptionResult::new();

        // Vision
        for entity in world.spatial_query_cone(
            agent.transform.position,
            agent.transform.forward(),
            self.vision.field_of_view,
            self.vision.range,
        ) {
            let visibility = self.calculate_visibility(agent, entity, world);
            if visibility > 0.0 {
                perceived.add_visual(entity, visibility);
            }
        }

        // Hearing
        for sound in world.recent_sounds() {
            let audibility = self.calculate_audibility(agent, sound);
            if audibility > 0.0 {
                perceived.add_auditory(sound.source, sound.location, audibility);
            }
        }

        // Smell
        for scent_trail in world.scent_grid.query_radius(
            agent.transform.position,
            self.smell.range,
        ) {
            let detectability = self.calculate_scent(agent, scent_trail);
            if detectability > 0.0 {
                perceived.add_olfactory(scent_trail, detectability);
            }
        }

        perceived
    }

    fn calculate_visibility(
        &self,
        agent: &AIAgent,
        target: &Entity,
        world: &World,
    ) -> f32 {
        let direction = target.position - agent.transform.position;
        let distance = direction.length();

        // Distance falloff
        let distance_factor = 1.0 - (distance / self.vision.range).min(1.0);

        // Angle from forward
        let angle = agent.transform.forward().angle_between(direction.normalize());
        let angle_factor = 1.0 - (angle / (self.vision.field_of_view / 2.0)).min(1.0);

        // Occlusion check
        let occluded = world.raycast(agent.eye_position(), target.position).hit;
        if occluded {
            return 0.0;
        }

        // Target visibility (size, movement, camouflage)
        let target_visibility = target.visibility_factor();

        // Lighting conditions
        let light_factor = world.light_level_at(target.position)
            .max(self.vision.night_vision);

        // Motion detection bonus
        let motion_bonus = if target.velocity.length() > 0.1 {
            self.vision.motion_sensitivity * 0.3
        } else {
            0.0
        };

        (distance_factor * angle_factor * target_visibility * light_factor + motion_bonus)
            .min(1.0)
    }
}
```

### 3.2 Memory System

Agents remember past experiences:

```rust
pub struct AgentMemory {
    pub short_term: VecDeque<MemoryEvent>,
    pub long_term: HashMap<EntityId, EntityMemory>,
    pub locations: SpatialMemory,
    pub threats: ThreatMemory,
}

#[derive(Clone)]
pub struct MemoryEvent {
    pub timestamp: f64,
    pub event_type: EventType,
    pub location: Vec3,
    pub entities_involved: Vec<EntityId>,
    pub emotional_valence: f32,  // -1.0 (bad) to 1.0 (good)
    pub importance: f32,
}

#[derive(Clone)]
pub struct EntityMemory {
    pub last_seen: f64,
    pub last_position: Vec3,
    pub relationship: f32,           // -1.0 (hostile) to 1.0 (friendly)
    pub threat_level: f32,           // 0.0 to 1.0
    pub interactions: Vec<InteractionMemory>,
}

impl AgentMemory {
    pub fn record_event(&mut self, event: MemoryEvent) {
        // Add to short-term
        self.short_term.push_back(event.clone());

        // Consolidate important events to long-term
        if event.importance > 0.7 {
            self.consolidate_to_long_term(&event);
        }

        // Memory decay
        self.decay(event.timestamp);
    }

    fn decay(&mut self, current_time: f64) {
        // Short-term decay (forget after ~30 seconds)
        while let Some(front) = self.short_term.front() {
            if current_time - front.timestamp > 30.0 {
                self.short_term.pop_front();
            } else {
                break;
            }
        }

        // Long-term decay (fade over hours of game time)
        for memory in self.long_term.values_mut() {
            let age = current_time - memory.last_seen;
            memory.relationship *= (-age / 3600.0).exp() as f32;
        }
    }
}
```

---

## 4. Decision Making

### 4.1 Utility-Based Selection

Actions are chosen based on utility scores:

```rust
pub struct UtilityEvaluator {
    pub considerations: Vec<Box<dyn Consideration>>,
}

pub trait Consideration: Send + Sync {
    fn evaluate(&self, agent: &AIAgent, action: &ActionTemplate, world: &World) -> f32;
    fn weight(&self) -> f32;
}

impl UtilityEvaluator {
    pub fn evaluate_action(
        &self,
        agent: &AIAgent,
        action: &ActionTemplate,
        world: &World,
    ) -> f32 {
        let mut total_utility = 1.0;
        let mut total_weight = 0.0;

        for consideration in &self.considerations {
            let score = consideration.evaluate(agent, action, world);
            let weight = consideration.weight();

            // Multiplicative combination (all factors must be satisfied)
            total_utility *= score.powf(weight);
            total_weight += weight;
        }

        // Normalize
        if total_weight > 0.0 {
            total_utility.powf(1.0 / total_weight)
        } else {
            0.0
        }
    }
}

// Example considerations

pub struct HungerConsideration;

impl Consideration for HungerConsideration {
    fn evaluate(&self, agent: &AIAgent, action: &ActionTemplate, _world: &World) -> f32 {
        match action.category {
            ActionCategory::Eat => {
                // Higher utility when hungrier
                response_curve_exponential(agent.needs.sustenance, 2.0)
            }
            ActionCategory::Hunt => {
                // Slightly lower threshold to start hunting
                response_curve_exponential(agent.needs.sustenance * 0.8, 2.0)
            }
            _ => 1.0, // Neutral for unrelated actions
        }
    }

    fn weight(&self) -> f32 { 1.5 }
}

pub struct ThreatConsideration;

impl Consideration for ThreatConsideration {
    fn evaluate(&self, agent: &AIAgent, action: &ActionTemplate, world: &World) -> f32 {
        let threat_level = agent.memory.threats.current_threat_level();

        match action.category {
            ActionCategory::Flee => {
                // High utility when threatened
                response_curve_exponential(threat_level, 3.0)
            }
            ActionCategory::Fight => {
                // Only if cornered or protecting young
                let fight_threshold = if agent.has_offspring_nearby(world) { 0.3 } else { 0.8 };
                if threat_level > fight_threshold {
                    response_curve_linear(threat_level, 0.5, 1.0)
                } else {
                    0.0
                }
            }
            _ => {
                // Suppress non-survival actions when threatened
                1.0 - threat_level * 0.8
            }
        }
    }

    fn weight(&self) -> f32 { 2.0 } // High priority
}

fn response_curve_exponential(x: f32, exponent: f32) -> f32 {
    x.powf(exponent)
}

fn response_curve_linear(x: f32, min: f32, max: f32) -> f32 {
    ((x - min) / (max - min)).clamp(0.0, 1.0)
}
```

### 4.2 Goal System

Agents pursue hierarchical goals:

```rust
pub struct GoalStack {
    goals: Vec<Goal>,
}

pub struct Goal {
    pub goal_type: GoalType,
    pub priority: f32,
    pub insistence: f32,  // How much it demands attention
    pub target: Option<EntityId>,
    pub location: Option<Vec3>,
    pub expiry: Option<f64>,
    pub subgoals: Vec<Goal>,
}

#[derive(Clone, PartialEq)]
pub enum GoalType {
    // Survival
    Survive,
    Flee { from: EntityId },
    Hide,

    // Sustenance
    FindFood,
    Eat { target: EntityId },
    FindWater,
    Drink,

    // Rest
    FindShelter,
    Sleep,
    Rest,

    // Social
    FindMate,
    Court { target: EntityId },
    ProtectOffspring,
    FollowHerd,

    // Territory
    Patrol,
    MarkTerritory,
    DefendTerritory,

    // Exploration
    Wander,
    Investigate { location: Vec3 },
}

impl GoalStack {
    pub fn update(&mut self, agent: &AIAgent, world: &World) {
        // Generate new goals based on needs
        self.generate_goals(agent, world);

        // Prune completed/expired goals
        self.prune();

        // Sort by priority and insistence
        self.goals.sort_by(|a, b| {
            let a_score = a.priority * a.insistence;
            let b_score = b.priority * b.insistence;
            b_score.partial_cmp(&a_score).unwrap()
        });
    }

    fn generate_goals(&mut self, agent: &AIAgent, world: &World) {
        // Survival goals (highest priority)
        if agent.memory.threats.is_threatened() {
            self.push_if_new(Goal {
                goal_type: GoalType::Survive,
                priority: 1.0,
                insistence: agent.memory.threats.current_threat_level(),
                ..Default::default()
            });
        }

        // Sustenance goals
        if agent.needs.sustenance > 0.4 {
            self.push_if_new(Goal {
                goal_type: GoalType::FindFood,
                priority: 0.8,
                insistence: agent.needs.sustenance,
                ..Default::default()
            });
        }

        // Social goals (if not stressed)
        if agent.needs.social > 0.5 && agent.needs.survival < 0.3 {
            self.push_if_new(Goal {
                goal_type: GoalType::FollowHerd,
                priority: 0.4,
                insistence: agent.needs.social,
                ..Default::default()
            });
        }

        // Default wandering
        if self.goals.is_empty() {
            self.push_if_new(Goal {
                goal_type: GoalType::Wander,
                priority: 0.1,
                insistence: 0.5,
                ..Default::default()
            });
        }
    }
}
```

### 4.3 Action Selection

The best action for current goals:

```rust
pub struct ActionSelector {
    action_templates: Vec<ActionTemplate>,
    utility_evaluator: UtilityEvaluator,
}

impl ActionSelector {
    pub fn select_action(
        &self,
        agent: &AIAgent,
        world: &World,
    ) -> Option<ActionInstance> {
        let current_goal = agent.goals.current()?;

        // Filter actions applicable to current goal
        let applicable: Vec<_> = self.action_templates
            .iter()
            .filter(|a| a.supports_goal(&current_goal.goal_type))
            .filter(|a| a.preconditions_met(agent, world))
            .collect();

        if applicable.is_empty() {
            return None;
        }

        // Evaluate utility of each action
        let mut scored: Vec<_> = applicable
            .iter()
            .map(|action| {
                let utility = self.utility_evaluator.evaluate_action(agent, action, world);
                (action, utility)
            })
            .collect();

        // Sort by utility (descending)
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        // Select with weighted randomness (avoid predictability)
        let selected = self.weighted_random_select(&scored);

        Some(selected.instantiate(agent, world))
    }

    fn weighted_random_select<'a>(
        &self,
        scored: &[(&'a ActionTemplate, f32)],
    ) -> &'a ActionTemplate {
        // Top 3 actions compete with weights
        let top_n = scored.iter().take(3).collect::<Vec<_>>();

        let total_weight: f32 = top_n.iter().map(|(_, w)| w).sum();
        let mut random = rand::random::<f32>() * total_weight;

        for (action, weight) in top_n {
            random -= weight;
            if random <= 0.0 {
                return action;
            }
        }

        scored[0].0
    }
}
```

---

## 5. Species Behaviors

### 5.1 Behavior Archetypes

```rust
pub enum BehaviorArchetype {
    // Prey animals
    Grazer {
        herd_size: Range<usize>,
        vigilance: f32,
        flight_distance: f32,
    },

    // Predators
    Stalker {
        hunt_style: HuntStyle,
        territory_size: f32,
        pack_size: Range<usize>,
    },

    // Omnivores
    Opportunist {
        diet_preference: f32,  // 0.0 = herbivore, 1.0 = carnivore
        curiosity: f32,
    },

    // Special behaviors
    Territorial {
        territory_size: f32,
        aggression: f32,
    },

    Migratory {
        migration_season: Season,
        migration_distance: f32,
    },
}

pub enum HuntStyle {
    Ambush,      // Wait and pounce
    Pursuit,     // Chase down
    Pack,        // Coordinated group hunting
    Scavenge,    // Follow other predators
}
```

### 5.2 Deer Behavior Example

```rust
pub fn create_deer_behavior() -> BehaviorArchetype {
    BehaviorArchetype::Grazer {
        herd_size: 3..12,
        vigilance: 0.7,
        flight_distance: 30.0,
    }
}

pub fn deer_action_set() -> Vec<ActionTemplate> {
    vec![
        ActionTemplate {
            name: "graze",
            category: ActionCategory::Eat,
            duration: Duration::Continuous,
            preconditions: vec![
                Precondition::NearResource(ResourceType::Grass, 2.0),
                Precondition::NotThreatened,
            ],
            effects: vec![
                Effect::ReduceNeed(NeedType::Sustenance, 0.1),
            ],
        },
        ActionTemplate {
            name: "flee",
            category: ActionCategory::Flee,
            duration: Duration::UntilSafe,
            preconditions: vec![
                Precondition::ThreatDetected,
            ],
            effects: vec![
                Effect::MoveAwayFrom(ThreatSource),
                Effect::AlertHerd,
            ],
        },
        ActionTemplate {
            name: "scan_for_threats",
            category: ActionCategory::Vigilance,
            duration: Duration::Fixed(3.0),
            preconditions: vec![],
            effects: vec![
                Effect::UpdateThreatKnowledge,
            ],
        },
        ActionTemplate {
            name: "follow_herd",
            category: ActionCategory::Social,
            duration: Duration::Continuous,
            preconditions: vec![
                Precondition::HerdNearby,
                Precondition::NotThreatened,
            ],
            effects: vec![
                Effect::MoveTowardHerd,
                Effect::ReduceNeed(NeedType::Social, 0.05),
            ],
        },
        ActionTemplate {
            name: "rest",
            category: ActionCategory::Rest,
            duration: Duration::Fixed(60.0),
            preconditions: vec![
                Precondition::StaminaBelow(0.3),
                Precondition::NotThreatened,
                Precondition::NearCover,
            ],
            effects: vec![
                Effect::RestoreStamina(0.5),
            ],
        },
    ]
}
```

### 5.3 Wolf Pack Behavior

```rust
pub fn create_wolf_behavior() -> BehaviorArchetype {
    BehaviorArchetype::Stalker {
        hunt_style: HuntStyle::Pack,
        territory_size: 500.0,
        pack_size: 4..8,
    }
}

pub struct PackBehavior {
    alpha: Option<AgentId>,
    members: Vec<AgentId>,
    territory_center: Vec3,
    current_hunt: Option<HuntState>,
}

pub struct HuntState {
    target: EntityId,
    phase: HuntPhase,
    participants: Vec<(AgentId, HuntRole)>,
}

pub enum HuntPhase {
    Stalking,
    Flanking,
    Chase,
    Attack,
    Feeding,
}

pub enum HuntRole {
    Driver,      // Push prey toward ambush
    Flanker,     // Cut off escape routes
    Attacker,    // Primary attack
    Reserve,     // Fresh energy for long chases
}

impl PackBehavior {
    pub fn coordinate_hunt(
        &mut self,
        pack: &[&mut AIAgent],
        target: &Entity,
        world: &World,
    ) {
        match self.current_hunt.as_ref().map(|h| &h.phase) {
            Some(HuntPhase::Stalking) => {
                // All wolves approach slowly, staying downwind
                let wind_dir = world.wind_direction();
                for agent in pack {
                    let approach_vector = self.calculate_downwind_approach(
                        agent.transform.position,
                        target.position,
                        wind_dir,
                    );
                    agent.goals.push(Goal {
                        goal_type: GoalType::MoveTo {
                            target: approach_vector,
                            speed: MovementSpeed::Slow,
                        },
                        priority: 0.9,
                        ..Default::default()
                    });
                }
            }
            Some(HuntPhase::Flanking) => {
                // Assign roles based on position
                self.assign_hunt_roles(pack, target);

                for (agent, role) in pack.iter_mut().zip(self.get_roles()) {
                    match role {
                        HuntRole::Driver => {
                            // Move behind prey
                            let behind = target.position - target.velocity.normalize() * 20.0;
                            agent.goals.push(Goal::move_to(behind, MovementSpeed::Normal));
                        }
                        HuntRole::Flanker => {
                            // Move to sides
                            let flank = self.calculate_flank_position(agent, target);
                            agent.goals.push(Goal::move_to(flank, MovementSpeed::Normal));
                        }
                        HuntRole::Attacker => {
                            // Wait for signal
                            agent.goals.push(Goal::wait_for_signal(HuntSignal::Attack));
                        }
                        _ => {}
                    }
                }
            }
            Some(HuntPhase::Chase) => {
                // All wolves pursue at full speed
                for agent in pack {
                    if agent.physical_stats.stamina > 0.2 {
                        agent.goals.push(Goal {
                            goal_type: GoalType::Pursue { target: target.id },
                            priority: 1.0,
                            ..Default::default()
                        });
                    } else {
                        // Exhausted wolves drop back
                        agent.goals.push(Goal::rest());
                    }
                }
            }
            _ => {}
        }
    }
}
```

---

## 6. Emergent Behaviors

### 6.1 Ecosystem Dynamics

Simple rules create complex ecosystem behavior:

```rust
pub struct EcosystemSimulation {
    pub predator_prey_graph: PredatorPreyGraph,
    pub resource_distribution: ResourceGrid,
    pub population_dynamics: PopulationTracker,
}

impl EcosystemSimulation {
    pub fn update(&mut self, world: &World, delta: f32) {
        // Track population health
        for species in self.species_list() {
            let population = world.count_species(species);
            let avg_health = world.average_health(species);
            let reproduction_rate = world.reproduction_rate(species);

            self.population_dynamics.record(species, PopulationSnapshot {
                count: population,
                health: avg_health,
                reproduction: reproduction_rate,
            });
        }

        // Dynamic spawn adjustment
        for species in self.species_list() {
            let target = self.target_population(species);
            let current = world.count_species(species);

            if current < target * 0.7 {
                // Population too low, increase spawns
                self.spawner.increase_rate(species, 1.2);
            } else if current > target * 1.3 {
                // Population too high, reduce spawns
                self.spawner.decrease_rate(species, 0.8);
            }
        }
    }

    fn target_population(&self, species: SpeciesType) -> usize {
        // Based on available resources and predator pressure
        let food_availability = self.resource_distribution.availability_for(species);
        let predator_count = self.count_predators_of(species);

        let base = species.baseline_population();
        let food_factor = food_availability / species.food_requirement();
        let predator_factor = 1.0 / (1.0 + predator_count as f32 * 0.1);

        (base as f32 * food_factor * predator_factor) as usize
    }
}
```

### 6.2 Territorial Behavior Emergence

```rust
pub struct TerritorySystem {
    territories: HashMap<AgentId, Territory>,
    scent_grid: ScentGrid,
}

pub struct Territory {
    owner: AgentId,
    center: Vec3,
    radius: f32,
    strength: f32,  // How well-marked/defended
    contested_by: Vec<AgentId>,
}

impl TerritorySystem {
    pub fn update_territories(&mut self, agents: &[AIAgent], delta: f32) {
        for agent in agents {
            if agent.archetype.is_territorial() {
                // Update scent marking
                if agent.current_action.is_marking_territory() {
                    self.scent_grid.add_scent(
                        agent.transform.position,
                        agent.id,
                        agent.scent_strength(),
                    );
                }

                // Check for intruders
                let intruders = self.find_intruders(agent);
                for intruder in intruders {
                    // Record conflict, may trigger aggression
                    self.territories
                        .entry(agent.id)
                        .or_insert_with(|| Territory::new(agent))
                        .contested_by
                        .push(intruder);
                }
            }
        }

        // Decay scent marks
        self.scent_grid.decay(delta);

        // Resolve territorial disputes
        self.resolve_disputes();
    }

    fn resolve_disputes(&mut self) {
        for territory in self.territories.values_mut() {
            if !territory.contested_by.is_empty() {
                // Strength-based resolution
                // Stronger/older animals maintain territory
                // Weaker challengers back down or escalate to combat
            }
        }
    }
}
```

---

## 7. Performance Optimization

### 7.1 LOD-Based AI Updates

Distant agents use simplified AI:

```rust
pub struct AIUpdateScheduler {
    update_buckets: [Vec<AgentId>; 4],
    current_bucket: usize,
}

pub enum AIUpdateLOD {
    Full,       // Every frame, full perception and decision
    Reduced,    // Every 4 frames, limited perception
    Minimal,    // Every 16 frames, goal updates only
    Dormant,    // Every 60 frames, existence check only
}

impl AIUpdateScheduler {
    pub fn get_agents_to_update(&mut self) -> Vec<(AgentId, AIUpdateLOD)> {
        let mut to_update = Vec::new();

        // Full update for nearby agents
        for id in &self.buckets[AIUpdateLOD::Full] {
            to_update.push((*id, AIUpdateLOD::Full));
        }

        // Staggered updates for others
        let frame = self.current_frame;
        if frame % 4 == 0 {
            for id in &self.buckets[AIUpdateLOD::Reduced] {
                to_update.push((*id, AIUpdateLOD::Reduced));
            }
        }
        if frame % 16 == 0 {
            for id in &self.buckets[AIUpdateLOD::Minimal] {
                to_update.push((*id, AIUpdateLOD::Minimal));
            }
        }
        if frame % 60 == 0 {
            for id in &self.buckets[AIUpdateLOD::Dormant] {
                to_update.push((*id, AIUpdateLOD::Dormant));
            }
        }

        to_update
    }

    pub fn categorize_agent(&mut self, agent_id: AgentId, player_distance: f32) {
        let lod = if player_distance < 50.0 {
            AIUpdateLOD::Full
        } else if player_distance < 150.0 {
            AIUpdateLOD::Reduced
        } else if player_distance < 500.0 {
            AIUpdateLOD::Minimal
        } else {
            AIUpdateLOD::Dormant
        };

        self.assign_to_bucket(agent_id, lod);
    }
}
```

### 7.2 Spatial Partitioning

Efficient neighbor queries:

```rust
pub struct SpatialHash {
    cells: HashMap<CellCoord, Vec<AgentId>>,
    cell_size: f32,
}

impl SpatialHash {
    pub fn query_radius(&self, center: Vec3, radius: f32) -> Vec<AgentId> {
        let min_cell = self.coord_for(center - Vec3::splat(radius));
        let max_cell = self.coord_for(center + Vec3::splat(radius));

        let mut results = Vec::new();

        for x in min_cell.x..=max_cell.x {
            for z in min_cell.z..=max_cell.z {
                if let Some(agents) = self.cells.get(&CellCoord { x, z }) {
                    results.extend(agents.iter().copied());
                }
            }
        }

        results
    }
}
```

### 7.3 Parallel Processing

AI updates are parallelized:

```rust
pub fn update_ai_parallel(agents: &mut [AIAgent], world: &World) {
    // Split agents into independent groups
    // (Agents far apart can be processed in parallel)
    let groups = partition_by_spatial_independence(agents);

    groups.par_iter_mut().for_each(|group| {
        for agent in group {
            // Perception
            let perception = agent.senses.perceive(agent, world);

            // Memory update
            agent.memory.integrate(perception);

            // Goal update
            agent.goals.update(agent, world);

            // Action selection
            if let Some(action) = select_action(agent, world) {
                agent.current_action = Some(action);
            }

            // Execute current action
            if let Some(action) = &mut agent.current_action {
                action.execute(agent, world);
            }
        }
    });
}
```

---

## 8. Benchmarks

### 8.1 Performance Metrics

| Agent Count | Update Time | Memory | FPS Impact |
|-------------|-------------|--------|------------|
| 100 | 0.8ms | 12 MB | None |
| 500 | 2.1ms | 55 MB | None |
| 1,000 | 4.5ms | 108 MB | Minimal |
| 5,000 | 15ms | 520 MB | Moderate |
| 10,000 | 28ms | 1.0 GB | Significant |

*With LOD system enabled:*

| Agent Count | Full LOD | Update Time | FPS Impact |
|-------------|----------|-------------|------------|
| 1,000 | 50 | 1.2ms | None |
| 5,000 | 100 | 3.5ms | None |
| 10,000 | 150 | 6.2ms | Minimal |

---

## 9. Conclusion

The Roanoke AI system demonstrates that believable, emergent behaviors arise from well-designed utility systems and hierarchical goals. By combining biological inspiration with computational efficiency, we create worlds that feel alive—where players witness predators hunting, herds migrating, and ecosystems evolving.

The modular architecture enables easy extension for new species, behaviors, and environmental factors, supporting Roanoke's vision of infinite, living worlds.

---

*© 2025 Roanoke Interactive, Inc. | Technical Whitepaper WP-003*
