# Animal Model System - Proof of Concept Checklist

## Pre-requisites

- [ ] Fix NPC duplicate `add_memory` method errors in `npc/relationships.rs` and `npc/interaction.rs`
- [ ] Project compiles with `cargo build --release`
- [ ] GLTF files present in `assets/models/animals/`:
  - [ ] Deer.gltf
  - [ ] Stag.gltf
  - [ ] Horse.gltf
  - [ ] Donkey.gltf
  - [ ] Fox.gltf
  - [ ] Husky.gltf
  - [ ] Wolf.gltf

---

## Visual Verification

### Model Loading
- [ ] Game starts without GLTF loading errors in console
- [ ] Log shows "Loaded 'Wolf': X meshes" (or similar) for each model
- [ ] No "Failed to load GLTF" warnings

### Model Rendering
- [ ] Wolves render as 3D wolf models (not orbs)
- [ ] Deer render as 3D deer models
- [ ] Horses render as 3D horse models
- [ ] Foxes render as 3D fox models

### Fallback to Orbs
- [ ] Bears still render as brown orbs (no model available)
- [ ] Snakes still render as orbs (no model available)
- [ ] Alligators still render as orbs (no model available)

### Scale & Positioning
- [ ] Models are appropriately sized relative to player
- [ ] Models are grounded (not floating or buried)
- [ ] Models face correct direction based on movement

---

## Behavior Verification

### State-Based Coloring
- [ ] Idle animals have natural coloring
- [ ] Fleeing animals appear washed out/pale
- [ ] Alert animals have slight yellow tint
- [ ] Attacking animals have red tint + glow

### Damage Flash
- [ ] Hitting an animal causes brief red/white flash
- [ ] Flash fades smoothly

### Fog Integration
- [ ] Distant animals fade into fog correctly
- [ ] Fog color matches terrain fog

---

## Performance Verification

- [ ] FPS remains stable with 10+ visible animals
- [ ] FPS remains stable with 50+ visible animals
- [ ] No noticeable stutter when new species loads for first time
- [ ] Memory usage is reasonable (check with task manager)

---

## Species Checklist

| Species | Has Model | Renders As | Verified |
|---------|-----------|------------|----------|
| GrayWolf | Wolf.gltf | 3D Model | [ ] |
| RedWolf | Wolf.gltf | 3D Model | [ ] |
| WhitetailDeer | Deer.gltf | 3D Model | [ ] |
| Stag | Stag.gltf | 3D Model | [ ] |
| Horse | Horse.gltf | 3D Model | [ ] |
| Donkey | Donkey.gltf | 3D Model | [ ] |
| Fox | Fox.gltf | 3D Model | [ ] |
| Husky | Husky.gltf | 3D Model | [ ] |
| Bobcat | Fox.gltf (placeholder) | 3D Model | [ ] |
| BlackBear | None | Orb | [ ] |
| EasternCougar | None | Orb | [ ] |
| WildBoar | None | Orb | [ ] |
| TimberRattlesnake | None | Orb | [ ] |
| Copperhead | None | Orb | [ ] |
| Cottonmouth | None | Orb | [ ] |
| AmericanAlligator | None | Orb | [ ] |

---

## Known Issues to Watch For

1. **Model scale wrong** - Adjust `model_scale()` in `animals/types.rs`
2. **Model rotated wrong** - Check `animal.rotation` quaternion in main.rs
3. **Textures missing** - Ensure texture files are alongside GLTF or embedded
4. **Z-fighting with terrain** - May need small Y offset in shader
5. **Backface culling issues** - Some models may need `cull_mode: None`

---

## Files Modified

```
roanoke_game/Cargo.toml              - Added gltf crate
roanoke_game/src/main.rs             - Pipeline init + rendering
roanoke_game/src/gltf_loader.rs      - NEW: GLTF loading
roanoke_game/src/animals/types.rs    - New species + model mapping

crates/croatoan_render/src/lib.rs                    - Export new pipeline
crates/croatoan_render/src/animal_model_pipeline.rs  - NEW: Render pipeline

assets/shaders/animal_model.wgsl     - NEW: Model shader

roanoke_game/src/encyclopedia/entries.rs  - New species data
roanoke_game/src/encyclopedia/mod.rs      - New species data
roanoke_game/src/ecology/mod.rs           - New species ecology
roanoke_game/src/ecology/population.rs    - New species population
```

---

## Next Steps After POC

1. [ ] Add more animal models (Bear, Boar, Cougar)
2. [ ] Add texture support to shader (currently vertex color only)
3. [ ] Add animation support (idle, walk, run cycles)
4. [ ] Add LOD system for distant animals
5. [ ] Add shadow casting for animal models
