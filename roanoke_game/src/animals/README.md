# Animal System

## Overview

The animal system provides dangerous wildlife encounters in Roanoke Engine. Animals spawn procedurally based on terrain, time of day, and habitat. They exhibit emergent behavior through a hierarchical finite state machine (HFSM) AI system.

## Architecture

```
animals/
├── mod.rs          # Module root and public exports
├── types.rs        # Species definitions, stats, attacks, status effects
├── entity.rs       # Runtime Animal struct and related types
├── spatial.rs      # Spatial hashing for efficient proximity queries
├── manager.rs      # AnimalManager - central orchestration
├── behavior.rs     # AI state machine and behavior logic
├── spawner.rs      # Chunk-based procedural spawning
└── combat.rs       # Damage processing (partially implemented)
```

## Quick Start

The animal system is automatically initialized in `main.rs`:

```rust
// In SharedState initialization
animal_manager: AnimalManager::new(Difficulty::Normal),
animal_spawner: AnimalSpawner::new(seed),
```

Animals spawn when chunks load and update each frame:

```rust
// On chunk load
animal_spawner.on_chunk_loaded(chunk_x, chunk_z, chunk_size, &mut animal_manager, player_pos, seed);

// Each frame
animal_manager.update(dt, player_pos, player_velocity);
```

## Species

| Species | Health | Speed | Danger | Behavior | Habitat |
|---------|--------|-------|--------|----------|---------|
| Black Bear | 150 | 35 | 7 | Territorial | Forest, Mountain, Swamp |
| Eastern Cougar | 100 | 50 | 8 | Stalker | Forest, Mountain, Rocky |
| Gray Wolf | 80 | 45 | 6 | Pack Hunter | Forest, Plain, Mountain |
| Timber Rattlesnake | 30 | 15 | 5 | Ambush | Forest, Rocky, Meadow |
| American Alligator | 200 | 20/35* | 9 | Ambush | Swamp, River, Marsh |
| Wild Boar | 90 | 30 | 4 | Aggressive | Forest, Swamp, Field |
| Copperhead | 25 | 12 | 3 | Hidden | Forest, Rocky, Near Water |
| Red Wolf | 70 | 42 | 5 | Pack Hunter | Forest, Swamp, Coast |
| Bobcat | 60 | 40 | 3 | Stalker | Forest, Swamp, Rocky |
| Cottonmouth | 35 | 10/18* | 4 | Aggressive | Swamp, River, Marsh |

*Water speed when applicable

## Behavior States

Animals use a hierarchical finite state machine:

```
┌─────────┐     awareness > 0.3     ┌─────────┐
│  Idle   │ ──────────────────────► │  Alert  │
└─────────┘                         └─────────┘
     │                                   │
     │ random                            │ awareness > 0.9
     ▼                                   │ && should_attack
┌─────────┐                              ▼
│ Patrol  │                         ┌─────────┐
└─────────┘                         │ Pursue  │
                                    └─────────┘
                                         │
                                         │ in attack range
                                         ▼
┌─────────┐     health < flee_hp    ┌─────────┐
│  Flee   │ ◄────────────────────── │ Attack  │
└─────────┘                         └─────────┘
```

### Behavior Types

- **Territorial** (Bear, Boar): Defends home area, warns before attacking
- **Stalker** (Cougar, Bobcat): Follows prey from detection edge, pounces when back is turned
- **Pack Hunter** (Wolves): Coordinates with pack members, flanking tactics
- **Ambush** (Snakes, Alligator): Waits hidden, strikes when close
- **Aggressive** (Boar, Cottonmouth): Attacks readily on detection
- **Hidden** (Copperhead): Camouflaged, defensive strikes only

## Spawning

Animals spawn procedurally when terrain chunks load:

1. **Habitat Detection**: Terrain height and moisture determine habitats
2. **Species Selection**: Filter by habitat, time of day, spawn rate
3. **Position Generation**: Noise-based placement within chunk
4. **Pack Spawning**: Wolves spawn in groups of 2-6

### Spawn Configuration

```rust
pub struct AnimalSpawner {
    max_animals: usize,        // Global cap (default: 50)
    min_spawn_distance: f32,   // From player (default: 40m)
    animals_per_chunk: f32,    // Density (default: 0.5)
}
```

### Time of Day

Animals have active periods:
- **Dawn** (5:00-8:00): Bears, Boars, Cougars, Wolves, Bobcats
- **Day** (8:00-17:00): Rattlesnakes
- **Dusk** (17:00-20:00): Bears, Boars, Rattlesnakes, Wolves, Copperheads, Bobcats
- **Night** (20:00-5:00): Bears, Cougars, Wolves, Copperheads, Bobcats
- **Any**: Alligators, Cottonmouths

## Spatial Queries

The `SpatialHash<AnimalId>` provides O(1) proximity queries:

```rust
// Find animals within 50 meters
let nearby = manager.query_radius(player_pos, 50.0);

// Find animals in a chunk
let in_chunk = manager.query_chunk(chunk_x, chunk_z, chunk_size);
```

Cell size is 16 units for optimal cache coherence.

## Attacks & Effects

Each species has unique attacks:

```rust
// Example: Black Bear
AttackDef { name: "claw_swipe", damage: 25.0, cooldown: 2.0, effect: Some(Bleeding) }
AttackDef { name: "bite", damage: 30.0, cooldown: 3.0, effect: None }
```

### Status Effects

| Effect | Duration | DPS | Speed Mod | Description |
|--------|----------|-----|-----------|-------------|
| Bleeding | 10s | 2 | 1.0 | Damage over time |
| Poison | 15s | 3 | 0.8 | DoT + slow |
| Knockdown | 2s | 0 | 0.0 | Prone, immobile |
| Knockback | 0.5s | 0 | 0.0 | Pushed back |
| Stun | 3s | 0 | 0.0 | Cannot act |
| Fear | 2s | 0 | 1.2 | Erratic movement |
| Slow | 5s | 0 | 0.5 | Half speed |

## Difficulty Scaling

| Difficulty | Health | Damage | Spawn Rate |
|------------|--------|--------|------------|
| Easy | 0.75x | 0.75x | 0.8x |
| Normal | 1.0x | 1.0x | 1.0x |
| Hard | 1.5x | 1.25x | 1.3x |
| Survival | 2.0x | 1.5x | 1.5x |

## Debug Information

In-game debug info is shown in Dev Stats:

```
Animals: 12 | Packs: 2 | Spawned: 45 | Killed: 3
Nearby (50m): 2
  Gray Wolf @ 23m - Pursue(Chasing)
  Gray Wolf @ 31m - Pursue(Circling)
```

## Implementation Status

### Complete
- [x] Species definitions with stats
- [x] Entity management (spawn, despawn, update)
- [x] Spatial hashing for queries
- [x] HFSM behavior system
- [x] Chunk-based spawning
- [x] Habitat/time filtering
- [x] Pack spawning for wolves
- [x] Stalker behavior (cougar)
- [x] Awareness/alertness system
- [x] Debug UI integration

### Planned
- [ ] Combat damage integration
- [ ] Visual rendering (mesh instances)
- [ ] Audio cues (growls, howls)
- [ ] Loot drops on death
- [ ] Save/load persistence
- [ ] NavMesh pathfinding

## Performance

- **Target**: 50+ active animals at 60 FPS
- **Spatial Hash**: O(1) insert/remove, O(n) radius query where n = nearby
- **Update**: Only updates animals in loaded chunks
- **Despawn**: Animals despawn 30s after chunk unloads

## API Reference

### AnimalManager

```rust
impl AnimalManager {
    fn new(difficulty: Difficulty) -> Self;
    fn spawn(species, position, chunk, pack_id) -> AnimalId;
    fn despawn(id: AnimalId);
    fn update(dt, player_pos, player_velocity);
    fn query_radius(center, radius) -> Vec<AnimalId>;
    fn animals_near(center, radius) -> Vec<&Animal>;
    fn damage_animal(id, damage) -> bool; // returns true if killed
    fn debug_info() -> String;
}
```

### AnimalSpawner

```rust
impl AnimalSpawner {
    fn new(seed: u32) -> Self;
    fn on_chunk_loaded(chunk_x, chunk_z, chunk_size, manager, player_pos, seed);
    fn on_chunk_unloaded(chunk_x, chunk_z, chunk_size, manager);
}
```

### Animal Entity

```rust
impl Animal {
    fn is_alive() -> bool;
    fn is_dead() -> bool;
    fn current_speed() -> f32;
    fn look_at(target: Vec3);
    fn take_damage(amount, source);
    fn apply_effect(effect_type, source);
    fn select_attack(distance) -> Option<usize>;
}
```
