# NPC System

## Overview

The NPC (Non-Player Character) system provides interactive Native American village inhabitants for the Roanoke Engine. NPCs have daily schedules, dialogue trees, trading inventories, and relationship tracking with memory systems.

## Architecture

```
npc/
├── mod.rs              # Module root and public exports
├── npc_manager.rs      # NpcManager - central orchestration, spawning
├── dialogue.rs         # Branching dialogue trees with conditions/effects
├── relationships.rs    # Relationship tracking, memory, dispositions
└── trading.rs          # Trade inventories, bartering, faction requirements
```

## Related Systems

```
croatoan_procgen/src/
├── npc.rs              # NPC appearance & mesh generation
├── village.rs          # Village layout, fire pits, corn fields
└── longhouse.rs        # Longhouse mesh generation

croatoan_wfc/src/
└── villages.rs         # World integration, site selection, streaming
```

## Quick Start

The NPC system is initialized in `main.rs`:

```rust
// In SharedState initialization
npc_manager: NpcManager::new(),
```

NPCs are spawned when villages are discovered:

```rust
// Village discovery spawns initial NPCs
npc_manager.initialize_village_npcs(&village_layout);

// Each frame
npc_manager.update(dt, time_of_day, player_pos);
```

## Initial NPCs

| Name | Role | Daily Schedule |
|------|------|----------------|
| Tawenho | Elder | Teach, Pray, Socialize |
| Askook | Warrior | Patrol, Train, Work |
| Kanehti | Shaman | Pray, Gather, Trade |
| Onatah | Farmer | Tend crops, Work |

## NPC Roles

| Role | Description | Typical Activities |
|------|-------------|-------------------|
| Chief | Village leader | Council, ceremonies, diplomacy |
| Shaman | Spiritual leader | Prayers, healing, rituals |
| Warrior | Village defender | Patrol, training, hunting |
| Hunter | Food provider | Hunting, tracking, skinning |
| Farmer | Crop tender | Planting, tending, harvesting |
| Craftsperson | Artisan | Crafting, trading goods |
| Elder | Knowledge keeper | Teaching, storytelling |
| Child | Young villager | Learning, playing |
| Villager | General resident | Various tasks |

## Relationship System

NPCs track relationships with the player through levels:

```
Stranger → Acquaintance → Friend → CloseFriend → Romantic
    ↓
  Enemy ← Hostile
```

### Memory System

NPCs remember interactions:
- Gifts received
- Conversations had
- Help provided
- Offenses committed

## Dialogue System

Branching dialogue trees with:
- **Conditions**: Faction reputation, relationship level, items owned
- **Effects**: Reputation changes, item transfers, quest triggers
- **Topics**: Greeting, Trade, Quest, Lore, Farewell

```rust
// Example dialogue check
if player.faction_reputation(Faction::Powhatan) >= 50 {
    show_dialogue_option("Ask about sacred grounds");
}
```

## Trading System

NPCs have individual trade inventories:

| Trade Type | Requirements |
|------------|--------------|
| Basic goods | Any reputation |
| Rare items | Friend+ relationship |
| Faction goods | Faction reputation 50+ |
| Sacred items | CloseFriend + reputation 75+ |

## Daily Schedule

NPCs follow hourly schedules based on role:

| Hour | Elder | Warrior | Farmer |
|------|-------|---------|--------|
| 6-8 | Wake, Eat | Wake, Train | Wake, Fields |
| 8-12 | Teach | Patrol | Tend crops |
| 12-14 | Rest, Eat | Rest, Eat | Rest, Eat |
| 14-18 | Council | Train | Harvest |
| 18-20 | Pray | Socialize | Socialize |
| 20-22 | Stories | Guard | Rest |

## API Reference

### NpcManager

```rust
impl NpcManager {
    pub fn new() -> Self;
    pub fn update(&mut self, dt: f32, time: f32, player_pos: Vec3);
    pub fn get_nearby_npcs(&self, pos: Vec3, radius: f32) -> Vec<&Npc>;
    pub fn start_dialogue(&mut self, npc_id: NpcId) -> Option<&DialogueTree>;
    pub fn get_trade_inventory(&self, npc_id: NpcId) -> Option<&TradeInventory>;
}
```

### Relationships

```rust
impl RelationshipManager {
    pub fn get_disposition(&self, npc_id: NpcId) -> Disposition;
    pub fn add_memory(&mut self, npc_id: NpcId, memory: Memory);
    pub fn improve_relationship(&mut self, npc_id: NpcId, amount: f32);
}
```

## Specification

For complete system design, see:
- `docs/specs/NPC_VILLAGE_SPECIFICATION.md` - Full specification
- `docs/specs/FACTION_SYSTEM_SPEC.md` - Faction reputation system

## Implementation Status

| Component | Status |
|-----------|--------|
| NPC Manager | Complete |
| Dialogue Trees | Complete |
| Trading System | Complete |
| Relationships | Complete |
| Memory System | Complete |
| Daily Schedules | Complete |
| Behavior AI | Planned (Phase 2) |
| Animations | Planned (Phase 4) |

---

*Last updated: 2024-12-06*
