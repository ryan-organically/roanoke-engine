# Stub and Placeholder Implementations

This document catalogs all stub, placeholder, and TODO implementations in `roanoke_game/src/`.

Generated: 2026-01-22

---

## Table of Contents

1. [Animals Module](#animals-module)
2. [Network Module](#network-module)
3. [Progression Module](#progression-module)
4. [Economy Module](#economy-module)
5. [UI Module](#ui-module)
6. [Water System](#water-system)
7. [Main/Rendering](#mainrendering)
8. [Flora Module](#flora-module)

---

## Animals Module

### Missing Animal Models

**File:** `/mnt/c/dev/roanoke engine/roanoke_game/src/animals/types.rs`
**Line:** 65-74
**Function:** `AnimalSpecies::model_name()`

**What it's supposed to do:** Return the 3D model name for each animal species.

**Current placeholder behavior:**
- Bobcat uses Fox model as placeholder (`Some("Fox")`)
- Several species return `None` (no model):
  - BlackBear
  - EasternCougar
  - TimberRattlesnake
  - AmericanAlligator
  - WildBoar
  - Copperhead
  - Cottonmouth

**Priority:** **High** - Missing models mean these animals render as orbs instead of 3D models.

---

### Corner Detection Not Implemented

**File:** `/mnt/c/dev/roanoke engine/roanoke_game/src/animals/behavior.rs`
**Line:** 388-391
**Function:** `is_cornered()`

**What it's supposed to do:** Detect when an animal is cornered by terrain/obstacles during flee behavior, triggering defensive behavior.

**Current placeholder behavior:** Always returns `false`.

```rust
fn is_cornered(_animal: &Animal, _player_pos: Vec3) -> bool {
    // TODO: Implement proper corner detection using terrain
    false
}
```

**Priority:** **Medium** - Affects animal AI realism; animals should turn and fight when cornered.

---

### Pack Hunter Coordination Simplified

**File:** `/mnt/c/dev/roanoke engine/roanoke_game/src/animals/behavior.rs`
**Line:** 279-283
**Function:** Pack hunter behavior in `transition_to_hostile()`

**What it's supposed to do:** Implement flanking and pack coordination for wolves and other pack hunters.

**Current placeholder behavior:** Just chases directly like solo hunters. Comment indicates "pack coordination in pack module."

**Priority:** **Medium** - Pack animals should exhibit more sophisticated group tactics.

---

### Simplified Line of Sight

**File:** `/mnt/c/dev/roanoke engine/roanoke_game/src/animals/behavior.rs`
**Line:** 863-865
**Function:** `has_line_of_sight()`

**What it's supposed to do:** Check if an animal has visual line of sight to its target, considering terrain and obstacles.

**Current placeholder behavior:** Only checks distance and height difference, no actual raycast or terrain occlusion.

**Priority:** **Low** - Current simplified version works adequately for most gameplay scenarios.

---

### Terrain Height Query Placeholder

**File:** `/mnt/c/dev/roanoke engine/roanoke_game/src/animals/spawner.rs`
**Line:** 239, 282-289
**Function:** `get_terrain_height()`

**What it's supposed to do:** Query actual terrain height for animal spawn placement.

**Current placeholder behavior:** Calls into `croatoan_wfc::mesh_gen::get_height_at` which is now implemented, but comments indicate this was originally a placeholder.

**Priority:** **Low** - Implementation exists, comments may be outdated.

---

## Network Module

### Remote Player Rendering Placeholder

**File:** `/mnt/c/dev/roanoke engine/roanoke_game/src/network/remote_renderer.rs`
**Lines:** 1-122 (entire file)
**Structs:** `PlayerCapsule`, `RemotePlayerOrb`, `PlayerNametag`, `RemotePlayerBatch`

**What it's supposed to do:** Render other players with proper character models in multiplayer.

**Current placeholder behavior:** Renders remote players as colored capsules/orbs with nametags above their heads.

```rust
//! Remote Player Rendering
//!
//! Placeholder for rendering other players in the world.
//! Currently renders as colored capsules/orbs.
```

**Priority:** **High** - Critical for multiplayer visual fidelity.

---

### Server Chat Broadcast Not Implemented

**File:** `/mnt/c/dev/roanoke engine/roanoke_game/src/network/manager.rs`
**Line:** 418
**Function:** `send_chat()`

**What it's supposed to do:** Broadcast chat messages to all connected players when hosting a server.

**Current placeholder behavior:** Only sends chat as client; server-side broadcast is TODO.

```rust
// TODO: Server chat broadcast
```

**Priority:** **Medium** - Needed for functional multiplayer chat.

---

## Progression Module

### Modified Relationships Not Tracked

**File:** `/mnt/c/dev/roanoke engine/roanoke_game/src/progression/faction_integration.rs`
**Line:** 844
**Function:** `FactionSaveData::capture()`

**What it's supposed to do:** Track and save relationships between factions that have been modified by player actions.

**Current placeholder behavior:** Returns empty vector.

```rust
modified_relationships: Vec::new(), // TODO: Track modified relationships
```

**Priority:** **Medium** - Affects save/load persistence of faction dynamics.

---

### Stub Faction Skill Trees

**File:** `/mnt/c/dev/roanoke engine/roanoke_game/src/progression/faction_skills.rs`
**Line:** 1376+
**Functions:** `get_tuscarora_skills()` and remaining faction skill functions

**What it's supposed to do:** Define complete skill trees for all factions.

**Current placeholder behavior:** Comment indicates these are "stub implementations following same pattern." However, the Tuscarora implementation appears fairly complete.

```rust
// Stub implementations for remaining factions - following same pattern
```

**Priority:** **Low** - Tuscarora skills appear implemented; verify other faction completeness.

---

## Economy Module

### Currency Drop Placeholder Item

**File:** `/mnt/c/dev/roanoke engine/roanoke_game/src/economy/drops.rs`
**Line:** 67-82
**Function:** `DroppedItem::new_currency()`

**What it's supposed to do:** Create a proper visual representation for dropped currency.

**Current placeholder behavior:** Creates a generic "wampum_pouch" or "tobacco_bundle" item with basic properties.

```rust
// Create a placeholder item for currency using the proper constructor
```

**Priority:** **Low** - Functional but could use dedicated currency visuals/icons.

---

## UI Module

### Perks Journal Placeholder Entries

**File:** `/mnt/c/dev/roanoke engine/roanoke_game/src/ui/perks_journal.rs`
**Line:** 1035-1043
**Function:** Perks tab rendering

**What it's supposed to do:** Display actual player perks from the progression system.

**Current placeholder behavior:** Shows hardcoded static perk entries:

```rust
// Placeholder entries
let perks = [
    ("Tier I Perk", "Basic ability", true),
    ("Tier I Perk B", "Another starter", true),
    ("Tier II Perk", "Intermediate", false),
    ("Tier III Perk", "Advanced", false),
    ("Tier IV Perk", "Expert", false),
    ("Tier V Perk", "Legendary", false),
];
```

**Priority:** **High** - Players see fake perks instead of their actual progression.

---

### Settings Menu Not Implemented

**File:** `/mnt/c/dev/roanoke engine/roanoke_game/src/main.rs`
**Line:** 5092
**Function:** Main menu

**What it's supposed to do:** Provide game settings (graphics, audio, controls, etc.).

**Current placeholder behavior:** Menu item exists but is disabled.

```rust
("Settings", false), // Not implemented
```

**Priority:** **High** - Essential UX feature for any game.

---

### NPC Interaction Icon Placeholder

**File:** `/mnt/c/dev/roanoke engine/roanoke_game/src/main.rs`
**Line:** 5611
**Function:** NPC interaction prompt HUD

**What it's supposed to do:** Display contextual icon for NPC interaction type.

**Current placeholder behavior:** Uses diamond unicode character as placeholder icon.

```rust
// Icon placeholder (diamond shape)
ui.label(egui::RichText::new("◇")
```

**Priority:** **Low** - Functional but could use proper contextual icons.

---

### Health Bar Not Wired to Player Health

**File:** `/mnt/c/dev/roanoke engine/roanoke_game/src/main.rs`
**Line:** 5495-5496
**Function:** HUD health display

**What it's supposed to do:** Display actual player health from progression system.

**Current placeholder behavior:** Shows hardcoded 100/100 health.

```rust
// Health bar (placeholder - use progression's player_health when available)
let health = 100.0_f32; // TODO: Wire to actual player health
```

**Priority:** **High** - Player can't see their actual health status.

---

### Character Preview Placeholder

**File:** `/mnt/c/dev/roanoke engine/roanoke_game/src/main.rs`
**Line:** 955-974
**Function:** Character page in journal

**What it's supposed to do:** Render 3D character model preview.

**Current placeholder behavior:** Shows a dark rectangle with "3D Model" text and rotation indicator.

```rust
// Character silhouette placeholder (will be replaced with 3D model)
```

**Priority:** **Medium** - Visual polish for character customization screen.

---

## Water System

### Shore Distance Texture Placeholder

**File:** `/mnt/c/dev/roanoke engine/roanoke_game/src/water_system.rs`
**Line:** 235-279
**Function:** Shore distance texture initialization

**What it's supposed to do:** Use actual terrain data to calculate shore distances for realistic wave foam and water depth effects.

**Current placeholder behavior:** Generates a simple gradient pattern assuming shore at edges, deep water toward center.

```rust
// Initialize shore distance with gradient (placeholder - real data from terrain)
```

**Priority:** **Medium** - Affects water rendering quality near shores.

---

### Butterfly Texture Placeholder

**File:** `/mnt/c/dev/roanoke engine/roanoke_game/src/water_system.rs`
**Line:** 281-291
**Function:** Butterfly texture for water effects

**What it's supposed to do:** Provide butterfly displacement texture for FFT water simulation.

**Current placeholder behavior:** Creates empty texture with no data initialization.

```rust
// Butterfly Texture (Placeholder)
let butterfly_texture = device.create_texture(&wgpu::TextureDescriptor {
    ...
});
```

**Priority:** **Low** - Water rendering works without this advanced feature.

---

### River Water Not Implemented

**File:** `/mnt/c/dev/roanoke engine/roanoke_game/src/main.rs`
**Line:** 7867
**Function:** Chunk loading

**What it's supposed to do:** Render river water with proper flow and current effects.

**Current placeholder behavior:** Returns empty vector.

```rust
river_water: Vec::new(), // TODO: implement river water
```

**Priority:** **Medium** - Rivers exist in world generation but have no water.

---

## Main/Rendering

### Moon Phase Hardcoded

**File:** `/mnt/c/dev/roanoke engine/roanoke_game/src/main.rs`
**Line:** 8283
**Function:** Moon rendering

**What it's supposed to do:** Calculate moon phase based on in-game time progression (29.5 day cycle).

**Current placeholder behavior:** Always renders full moon (0.5).

```rust
let moon_phase = 0.5; // Full moon for now (TODO: calculate from game days)
```

**Priority:** **Low** - Visual polish; doesn't affect gameplay.

---

### Disabled Single-LOD Placeholders

**File:** `/mnt/c/dev/roanoke engine/roanoke_game/src/main.rs`
**Lines:** 2371, 2760
**Function:** Tree and shrub loading

**What it's supposed to do:** Support full LOD chains for all vegetation.

**Current status:** Several models disabled as they were single-LOD placeholders:
- `tree_0`, `tree_1` - disabled single-LOD trees
- `shrub_0`, `bush_0`, `grass_0` - disabled single-LOD shrubs
- `beach_grass_0` - replaced by grass3 LOD system

```rust
// NOTE: tree_0, tree_1 are single-LOD placeholders - DISABLED
// NOTE: shrub_0, bush_0, grass_0, beach_grass_0 disabled - using grass3 LOD system
```

**Priority:** **Low** - Replaced with proper multi-LOD models.

---

### Fallback Texture for Terrain

**File:** `/mnt/c/dev/roanoke engine/roanoke_game/src/main.rs`
**Line:** 3634-3636
**Function:** Terrain texture loading

**What it's supposed to do:** Load proper terrain textures from assets.

**Current fallback behavior:** Creates procedural fallback texture if loading fails.

```rust
println!("[GPU] Using fallback placeholder texture");
let fallback = TerrainTextures::create_fallback(ctx.device(), ctx.queue());
```

**Priority:** **Low** - Error handling path; not normally triggered.

---

### Village System Disabled

**File:** `/mnt/c/dev/roanoke engine/roanoke_game/src/main.rs`
**Lines:** 5160, 5169, 5209, 5221, 5437
**Function:** Various save/load and game state operations

**What it's supposed to do:** Integrate village NPCs and buildings into save/load system.

**Current placeholder behavior:** Village system integration disabled in multiple places.

```rust
// Village system disabled for now
// Village animals disabled for now
```

**Priority:** **Medium** - Villages exist but state doesn't persist properly.

---

## Flora Module

### Medicinal Effects Stub

**File:** `/mnt/c/dev/roanoke engine/roanoke_game/src/flora/medicinal.rs`
**Line:** 583-607
**Function:** `get_primary_effect()`

**What it's supposed to do:** Return medicinal properties for plant species, potentially with more complex effects.

**Current status:** Marked as "stub implementation" but appears fairly complete with 19 species mapped to effects. May need expansion for additional species.

```rust
/// Get primary medicinal effect for a species (stub implementation)
fn get_primary_effect(&self, species: FloraSpecies) -> Option<PlantEffect> {
```

**Priority:** **Low** - Implementation appears functional; comment may be outdated.

---

## Summary by Priority

### High Priority
1. **Missing Animal Models** - 7 species render as orbs
2. **Remote Player Rendering** - Multiplayer shows capsules
3. **Perks Journal** - Shows fake perks
4. **Settings Menu** - Not implemented
5. **Health Bar** - Hardcoded, not connected to player state

### Medium Priority
1. **Corner Detection** - Affects animal AI realism
2. **Pack Hunter Coordination** - Wolves don't flank
3. **Server Chat Broadcast** - Multiplayer chat incomplete
4. **Modified Relationships** - Save/load incomplete
5. **Shore Distance Texture** - Water quality
6. **River Water** - No water in rivers
7. **Village System** - State doesn't persist
8. **Character Preview** - Shows placeholder rectangle

### Low Priority
1. **Line of Sight** - Simplified but functional
2. **Terrain Height Query** - May be outdated comment
3. **Currency Drop Visuals** - Functional
4. **NPC Interaction Icon** - Unicode placeholder works
5. **Butterfly Texture** - Advanced feature
6. **Moon Phase** - Visual polish only
7. **Disabled LOD Models** - Replaced with better versions
8. **Fallback Texture** - Error handling
9. **Faction Skills** - May be complete, verify
10. **Medicinal Effects** - Comment may be outdated
