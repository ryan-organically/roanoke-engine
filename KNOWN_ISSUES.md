# Known Issues

## Flora/Medicinal System Enum Mismatch

**Status:** Build-blocking
**Location:** `roanoke_game/src/flora/medicinal.rs`

### Description
The `PlantEffect` enum in `medicinal.rs` has mismatched variants. Code references variants that don't exist in the enum definition:

**Missing variants in enum (line 11):**
- `Antibacterial`
- `Antiseptic`
- `AntiNausea`
- `Antihistamine`
- `FeverReducer` (should be `FeverReduction`)
- `BloodClotting`

**Affected code:** Lines 588-600+ in `medicinal.rs`

### Fix
Either:
1. Add missing variants to `PlantEffect` enum
2. Or update the match arms to use existing variants

### Related Files
- `roanoke_game/src/flora/mod.rs` - FloraSpecies methods
- `roanoke_game/src/flora/medicinal.rs` - PlantEffect enum and usage
- `roanoke_game/src/encyclopedia/mod.rs` - PlantCategory usage
