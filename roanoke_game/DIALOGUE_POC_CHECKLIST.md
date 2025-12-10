# Dialogue System Proof of Concept Checklist

## Prerequisites
- [ ] Game builds successfully (`cargo build --release`)
- [ ] Game launches without crashes
- [ ] Player can move and look around

## Village & NPC Detection
- [ ] Village exists in the world (check debug menu for village info)
- [ ] NPCs are visible as colored orbs in the village
- [ ] NPCs are moving around (schedule-based activity)

## Focus Detection (Looking at NPCs)
- [ ] Look directly at an NPC orb within ~15 meters
- [ ] "[Name] - [Role]" prompt appears on screen
- [ ] Distance shown (e.g., "5.2m away")
- [ ] "[E] Talk" prompt visible
- [ ] Focus beam (cyan line) connects player to focused NPC
- [ ] Prompt disappears when looking away

## Starting Dialogue
- [ ] Press E while looking at NPC
- [ ] Dialogue window appears at bottom of screen
- [ ] Speaker name shown in gold/yellow
- [ ] Relationship status shown (e.g., "Stranger")
- [ ] NPC's dialogue text displayed
- [ ] Multiple dialogue choices visible (numbered [1], [2], etc.)
- [ ] "[ESC] Leave conversation" hint shown

## Dialogue Navigation
- [ ] Press 1 to select first choice
- [ ] Dialogue advances to next node
- [ ] Press 2/3/4 for other choices (if available)
- [ ] Press E to advance when no choices (continue prompt)
- [ ] Dialogue ends naturally when reaching terminal node

## Closing Dialogue
- [ ] Press ESC while in dialogue
- [ ] Dialogue window closes
- [ ] Game does NOT pause (ESC closes dialogue first)
- [ ] Press ESC again - game pauses normally

## Dialogue Effects
- [ ] Make a choice that affects reputation
- [ ] Check log output for "[DIALOGUE EFFECT] Reputation..." message
- [ ] Make a choice that gives an item
- [ ] Check log for "[DIALOGUE EFFECT] Received..." message
- [ ] Item appears in inventory (if applicable)

## Relationship Tracking
- [ ] Talk to same NPC multiple times
- [ ] Relationship status may change based on choices
- [ ] Different dialogue options may unlock with better relationships

## Different NPC Roles
Test dialogue with different NPC types:
- [ ] Warrior - defensive/suspicious dialogue
- [ ] Villager - curious/welcoming dialogue
- [ ] Elder/Chief - wise/authoritative dialogue
- [ ] Craftsman - trade-focused dialogue
- [ ] Hunter - hunting tips dialogue
- [ ] Shaman - spiritual dialogue

## Edge Cases
- [ ] Walk away during dialogue - dialogue stays open
- [ ] Try to pick up item while in dialogue - E advances dialogue instead
- [ ] Number keys 1-4 select dialogue, not hotbar while in dialogue
- [ ] Number keys 5-0 still work for hotbar during dialogue

## Performance
- [ ] No FPS drop when dialogue UI is shown
- [ ] No stuttering when starting/ending dialogue
- [ ] Multiple NPCs can be focused sequentially without issues

## Log Verification
Run game and check console for:
- [ ] `[DIALOGUE] Started with [Name] ([Role])` when starting
- [ ] `[DIALOGUE] Selected choice X` when making choices
- [ ] `[DIALOGUE] Ended` when dialogue completes
- [ ] `[DIALOGUE EFFECT] ...` for any effects triggered

---

## Test Commands (Debug)

To quickly test, you can:
1. Start new game
2. Move toward village (check debug menu for village location)
3. Find NPC orbs (colored spheres)
4. Look at one and press E
5. Navigate through dialogue with 1/2/3/4 keys
6. Press ESC to exit

## Known Limitations

- Dialogue trees are hardcoded (Warrior, Villager, Craftsman types)
- No save/load of relationship state yet
- No visual portraits (placeholder system)
- Item rewards use generic "Material" type

## Success Criteria

The POC is successful if:
1. Player can identify NPCs to talk to (focus prompt)
2. Player can initiate dialogue (E key)
3. Player can navigate dialogue choices (number keys)
4. Player can exit dialogue (ESC or natural end)
5. Dialogue effects are processed (reputation changes logged)
6. System doesn't interfere with normal gameplay when not in dialogue
