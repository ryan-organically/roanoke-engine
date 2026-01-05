# Multiplayer Architecture at Scale

## The Problem

**Target:** Millions of concurrent users
**Constraint:** Keep costs sustainable
**Requirement:** Secure enough for wagered play
**Philosophy:** Blaze trails, don't just copy existing solutions

---

## Cost Analysis: Traditional vs. Novel

### Traditional Dedicated Servers (What Everyone Does)

```
1 million concurrent players
├── 50 players per match = 20,000 active matches
├── 1 server per match = 20,000 server instances
├── Cheapest viable (2 vCPU, 4GB) = ~$0.05/hour
├── Average match = 1 hour
└── Cost = $1,000/hour = $720,000/month

At 10M concurrent: $7.2M/month
```

**This is why Fortnite/PUBG need billions in revenue.**

### What If We Didn't Need That?

```
Hybrid P2P + Validation
├── Players host their own matches (casual/free)
├── Edge validators spot-check for cheating
├── Dedicated servers ONLY for wagered play
└── Cost = 95% reduction for casual, full security for money

At 1M concurrent (95% casual, 5% wagered):
├── 950K in P2P matches = $0 server cost
├── 50K in wagered matches = 1,000 servers = $50/hour
└── Total = $36,000/month (vs $720,000)
```

---

## Architecture Options

### Option 1: Hierarchical Trust Tiers

Different infrastructure for different stakes.

```
┌─────────────────────────────────────────────────────────────────┐
│                        TRUST TIERS                               │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  TIER 0: Local/LAN (Current scaffold)                          │
│  ├── Player-hosted                                              │
│  ├── No validation                                              │
│  ├── Cost: $0                                                   │
│  └── Use case: Testing, friends, offline                        │
│                                                                  │
│  TIER 1: Casual Public                                          │
│  ├── Player-hosted with matchmaking                             │
│  ├── Reputation system (good hosts rise, bad hosts sink)        │
│  ├── Optional: Spectator validation                             │
│  ├── Cost: ~$0.001 per match (matchmaking API only)             │
│  └── Use case: Free public play, no stakes                      │
│                                                                  │
│  TIER 2: Ranked/Competitive                                     │
│  ├── Edge-hosted (Cloudflare Workers / Fly.io)                  │
│  ├── Server-authoritative positions                             │
│  ├── Anti-cheat validation                                      │
│  ├── Cost: ~$0.01 per match                                     │
│  └── Use case: Leaderboards, rankings, reputation               │
│                                                                  │
│  TIER 3: Wagered Play                                           │
│  ├── Dedicated match servers (isolated VMs)                     │
│  ├── Full server authority                                      │
│  ├── Match recording for disputes                               │
│  ├── Escrow integration                                         │
│  ├── Cost: ~$0.10 per match                                     │
│  └── Use case: Seed Wars, tournaments, real money               │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

**Pros:** Scales costs with stakes. Most players never cost you server money.
**Cons:** Complex. Multiple codepaths.

---

### Option 2: Deterministic Lockstep + Fraud Proofs

Every client runs identical simulation. Cheating is mathematically detectable.

```
┌─────────────────────────────────────────────────────────────────┐
│                   DETERMINISTIC LOCKSTEP                         │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  All clients receive same inputs → Must produce same outputs    │
│                                                                  │
│  Frame 0: Seed=12345, Players spawn at deterministic positions  │
│  Frame 1: Player A presses W → All clients simulate A moving    │
│  Frame 2: Player B presses Space → All clients simulate B jump  │
│  ...                                                             │
│                                                                  │
│  Every N frames: Clients broadcast state hash                   │
│  If hashes diverge → Someone is cheating or desynced            │
│                                                                  │
│  ┌─────────┐    ┌─────────┐    ┌─────────┐                      │
│  │Client A │    │Client B │    │Client C │                      │
│  │Hash: 7f3│    │Hash: 7f3│    │Hash: 7f3│  ✓ All match         │
│  └─────────┘    └─────────┘    └─────────┘                      │
│                                                                  │
│  ┌─────────┐    ┌─────────┐    ┌─────────┐                      │
│  │Client A │    │Client B │    │Client C │                      │
│  │Hash: 7f3│    │Hash: 7f3│    │Hash: a19│  ✗ C is cheating     │
│  └─────────┘    └─────────┘    └─────────┘                      │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

```rust
/// Deterministic game state that can be hashed
#[derive(Hash, Clone)]
pub struct DeterministicState {
    pub frame: u64,
    pub rng_state: u64,  // Seeded RNG, reproducible
    pub players: BTreeMap<PlayerId, PlayerState>,  // Ordered!
    pub entities: BTreeMap<EntityId, EntityState>,
}

impl DeterministicState {
    /// Advance simulation by one frame given inputs
    pub fn step(&mut self, inputs: &FrameInputs) {
        // MUST be deterministic:
        // - No floating point (use fixed-point)
        // - No HashMap iteration (use BTreeMap)
        // - No random() calls (use seeded RNG)
        // - No system time
    }

    /// Hash for comparison
    pub fn hash(&self) -> [u8; 32] {
        sha256(bincode::serialize(self))
    }
}
```

**Pros:**
- P2P with cheat detection
- No server compute needed
- Mathematically provable fairness

**Cons:**
- Requires fully deterministic simulation (hard with physics)
- Latency = input delay (waiting for all inputs)
- All clients must run full simulation

---

### Option 3: Optimistic Execution + Fraud Proofs (Rollup-style)

Assume everyone is honest. If someone proves fraud, penalize the cheater.

```
┌─────────────────────────────────────────────────────────────────┐
│              OPTIMISTIC EXECUTION (ROLLUP-STYLE)                 │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  1. Players submit signed inputs to relay                       │
│  2. Everyone executes locally, assumes others are honest        │
│  3. State commitments posted periodically                       │
│  4. Challenge period: Anyone can submit fraud proof             │
│  5. If fraud proven → Cheater loses stake/reputation            │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │                     INPUT RELAY                          │   │
│  │  (Cheap - just forwards signed messages, no compute)     │   │
│  └──────────────────────────────────────────────────────────┘   │
│           │                    │                    │            │
│           ▼                    ▼                    ▼            │
│     ┌──────────┐         ┌──────────┐         ┌──────────┐      │
│     │ Player A │         │ Player B │         │ Player C │      │
│     │ Executes │         │ Executes │         │ Executes │      │
│     │ locally  │         │ locally  │         │ locally  │      │
│     └──────────┘         └──────────┘         └──────────┘      │
│           │                    │                    │            │
│           └────────────────────┼────────────────────┘            │
│                                ▼                                 │
│                    ┌───────────────────────┐                    │
│                    │   STATE COMMITMENTS   │                    │
│                    │   (Posted on-chain    │                    │
│                    │    or to validator)   │                    │
│                    └───────────────────────┘                    │
│                                │                                 │
│                                ▼                                 │
│                    ┌───────────────────────┐                    │
│                    │   CHALLENGE PERIOD    │                    │
│                    │   Anyone can prove    │                    │
│                    │   fraud with replay   │                    │
│                    └───────────────────────┘                    │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

**Pros:**
- Near-zero latency (optimistic execution)
- Minimal server cost (relay only)
- Cryptographic security for wagered play
- Could settle on-chain for trustless payouts

**Cons:**
- Complex implementation
- Challenge period delays final settlement
- Still needs deterministic simulation for fraud proofs

---

### Option 4: Edge Computing + Stateless Validation

Match servers at the edge (Cloudflare Workers, Fly.io, Deno Deploy).

```
┌─────────────────────────────────────────────────────────────────┐
│                    EDGE-FIRST ARCHITECTURE                       │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Cloudflare has 300+ edge locations worldwide                   │
│  Fly.io can spin up VMs in <500ms at any edge                   │
│                                                                  │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                    MATCHMAKING                           │    │
│  │              (Central, lightweight)                      │    │
│  └─────────────────────────────────────────────────────────┘    │
│                            │                                     │
│                            │ Players matched                     │
│                            ▼                                     │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │              EDGE LOCATION SELECTION                     │    │
│  │    Pick closest edge to all players (minimize ping)      │    │
│  └─────────────────────────────────────────────────────────┘    │
│                            │                                     │
│        ┌───────────────────┼───────────────────┐                │
│        ▼                   ▼                   ▼                │
│  ┌───────────┐       ┌───────────┐       ┌───────────┐         │
│  │ Edge: LAX │       │ Edge: FRA │       │ Edge: SIN │         │
│  │ (LA)      │       │ (Frankfurt)│      │ (Singapore)│         │
│  └───────────┘       └───────────┘       └───────────┘         │
│        │                                                         │
│        ▼                                                         │
│  ┌───────────────────────────────────────────────────────┐      │
│  │              STATELESS MATCH SERVER                    │      │
│  │  - Spins up in <1 second                              │      │
│  │  - Validates inputs, broadcasts state                 │      │
│  │  - Persists nothing (state in Redis/KV)               │      │
│  │  - Dies when match ends                               │      │
│  └───────────────────────────────────────────────────────┘      │
│                                                                  │
│  COST MODEL (Cloudflare Workers):                               │
│  - First 100K requests/day: FREE                                │
│  - $5/10M requests after that                                   │
│  - 1M concurrent × 20 msgs/sec = 20M msgs/sec                   │
│  - = ~$0.000005 per message                                      │
│  - Match (1 hour, 50 players, 20 msg/sec) = $0.18               │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

**Pros:**
- Low latency (edge locations worldwide)
- Pay-per-use (no idle servers)
- Scales automatically
- Server-authoritative (secure)

**Cons:**
- Cloudflare Workers have CPU limits (10-50ms)
- Need to offload heavy compute or use Fly.io for real VMs
- State management across stateless workers is tricky

---

### Option 5: Hybrid Mesh (The Trail-Blazer)

Combine the best ideas into something new.

```
┌─────────────────────────────────────────────────────────────────┐
│                     HYBRID MESH NETWORK                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  INSIGHT: Most computation is rendering. Game logic is cheap.   │
│                                                                  │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                 CENTRAL SERVICES (Minimal)               │    │
│  │  - Matchmaking                                           │    │
│  │  - Account/Auth                                          │    │
│  │  - Leaderboards                                          │    │
│  │  - Payment processing                                    │    │
│  │  Cost: Fixed, ~$500/month for millions of users          │    │
│  └─────────────────────────────────────────────────────────┘    │
│                            │                                     │
│                            ▼                                     │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                   MATCH FORMATION                        │    │
│  │  Players grouped by: region, skill, stake level          │    │
│  └─────────────────────────────────────────────────────────┘    │
│                            │                                     │
│           ┌────────────────┴────────────────┐                   │
│           ▼                                 ▼                    │
│  ┌─────────────────────┐          ┌─────────────────────┐       │
│  │   CASUAL MATCH      │          │   WAGERED MATCH     │       │
│  │                     │          │                     │       │
│  │  P2P Mesh + Relay   │          │  Edge Server        │       │
│  │                     │          │  (Fly.io/CF Worker) │       │
│  │  ┌───┐ ┌───┐ ┌───┐  │          │                     │       │
│  │  │ A │◄┼►│ B │◄┼►│ C │ │          │  Full authority    │       │
│  │  └───┘ └───┘ └───┘  │          │  Match recording    │       │
│  │    ▲     ▲     ▲    │          │  Escrow integration │       │
│  │    └─────┼─────┘    │          │                     │       │
│  │          ▼          │          │                     │       │
│  │  ┌──────────────┐   │          └─────────────────────┘       │
│  │  │ Relay Server │   │                                        │
│  │  │ (WebSocket)  │   │                                        │
│  │  │ No compute,  │   │                                        │
│  │  │ just forward │   │                                        │
│  │  └──────────────┘   │                                        │
│  │                     │                                        │
│  │  Validation:        │                                        │
│  │  - Hash checkpoints │                                        │
│  │  - Peer attestation │                                        │
│  │  - Reputation stake │                                        │
│  └─────────────────────┘                                        │
│                                                                  │
│  COST AT 1M CONCURRENT:                                         │
│  ├── 95% Casual (P2P + Relay): ~$5K/month                       │
│  ├── 5% Wagered (Edge servers): ~$10K/month                     │
│  ├── Central services: ~$500/month                              │
│  └── TOTAL: ~$15K/month (vs $720K traditional)                  │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

---

## The Trail-Blazer Concept: Peer Validation Staking

**Novel idea:** Players stake reputation (or small crypto amount) to validate others.

```rust
/// Players can volunteer as validators
pub struct ValidatorNode {
    pub player_id: PlayerId,
    pub stake: u64,              // Reputation points or tokens
    pub validated_matches: u32,
    pub disputes_against: u32,
    pub accuracy_score: f32,     // How often their validations hold up
}

/// During a match, random validators are selected
pub struct MatchValidation {
    pub match_id: MatchId,
    pub validators: Vec<ValidatorNode>,  // 3-5 random validators
    pub player_stakes: HashMap<PlayerId, u64>,
}

impl MatchValidation {
    /// Validators run simulation in background, compare hashes
    pub fn validate_frame(&self, frame: u64, player_hashes: &HashMap<PlayerId, Hash>) {
        // If all player hashes match: OK
        // If one differs: Flag for review
        // If validator catches cheater: Reward from cheater's stake
        // If validator false-flags: Lose stake
    }
}
```

**Economics:**
- Validators earn small rewards for honest validation
- Cheaters lose stake to validators who catch them
- False accusations cost the accuser
- Self-balancing: Cheating becomes unprofitable

---

## Recommended Architecture for Roanoke

### Phase 1: Foundation (Now → 3 months)
- Current WebSocket scaffold for local/LAN
- Add simple relay server for NAT traversal
- P2P mesh for casual play
- **Cost: ~$50/month for relay**

### Phase 2: Public Play (3-6 months)
- Matchmaking service
- Account system (simple JWT auth)
- Reputation tracking
- Hash-based cheat detection (deterministic core)
- **Cost: ~$200/month**

### Phase 3: Competitive (6-12 months)
- Edge servers for ranked play (Fly.io)
- Server-authoritative for ranked matches
- Leaderboards
- **Cost: ~$2K/month at 100K users**

### Phase 4: Wagered Play (12+ months)
- Dedicated match servers for Seed Wars
- Full match recording
- Payment integration (Stripe + crypto)
- Escrow smart contracts
- Peer validation staking
- **Cost: Scales with wagered volume (~2-5% of pot)**

---

## Key Technical Decisions

### 1. Deterministic Simulation

Required for fraud proofs and replay validation.

```rust
// Use fixed-point math, not floats
type Fixed = i64;  // 1.0 = 1_000_000

// Use seeded RNG
pub struct GameRng {
    state: u64,
}

impl GameRng {
    pub fn next(&mut self) -> u64 {
        // Xorshift or similar - same seed = same sequence
        self.state ^= self.state << 13;
        self.state ^= self.state >> 7;
        self.state ^= self.state << 17;
        self.state
    }
}

// Use ordered collections
pub type EntityMap = BTreeMap<EntityId, Entity>;  // NOT HashMap
```

### 2. Input-Only Networking

Clients send inputs, not positions. Server/peers compute positions.

```rust
#[derive(Serialize)]
pub struct FrameInput {
    pub frame: u64,
    pub player_id: PlayerId,
    pub input: PlayerInput,
    pub signature: Signature,  // Proves this player sent it
}

#[derive(Serialize)]
pub struct PlayerInput {
    pub move_dir: [i8; 2],     // -1, 0, or 1 for each axis
    pub look_delta: [i16; 2],  // Mouse movement
    pub actions: ActionFlags,  // Jump, attack, interact, etc.
}
```

### 3. Checkpoint Hashing

Periodic state hashes detect desync/cheating.

```rust
pub struct Checkpoint {
    pub frame: u64,
    pub state_hash: [u8; 32],
    pub player_hashes: HashMap<PlayerId, [u8; 32]>,
}

// Every 60 frames (~1 second), broadcast checkpoint
// If hashes don't match, investigate
```

---

## Questions to Answer

1. **How deterministic can we make the simulation?**
   - Physics engine? (Rapier is not deterministic across platforms)
   - Floating point? (Need fixed-point for cross-platform)

2. **What's the minimum viable trust model for early wagered play?**
   - Could start with "trusted host" (you run the servers)
   - Expand to peer validation later

3. **On-chain or off-chain for settlements?**
   - On-chain: Trustless, but gas fees eat small wagers
   - Off-chain: Cheaper, but requires trust in you
   - Hybrid: Off-chain with on-chain dispute resolution

4. **WebSocket or WebRTC for P2P?**
   - WebSocket: Simpler, needs relay
   - WebRTC: True P2P, complex NAT traversal

---

*This document is a living exploration. Update as we make decisions.*
