# VRAM Observability Specification

## Goal
Implement a system to monitor and display Video RAM (VRAM) usage and GPU resource allocation in the Roanoke Engine. This will help in optimizing performance and debugging memory leaks.

## Strategies

We will implement a tiered approach, starting with `wgpu` internal counters (easiest and most portable) and optionally moving to manual tracking for more precision.

### Level 1: `wgpu` Internal Counters (Recommended First Step)
`wgpu` maintains internal counters for resources (buffers, textures, bind groups, etc.) if the `wgpu-core` backend is used (which is the default for native builds).

**Data Available:**
- Number of Buffers
- Number of Textures
- Number of Bind Groups
- Number of Render Pipelines
- (Note: Does not directly provide *bytes* allocated, only counts).

**Implementation:**
1.  **Store `wgpu::Instance`**: Modify `croatoan_render::GraphicsContext` to store the `wgpu::Instance`. Currently, it is created locally in `new_async` and dropped.
2.  **Query Report**: Use `instance.generate_report()` to get a `wgpu::GlobalReport`.
3.  **Expose Data**: Add a method `GraphicsContext::get_resource_counts()` that returns this data.

### Level 2: Manual Allocation Tracking (Precision)
To get the actual *size* (in bytes) of VRAM used, we need to wrap resource creation since `wgpu` 0.19 does not expose memory usage directly in a cross-platform way.

**Implementation:**
1.  **Create `VramTracker`**: A thread-safe struct (using `AtomicU64` or `Mutex`) to track total bytes allocated.
2.  **Wrap Device**: Create a wrapper around `wgpu::Device` (or helper methods in `GraphicsContext`) for creating resources.
    - `create_buffer_tracked(...)`: Calls `device.create_buffer` and adds `size` to tracker.
    - `create_texture_tracked(...)`: Calls `device.create_texture` and estimates size (width * height * bpp) to add to tracker.
    - **Deallocation**: This is harder with `wgpu` as resources are ref-counted. We might need to rely on `Drop` implementation or just track *peak* allocation / cumulative allocation, or use `wgpu`'s `on_submitted_work_done` callback to check if resources are destroyed (complex).
    - *Alternative*: Just track "Scene Size" by summing up known large assets (Terrain buffers, Tree meshes) and ignore small transient buffers.

### Level 3: OS-Level Monitoring (Platform Specific)
Use external crates to query the OS driver.
- **Windows/Linux (NVIDIA)**: `nvml-wrapper`
- **General**: `sysinfo` (limited GPU support)

*Decision*: We will stick to **Level 1** for now, as it provides immediate insight into "Leaking Resources" (e.g., if buffer count keeps going up). We can implement a simplified **Level 2** by just summing up the size of major assets in the `MeshRegistry`.

## UI Integration
We will add a "Debug / Performance" window using `egui` in `roanoke_game/src/main.rs`.

**Proposed UI Layout:**
```text
+--------------------------------+
| Debug                          |
+--------------------------------+
| FPS: 144.0                     |
| Frame Time: 6.94 ms            |
|                                |
| [ GPU Resources ]              |
| Buffers: 150                   |
| Textures: 45                   |
| BindGroups: 200                |
|                                |
| [ Asset Registry ]             |
| Loaded Meshes: 12              |
| Loaded Textures: 5             |
| Est. VRAM (Assets): 124 MB     |
+--------------------------------+
```

## Implementation Plan

### Phase 1: Expose `wgpu` Counters
1.  **Modify `crates/croatoan_render/src/lib.rs`**:
    - Update `GraphicsContext` struct to hold `pub instance: wgpu::Instance`.
    - Update `new_async` to store the instance.
2.  **Add Accessor**:
    - Add `pub fn get_render_stats(&self) -> wgpu::GlobalReport` to `GraphicsContext`.

### Phase 2: Estimate Asset VRAM
1.  **Update `TreeMesh` / `BuildingMesh`**:
    - Add a field `pub size_bytes: u64` to these structs.
    - Calculate this during creation (vertex buffer size + index buffer size).
2.  **Update `SharedState`**:
    - Add a method to calculate total registry size.

### Phase 3: Debug UI
1.  **Update `roanoke_game/src/main.rs`**:
    - In the `egui` render loop, add a new Window "Performance".
    - Query `ctx.get_render_stats()` (we need to pass `ctx` to the render callback, which we do).
    - Display the stats.

## Detailed Steps

1.  **Edit `crates/croatoan_render/src/lib.rs`**:
    ```rust
    pub struct GraphicsContext {
        pub instance: wgpu::Instance, // Added
        // ...
    }
    ```
2.  **Edit `roanoke_game/src/main.rs`**:
    - Inside `app.set_render_callback`, access `ctx.instance.generate_report()`.
    - Pass this data to the `egui` draw closure (might need to store it in `SharedState` or pass it temporarily).

## Future Work
- Implement a full `wgpu::Device` wrapper for precise memory tracking if needed.
- Add "Texture Viewer" to inspect loaded textures in VRAM.
