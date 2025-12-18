# Shader Hot-Reload System Spec

## Overview
File-watcher based shader reloading without game restart. Press a key or auto-detect file changes to recompile WGSL shaders at runtime.

## Goals
- Sub-second shader iteration during development
- No game restart required for shader tweaks
- Graceful fallback on compile errors (keep old shader)
- Minimal runtime overhead when not in use

## Architecture

### 1. Shader Registry
```rust
struct ShaderRegistry {
    shaders: HashMap<String, ShaderEntry>,
    watcher: Option<FileWatcher>,
    pending_reloads: Vec<String>,
}

struct ShaderEntry {
    path: PathBuf,                    // "assets/shaders/water.wgsl"
    module: wgpu::ShaderModule,       // Current compiled module
    last_modified: SystemTime,
    pipelines: Vec<PipelineRef>,      // Pipelines using this shader
}
```

### 2. File Watching
- Use `notify` crate for cross-platform file watching
- Watch `assets/shaders/` directory
- Debounce rapid saves (100ms window)
- Queue changed files for next frame reload

### 3. Reload Flow
```
File Changed → Queue Reload → Next Frame:
  1. Read shader source from disk
  2. Attempt wgpu::Device::create_shader_module()
  3. On success:
     - Recreate affected pipelines
     - Swap in new pipelines
     - Log success
  4. On failure:
     - Log error with line numbers
     - Keep existing shader/pipeline
     - Optional: overlay error on screen
```

### 4. Pipeline Recreation
Each pipeline type needs a `recreate_with_shader()` method:
- `WaterSystem::recreate_shaders(device, registry)`
- `TreePipeline::recreate_shaders(device, registry)`
- `GrassPipeline::recreate_shaders(device, registry)`
- etc.

### 5. Keybinds
| Key | Action |
|-----|--------|
| F5 | Force reload all shaders |
| F6 | Toggle auto-reload on/off |
| ~ | Toggle shader error overlay |

### 6. Config
```toml
# claude.toml or engine.toml
[dev]
shader_hot_reload = true
shader_watch_path = "assets/shaders"
shader_reload_debounce_ms = 100
```

## Implementation Phases

### Phase 1: Manual Reload (F5)
- Read shaders from disk instead of `include_wgsl!`
- F5 triggers full shader reload
- ~2 hours work

### Phase 2: File Watcher
- Add `notify` crate dependency
- Auto-detect changes
- ~1 hour additional

### Phase 3: Error Overlay
- Render shader compile errors on screen
- Show file:line:col information
- ~2 hours additional

## Files to Modify
1. `Cargo.toml` - add `notify` crate
2. New `shader_registry.rs` - central management
3. Each pipeline file - add `recreate_shaders()` method
4. `main.rs` - integrate registry, add keybinds
5. Move from `include_wgsl!()` to runtime loading

## Shader Files Affected
- `assets/shaders/water.wgsl`
- `assets/shaders/water_compute.wgsl`
- `assets/shaders/tree.wgsl`
- `assets/shaders/grass.wgsl`
- `assets/shaders/terrain.wgsl`
- `assets/shaders/sky.wgsl`
- (others as discovered)

## Risks
- Pipeline recreation may cause frame stutter (~50-100ms)
- Some pipelines have complex bind group layouts
- Compute pipelines may need special handling

## Debug Output
```
[SHADER] Watching: assets/shaders/
[SHADER] Changed: water.wgsl
[SHADER] Recompiling water.wgsl... OK (12ms)
[SHADER] Recreating WaterSystem pipelines... OK
[SHADER] ERROR in grass.wgsl:47:12 - expected ';'
[SHADER] Keeping previous grass.wgsl
```
