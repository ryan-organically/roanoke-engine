# Roanoke Version History

**Current**: v0.0.1
**Code Location**: `main.rs:862` (menu display)

<!-- AGENT: When releasing a new version:
1. Update version number below
2. Update main.rs:862 to match
3. Move "Unreleased" items to new version section
4. Add date
-->

---

## Unreleased

<!-- AGENT: Add completed features here before release -->

### Added
- (pending features go here)

### Fixed
- (pending fixes go here)

### Changed
- (pending changes go here)

---

## v0.0.1 (2024-11-28) - Foundation

### Added
- Procedural terrain with chunked streaming (64x64 chunks, 3 chunk view distance)
- Tree, rock, vegetation, building placement via Wave Function Collapse
- Dynamic weather system: Clear, PartlyCloudy, Overcast, Stormy, Foggy
- Day/night cycle with T/Y key controls
- Sky rendering with world-space procedural clouds
- Water compute shader (built, not yet integrated)
- Save/Load game system with JSON serialization
- Home screen with stylistic right-aligned menu
- ROANOKE v0.0.1 title branding

### Technical
- Rust + wgpu (WebGPU) graphics backend
- egui for immediate-mode UI
- glam for math
- Inverse view-projection for proper sky ray reconstruction

---

## Roadmap

<!-- AGENT: Update estimates and check off completed items -->

### v0.0.2 - Polish (Target: TBD)
- [ ] Custom serif font
- [ ] Settings menu
- [ ] Wire up water system
- [ ] Audio foundation

### v0.0.3 - Gameplay (Target: TBD)
- [ ] Inventory system
- [ ] Basic interactions
- [ ] NPC foundation

### v0.1.0 - Alpha (Target: TBD)
- [ ] Multiplayer foundation
- [ ] Quest system
- [ ] Marketplace

---

## Version Numbering

```
MAJOR.MINOR.PATCH

0.0.x - Pre-alpha (current)
0.x.0 - Alpha features
1.0.0 - First stable release
```

---

*See AGENT_SCOPE.md for current work and improvement areas.*
