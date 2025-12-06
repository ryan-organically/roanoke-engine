# MARCHING ORDERS - Roanoke Engine

**Date**: 2024-12-05
**Goal**: Make the game look good and run smoothly

---

## PHASE 1: TEXTURES (You Do This)

### Step 1: Generate Terrain Textures in Midjourney

Open Midjourney and run these prompts. Save each result to the specified path.

**Grass:**
```
/imagine seamless tileable grass texture, photorealistic, top-down view, green meadow, natural variation, 4k --ar 1:1 --tile --v 6
```
Save as: `assets/textures/grass_diffuse.png` (1024x1024)

**Dirt:**
```
/imagine seamless tileable brown dirt soil texture, photorealistic, earth ground, some small stones, 4k --ar 1:1 --tile --v 6
```
Save as: `assets/textures/dirt_diffuse.png` (1024x1024)

**Rock:**
```
/imagine seamless tileable grey rock stone texture, photorealistic, rough granite surface, natural weathering, 4k --ar 1:1 --tile --v 6
```
Save as: `assets/textures/rock_diffuse.png` (1024x1024)

**Sand:**
```
/imagine seamless tileable beach sand texture, photorealistic, fine grain, coastal, 4k --ar 1:1 --tile --v 6
```
Save as: `assets/textures/sand_diffuse.png` (1024x1024)

### Step 2: Generate Tree Textures

**Oak Bark:**
```
/imagine seamless tileable oak tree bark texture, photorealistic, deep grooves, brown grey, forest tree, 4k --ar 1:1 --tile --v 6
```
Save as: `assets/textures/bark_oak.png` (512x512)

**Leaf Cluster (needs cleanup):**
```
/imagine oak tree leaf cluster, transparent background, top-down view, scattered green leaves, isolated --ar 1:1 --v 6
```
Save as: `assets/textures/leaf_oak.png` (256x256)
**NOTE:** Open in GIMP/Photoshop and make background transparent

---

## PHASE 2: TREE MODELS (You Do This)

### Option A: Quaternius Pack (Recommended - FREE)

1. Go to: https://quaternius.com/packs/ultimatestylizednature.html
2. Download the pack
3. Open in Blender
4. Export trees as OBJ:
   - File > Export > Wavefront (.obj)
   - Settings: Forward = -Z, Up = Y, Triangulate = ON
5. Save to: `assets/models/trees/`

### Option B: Sketchfab

1. Go to: https://sketchfab.com
2. Search: "low poly tree"
3. Filter: Downloadable = Yes, Price = Free
4. Download OBJ format
5. Look for trees with < 2000 triangles

### Target Specs
- Oak tree: < 2000 triangles
- Pine tree: < 1500 triangles
- Birch tree: < 1500 triangles

---

## PHASE 3: ANIMAL MODELS (You Do This - Later)

### Quaternius Animals (FREE)

1. Go to: https://quaternius.com/packs/ultimateanimals.html
2. Download pack
3. Contains: deer, wolf, bear, rabbit, fox, birds
4. ~500-1000 triangles each (perfect)

Save to: `assets/models/animals/`

---

## PHASE 4: CODE CHANGES (Claude Does This)

Once you have the assets, tell me and I will:

1. **Integrate terrain textures** - Add texture sampling to terrain shader
2. **Replace tree geometry** - Load your OBJ models instead of simple shapes
3. **Add tree textures** - Apply bark texture to trees
4. **Fix remaining fog** - Add fog to grass and detritus shaders
5. **Implement animals** - Replace orbs with actual animal meshes

---

## DIRECTORY STRUCTURE

Create these folders and put your assets here:

```
assets/
├── textures/
│   ├── grass_diffuse.png      <-- Midjourney
│   ├── dirt_diffuse.png       <-- Midjourney
│   ├── rock_diffuse.png       <-- Midjourney
│   ├── sand_diffuse.png       <-- Midjourney
│   ├── bark_oak.png           <-- Midjourney
│   └── leaf_oak.png           <-- Midjourney (with alpha)
│
└── models/
    ├── trees/
    │   ├── oak.obj            <-- Quaternius/Sketchfab
    │   ├── pine.obj           <-- Quaternius/Sketchfab
    │   └── birch.obj          <-- Quaternius/Sketchfab
    │
    └── animals/
        ├── deer.obj           <-- Quaternius
        ├── wolf.obj           <-- Quaternius
        └── bear.obj           <-- Quaternius
```

---

## QUICK REFERENCE CARD

| Task | Tool | Time |
|------|------|------|
| 4 terrain textures | Midjourney | 30 min |
| 2 tree textures | Midjourney | 15 min |
| Tree models | Quaternius download | 10 min |
| Animal models | Quaternius download | 10 min |
| **Total your work** | | **~1 hour** |

---

## CHECKLIST

### Textures (Midjourney)
- [ ] grass_diffuse.png
- [ ] dirt_diffuse.png
- [ ] rock_diffuse.png
- [ ] sand_diffuse.png
- [ ] bark_oak.png
- [ ] leaf_oak.png (with transparency)

### Models (Download)
- [ ] Oak tree OBJ (< 2000 tris)
- [ ] Pine tree OBJ (< 1500 tris)
- [ ] Deer OBJ
- [ ] Wolf OBJ

### Tell Claude When Ready
- [ ] "I have the terrain textures ready"
- [ ] "I have tree models ready"
- [ ] "I have animal models ready"

---

## QUESTIONS FOR YOU

1. **What is "nano banana"?** You mentioned it for texture generation. What is it and how do you use it?

2. **Art style preference?**
   - Photorealistic (Midjourney prompts above)
   - Stylized/painterly (different prompts needed)
   - Low-poly flat shaded (no textures needed)

3. **Do you have Blender?** Needed for converting model formats.

---

## CURRENT STATUS

| System | Status | Blocking? |
|--------|--------|-----------|
| FPS | Improved | No |
| Trees visible | Yes (basic shapes) | No |
| Tree fog | Working | No |
| Grass fog | Not done | No |
| Terrain textures | None | **YES** |
| Tree models | Basic | **YES** |
| Animal models | Orbs only | No |

**Your priority**: Get terrain textures first. They will have the biggest visual impact.

---

*When you have assets ready, just tell me which ones and I'll integrate them.*
