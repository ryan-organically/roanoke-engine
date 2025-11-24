# VOBJ Format Specification v1.0

## Overview

The Vector Object (.vobj) format is a lightweight, compressed 3D object file format designed for efficient storage and runtime performance. It addresses common inefficiencies in traditional formats like OBJ, FBX, and GLTF by providing aggressive compression, optimized data structures, and built-in support for modern rendering pipelines.

## Design Goals

1. **Minimal File Size**: Achieve 50-70% compression compared to equivalent OBJ files
2. **Fast Parsing**: Binary format optimized for quick deserialization
3. **GPU-Ready Data**: Vertex data pre-organized for direct buffer uploads
4. **Material Optimization**: Shared material instances and texture atlasing support
5. **LOD Support**: Built-in level-of-detail mesh variants
6. **Streaming Friendly**: Chunked structure for progressive loading

## File Structure

### Header (32 bytes)
```
Offset  Size  Type      Description
------  ----  --------  -----------
0x00    4     char[4]   Magic number "VOBJ"
0x04    2     uint16    Version (major.minor) - currently 0x0100 for 1.0
0x06    2     uint16    Flags (compression, encoding options)
0x08    4     uint32    Total file size in bytes
0x0C    4     uint32    Chunk count
0x10    8     uint64    CRC64 checksum
0x18    8     reserved  Reserved for future use
```

### Flags Bitfield
```
Bit 0-3:   Compression type (0=None, 1=LZ4, 2=Zstd, 3=Custom)
Bit 4:     Quantized positions (16-bit vs 32-bit floats)
Bit 5:     Quantized normals (octahedral encoding)
Bit 6:     Quantized UVs (16-bit normalized)
Bit 7:     Contains LOD data
Bit 8:     Contains animation data
Bit 9:     Indexed geometry
Bit 10:    Triangle strips vs triangle lists
Bit 11-15: Reserved
```

## Chunk Structure

Each chunk follows this format:
```
Offset  Size  Type      Description
------  ----  --------  -----------
0x00    4     char[4]   Chunk type identifier
0x04    4     uint32    Chunk data size (excluding header)
0x08    4     uint32    Uncompressed size (if compressed)
0x0C    4     uint32    Chunk flags
0x10    N     byte[]    Chunk data
```

## Chunk Types

### META - Metadata Chunk
Contains object metadata and bounding information.

```
struct VobjMetadata {
    char name[64];              // Object name
    float boundingBox[6];       // min_x, min_y, min_z, max_x, max_y, max_z
    float boundingSphere[4];    // center_x, center_y, center_z, radius
    uint32 vertexCount;
    uint32 indexCount;
    uint32 materialCount;
    uint8 lodLevels;
    uint8 reserved[3];
}
```

### VBUF - Vertex Buffer Chunk
GPU-ready vertex data with interleaved or separate attributes.

```
struct VobjVertexBuffer {
    uint8 format;               // 0=Interleaved, 1=Separate streams
    uint8 attributeCount;
    uint16 stride;              // Bytes per vertex (interleaved only)

    VobjAttribute attributes[attributeCount];
    byte vertexData[];
}

struct VobjAttribute {
    uint8 semantic;             // 0=Position, 1=Normal, 2=Tangent, 3=UV0, 4=UV1, 5=Color, 6=Joints, 7=Weights
    uint8 type;                 // 0=Float32, 1=Float16, 2=Int16Norm, 3=Int8Norm, 4=UInt16Norm, 5=UInt8Norm
    uint8 componentCount;       // 1-4
    uint8 offset;               // Offset in vertex (interleaved) or stream index
}
```

### IBUF - Index Buffer Chunk
Optimized index data for vertex reuse.

```
struct VobjIndexBuffer {
    uint8 indexSize;            // 1=uint8, 2=uint16, 4=uint32
    uint8 primitiveType;        // 0=Triangles, 1=Triangle Strip, 2=Triangle Fan
    uint16 reserved;
    uint32 indexCount;

    byte indexData[];           // Indices, optionally delta-encoded
}
```

### MTRL - Material Chunk
Material definitions with PBR workflow support.

```
struct VobjMaterial {
    char name[32];
    uint32 shadingModel;        // 0=Unlit, 1=Lambert, 2=PBR Metallic, 3=PBR Specular

    // Base properties
    float baseColor[4];         // RGBA
    float metallic;
    float roughness;
    float emissive[3];          // RGB
    float emissiveStrength;

    // Texture references (index into TXTR chunk, 0xFFFFFFFF = none)
    uint32 baseColorTex;
    uint32 normalTex;
    uint32 metallicRoughnessTex; // Combined or separate
    uint32 occlusionTex;
    uint32 emissiveTex;

    // Texture transform
    float uvScale[2];
    float uvOffset[2];

    // Rendering flags
    uint16 flags;               // Bit 0: Double-sided, Bit 1: Alpha blend, Bit 2: Alpha test
    uint16 renderQueue;         // Sort order hint
}
```

### TXTR - Texture Reference Chunk
Texture metadata and optional embedded data.

```
struct VobjTexture {
    char name[64];
    uint8 embed;                // 0=External reference, 1=Embedded data
    uint8 format;               // 0=PNG, 1=JPEG, 2=KTX2, 3=DDS, 4=Raw
    uint16 width;
    uint16 height;
    uint16 mipLevels;

    uint32 dataSize;            // Size of embedded data or 0 for external

    union {
        char externalPath[256]; // Relative path if embed=0
        byte textureData[];     // Compressed texture data if embed=1
    };
}
```

### MESH - Mesh Chunk
References to vertex/index buffers and materials.

```
struct VobjMesh {
    char name[32];
    uint32 vertexBufferIndex;   // Index of VBUF chunk
    uint32 indexBufferIndex;    // Index of IBUF chunk
    uint32 materialIndex;       // Index of MTRL chunk

    uint32 indexOffset;         // Start index in index buffer
    uint32 indexCount;          // Number of indices to draw
    uint32 vertexOffset;        // Base vertex offset

    float localTransform[16];   // 4x4 matrix for local positioning
}
```

### LODM - Level of Detail Chunk
Multiple mesh variants for distance-based rendering.

```
struct VobjLOD {
    uint8 levelCount;
    uint8 reserved[3];

    VobjLODLevel levels[levelCount];
}

struct VobjLODLevel {
    float distance;             // Distance threshold for this LOD
    uint32 meshIndex;           // Index of MESH chunk for this LOD
    float screenCoverage;       // Alternative: screen space coverage threshold
}
```

## Compression Strategy

### Vertex Data Compression

1. **Position Quantization**
   - Store positions as 16-bit integers relative to bounding box
   - Formula: `quantized = (position - bbox_min) / (bbox_max - bbox_min) * 65535`
   - Reconstruction: `position = bbox_min + (quantized / 65535.0) * (bbox_max - bbox_min)`

2. **Normal Encoding**
   - Octahedral encoding: Store normals as 2 components (16-bit each)
   - 75% size reduction with minimal quality loss
   - Implementation: Project unit sphere onto octahedron, unfold to 2D

3. **UV Quantization**
   - Store UVs as 16-bit unsigned normalized integers [0, 65535]
   - Sufficient precision for most texture coordinates

4. **Index Compression**
   - Delta encoding: Store first index, then differences
   - Reduces bit width requirements
   - Triangle strip conversion when beneficial

### Material Optimization

1. **Material Instancing**
   - Detect duplicate materials with different names
   - Create shared material with parameter overrides

2. **Texture Atlasing Hints**
   - Store UV atlas regions for merged textures
   - Enable runtime texture atlas packing

## Usage Example

### Writing a VOBJ File

```rust
// Pseudo-code
let mut vobj = VobjWriter::new();

// Add metadata
vobj.add_metadata(name, bounds, vertex_count, index_count);

// Add vertex data with quantization
vobj.add_vertex_buffer()
    .quantize_positions(true)
    .encode_normals_octahedral(true)
    .add_attribute(Semantic::Position, Float16, 3)
    .add_attribute(Semantic::Normal, Int16Norm, 2)  // Octahedral
    .add_attribute(Semantic::UV0, UInt16Norm, 2)
    .write_data(vertices);

// Add index buffer
vobj.add_index_buffer()
    .delta_encode(true)
    .write_data(indices);

// Add materials
vobj.add_material()
    .set_base_color(color)
    .set_pbr_params(metallic, roughness)
    .set_texture("baseColor", texture_ref);

// Add mesh references
vobj.add_mesh(name, vbuf_idx, ibuf_idx, mat_idx);

// Compress and write
vobj.compress(CompressionType::Zstd)
    .write("model.vobj")?;
```

### Reading a VOBJ File

```rust
// Pseudo-code
let vobj = VobjReader::open("model.vobj")?;

// Read metadata
let meta = vobj.read_metadata()?;
println!("Object: {}, vertices: {}", meta.name, meta.vertex_count);

// Read vertex buffer (auto-dequantized)
let vbuf = vobj.read_vertex_buffer(0)?;
let positions = vbuf.get_attribute(Semantic::Position)?;

// Read materials
for mat in vobj.read_materials()? {
    println!("Material: {}", mat.name);
    if let Some(tex) = mat.base_color_texture() {
        load_texture(tex.path());
    }
}
```

## File Size Comparison

Expected compression ratios compared to OBJ format:

| Feature | OBJ Size | VOBJ Size | Reduction |
|---------|----------|-----------|-----------|
| Positions (float32) | 100% | 50% (float16) | 50% |
| Normals (float32) | 100% | 33% (oct16) | 67% |
| UVs (float32) | 100% | 50% (uint16) | 50% |
| Indices (explicit) | 100% | 20-40% (delta) | 60-80% |
| Materials (text) | 100% | 10-20% (binary) | 80-90% |
| Overall | 100% | 30-50% | 50-70% |

## Implementation Roadmap

### Phase 1: Core Format
- [ ] Basic reader/writer for binary format
- [ ] Header and chunk parsing
- [ ] VBUF, IBUF, MESH chunks
- [ ] OBJ to VOBJ converter

### Phase 2: Compression
- [ ] Vertex quantization
- [ ] Octahedral normal encoding
- [ ] Index delta encoding
- [ ] Zstd integration

### Phase 3: Materials
- [ ] MTRL chunk implementation
- [ ] TXTR chunk with external references
- [ ] PBR material support
- [ ] Material instancing

### Phase 4: Advanced Features
- [ ] LOD support
- [ ] Embedded texture data
- [ ] Triangle strip optimization
- [ ] Streaming/progressive loading

### Phase 5: Tools
- [ ] Blender exporter plugin
- [ ] GLTF to VOBJ converter
- [ ] Validation tool
- [ ] Inspector/viewer utility

## Technical Considerations

### Endianness
- Little-endian throughout (matches x86/x64, ARM)
- Big-endian systems must swap on read/write

### Alignment
- All chunk headers aligned to 8-byte boundaries
- Vertex attributes naturally aligned for GPU access

### Version Compatibility
- Major version changes break compatibility
- Minor version changes maintain backward compatibility
- Readers should reject unknown major versions

### Error Handling
- CRC64 verification for data integrity
- Graceful degradation on unknown chunks
- Validation mode for strict conformance checking

## License & Attribution

This format specification is designed for the Roanoke Engine.
Implementations should include format version in exported files.

## Changelog

**v1.0 (Initial Draft)**
- Core format structure
- Basic compression features
- PBR material support
- LOD system design
