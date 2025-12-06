# EXTERNAL ASSETS NEEDED

**Last Updated**: 2024-12-05
**Purpose**: Comprehensive list of textures, models, and assets needed from external sources

---

## IMMEDIATE PRIORITIES

### Current Problems
| Issue | Cause | Solution |
|-------|-------|----------|
| Trees look bad | Simple icosphere + cylinder (36 tris) | Better tree model OR keep simple but add texture |
| FPS still poor | Multiple factors (see below) | Reduce render distance, optimize shaders |
| Fog only on ground | Grass/detritus shaders missing fog | Code fix in progress |
| No textures | Procedural only | Generate with Midjourney |

---

## 1. TEXTURES (Generate with Midjourney)

### How to Generate
Use Midjourney with `--tile` flag for seamless textures. Download highest resolution available (usually 1024x1024 or higher), then resize as needed.

### TERRAIN TEXTURES (HIGHEST PRIORITY)

**Grass Diffuse**
```
/imagine seamless tileable grass texture, photorealistic, top-down view, green meadow, natural variation, 4k --ar 1:1 --tile --v 6
```
- Save as: `assets/textures/grass_diffuse.png`
- Size: 1024x1024

**Grass Normal Map**
```
/imagine seamless tileable grass normal map, blue purple tones, bumpy texture, displacement map style, 4k --ar 1:1 --tile --v 6
```
- Save as: `assets/textures/grass_normal.png`
- Size: 1024x1024

**Dirt/Soil**
```
/imagine seamless tileable brown dirt soil texture, photorealistic, earth ground, some small stones, 4k --ar 1:1 --tile --v 6
```
- Save as: `assets/textures/dirt_diffuse.png`
- Size: 1024x1024

**Rock/Stone**
```
/imagine seamless tileable grey rock stone texture, photorealistic, rough granite surface, natural weathering, 4k --ar 1:1 --tile --v 6
```
- Save as: `assets/textures/rock_diffuse.png`
- Size: 1024x1024

**Sand (Beach)**
```
/imagine seamless tileable beach sand texture, photorealistic, fine grain, wet sand near water, 4k --ar 1:1 --tile --v 6
```
- Save as: `assets/textures/sand_diffuse.png`
- Size: 1024x1024

### TREE TEXTURES

**Oak Bark**
```
/imagine seamless tileable oak tree bark texture, photorealistic, deep grooves, brown grey, forest tree, 4k --ar 1:1 --tile --v 6
```
- Save as: `assets/textures/bark_oak.png`
- Size: 512x512

**Pine Bark**
```
/imagine seamless tileable pine tree bark texture, photorealistic, reddish brown, vertical grooves, 4k --ar 1:1 --tile --v 6
```
- Save as: `assets/textures/bark_pine.png`
- Size: 512x512

**Birch Bark**
```
/imagine seamless tileable birch tree bark texture, white with black horizontal marks, paper-like, 4k --ar 1:1 --tile --v 6
```
- Save as: `assets/textures/bark_birch.png`
- Size: 512x512

**Leaf Cluster (requires alpha cleanup in GIMP/Photoshop)**
```
/imagine oak tree leaf cluster, transparent background, top-down view, scattered natural arrangement, green leaves, isolated --ar 1:1 --v 6
```
- Save as: `assets/textures/leaf_cluster_oak.png`
- Size: 256x256 with alpha channel
- **NOTE**: You'll need to clean up the background to true transparency in GIMP

### BUILDING TEXTURES

**Thatch Roof**
```
/imagine seamless tileable thatched roof straw texture, photorealistic, colonial era, tan golden, 4k --ar 1:1 --tile --v 6
```
- Save as: `assets/textures/thatch_roof.png`
- Size: 512x512

**Wood Planks**
```
/imagine seamless tileable wooden plank texture, old weathered colonial wood, horizontal boards, 4k --ar 1:1 --tile --v 6
```
- Save as: `assets/textures/wood_planks.png`
- Size: 512x512

**Stone Wall**
```
/imagine seamless tileable stone wall texture, irregular fieldstone, colonial era masonry, grey brown, 4k --ar 1:1 --tile --v 6
```
- Save as: `assets/textures/stone_wall.png`
- Size: 512x512

---

## 2. 3D MODELS (Download from Web)

### FREE SOURCES

| Source | URL | Best For |
|--------|-----|----------|
| **Quaternius** | quaternius.com | Low-poly animals, nature |
| **Sketchfab** | sketchfab.com (filter: free, downloadable) | Trees, rocks, buildings |
| **OpenGameArt** | opengameart.org | Various CC0 assets |
| **Kenney** | kenney.nl | Simple stylized assets |
| **Poly Pizza** | poly.pizza | Low-poly everything |

### TREE MODELS (PRIORITY)

**Option A: Simple Low-Poly Trees (Recommended)**

Search on Sketchfab: "low poly tree" with filters:
- Downloadable: Yes
- License: CC-BY or CC0
- Format: OBJ or GLTF
- Price: Free

Look for trees with **< 2000 triangles** each.

**Specific Search Terms:**
- "low poly oak tree"
- "low poly pine tree"
- "stylized tree pack"
- "simple forest tree"

**Option B: Quaternius Nature Pack**
- URL: https://quaternius.com/packs/ultimatestylizednature.html
- Contains: Multiple tree types, rocks, bushes
- Format: FBX (convert to OBJ with Blender)
- License: CC0 (free to use)

### ANIMAL MODELS

**Quaternius Animal Pack** (RECOMMENDED)
- URL: https://quaternius.com/packs/ultimateanimals.html
- Contains: Deer, wolf, bear, rabbit, birds, etc.
- Poly count: ~300-1000 tris each (perfect for games)
- License: CC0

**Search Terms on Sketchfab:**
- "low poly deer"
- "low poly wolf"
- "low poly bear"
- "stylized forest animals"

### ROCK/BOULDER MODELS

Current procedural rocks are fine, but for variety:
- Search: "low poly rock pack"
- Look for: 5-10 rock variations
- Poly count: < 500 tris each

---

## 3. ASSET INTEGRATION GUIDE

### For Textures

1. **Create directory structure:**
```
assets/
  textures/
    terrain/
      grass_diffuse.png
      grass_normal.png
      dirt_diffuse.png
      rock_diffuse.png
      sand_diffuse.png
    trees/
      bark_oak.png
      bark_pine.png
      leaf_cluster_oak.png
    buildings/
      thatch_roof.png
      wood_planks.png
```

2. **Texture requirements:**
   - Format: PNG (RGB or RGBA for alpha)
   - Power of 2 sizes: 256, 512, 1024, 2048
   - Tileable for terrain
   - sRGB color space for diffuse maps

### For 3D Models

1. **Convert to OBJ format** (if not already):
   - Open in Blender
   - Export as OBJ with these settings:
     - Forward: -Z
     - Up: Y
     - Include: Normals, UVs
     - Triangulate Faces: Yes

2. **Place in assets:**
```
assets/
  models/
    trees/
      oak.obj
      pine.obj
    animals/
      deer.obj
      wolf.obj
```

3. **Poly count targets:**
   - Trees: < 2000 triangles (current simple tree is 36)
   - Animals: < 1000 triangles
   - Rocks: < 500 triangles
   - Buildings: < 3000 triangles

---

## 4. IMMEDIATE CODE FIXES NEEDED

### FPS Issues (In Order of Impact)

1. **Reduce Tree Render Distance** (QUICK FIX)
   - File: `roanoke_game/src/main.rs:2470`
   - Change: `let tree_max_distance = state.render_distance * 0.5;`
   - Impact: 50% fewer tree draw calls

2. **Reduce Detritus Density** (MEDIUM)
   - File: `crates/croatoan_wfc/src/vegetation.rs:137`
   - Change: `let detritus_density = 0.02;` (from 0.08)
   - Impact: 75% fewer detritus items

3. **Enable Rocks with Distance Culling** (OPTIONAL)
   - Currently disabled, can re-enable with aggressive culling
   - Only render rocks within 50m

### Fog on All Objects

Tree fog is now working. Still need:
- Grass shader fog (modify grass_pipeline.rs + grass.wgsl)
- Detritus shader fog (modify detritus_pipeline.rs + detritus.wgsl)

---

## 5. TEXTURE PIPELINE IMPLEMENTATION

Once you have textures, these code changes are needed:

### For Terrain Textures
- Already supports textures in terrain.wgsl
- Need to load and bind textures in terrain_pipeline.rs

### For Tree Textures
- tree_pipeline.rs already has texture bind group support
- Need to load bark texture and pass to pipeline

### For Buildings
- building_pipeline.rs needs texture support added

---

## PRIORITY CHECKLIST

### This Week
- [ ] Generate grass_diffuse.png with Midjourney
- [ ] Generate dirt_diffuse.png with Midjourney
- [ ] Download Quaternius tree pack
- [ ] Reduce tree render distance (code change)
- [ ] Reduce detritus density (code change)

### Next Week
- [ ] Generate all terrain textures
- [ ] Integrate tree models (replace simple geometry)
- [ ] Add fog to grass/detritus shaders

### Later
- [ ] Download animal models
- [ ] Implement animal mesh rendering
- [ ] Add building textures
- [ ] Normal mapping for terrain

---

## NOTES

### About "Nano Banana"
If you have a tool called "nano banana" for texture generation, please let me know what it is and I can provide specific instructions for it.

### Blender Export Settings
If creating custom models in Blender:
```
Export OBJ Settings:
- Include: Normals, UVs, Materials
- Transform:
  - Scale: 1.0
  - Forward: -Z Forward
  - Up: Y Up
- Geometry:
  - Triangulate Faces: ON
  - Write Normals: ON
```

### Performance Budget
| Category | Triangle Budget |
|----------|-----------------|
| Terrain (per chunk) | 65K |
| Trees (per chunk) | 50K max |
| Grass (per chunk) | 30K |
| Rocks (per chunk) | 20K |
| Detritus (per chunk) | 15K |
| Buildings (per chunk) | 10K |
| Animals (total visible) | 5K |
| **Total per frame** | **~200K** |

---

*Document maintained by Claude Code*
