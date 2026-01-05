# Known Issues

## Flora/Medicinal System Enum Mismatch

**Status:** ✅ Resolved (2026-01-05)
**Location:** `roanoke_game/src/flora/medicinal.rs`

### Description
~~The `PlantEffect` enum in `medicinal.rs` has mismatched variants.~~

**Resolution:** The enum and all usages are now consistent. The codebase compiles cleanly and all tests pass. The originally reported missing variants were either fixed or never existed in the actual codebase.

### Related Files
- `roanoke_game/src/flora/mod.rs` - FloraSpecies methods
- `roanoke_game/src/flora/medicinal.rs` - PlantEffect enum and usage
- `roanoke_game/src/encyclopedia/mod.rs` - PlantCategory usage

---

*No current build-blocking issues.*
