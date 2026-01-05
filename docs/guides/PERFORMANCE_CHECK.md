# Performance Check

Reference checklist for validating new additions don't degrade performance.

## Mesh Budgets

| Asset Type | Max Polys (LOD0) | Max Verts | LOD1 Target | LOD2 Target |
|------------|------------------|-----------|-------------|-------------|
| Player/NPC | 8,000 | 10,000 | 50% | 25% |
| Weapon | 2,000 | 2,500 | 50% | 25% |
| Foliage (tree) | 4,000 | 5,000 | 40% | 15% |
| Foliage (small) | 500 | 600 | 50% | 20% |
| Props | 1,500 | 2,000 | 50% | 25% |
| Buildings | 10,000 | 12,000 | 50% | 20% |

**UV Requirements:**
- Single UV channel preferred
- No overlapping UVs (breaks lightmaps)
- UV islands within 0-1 space

## Texture Budgets

| Asset Type | Max Resolution | Format | Channels |
|------------|----------------|--------|----------|
| Character | 2048x2048 | PNG/DDS | RGB+A if needed |
| Weapon | 1024x1024 | PNG/DDS | RGB+A if needed |
| Foliage | 1024x1024 | PNG | RGBA (alpha cutout) |
| Props | 512x512 | PNG/DDS | RGB |
| Terrain tiles | 2048x2048 | PNG | RGB |
| UI icons | 256x256 | PNG | RGBA |

**File Size Targets:**
- Single texture: < 4MB uncompressed
- Model + textures combined: < 8MB per asset
- Audio files: < 2MB per clip (use OGG/compressed)

## Shader/Pipeline Checks

Before adding a new render pipeline:
1. Does an existing pipeline already handle this? Reuse if possible
2. Limit unique pipelines - each adds GPU state switches
3. Batch similar materials together
4. Avoid per-pixel branching in fragment shaders

**Per-Pipeline Limits:**
- Max texture samplers: 4-6
- Max uniform buffer size: 16KB
- Avoid dynamic loops in shaders

## Runtime Checks

**Draw Calls:**
- Target: < 500 draw calls per frame
- Use instancing for repeated objects (foliage, rocks)

**Memory:**
- Monitor VRAM usage - stay under 2GB for broad compatibility
- Unload distant chunk data

**Frame Budget (60fps = 16.6ms):**
- CPU game logic: < 4ms
- CPU render prep: < 4ms
- GPU render: < 8ms

## Validation Commands

```bash
# Check mesh stats (Blender Python)
bpy.context.object.data.polygons  # poly count
bpy.context.object.data.vertices  # vert count

# Check texture sizes
ls -lh assets/textures/

# Check total asset folder size
du -sh assets/

# Profile a run (add to main.rs temporarily)
# Use wgpu timestamp queries or external tools
```

## Red Flags

- Single mesh > 20k polys without LODs
- Texture > 4096x4096
- More than 8 texture samplers in one shader
- Unbounded loops in shaders
- New pipeline for a one-off effect
- Asset file > 10MB
- Audio file uncompressed WAV in release

## Pre-Commit Checklist

- [ ] LODs created for visible meshes
- [ ] Textures power-of-2 dimensions
- [ ] No unused UV channels
- [ ] Shader compiles without warnings
- [ ] Tested at lowest quality setting
- [ ] File sizes within budget
