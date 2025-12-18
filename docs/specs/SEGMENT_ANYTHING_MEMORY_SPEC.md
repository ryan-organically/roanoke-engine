# Segment Anything Model (SAM) for Memory Optimization

**Status**: Speculative / Research
**Priority**: Future Investigation
**Dependencies**: Stable rendering pipeline first

---

## The Problem

Game engines face constant memory pressure:
- **VRAM**: GPU texture memory, mesh buffers, render targets
- **RAM**: Asset staging, CPU-side data, streaming buffers
- **Bandwidth**: CPU↔GPU transfers, disk streaming

Traditional solutions are brute-force:
- Fixed LOD levels (wasteful - same detail for boring vs interesting areas)
- Distance-based culling (ignores visual importance)
- Uniform texture resolution (rocks get same detail as character faces)
- Manual artist markup (doesn't scale, expensive)

**What if we could automatically identify what matters?**

---

## The Idea: SAM-Driven Intelligent Memory

Use Segment Anything Model to:
1. **Analyze rendered frames** → identify visually distinct regions
2. **Score importance** → player focus, motion, contrast, semantic meaning
3. **Allocate resources dynamically** → more memory for important stuff, less for background

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    SAM-DRIVEN MEMORY PIPELINE                               │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│   FRAME N                    SAM ANALYSIS                FRAME N+1         │
│   ────────                   ────────────                ────────           │
│                                                                             │
│   ┌─────────┐               ┌─────────────┐             ┌─────────┐        │
│   │ Render  │──────────────▶│  Segment    │────────────▶│ Adjust  │        │
│   │ Frame   │               │  + Score    │             │ Quality │        │
│   └─────────┘               └─────────────┘             └─────────┘        │
│                                    │                                        │
│                                    ▼                                        │
│                             ┌─────────────┐                                │
│                             │  Importance │                                │
│                             │     Map     │                                │
│                             └─────────────┘                                │
│                                    │                                        │
│                    ┌───────────────┼───────────────┐                       │
│                    ▼               ▼               ▼                       │
│              ┌──────────┐   ┌──────────┐   ┌──────────┐                   │
│              │ Texture  │   │   LOD    │   │ Streaming│                   │
│              │ Quality  │   │ Selection│   │ Priority │                   │
│              └──────────┘   └──────────┘   └──────────┘                   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Use Cases

### 1. Adaptive Texture Resolution

**Problem**: All textures loaded at same resolution. Distant mountain gets same VRAM as player's hands.

**SAM Solution**:
- Segment frame into regions
- Score each region by screen coverage + visual complexity
- Dynamically mipmap textures based on importance

```
BEFORE (Fixed):
┌────────────────────────────────────┐
│ Sky: 2K        │ Mountain: 2K     │  Total: 8K textures
│ Ground: 2K     │ Tree: 2K         │  (wasteful)
└────────────────────────────────────┘

AFTER (SAM-Driven):
┌────────────────────────────────────┐
│ Sky: 256       │ Mountain: 512    │  Total: 2.3K textures
│ Ground: 1K     │ Tree: 512        │  (smart)
│ Player Hands: 2K (focus area)     │
└────────────────────────────────────┘
```

**VRAM Savings**: 50-70% reduction with minimal perceptual loss

---

### 2. Intelligent LOD Selection

**Problem**: LOD switches based purely on distance. A detailed rock 50m away gets same LOD as a boring flat wall 50m away.

**SAM Solution**:
- Segment objects by visual distinctiveness
- High-contrast/complex segments → keep detail longer
- Low-contrast/simple segments → aggressive LOD earlier

```rust
struct SamLodPolicy {
    // Traditional: just distance
    // distance_threshold: f32,

    // SAM-enhanced: importance-weighted
    fn select_lod(&self, object: &Object, importance: f32) -> LodLevel {
        let effective_distance = object.distance / importance;
        // High importance (1.0) = full distance considered
        // Low importance (0.1) = acts like 10x further away
        self.distance_to_lod(effective_distance)
    }
}
```

---

### 3. Streaming Priority Queue

**Problem**: Asset streaming loads chunks by distance. Player looking at horizon loads nearby underground caves first.

**SAM Solution**:
- Segment visible frame
- Identify what player is actually looking at (gaze estimation from frame center)
- Prioritize streaming for visible segments, deprioritize occluded

```
STREAMING PRIORITY (Traditional):
1. Chunk at (0,0) - closest
2. Chunk at (1,0) - second closest
3. Chunk at (0,1) - third closest
...

STREAMING PRIORITY (SAM-Enhanced):
1. Mountain chunk - player looking at it (center of frame)
2. Ground chunk - visible, near player
3. Sky chunk - visible but low detail needed
4. Cave chunk - not visible, defer loading
```

---

### 4. Render Target Resolution

**Problem**: Entire frame rendered at same resolution. VR already does foveated rendering, but requires eye tracking hardware.

**SAM Solution**:
- Software foveated rendering without eye tracking
- Segment frame, identify likely focus regions (center, motion, high contrast)
- Render focus regions at full res, periphery at lower res

```
┌─────────────────────────────────────┐
│ ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░ │  ░ = 50% resolution
│ ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░ │
│ ░░░░░░░░░┌───────────┐░░░░░░░░░░░ │
│ ░░░░░░░░░│ FULL RES  │░░░░░░░░░░░ │  █ = 100% resolution
│ ░░░░░░░░░│  (focus)  │░░░░░░░░░░░ │      (SAM-identified
│ ░░░░░░░░░└───────────┘░░░░░░░░░░░ │       important region)
│ ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░ │
│ ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░ │
└─────────────────────────────────────┘

GPU savings: 30-50% fill rate reduction
```

---

### 5. Procedural Detail Allocation

**Problem**: Procedural generation creates uniform detail. Forest floor has same pebble density whether visible or obscured by grass.

**SAM Solution**:
- Pre-analyze procedural output with SAM
- Identify regions that will be visually occluded
- Skip detail generation for hidden areas

```rust
fn generate_ground_detail(&self, chunk: &Chunk, sam_mask: &ImportanceMask) {
    for cell in chunk.cells() {
        let importance = sam_mask.sample(cell.position);

        if importance < 0.1 {
            // This area is under dense grass/foliage
            // Skip pebble generation entirely
            continue;
        }

        // Scale detail density by importance
        let pebble_density = self.base_density * importance;
        self.spawn_pebbles(cell, pebble_density);
    }
}
```

**Potential Fix for Rock Problem**: If 78K rocks are spawned but SAM identifies most areas as visually unimportant (under trees, behind hills), we could cull at spawn time, not render time.

---

## Implementation Approaches

### Option A: Offline Pre-Processing

Run SAM on representative camera positions during development. Bake importance maps into world data.

**Pros**:
- Zero runtime cost
- Can use full SAM model (slow but accurate)
- Deterministic

**Cons**:
- Doesn't adapt to player behavior
- Storage overhead for importance maps
- Stale if world changes

```
BUILD PIPELINE:
┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐
│ Generate │───▶│ Render   │───▶│ Run SAM  │───▶│ Bake     │
│ World    │    │ Samples  │    │ Analysis │    │ Maps     │
└──────────┘    └──────────┘    └──────────┘    └──────────┘
```

---

### Option B: Runtime Analysis (Async)

Run SAM on rendered frames asynchronously. Results arrive 1-3 frames later.

**Pros**:
- Adapts to actual player view
- No baked data needed
- Works with dynamic content

**Cons**:
- Latency (stale by 1-3 frames)
- GPU/CPU cost for inference
- Needs lightweight SAM variant

```
RUNTIME PIPELINE:
Frame N: Render → Send to SAM (async)
Frame N+1: Render with old importance data
Frame N+2: Render with old importance data
Frame N+3: SAM results arrive → Update importance → Render with new data
```

---

### Option C: Hybrid (Recommended)

Baked coarse importance + runtime refinement.

**Workflow**:
1. Offline: Generate coarse importance maps per chunk (low-res, static)
2. Runtime: Lightweight edge detection identifies frame regions
3. Runtime: Blend baked + detected importance
4. Every N seconds: Run full SAM analysis to recalibrate

```rust
struct HybridImportanceSystem {
    baked_maps: HashMap<ChunkId, ImportanceMap>,      // Coarse, static
    runtime_edges: EdgeDetector,                       // Fast, per-frame
    sam_refinement: Option<SamAnalyzer>,              // Slow, periodic

    fn get_importance(&self, chunk: ChunkId, screen_pos: Vec2) -> f32 {
        let baked = self.baked_maps[chunk].sample(screen_pos);
        let edges = self.runtime_edges.importance(screen_pos);
        let sam = self.sam_refinement.latest_importance(screen_pos);

        // Blend: edges for responsiveness, SAM for accuracy, baked as fallback
        edges * 0.4 + sam * 0.4 + baked * 0.2
    }
}
```

---

## SAM Model Options

### Full SAM (Meta)
- **Size**: ~2.5GB
- **Speed**: ~50ms/frame on RTX 3080
- **Use**: Offline baking only

### SAM-HQ (High Quality variant)
- **Size**: ~2.5GB
- **Speed**: ~60ms/frame
- **Use**: Offline, higher accuracy for edges

### MobileSAM
- **Size**: ~40MB
- **Speed**: ~10ms/frame on RTX 3080
- **Use**: Runtime async analysis

### FastSAM (YOLO-based)
- **Size**: ~140MB
- **Speed**: ~4ms/frame on RTX 3080
- **Use**: Near-realtime, lower quality

### EdgeSAM
- **Size**: ~10MB
- **Speed**: ~2ms/frame
- **Use**: Realtime, edge-focused

**Recommendation**: EdgeSAM for runtime, Full SAM for offline baking.

---

## Memory Budget Analysis

### Current Estimated Usage (Roanoke)
```
VRAM:
- Terrain textures: ~200MB
- Foliage instances: ~100MB
- Shadow maps: ~64MB
- Render targets: ~150MB
- Mesh buffers: ~100MB
- Other: ~50MB
TOTAL: ~664MB VRAM

RAM:
- Chunk data: ~300MB
- Asset staging: ~200MB
- Audio: ~50MB
- Other: ~100MB
TOTAL: ~650MB RAM
```

### With SAM Optimization (Projected)
```
VRAM:
- Terrain textures: ~120MB (-40%, adaptive mips)
- Foliage instances: ~60MB (-40%, importance culling)
- Shadow maps: ~64MB (unchanged)
- Render targets: ~100MB (-33%, variable resolution)
- Mesh buffers: ~70MB (-30%, smart LOD)
- SAM model (EdgeSAM): ~10MB
- Importance buffer: ~4MB
TOTAL: ~428MB VRAM (-35%)

RAM:
- Chunk data: ~200MB (-33%, skip unimportant detail)
- Asset staging: ~150MB (-25%, smarter streaming)
- SAM inference: ~50MB
- Other: ~100MB
TOTAL: ~500MB RAM (-23%)
```

---

## Integration Points in Roanoke

### 1. Foliage Pipeline
```rust
// crates/croatoan_render/src/foliage_pipeline.rs

fn cull_instances(&self, instances: &[FoliageInstance], importance: &ImportanceMap) -> Vec<FoliageInstance> {
    instances.iter()
        .filter(|inst| {
            let screen_pos = self.world_to_screen(inst.position);
            let imp = importance.sample(screen_pos);
            // Keep if important OR randomly based on importance
            imp > 0.3 || self.rng.gen::<f32>() < imp
        })
        .cloned()
        .collect()
}
```

### 2. Rock Spawner (Fix for 78K→100 problem)
```rust
// crates/croatoan_wfc/src/rock_spawner.rs

fn spawn_rocks(&self, chunk: &Chunk, importance: &ImportanceMap) -> Vec<RockInstance> {
    let mut rocks = Vec::new();

    for cell in chunk.cells() {
        let imp = importance.sample_world(cell.position);

        // Skip spawning in unimportant areas entirely
        if imp < 0.05 {
            continue;
        }

        // Reduce density in medium-importance areas
        let density = self.base_density * imp;
        let count = (density * cell.area) as usize;

        for _ in 0..count {
            rocks.push(self.generate_rock(cell));
        }
    }

    rocks
}
```

### 3. Texture Streaming
```rust
// crates/croatoan_render/src/texture_manager.rs

fn update_mip_levels(&mut self, importance: &ImportanceMap) {
    for (id, texture) in &mut self.textures {
        let screen_coverage = self.get_screen_coverage(id);
        let importance = importance.average_for_texture(id);

        // High importance + high coverage = high detail
        let target_mip = self.calculate_mip(screen_coverage * importance);

        if texture.current_mip != target_mip {
            self.schedule_mip_transition(id, target_mip);
        }
    }
}
```

---

## Risks and Concerns

### Performance Overhead
- Even EdgeSAM adds ~2ms latency
- Memory for model weights
- CPU/GPU contention

**Mitigation**: Run on separate thread, async results, skip frames under load

### Temporal Stability
- Importance map changes frame-to-frame
- Could cause popping/flickering

**Mitigation**: Temporal smoothing, hysteresis thresholds

### Accuracy Limitations
- SAM designed for natural images, not game renders
- May misidentify stylized content

**Mitigation**: Fine-tune on game screenshots, fallback to distance-based

### Complexity
- Another system to maintain
- Debugging harder (why did this rock disappear?)

**Mitigation**: Debug visualization mode, importance overlay

---

## Research Questions

1. **Does SAM work on procedural/stylized renders?**
   - Need to test with Roanoke screenshots
   - May need fine-tuning or custom training

2. **What's the minimum viable model?**
   - EdgeSAM? Custom distilled model?
   - Can we use just edge detection + heuristics?

3. **How much does temporal instability matter?**
   - Need player testing
   - Different thresholds for different use cases

4. **Is this overkill?**
   - Simple screen-space heuristics might get 80% of benefit
   - Center-weighted + edge detection + distance

---

## Alternative: SAM-Free Approximation

If SAM proves too heavy, approximate with:

```rust
struct SimpleImportanceEstimator {
    fn estimate(&self, screen_pos: Vec2, depth: f32, motion: Vec2) -> f32 {
        // Center bias (foveation)
        let center_dist = (screen_pos - Vec2::new(0.5, 0.5)).length();
        let center_weight = 1.0 - center_dist.min(1.0);

        // Depth bias (closer = more important)
        let depth_weight = 1.0 / (1.0 + depth * 0.1);

        // Motion bias (moving = attention-grabbing)
        let motion_weight = motion.length().min(1.0);

        // Edge detection (high contrast = important)
        let edge_weight = self.sobel_magnitude(screen_pos);

        center_weight * 0.3 + depth_weight * 0.3 + motion_weight * 0.2 + edge_weight * 0.2
    }
}
```

This gets maybe 60% of SAM's benefit at 0.1% of the cost.

---

## Next Steps

### Phase 1: Validate Concept
- [ ] Capture 100 representative Roanoke screenshots
- [ ] Run full SAM offline, analyze segmentation quality
- [ ] Identify if game renders segment well

### Phase 2: Prototype Integration
- [ ] Implement simple importance estimator (SAM-free)
- [ ] Add importance-weighted rock spawning
- [ ] Measure memory/performance impact

### Phase 3: SAM Integration
- [ ] Integrate EdgeSAM or FastSAM
- [ ] Async analysis pipeline
- [ ] Compare ML vs heuristic results

### Phase 4: Production
- [ ] Hybrid baked + runtime system
- [ ] Temporal stability tuning
- [ ] Debug visualization tools

---

## Conclusion

SAM-driven memory optimization is a **speculative but promising** approach to intelligent resource allocation. Instead of uniform quality everywhere, allocate detail where it matters.

**Immediate value**: Could help diagnose the rock rendering problem. If SAM shows most rock positions are visually unimportant (under trees, behind terrain), we can cull at spawn time.

**Long-term value**: Adaptive quality that scales with hardware. Low-end machines get aggressive importance culling. High-end machines render everything.

**Risk**: Complexity and overhead may outweigh benefits. Start with SAM-free heuristics, graduate to ML if needed.

---

*"The best optimization is not rendering what no one will see."*

---

**Document Version**: 1.0
**Author**: Claude Code Analysis
**Status**: Speculative Research
