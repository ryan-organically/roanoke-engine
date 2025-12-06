# Roanoke Engine - Project Structure Diagrams

## 1. High-Level Architecture

```
+===========================================================================+
|                          ROANOKE GAME ENGINE                               |
+===========================================================================+

                          +-------------------------+
                          |     roanoke_game/       |
                          |      (Main Binary)      |
                          |  main.rs | game_state   |
                          +------------+------------+
                                       |
          +----------------------------+----------------------------+
          |                            |                            |
          v                            v                            v
+-------------------+      +-------------------+      +-------------------+
|   Game Systems    |      |   World Systems   |      |  Player Systems   |
|                   |      |                   |      |                   |
| - animals/        |      | - chunk_manager   |      | - progression/    |
| - npc/            |      | - village_manager |      | - economy/        |
| - audio_system    |      | - water_system    |      | - encyclopedia/   |
| - weather_system  |      | - atmosphere      |      |                   |
+--------+----------+      +--------+----------+      +--------+----------+
         |                          |                          |
         +-------------+------------+-------------+------------+
                       |                          |
                       v                          v
          +========================+   +========================+
          |    Engine Crates       |   |    External Deps       |
          |                        |   |                        |
          | - croatoan_core        |   | - wgpu (graphics)      |
          | - croatoan_render      |   | - winit (windowing)    |
          | - croatoan_wfc         |   | - egui (UI)            |
          | - croatoan_procgen     |   | - kira (audio)         |
          | - croatoan_neural      |   | - glam (math)          |
          +========================+   +========================+
```

## 2. Crates Dependency Graph

```
                    +------------------+
                    |  roanoke_game    |
                    |  (main binary)   |
                    +--------+---------+
                             |
       +---------------------+---------------------+
       |                     |                     |
       v                     v                     v
+--------------+    +-----------------+    +-----------------+
|croatoan_core |    | croatoan_render |    |  croatoan_wfc   |
|              |    |                 |    |                 |
| - App loop   |    | - Pipelines     |    | - Terrain gen   |
| - Events     |    | - Camera        |    | - Biomes (15+)  |
| - Window     |    | - Shadows       |    | - Caves/Rivers  |
+------+-------+    +--------+--------+    +--------+--------+
       |                     |                      |
       |                     v                      |
       |            +------------------+            |
       +----------->| croatoan_procgen |<-----------+
                    |                  |
                    | - Trees/Grass    |
                    | - Buildings      |
                    | - NPCs/Villages  |
                    +------------------+

+-----------------+
|croatoan_neural  |   (Placeholder - AI/ML future)
+-----------------+
```

## 3. Directory Structure

```
roanoke engine/
|
+-- Cargo.toml                    # Workspace root
+-- Cargo.lock
|
+-- crates/                       # Engine modules
|   +-- croatoan_core/            # Core engine loop
|   |   +-- src/lib.rs
|   |
|   +-- croatoan_render/          # Graphics rendering
|   |   +-- src/
|   |       +-- lib.rs
|   |       +-- camera.rs
|   |       +-- shadows.rs
|   |       +-- frustum.rs
|   |       +-- *_pipeline.rs     # 10+ render pipelines
|   |
|   +-- croatoan_wfc/             # Wave Function Collapse
|   |   +-- src/
|   |       +-- lib.rs
|   |       +-- biome.rs          # 15+ biome definitions
|   |       +-- terrain.rs
|   |       +-- caves.rs
|   |       +-- rivers.rs
|   |       +-- villages.rs
|   |
|   +-- croatoan_procgen/         # Procedural generation
|   |   +-- src/
|   |       +-- lib.rs
|   |       +-- tree.rs
|   |       +-- grass.rs
|   |       +-- building.rs
|   |       +-- longhouse.rs
|   |       +-- npc.rs
|   |       +-- village.rs
|   |
|   +-- croatoan_neural/          # AI systems (placeholder)
|
+-- roanoke_game/                 # Main game application
|   +-- Cargo.toml
|   +-- src/
|       +-- main.rs               # Entry point (143KB)
|       +-- game_state.rs
|       +-- asset_loader.rs
|       +-- player.rs
|       +-- chunk_manager.rs
|       +-- village_manager.rs
|       +-- audio_system.rs
|       +-- weather_system.rs
|       +-- water_system.rs
|       +-- atmosphere.rs
|       +-- procedural_synth.rs
|       |
|       +-- animals/              # Fauna system
|       +-- npc/                  # NPC & dialogue
|       +-- progression/          # Skills, factions, quests
|       +-- economy/              # Items & trading
|       +-- encyclopedia/         # Discovery system
|
+-- assets/                       # Game assets
|   +-- shaders/                  # WGSL shaders (13 files)
|   +-- audio/
|   |   +-- ambience/
|   |   +-- music/
|   |   +-- sfx/
|   +-- ui/
|
+-- marketing/                    # Business documentation
|   +-- whitepapers/
|   +-- investor/
|   +-- press/
|
+-- saves/                        # Game save data
+-- trees/                        # Tree data/assets
+-- [30+ spec files]              # Design documentation
```

## 4. Game Systems Module Structure

```
roanoke_game/src/
|
+====================+
|  CORE SYSTEMS      |
+====================+
|
+-- main.rs              Game loop, initialization, input handling
+-- game_state.rs        State management, game modes
+-- asset_loader.rs      Resource loading pipeline
+-- chunk_manager.rs     World streaming, LOD management
|
+====================+
|  WORLD SYSTEMS     |
+====================+
|
+-- village_manager.rs   Village lifecycle, NPC population
+-- atmosphere.rs        Sky, fog, ambient lighting
+-- weather_system.rs    Rain, storms, weather cycles
+-- water_system.rs      Water rendering, physics
|
+====================+
|  GAME MODULES      |
+====================+
|
+-- animals/
|   +-- mod.rs           Module exports
|   +-- types.rs         22KB - Animal definitions (50+ species)
|   +-- behavior.rs      AI state machines, behavior trees
|   +-- spawner.rs       Biome-based spawn logic
|   +-- manager.rs       Lifecycle, pooling
|   +-- combat.rs        Attack, flee, hunt behaviors
|   +-- entity.rs        ECS components
|   +-- spatial.rs       Spatial partitioning
|   +-- player_tracking.rs  Awareness, detection
|
+-- npc/
|   +-- mod.rs
|   +-- npc_manager.rs   Spawning, scheduling
|   +-- dialogue.rs      Dialogue trees, branching
|   +-- relationships.rs Trust, friendship systems
|   +-- trading.rs       Barter, economy interface
|
+-- progression/
|   +-- mod.rs
|   +-- player_state.rs  XP, level, stats
|   +-- skills.rs        Skill trees, unlocks
|   +-- faction.rs       98KB - Faction definitions
|   +-- faction_manager.rs  Relations, wars, alliances
|   +-- faction_skills.rs   94KB - Faction-specific skills
|   +-- quests.rs        50KB - Quest system
|   +-- events.rs        Triggers, conditions
|   +-- reputation.rs    Standing with NPCs
|
+-- economy/
|   +-- mod.rs
|   +-- item.rs          15KB - Item definitions
|
+-- encyclopedia/
|   +-- mod.rs           Discovery tracking
|   +-- entries.rs       Encyclopedia content
|   +-- observer.rs      Event observation
|
+-- flora/               (Placeholder)
+-- ecology/             (Placeholder)
+-- naval/               (Placeholder)
```

## 5. Rendering Pipeline Architecture

```
+=====================================================================+
|                      CROATOAN_RENDER PIPELINES                       |
+=====================================================================+

+-------------------+     +-------------------+     +-------------------+
|  TERRAIN LAYER    |     |  VEGETATION LAYER |     |   SKY LAYER       |
|                   |     |                   |     |                   |
| terrain_pipeline  |     | grass_pipeline    |     | sky_pipeline      |
| water_pipeline    |     | tree_pipeline     |     | sun_pipeline      |
| coastline_pipeline|     | detritus_pipeline |     | light_shaft_pipe  |
+-------------------+     +-------------------+     +-------------------+
         |                         |                         |
         v                         v                         v
+-----------------------------------------------------------------------+
|                          RENDER PASS                                   |
|                                                                        |
|   Shadow Pass  -->  Depth Pass  -->  Color Pass  -->  Post-Process    |
+-----------------------------------------------------------------------+
         |
         v
+-------------------+     +-------------------+     +-------------------+
| ENTITY LAYER      |     |  EFFECTS LAYER    |     |   UI LAYER        |
|                   |     |                   |     |                   |
| building_pipeline |     | animal_orb_pipe   |     | egui integration  |
| viewmodel_pipeline|     | water_compute     |     |                   |
+-------------------+     +-------------------+     +-------------------+

Shader Files (assets/shaders/):
+-------------+-------------+-------------+-------------+
| terrain.wgsl| grass.wgsl  | tree.wgsl   | sky.wgsl    |
| water.wgsl  | sun.wgsl    | building.wgsl| detritus.wgsl|
| coastline.wgsl | light_shafts.wgsl | animal_orb.wgsl | viewmodel.wgsl |
| water_compute.wgsl |
+-------------+-------------+-------------+-------------+
```

## 6. Asset Organization

```
assets/
|
+-- shaders/                      # WGSL GPU shaders
|   +-- terrain.wgsl              Terrain mesh rendering
|   +-- grass.wgsl                Instanced grass blades
|   +-- tree.wgsl                 LOD tree rendering
|   +-- building.wgsl             Structure rendering
|   +-- water.wgsl                Water surface (fragment)
|   +-- water_compute.wgsl        Water simulation (compute)
|   +-- sky.wgsl                  Atmospheric scattering
|   +-- sun.wgsl                  Sun disc rendering
|   +-- light_shafts.wgsl         God rays / crepuscular
|   +-- animal_orb.wgsl           Animal glow indicators
|   +-- coastline.wgsl            Beach/shore rendering
|   +-- detritus.wgsl             Debris, ground clutter
|   +-- viewmodel.wgsl            First-person weapons
|
+-- audio/
|   +-- README.txt
|   +-- ambience/                 Environmental sounds
|   |   +-- wind, water, forest, cave loops
|   +-- music/                    Background music
|   |   +-- exploration, combat, village themes
|   +-- sfx/                      Sound effects
|       +-- footsteps, weapons, animals, UI
|
+-- ui/
|   +-- README.md
|   +-- roanoke1.png              Main UI atlas (2.3MB)
|   +-- loading/                  Loading screen assets
|
+-- grass-tile1.jpg               Ground texture
+-- oak-compressed.jpg            Tree bark texture
+-- taskbar icon.jpg              App icon
```

## 7. Documentation Structure

```
Root Documentation:
|
+-- SPECIFICATIONS (Game Design)
|   +-- ANIMAL_SYSTEM_SPEC.md          Fauna mechanics
|   +-- DOCILE_FAUNA_SPEC.md           Tameable animals
|   +-- FACTION_SYSTEM_SPEC.md         Faction system
|   +-- NPC_VILLAGE_SPECIFICATION.md   NPC & villages
|   +-- FLORA_PLANT_SYSTEM_SPEC.md     Plant system
|   +-- MARKETPLACE_LOOT_SYSTEM_SPEC.md  Economy
|   +-- NAVAL_COMBAT_SHIP_BATTLES_SPEC.md  Naval
|   +-- NATURE_DISCOVERY_ENCYCLOPEDIA_SPEC.md  Encyclopedia
|   +-- NATURE_MORALITY_WEATHER_KARMA_SPEC.md  Karma
|   +-- ARCHAEOLOGY_SKILL_TREE_SPEC.md  Skills
|   +-- HUNTING_SKILL_TREE_SPEC.md      Skills
|   +-- BOW_WEAPON_WHEEL_SPEC.md        Weapons
|
+-- TECHNICAL
|   +-- MASTER_AUDIT.md             System audit
|   +-- TREE_SYSTEM_AUDIT.md        Tree generation
|   +-- MATERIAL_SHADER_AUDIT.md    Shader materials
|   +-- FPS_OPTIMIZATIONS.md        Performance
|   +-- FPS_OPTIMIZATION_ROADMAP.md Performance roadmap
|   +-- VOBJ_SPECIFICATION.md       Vertex format
|   +-- VRAM_OBSERVABILITY_SPEC.md  Memory monitoring
|
+-- PLANNING
|   +-- ROADMAP.md                  Development roadmap
|   +-- MARCHING_ORDERS.md          Current priorities
|   +-- EXTERNAL_ASSETS_NEEDED.md   Asset procurement
|   +-- ECONOMY_IMPLEMENTATION.md   Economy implementation
|
+-- VISION
|   +-- TRILLION_DOLLAR_VISION.md   Long-term vision
|   +-- DECADE_FINANCIAL_ROADMAP.md Financial planning
|
+-- marketing/
    +-- MASTER_INDEX.md
    +-- whitepapers/
    |   +-- technical/              WP001-WP003
    |   +-- business/               WP010
    |   +-- community/              WP020, WP025
    |   +-- legal/                  WP030
    +-- investor/
    +-- press/
    +-- partnerships/
    +-- brand/
    +-- procurement/
```

## 8. Technology Stack

```
+=====================================================================+
|                         TECHNOLOGY STACK                             |
+=====================================================================+

LANGUAGE & RUNTIME
+-------------------+
| Rust 2021 Edition |
+-------------------+

GRAPHICS                         WINDOWING & INPUT
+-------------------+            +-------------------+
| wgpu 0.19         |            | winit 0.29        |
| (WebGPU/Vulkan/   |            | (Cross-platform   |
|  Metal/DX12)      |            |  windowing)       |
+-------------------+            +-------------------+

USER INTERFACE                   AUDIO
+-------------------+            +-------------------+
| egui 0.27         |            | kira 0.9          |
| egui-wgpu         |            | (Game audio       |
| egui-winit        |            |  engine)          |
+-------------------+            +-------------------+

MATH & PHYSICS                   SERIALIZATION
+-------------------+            +-------------------+
| glam 0.25         |            | serde 1.0         |
| (Fast linear      |            | serde_json        |
|  algebra)         |            |                   |
| noise 0.9         |            |                   |
| (Perlin noise)    |            |                   |
+-------------------+            +-------------------+

ECS FRAMEWORK                    3D MODELS
+-------------------+            +-------------------+
| bevy_ecs 0.13     |            | tobj              |
| (Entity Component |            | (OBJ loader)      |
|  System)          |            |                   |
+-------------------+            +-------------------+

ASSET PROCESSING                 LOGGING
+-------------------+            +-------------------+
| image             |            | log 0.4           |
| (Image loading)   |            | env_logger 0.11   |
| bytemuck          |            |                   |
| (Safe casting)    |            |                   |
+-------------------+            +-------------------+

FUTURE/PLACEHOLDER
+-------------------+
| burn 0.13         |
| (ML framework     |
|  for AI)          |
+-------------------+
```
