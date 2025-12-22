# Cheat-Proof Architecture

## The Goal

**Cheating should be logically impossible, not just detectable.**

Not "we'll catch you" - that's an arms race.
Not "we'll punish you" - that assumes you can cheat first.

**The system should make cheating as impossible as dividing by zero.**

---

## Why "Catch Cheaters" Fails

```
Traditional Anti-Cheat Arms Race:

  Cheater finds exploit
        ↓
  Uses it for weeks/months
        ↓
  Eventually detected
        ↓
  Banned (makes new account)
        ↓
  Exploit patched
        ↓
  Cheater finds new exploit
        ↓
  (Repeat forever)

PROBLEMS:
- Damage done before detection
- Reactive, not proactive
- Infinite cat-and-mouse
- Sophisticated cheats evade detection
- Post-quantum computing makes detection crypto breakable
```

---

## Structural Impossibility: The Approach

### Principle 1: No Hidden Information That Matters

If the client doesn't have information, it can't leak it.
If the client does have information, assume the cheater has it too.

```
WRONG (most games):
  Server: "Enemy is at position (45, 72) but behind wall"
  Client: Has this info for "efficient rendering"
  Cheater: Reads memory → wallhack

RIGHT:
  Server: Only sends what's visible to player
  Client: Cannot know enemy position because server didn't send it
  Cheater: Cannot extract information that doesn't exist locally
```

### Principle 2: Commit-Then-Reveal for All Decisions

No one can react to information they shouldn't have.

```
WRONG:
  Player A: "I attack!"
  Player B: Sees A's attack, dodges (reaction time: 0ms)

  Cheater B: Modified client removes network latency display
             Actually reacted before A's action arrived

RIGHT (Commit-Reveal):
  Frame N:
    Player A: Commits hash(attack_north + salt_A)
    Player B: Commits hash(move_south + salt_B)

  Frame N+1:
    Player A: Reveals (attack_north, salt_A) - hash verified
    Player B: Reveals (move_south, salt_B) - hash verified

  Execution:
    Both actions were locked before either knew the other's choice
    Cheater cannot react to A's attack - was already committed to move_south
```

### Principle 3: Deterministic Execution with Verified Inputs

Given the same inputs, every client MUST produce the same output.
Any deviation is not "detected" - it simply doesn't execute.

```
┌─────────────────────────────────────────────────────────────────┐
│              DETERMINISTIC EXECUTION MODEL                       │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  All clients run identical simulation                           │
│  Inputs are the ONLY variable                                   │
│  Same inputs → Same outputs (mathematical certainty)            │
│                                                                  │
│  Frame 100:                                                     │
│    Inputs: {A: move_north, B: attack, C: idle}                  │
│    ↓                                                            │
│    Deterministic simulation step                                │
│    ↓                                                            │
│    State hash: 7f3a2b...                                        │
│                                                                  │
│  ALL clients compute same hash                                  │
│  Not "verified" - it's mathematically inevitable               │
│                                                                  │
│  If a client claims different state:                           │
│    - Other clients simply don't see it                         │
│    - Consensus reality doesn't include the "cheat"             │
│    - Cheater is playing a different game alone                 │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

---

## The Cheat-Proof Protocol

### Phase 1: Input Commitment

```rust
/// All players commit their input BEFORE seeing others
pub struct InputCommitment {
    pub frame: u64,
    pub player_id: PlayerId,

    /// Hash of (input || random_salt)
    /// Uses SHA-3 (post-quantum secure)
    pub commitment: [u8; 32],

    /// Timestamp for ordering
    pub timestamp: u64,
}

impl InputCommitment {
    pub fn create(input: &PlayerInput, salt: &[u8; 32]) -> Self {
        let mut hasher = Sha3_256::new();
        hasher.update(&input.to_bytes());
        hasher.update(salt);

        Self {
            frame: input.frame,
            player_id: input.player_id,
            commitment: hasher.finalize().into(),
            timestamp: current_time(),
        }
    }
}
```

### Phase 2: Commitment Collection

```rust
/// Wait for all players to commit before revealing
pub struct FrameCommitments {
    pub frame: u64,
    pub commitments: HashMap<PlayerId, InputCommitment>,
    pub deadline: Timestamp,
}

impl FrameCommitments {
    pub fn is_complete(&self, expected_players: &[PlayerId]) -> bool {
        expected_players.iter().all(|p| self.commitments.contains_key(p))
    }

    /// Once complete, no one can change their input
    /// The future is cryptographically locked
    pub fn lock(&self) -> LockedFrame {
        LockedFrame {
            frame: self.frame,
            commitment_root: self.merkle_root(),
        }
    }
}
```

### Phase 3: Reveal and Verify

```rust
/// After all commit, reveal inputs
pub struct InputReveal {
    pub frame: u64,
    pub player_id: PlayerId,
    pub input: PlayerInput,
    pub salt: [u8; 32],
}

impl InputReveal {
    /// Verify reveal matches prior commitment
    pub fn verify(&self, commitment: &InputCommitment) -> bool {
        let mut hasher = Sha3_256::new();
        hasher.update(&self.input.to_bytes());
        hasher.update(&self.salt);
        let computed: [u8; 32] = hasher.finalize().into();

        computed == commitment.commitment
    }
}

/// If reveal doesn't match commitment:
/// - Player's input is treated as "null" (no action)
/// - They committed to something they won't reveal = forfeit turn
/// - No way to "try different inputs until one works"
```

### Phase 4: Deterministic Execution

```rust
/// Execution is pure function: State × Inputs → State'
pub fn execute_frame(
    state: &GameState,
    inputs: &HashMap<PlayerId, PlayerInput>,
) -> GameState {
    let mut new_state = state.clone();

    // Process inputs in deterministic order
    let mut sorted_players: Vec<_> = inputs.keys().collect();
    sorted_players.sort();  // Consistent ordering

    for player_id in sorted_players {
        let input = &inputs[player_id];
        apply_input(&mut new_state, *player_id, input);
    }

    // Advance physics, AI, etc - all deterministic
    new_state.tick();

    new_state
}

/// This function MUST be:
/// - Deterministic (no random(), no system time, no HashMap iteration)
/// - Pure (no side effects, no external state)
/// - Portable (same result on any CPU architecture)
```

---

## Post-Quantum Security

### Why Current Crypto Fails

```
Current (vulnerable to quantum):
├── ECDSA signatures - Shor's algorithm breaks it
├── RSA - Shor's algorithm breaks it
└── DH key exchange - Shor's algorithm breaks it

Post-quantum threats:
├── Forge signatures → impersonate players
├── Break commitments → see inputs before revealing
└── Decrypt communications → information advantage
```

### Post-Quantum Hardened Design

```rust
/// Post-quantum cryptographic primitives
pub mod crypto {
    /// Hash function: SHA-3 (Keccak)
    /// Quantum resistance: Grover's algorithm gives √N speedup
    /// 256-bit hash → 128-bit security post-quantum
    /// Use 384 or 512 for higher margins
    pub type Hash = Sha3_384;

    /// Commitments: Hash-based (inherently post-quantum)
    pub fn commit(data: &[u8], salt: &[u8]) -> [u8; 48] {
        let mut h = Hash::new();
        h.update(data);
        h.update(salt);
        h.finalize().into()
    }

    /// Signatures: SPHINCS+ (hash-based, stateless)
    /// - Based only on hash function security
    /// - No number-theoretic assumptions
    /// - Survives quantum computers
    pub type Signature = SphincsPlus256s;

    /// Key exchange: Kyber (lattice-based)
    /// - NIST post-quantum standard
    /// - Based on Learning With Errors
    pub type KeyExchange = Kyber1024;
}
```

### But Crypto Isn't The Point

Even with infinite compute, **the protocol design makes cheating impossible**:

```
Attack: "I'll break the hash and see your commitment before I commit"

Defense: Timing.
  - Commitment window closes at time T
  - All commitments collected before ANY reveals
  - Even with instant hash breaking, you can't go back in time
  - Your commitment was already locked

Attack: "I'll break signatures and submit fake inputs"

Defense: Consensus.
  - N players all saw the original commitment
  - Fake input doesn't match commitment hash
  - N-1 honest players reject your reveal
  - You don't get to execute

Attack: "I'll compute the game state faster and act on future info"

Defense: Irrelevance.
  - You committed before seeing others' inputs
  - Computing faster doesn't let you change your commitment
  - The future was fixed when you committed
```

---

## What Cheats Become Impossible

### Wallhacks: Impossible

```
WHY: Server only sends visible entities
     Client doesn't have enemy positions behind walls
     Can't display what doesn't exist in memory

IMPLEMENTATION:
  Server computes visibility per player
  Only transmits entities in line-of-sight
  Client renders what it receives
  No hidden data to extract
```

### Aimbots: Ineffective

```
WHY: Commit-reveal means you aim BEFORE seeing enemy move
     Enemy committed their dodge before you committed your shot
     Aimbot can only aim at where enemy WAS, not where they'll BE

IMPLEMENTATION:
  Shooting is input: commit(aim_direction)
  Dodging is input: commit(move_direction)
  Neither knows other's choice when committing
  Aimbot has no advantage
```

### Speed Hacks: Impossible

```
WHY: Movement is computed from inputs, not client-claimed position
     You submit input: "move_forward"
     Server/peers compute: position += speed × direction
     Claiming different position rejected by all other clients

IMPLEMENTATION:
  Client never sends position
  Client only sends input commands
  All parties compute position from inputs
  Disagreement = your reality is ignored
```

### Teleportation: Impossible

```
WHY: Position is derived, not transmitted
     Can't claim to be somewhere without input history to get there
     Every position must have a valid input path from spawn

IMPLEMENTATION:
  Full input history is part of state
  Position = f(spawn_point, input_history)
  No input sequence produces teleportation
  Invalid position doesn't exist in consensus reality
```

### Damage Hacks: Impossible

```
WHY: Damage is computed from game rules, not client claims
     "I hit for 1 million damage" doesn't mean anything
     Damage = weapon.base × modifiers × hit_location
     All parties compute same damage from same inputs

IMPLEMENTATION:
  Combat is input: "attack with weapon X in direction Y"
  Damage calculation is deterministic game logic
  All clients compute same result
  Hacked damage value rejected by consensus
```

### Inventory Hacks: Impossible

```
WHY: Inventory state is derived from action history
     Every item has provenance: how you got it
     Can't spawn items without valid acquisition event

IMPLEMENTATION:
  Inventory = accumulation of loot events
  Each loot event is in the ledger
  Item exists iff valid loot event exists
  Forging loot event requires forging history
  History is committed and witnessed by N parties
```

---

## The Remaining Attack Surface

What's still possible with this design:

### 1. Denial of Service
```
Attack: Don't submit inputs, stall the game
Mitigation: Timeout → null input, game continues without you
```

### 2. Collusion
```
Attack: Multiple accounts share information side-channel
Mitigation: No hidden information to share (visibility culling)
            Commit-reveal means even collusion can't react to hidden info
```

### 3. Client Manipulation (Self-Harm Only)
```
Attack: Modified client shows wrong information to YOU
Reality: You only hurt yourself
         Other players unaffected
         Your actions still follow your committed inputs
```

### 4. Physical Hardware
```
Attack: Faster reflexes via physical augmentation
Reality: Not a software problem
         Commit-reveal limits reaction advantage anyway
         Competitive integrity is about fair play, not raw speed
```

---

## Implementation Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                   CHEAT-PROOF GAME LOOP                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Time ──────────────────────────────────────────────────────▶   │
│                                                                  │
│  Frame N:                                                       │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │ COMMIT PHASE (0-100ms)                                  │    │
│  │                                                          │    │
│  │  All players:                                           │    │
│  │  1. Decide input based on current visible state         │    │
│  │  2. Generate random salt                                │    │
│  │  3. Create commitment = hash(input || salt)             │    │
│  │  4. Broadcast commitment to all peers                   │    │
│  │  5. Wait for all commitments                            │    │
│  │                                                          │    │
│  │  Window closes → commitments locked                     │    │
│  └─────────────────────────────────────────────────────────┘    │
│                            │                                     │
│                            ▼                                     │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │ REVEAL PHASE (100-150ms)                                │    │
│  │                                                          │    │
│  │  All players:                                           │    │
│  │  1. Broadcast (input, salt) reveal                      │    │
│  │  2. Verify all reveals match commitments                │    │
│  │  3. Invalid reveals → null input for that player        │    │
│  │                                                          │    │
│  │  All inputs now known and verified                      │    │
│  └─────────────────────────────────────────────────────────┘    │
│                            │                                     │
│                            ▼                                     │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │ EXECUTE PHASE (150-200ms)                               │    │
│  │                                                          │    │
│  │  All clients (in parallel):                             │    │
│  │  1. Execute deterministic simulation step               │    │
│  │  2. Apply all verified inputs                           │    │
│  │  3. Compute new game state                              │    │
│  │  4. Hash new state (optional: broadcast for verify)     │    │
│  │                                                          │    │
│  │  All clients now have identical state                   │    │
│  └─────────────────────────────────────────────────────────┘    │
│                            │                                     │
│                            ▼                                     │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │ RENDER PHASE (local only)                               │    │
│  │                                                          │    │
│  │  Each client:                                           │    │
│  │  1. Render from verified state                          │    │
│  │  2. Interpolate between frames for smoothness           │    │
│  │  3. Accept user input for NEXT frame's commit           │    │
│  │                                                          │    │
│  └─────────────────────────────────────────────────────────┘    │
│                            │                                     │
│                            ▼                                     │
│                      Frame N+1...                               │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

---

## Latency Considerations

### The Tradeoff

```
Commit-reveal adds latency:
  - Must wait for all commitments (~50-100ms)
  - Must wait for all reveals (~50ms)
  - Total: 100-150ms input delay

Traditional games: 0-50ms input delay
Cheat-proof games: 100-200ms input delay
```

### Mitigation Strategies

```
1. INPUT PREDICTION
   - Client predicts own movement immediately
   - Corrects when reveals come in
   - Feels responsive for player's own character
   - Others appear slightly delayed (acceptable)

2. VARIABLE TICK RATE
   - Strategic games: 5-10 ticks/sec (fine with delay)
   - Action games: 20-30 ticks/sec (needs prediction)
   - Survival games (Roanoke): 10-20 ticks/sec

3. COMMITMENT WINDOWS
   - Short windows for small groups (50ms)
   - Longer windows for large matches (100-200ms)

4. REGIONAL MATCHING
   - Lower latency players together
   - Reduces commitment window needed
```

### For Roanoke Specifically

```
Survival/hunting gameplay:
├── Not a twitch shooter (100ms delay acceptable)
├── Strategic positioning matters more than reflexes
├── Commit-reveal fits the slower pace
└── Territory Wars: Turn-based elements natural

Combat encounters:
├── Attacks are commitments (wind-up animations)
├── Defense is prediction (not reaction)
├── Adds tactical depth, not frustration
└── "I committed to dodge left" is a meaningful choice
```

---

## Determinism Requirements

For this to work, the simulation MUST be deterministic:

```rust
/// FORBIDDEN: Non-deterministic operations
fn bad_simulation() {
    let x = rand::random();              // NO: Different per client
    let t = SystemTime::now();            // NO: Different per client
    let h = HashMap::new();               // NO: Iteration order varies
    let f = some_value as f32 + 0.1;      // NO: Float rounding varies by CPU
    let p = some_pointer as usize;        // NO: Addresses differ
}

/// REQUIRED: Fully deterministic operations
fn good_simulation(rng_state: &mut u64) {
    let x = deterministic_rng(rng_state); // YES: Seeded, reproducible
    let t = game_tick_counter;            // YES: Derived from inputs
    let h = BTreeMap::new();              // YES: Ordered iteration
    let f = FixedPoint::from_bits(x);     // YES: Fixed-point math
    // No pointers in game logic
}
```

### Fixed-Point Math

```rust
/// Fixed-point number with 16 fractional bits
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Fixed(i64);

impl Fixed {
    pub const ONE: Fixed = Fixed(1 << 16);
    pub const ZERO: Fixed = Fixed(0);

    pub fn from_int(n: i32) -> Self {
        Fixed((n as i64) << 16)
    }

    pub fn from_f32(f: f32) -> Self {
        Fixed((f * 65536.0) as i64)
    }

    pub fn to_f32(self) -> f32 {
        (self.0 as f32) / 65536.0
    }
}

impl std::ops::Add for Fixed {
    type Output = Fixed;
    fn add(self, rhs: Fixed) -> Fixed {
        Fixed(self.0 + rhs.0)  // Exact, no rounding
    }
}

impl std::ops::Mul for Fixed {
    type Output = Fixed;
    fn mul(self, rhs: Fixed) -> Fixed {
        Fixed((self.0 * rhs.0) >> 16)  // Deterministic rounding
    }
}
```

---

## Summary: The Cheat-Proof Guarantee

```
┌─────────────────────────────────────────────────────────────────┐
│                                                                  │
│   GIVEN:                                                        │
│   ├── All inputs are committed before revealed                  │
│   ├── Execution is deterministic                                │
│   ├── Visibility is server-controlled                           │
│   └── State is derived from input history                       │
│                                                                  │
│   THEN:                                                         │
│   ├── Wallhacks: Impossible (no hidden data exists locally)     │
│   ├── Aimbots: Ineffective (commit before seeing enemy move)    │
│   ├── Speed hacks: Impossible (position is computed, not sent)  │
│   ├── Teleport: Impossible (no input sequence produces it)      │
│   ├── Damage hacks: Impossible (damage is computed by all)      │
│   └── Item spawning: Impossible (no valid acquisition event)    │
│                                                                  │
│   REGARDLESS OF:                                                │
│   ├── Computational power (even post-quantum)                   │
│   ├── Memory reading (no hidden data to read)                   │
│   ├── Client modification (only hurts yourself)                 │
│   └── Network manipulation (commitments are witnessed)          │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

**Cheating isn't caught. It's not punished. It's structurally impossible.**

---

*This is the target architecture. Implementation requires deterministic simulation as a prerequisite.*
