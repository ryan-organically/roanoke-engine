# Session Log: Agent Intelligence System Implementation
**Date:** 2026-01-22
**Branch:** render-fix

## Overview

This session implemented the foundational agent intelligence systems for NPCs and animals, establishing a unified character agent framework and introducing utility-based AI decision making with gossip propagation.

---

## Phase 0: CharacterAgent Trait Unification

### Goal
Unify NPC and Animal systems under a common `CharacterAgent` trait to enable cross-system coordination, shared awareness, and consistent behavior interfaces.

### Changes

#### 1. NpcInstance CharacterAgent Implementation
**File:** `roanoke_game/src/npc/npc_manager.rs`

- Added new fields to `NpcInstance`:
  ```rust
  pub awareness: f32,           // 0.0 to 1.0 - unified awareness
  pub emotional_state: EmotionalState,  // For visual representation
  ```

- Implemented `CharacterAgent` trait (13 methods):
  - `agent_id()` - Returns `AgentId::npc(self.id)`
  - `position()`, `set_position()` - Position accessors
  - `velocity()`, `set_velocity()` - Velocity accessors
  - `base_speed()` - Role-based speeds (Warriors: 5.0, Elders: 2.5, etc.)
  - `awareness()`, `set_awareness()` - Syncs with legacy `alertness`
  - `behavior_state()`, `set_behavior_state()` - Via conversion methods
  - `emotional_state()`, `set_emotional_state()` - Direct field access
  - `detection_radius()` - Role-based (Hunters: 40, Shamans: 30, others: 20)
  - `is_alive()` - Health > 0 check
  - `look_direction()`, `look_at()` - Facing direction
  - `update()` - Awareness decay and player detection
  - `orb_scale()` - Role-based (Elders: 1.3, Children: 0.7)

- Added behavior state conversion:
  ```rust
  impl NpcBehaviorState {
      pub fn to_unified(&self) -> UnifiedBehaviorState
      pub fn from_unified(unified: UnifiedBehaviorState) -> Self
  }
  ```

- Added `update_emotional_state()` method for context-aware emotion derivation

#### 2. Animal CharacterAgent Implementation
**File:** `roanoke_game/src/animals/entity.rs`

- Implemented `CharacterAgent` trait for `Animal`:
  - Uses `species.base_stats()` for speed and detection range
  - Emotional state derived from behavior (fearful when fleeing, hostile when attacking)
  - `set_emotional_state()` is no-op (animals derive emotion from behavior)

- Added behavior state conversion:
  ```rust
  impl BehaviorState {
      pub fn to_unified(&self) -> UnifiedBehaviorState
      pub fn from_unified(unified: UnifiedBehaviorState) -> Self
      pub fn to_emotional_state(&self) -> UnifiedEmotionalState
  }
  ```

#### 3. AgentCollection Trait Implementation
**File:** `roanoke_game/src/character_agent/unified_manager.rs`

- Implemented `AgentCollection` for `CombinedAgentAdapter`:
  ```rust
  impl<'a> AgentCollection for CombinedAgentAdapter<'a> {
      fn iter_ids(&self) -> Box<dyn Iterator<Item = AgentId> + '_>
      fn get_agent(&self, id: AgentId) -> Option<&dyn CharacterAgent>
      fn get_agent_mut(&mut self, id: AgentId) -> Option<&mut dyn CharacterAgent>
  }
  ```

#### 4. AnimalManager Extensions
**File:** `roanoke_game/src/animals/manager.rs`

- Added ID-based iterators:
  ```rust
  pub fn animals_with_ids(&self) -> impl Iterator<Item = (AnimalId, &Animal)>
  pub fn animals_with_ids_mut(&mut self) -> impl Iterator<Item = (AnimalId, &mut Animal)>
  ```

#### 5. Main Loop Integration
**File:** `roanoke_game/src/main.rs`

- Added `NpcManager` to `SharedState`
- Wired `UnifiedAgentManager.update()` into game loop:
  ```rust
  let mut adapter = CombinedAgentAdapter {
      npcs: &mut state.npc_manager,
      animals: &mut state.animal_manager,
  };
  unified_agents.update(delta, player_pos, player_vel, game_time, &mut adapter);
  ```

#### 6. Save/Load Persistence
**File:** `roanoke_game/src/main.rs`

- Extended `SaveData` with NPC relationships:
  ```rust
  #[serde(default)]
  npc_relationships: Option<npc::relationships::RelationshipManager>,
  ```
- Updated all 3 load locations to restore relationships
- Updated all 2 save locations to persist relationships

---

## Phase 1: Local Intelligence Layer

### Goal
Implement smart NPC decision-making without expensive API calls using utility AI scoring and gossip propagation.

### Changes

#### 1. Utility AI System
**File:** `roanoke_game/src/npc/utility_ai.rs` (NEW - 650 lines)

**Core Components:**

- **UtilityContext**: Gathered once per decision cycle
  - Spatial: player_distance, target_distance, home_distance
  - Temporal: current_hour, is_night
  - State: health_percent, alertness, mood
  - Relationship: player_affinity, player_trust, player_fear
  - Schedule: scheduled_activity, at_activity_location
  - Events: recently_attacked, heard_rumors

- **Considerations** (18 types):
  ```rust
  pub enum Consideration {
      PlayerNearby, PlayerFar, AtTargetLocation, AwayFromHome,
      PlayerIsFriend, PlayerIsEnemy, PlayerIsFeared, PlayerIsTrusted,
      IsAlert, IsCalm, HealthLow, MoodPositive, MoodNegative,
      ShouldBeWorking, ShouldBeResting, ShouldBeTrading, ShouldBeSocializing,
      IsDaytime, IsNighttime, WasRecentlyAttacked, HasHeardRumors
  }
  ```

- **Response Curves**:
  ```rust
  pub enum ResponseCurve {
      Linear,           // y = x
      Quadratic,        // y = x^2
      InverseQuadratic, // y = 1 - (1-x)^2
      Sigmoid,          // S-curve
      Threshold(f32),   // Binary cutoff
  }
  ```

- **NPC Actions** (13 types):
  ```rust
  pub enum NpcAction {
      Idle, WalkToTarget, WorkAtLocation,
      GreetPlayer, ApproachPlayer, TradeWithPlayer,
      FleeFromPlayer, AttackPlayer,
      BecomeAlert, Investigate, ReturnHome,
      Socialize, ShareGossip
  }
  ```

- **Action Profiles**: Pre-configured scoring profiles with:
  - Base score
  - Weighted considerations with curves
  - Role restrictions (e.g., only Warriors/Hunters can Attack)
  - Minimum score thresholds

- **UtilityEvaluator**:
  ```rust
  pub fn select_action(&self, ctx: &UtilityContext, role: NpcRole) -> (NpcAction, f32)
  pub fn evaluate_all(&self, ctx: &UtilityContext, role: NpcRole) -> Vec<(NpcAction, f32)>
  ```

**Integration:**
- Added `utility_evaluator` and `use_utility_ai` to `NpcManager`
- Added `apply_utility_action()` to `NpcInstance` for action execution
- NPC update loop now uses utility AI when enabled

#### 2. Gossip Propagation System
**File:** `roanoke_game/src/npc/relationships.rs`

**New Components:**

- **Rumor struct**:
  ```rust
  pub struct Rumor {
      pub subject: String,      // Who/what it's about
      pub description: String,  // What happened
      pub impact: i32,          // -100 to 100
      pub credibility: f32,     // 0.0 to 1.0
      pub age: f32,             // Hours since event
  }
  ```

- **RelationshipManager extensions**:
  ```rust
  pub fn spread_rumor(&mut self, speaker: u32, listener: u32, rumor: &Rumor, time: f64) -> bool
  pub fn propagate_gossip(&mut self, npc_pairs: &[(u32, u32)], time: f64)
  pub fn rumor_awareness(&self, subject: &str) -> usize
  pub fn has_heard_rumors(&self, npc_id: u32) -> bool
  ```

**Gossip Mechanics:**
- Rumors spread during socializing hours
- Impact decays through transmission (60% of original)
- Credibility affects final impact
- NPCs don't re-spread rumors they've already heard
- `HeardRumor` memory type now actively populated

**Integration:**
- Added `collect_socializing_pairs()` to find NPCs within talking distance
- `propagate_gossip()` called each update cycle when pairs exist

---

## Files Modified

| File | Changes |
|------|---------|
| `roanoke_game/src/npc/mod.rs` | Added `utility_ai` module export |
| `roanoke_game/src/npc/npc_manager.rs` | CharacterAgent impl, utility AI integration, gossip wiring |
| `roanoke_game/src/npc/relationships.rs` | Rumor struct, gossip propagation methods |
| `roanoke_game/src/npc/utility_ai.rs` | **NEW** - Complete utility AI system |
| `roanoke_game/src/animals/entity.rs` | CharacterAgent impl, behavior state conversion |
| `roanoke_game/src/animals/manager.rs` | Added ID-based iterators |
| `roanoke_game/src/character_agent/unified_manager.rs` | AgentCollection impl for CombinedAgentAdapter |
| `roanoke_game/src/main.rs` | NpcManager field, unified agent wiring, save/load |

---

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                     UnifiedAgentManager                          │
│  - Coordinates NPCs and Animals                                  │
│  - Spatial awareness queries                                     │
│  - Communication links                                           │
└──────────────────────┬──────────────────────────────────────────┘
                       │
        ┌──────────────┴──────────────┐
        │                             │
        ▼                             ▼
┌───────────────────┐       ┌───────────────────┐
│    NpcManager     │       │   AnimalManager   │
│                   │       │                   │
│ ┌───────────────┐ │       │ ┌───────────────┐ │
│ │UtilityEvaluator│ │       │ │   Animals     │ │
│ └───────────────┘ │       │ └───────────────┘ │
│         │         │       │         │         │
│         ▼         │       │         ▼         │
│ ┌───────────────┐ │       │ ┌───────────────┐ │
│ │  NpcInstance  │◄┼───────┼►│    Animal     │ │
│ │ (CharacterAgent)│       │ │(CharacterAgent)│ │
│ └───────────────┘ │       │ └───────────────┘ │
│         │         │       └───────────────────┘
│         ▼         │
│ ┌───────────────┐ │
│ │Relationships  │ │
│ │ + Gossip      │ │
│ └───────────────┘ │
└───────────────────┘
```

---

## Testing Notes

- All changes compile without errors
- Utility AI enabled by default (`use_utility_ai = true`)
- Gossip spreads during socializing hours (typically 14:00-18:00 for Elder)
- Legacy behavior preserved when `use_utility_ai = false`
- Save/load backwards compatible via `#[serde(default)]`

---

## Future Work (Not Completed)

1. **Behavior Trees**: For complex multi-step action sequences
2. **Template Dialogue**: Variable substitution in dialogue responses
3. **LLM Gateway**: API integration for advanced responses (see `docs/specs/LLM_GATEWAY_NPC_INTELLIGENCE_SPEC.md`)
