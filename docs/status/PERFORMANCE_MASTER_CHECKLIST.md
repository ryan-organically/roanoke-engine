# Performance Master Checklist

Generated: 2024-12-28

## Current Status Summary

| Category | Status | Notes |
|----------|--------|-------|
| Total Assets | **632MB** | Heavy - trees/shrubs dominate |
| Pipelines | 19 | Moderate - some consolidation possible |
| Shaders | 20 | OK |
| Audio | OK | All MP3, no WAVs |
| Textures | OK | Small/reasonable |
| main.rs | **9,530 lines** | Needs splitting |

---

## Critical Issues

### Asset Issues

#### Broken LODs (same size as LOD0)
| Model | LOD0 | LOD1 | LOD2 | Status |
|-------|------|------|------|--------|
| pine_0 | 26MB | 25MB | 25MB | [ ] Re-export |
| fir_0 | 25MB | 25MB | 25MB | [ ] Re-export |
| dead_conifer_0 | 12MB | 12MB | 12MB | [ ] Re-export |
| dead_log_0 | 12MB | 12MB | 12MB | [ ] Re-export |
| conifer_shrub_0 | 25MB | 25MB | 25MB | [ ] Re-export |
| birch_0 | 12MB | 8.7MB | 7.7MB | [x] Working |

#### Oversized Models (>10MB budget)
| Model | Size | Target | Status |
|-------|------|--------|--------|
| pine_0.glb | 26MB | <8MB | [ ] Reduce polys |
| fir_0.glb | 25MB | <8MB | [ ] Reduce polys |
| conifer_shrub_0.glb | 25MB | <8MB | [ ] Reduce polys |
| tree_0.glb | 23MB | <8MB | [ ] Reduce polys |
| tree_1.glb | 23MB | <8MB | [ ] Reduce polys |
| bush_0.glb | 23MB | <8MB | [ ] Reduce polys |
| grass_0.glb | 23MB | <8MB | [ ] Reduce polys |
| shrub_0.glb | 23MB | <8MB | [ ] Reduce polys |

#### Missing Optimized Folders
| Category | Source | Size | Status |
|----------|--------|------|--------|
| rocks | assets/models_optimized/rocks/ | 6.2MB | [x] boulder LODs (5.5M/560K/150K) - 62% reduction |
| containers | assets/models_optimized/containers/ | 10.4MB | [x] chest_closed + chest_open LODs complete |
| weapons | assets/models/weapons/ | 19MB | [ ] Create models_optimized/weapons/ |
| animals | assets/models/animals/ | 33MB | [ ] Create models_optimized/animals/ |

#### Folder Size Breakdown
```
trees/      295MB  <- primary target
shrubs/     146MB  <- secondary target
animals/     33MB
containers/  23MB
weapons/     19MB
rocks/       17MB
grass/      4.4MB
```

---

## Task Checklist

### Quick Wins (1-2 hours)

- [ ] **Cap shader loops**
  - [ ] `terrain.wgsl`: `min(uniforms.campfire_count, 8u)`
  - [ ] `tree.wgsl`: `min(camera.campfire_count, 8u)`
  - [ ] `light_shafts.wgsl`: `min(uniforms.num_samples, 64)`

- [ ] **Remove debug code**
  - [ ] `chunk_manager.rs:112-119`: Remove unsafe `CALL_COUNT` static

- [ ] **Add debug UI counters**
  - [ ] Draw call counter
  - [ ] Triangle counter
  - [ ] Instance count per pipeline

### Medium Effort (1-2 days)

- [ ] **Consolidate chunk iteration**
  - Currently 4 separate `iter_chunks()` loops in main.rs (lines 7885, 7992, 8284, 9232)
  - Target: Single pass collecting all render data

- [ ] **GPU timestamp queries**
  - [ ] Add wgpu timestamp query support
  - [ ] Per-pipeline timing in debug UI
  - [ ] Frame budget breakdown (CPU prep / GPU render)

- [ ] **Asset validation script**
  - [ ] Check model file sizes on import
  - [ ] Verify LOD size reduction (LOD1 < LOD0, LOD2 < LOD1)
  - [ ] Check texture dimensions (power of 2)
  - [ ] Flag files exceeding budgets

### Larger Refactors (1-2 weeks)

- [ ] **Split main.rs into modules**
  - [ ] `render_loop.rs` - main render pass logic
  - [ ] `input.rs` - keyboard/mouse handling
  - [ ] `ui.rs` - egui/HUD code
  - [ ] `chunk_loading.rs` - chunk generation orchestration
  - [ ] `systems_update.rs` - per-frame system updates

- [ ] **Consolidate similar pipelines**
  - [ ] grass2/grass3 share common grass pipeline
  - [ ] sun/moon share celestial pipeline
  - [ ] Review LOD0/1/2 variants for deduplication

- [ ] **CI performance regression tests**
  - [ ] Startup time benchmark
  - [ ] Frame time under load test
  - [ ] Memory usage tracking
  - [ ] Asset size gate (fail if total > threshold)

---

## Validation Commands

```bash
# Check total asset size
du -sh assets/

# Find files over 10MB
find assets/models -size +10M -exec ls -lh {} \;

# Check LOD size progression (should decrease)
ls -lh assets/models/trees/*pine*

# Count draw calls (grep render code)
grep -c "draw_indexed\|draw(" crates/croatoan_render/src/*.rs

# Check for unbounded loops in shaders
grep -n "for.*uniforms\." assets/shaders/*.wgsl
```

---

## Performance Budgets

| Metric | Budget | Current | Status |
|--------|--------|---------|--------|
| Total assets | <400MB | 632MB | Over |
| Single model | <8MB | 26MB max | Over |
| Draw calls/frame | <500 | Unknown | Need counter |
| Frame time | <16.6ms | Unknown | Need profiler |
| VRAM | <2GB | Unknown | Need tracking |

---

## Notes

- `models_optimized/` folder IS being used for trees/shrubs (good)
- Heavy files in `models/trees/` are legacy/source - not loaded at runtime
- Birch is the only tree with working LOD reduction
- Weapons: dagger/hatchet have good LODs, flintlock does not
