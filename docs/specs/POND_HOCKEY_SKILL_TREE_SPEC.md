# Pond Hockey Skill Tree Specification

## Overview

The Pond Hockey Skill Tree develops the player's ability to skate, handle a puck, and compete in stick-and-ball games on frozen ponds during winter. Beginning with basic ice movement, players progress through skating techniques, shooting abilities, and defensive skills. This is a **winter-only minigame** available when bodies of water freeze during cold snaps.

## Design Philosophy

"Shinny" or "bandy" — stick-and-ball games played on ice — existed in various forms among European colonists and were quickly adopted by Native peoples. In the harsh winters of the colonial frontier, frozen ponds became gathering places where rivalries were settled, alliances formed, and boredom was beaten back with wooden sticks and frozen leather balls. This skill tree reflects both the rough-and-tumble nature of frontier sport and the technical mastery that separates casual players from legends.

---

## Skill Tree Structure

```
                              [FROZEN LEGEND]
                                     |
                +--------------------+--------------------+
                |                                         |
         [Ice General]                             [Netminder Supreme]
                |                                         |
      +---------+---------+                     +---------+---------+
      |                   |                     |                   |
[Sniper Elite]     [Playmaker]           [Iron Curtain]      [Crease Master]
      |                   |                     |                   |
      +---------+---------+                     +---------+---------+
                |                                         |
         [Offensive Dynamo]                        [Guardian]
                |                                         |
                +--------------------+--------------------+
                                     |
                              [Complete Player]
                                     |
                +--------------------+--------------------+
                |                                         |
         [Shooting Form]                          [Defensive Stance]
                |                                         |
      +---------+---------+                     +---------+---------+
      |                   |                     |                   |
[One-Timer]        [Top Shelf]           [The Wall]        [Stick Lift]
      |                   |                     |                   |
      +---------+---------+                     +---------+---------+
                |                                         |
         [Sharp Shooter]                          [Solid Defender]
                |                                         |
                +--------------------+--------------------+
                                     |
                              [SKATING FOUNDATION]
                               (Starting Point)
```

---

## Tier 1: Foundation

### Ice Legs (Starting Skill)
**Unlock:** Step onto frozen pond for the first time
**Description:** Basic ability to remain upright and move on ice without falling.

**Effects:**
- Can move on ice at 50% normal walking speed
- 25% chance to fall when changing direction quickly
- Can pick up and carry a puck/ball
- Basic stick control (can push puck forward)

**Movement Mechanics:**
| Action | Effect |
|--------|--------|
| Walking | 50% speed, stable |
| Running attempt | 60% speed, 15% fall chance |
| Quick turn | Must stop first, 25% fall chance |
| Stopping | Slow slide, 2 second recovery |

---

## Tier 2: Basic Skating

### Steady Stride
**Unlock:** Skate 500m total on ice without falling
**Prerequisite:** Ice Legs
**Description:** Comfortable gliding motion across the ice.

**Effects:**
- Skating speed increased to 75% of running speed
- Fall chance on direction change reduced to 10%
- Can maintain speed while carrying puck
- Smooth acceleration (no stumbling on start)

**Stride Mechanics:**
- Push-glide motion feels natural
- Can recover from minor bumps
- Stamina drain reduced 20% on ice

---

### Quick Stops
**Unlock:** Successfully stop 20 times without falling
**Prerequisite:** Ice Legs
**Description:** The hockey stop — spraying ice while coming to a controlled halt.

**Effects:**
- Can stop instantly from full speed
- No fall chance on stopping
- Creates ice spray visual effect
- Can immediately change direction after stop
- +15% evasion when being chased

**Stop Types:**
| Stop | Speed Required | Effect |
|------|----------------|--------|
| Snowplow | Any | Slow, stable stop |
| Hockey Stop | >50% | Fast stop, ice spray |
| One-foot Stop | >75% | Fastest, +style points |

---

## Tier 3: Movement Mastery

### Crossover Turn
**Unlock:** Complete 50 tight turns without falling
**Prerequisites:** Steady Stride AND Quick Stops
**Description:** The skating crossover — feet crossing over each other for tight, fast turns.

**Effects:**
- +40% turning speed while skating
- No speed loss during turns
- Can turn in tight circles while maintaining momentum
- Unlocks spin move (360° turn)

**Turn Radius:**
| Skill Level | Turn Radius | Speed Maintained |
|-------------|-------------|------------------|
| No skill | 10m | 50% |
| With Crossover | 3m | 100% |
| Mastered | 1.5m | 100% |

---

### Backward Skate
**Unlock:** Skate backward for 100m total
**Prerequisites:** Steady Stride AND Quick Stops
**Description:** Confident movement in reverse — essential for defense.

**Effects:**
- Can skate backward at 60% forward speed
- Can transition smoothly forward-to-back
- Can track puck carrier while retreating
- +25% to block/steal attempts when skating backward

**Defensive Positioning:**
- Face attacker while retreating
- Maintain gap control
- Ready to pivot and chase

---

## Tier 4: Intermediate Skills

### Edge Master
**Unlock:** Win 3 races around the pond circuit
**Prerequisites:** Crossover Turn AND Backward Skate
**Description:** Complete control of skate edges for maximum agility.

**Effects:**
- Skating speed increased to 100% of running speed
- Can cut at any angle without slowing
- Perfect balance — fall chance eliminated
- Unlocks edge moves (dekes, fakes)
- +30% agility on ice

**Edge Techniques:**
| Technique | Effect |
|-----------|--------|
| Inside Edge Cut | Sharp inward turn |
| Outside Edge Glide | Wide, sweeping arc |
| Edge Fake | Fake one direction, go another |
| Mohawk Turn | Instant forward/backward switch |

---

### Phantom Stride
**Unlock:** Score a goal without being touched by defenders
**Prerequisites:** Crossover Turn AND Backward Skate
**Description:** Silent, ghostly movement across the ice.

**Effects:**
- Skating is nearly silent
- +25% stealth rating on ice
- Can approach from blind spots undetected
- AI defenders slower to react to your movement
- Unlocks "Ghost" skating animation (low, smooth)

**Stealth Skating:**
- No blade scraping sounds
- Lower body position
- Opponents lose track of you in chaos

---

## Tier 5: Core Competency

### Skating Foundation Complete
**Unlock:** All Tier 4 skating skills
**Prerequisites:** Edge Master AND Phantom Stride
**Description:** You have mastered movement on ice. Now choose your specialty.

**Effects:**
- All skating bonuses stack
- Maximum ice speed achieved
- Unlock Offensive and Defensive branches
- Title: "Ice Dancer"

**Passive Bonuses:**
| Stat | Bonus |
|------|-------|
| Ice Speed | +50% |
| Turn Speed | +40% |
| Stamina on Ice | -30% drain |
| Fall Chance | 0% |
| Direction Change | Instant |

---

## Tier 6: Offensive Branch

### Sharp Shooter
**Unlock:** Score 10 goals
**Prerequisite:** Skating Foundation Complete
**Description:** Developing accuracy and power in your shots.

**Effects:**
- Shot accuracy +25%
- Shot power +20%
- Can aim for specific targets (corners, five-hole)
- Unlock shot types: Wrist, Snap, Slap

**Shot Types:**
| Shot | Power | Accuracy | Release |
|------|-------|----------|---------|
| Wrist Shot | Medium | High | Fast |
| Snap Shot | Medium-High | Medium | Medium |
| Slap Shot | Very High | Low | Slow |

---

### Solid Defender
**Unlock:** Block 20 shots or steal puck 15 times
**Prerequisite:** Skating Foundation Complete
**Description:** Understanding defensive positioning and puck denial.

**Effects:**
- +20% chance to block shots
- +25% chance to steal puck
- Can body check without fouling (timing based)
- Poke check range +30%

**Defensive Techniques:**
| Technique | Success Rate | Effect |
|-----------|--------------|--------|
| Poke Check | Base 40% | Knock puck away |
| Stick Lift | Base 35% | Lift opponent's stick |
| Body Check | Base 50% | Separate player from puck |
| Shot Block | Base 30% | Sacrifice body |

---

## Tier 7: Specialization

### One-Timer
**Unlock:** Score 3 goals directly off passes
**Prerequisite:** Sharp Shooter
**Description:** Shooting passes without stopping the puck first.

**Effects:**
- Can shoot incoming passes immediately
- +50% shot power on one-timers
- Reduces goalie reaction time
- Timing window: 0.5 seconds after pass received

**One-Timer Mechanics:**
- Requires pass from teammate
- Must be in shooting stance
- Timing-based mini-game
- Perfect timing = unstoppable shot

---

### Top Shelf
**Unlock:** Score 5 goals in the upper corners of net
**Prerequisite:** Sharp Shooter
**Description:** Precision shooting to the hardest-to-reach areas.

**Effects:**
- +40% accuracy on high shots
- Upper corner shots +30% more likely to score
- Can "pick corners" consistently
- Unlocks "snipe" celebration

**Target Zones:**
| Zone | Difficulty | Goal Chance |
|------|------------|-------------|
| Five Hole | Medium | +20% |
| Low Corners | Medium | +15% |
| High Corners | Hard | +30% with skill |
| Top Shelf | Very Hard | +40% with skill |

---

### Stick Lift
**Unlock:** Perform 25 successful stick lifts
**Prerequisite:** Solid Defender
**Description:** Clean defensive play that lifts the opponent's stick without fouling.

**Effects:**
- Stick lift success rate +35%
- Can perform stick lifts while skating backward
- No foul calls on stick lifts
- Opponent loses puck control for 2 seconds

**Timing Windows:**
| Opponent Action | Stick Lift Success |
|-----------------|-------------------|
| Carrying puck | 60% |
| Preparing shot | 75% |
| Receiving pass | 80% |
| Post-deke | 90% |

---

### The Wall
**Unlock:** Allow 0 goals while playing defense for 3 games
**Prerequisite:** Solid Defender
**Description:** Impenetrable defensive presence — attackers rarely get past.

**Effects:**
- +50% puck steal chance in your zone
- Opponents -30% speed when near you
- Body checks always succeed
- Intimidation aura (AI plays more cautiously)

**Wall Presence:**
- Control the blue line
- Force bad shot angles
- Protect the goalie
- Clear rebounds instantly

---

## Tier 8: Advanced Specialization

### Sniper Elite
**Unlock:** Max both One-Timer AND Top Shelf
**Description:** Your shots are lethal — goalies fear you.

**Effects:**
- All shots +40% accuracy
- All shots +35% power
- Can score from anywhere on ice
- "Clapper" shot unlocked (maximum power slap shot)
- Title: "The Sniper"

**Sniper Abilities:**
| Ability | Effect | Cooldown |
|---------|--------|----------|
| Called Shot | Choose exact goal location | 2 min |
| Laser Wrist | Unhittable wrist shot | 3 min |
| The Clapper | Devastating slap shot | 5 min |

---

### Playmaker
**Unlock:** Assist on 20 goals
**Prerequisites:** Sharp Shooter + 15 assists minimum
**Description:** Vision and passing ability that creates opportunities.

**Effects:**
- Passing accuracy +50%
- Can see teammate positions through chaos
- Saucer passes (puck lifts over sticks)
- No-look passes (deceptive)
- Title: "The Setup Man"

**Passing Types:**
| Pass | Speed | Deception | Best Use |
|------|-------|-----------|----------|
| Direct | Fast | None | Open ice |
| Saucer | Medium | Medium | Over sticks |
| No-Look | Medium | High | Surprise plays |
| Bank Pass | Variable | Low | Around defenders |
| Drop Pass | Slow | Very High | Trailing player |

---

### Iron Curtain
**Unlock:** Max both Stick Lift AND The Wall
**Description:** When you're on defense, nothing gets through.

**Effects:**
- +60% block/steal chance
- Can defend 2v1 situations effectively
- Puck carrier must pass (can't get by you)
- Gap control is perfect
- Title: "The Shutdown"

**Iron Curtain Abilities:**
| Ability | Effect | Cooldown |
|---------|--------|----------|
| Lockdown | Target cannot pass or shoot for 3s | 2 min |
| Clear the Crease | Push all attackers away from net | 3 min |
| Intercept | Guarantee next pass is stolen | 5 min |

---

### Crease Master
**Unlock:** Block 50 shots as goaltender
**Prerequisites:** Solid Defender + 10 goalie games
**Description:** Goaltending fundamentals mastered.

**Effects:**
- Save percentage +30%
- Faster glove hand
- Better rebound control
- Can play the puck safely
- Title: "The Tender"

**Goalie Techniques:**
| Technique | Effect |
|-----------|--------|
| Glove Save | Catch high shots |
| Pad Stack | Butterfly save position |
| Poke Check | Aggressive stick poke |
| Cover Up | Freeze puck for stoppage |

---

## Tier 9: Mastery

### Ice General
**Unlock:** Complete both Sniper Elite AND Playmaker OR Iron Curtain
**Description:** You control the flow of the game from either end.

**Effects:**
- All offensive stats maximized
- Can call plays (AI teammates follow commands)
- Shift momentum to your team
- +20% team performance when on ice
- Title: "The General"

**Leadership Abilities:**
| Ability | Effect | Cooldown |
|---------|--------|----------|
| Rally | Team speed +20% for 30s | 5 min |
| Set Play | Teammates run choreographed play | 3 min |
| Captain's Goal | Next shot is +50% more likely to score | 10 min |

---

### Netminder Supreme
**Unlock:** Complete both Iron Curtain AND Crease Master
**Description:** Goaltending perfected — you are the last line and nothing passes.

**Effects:**
- Save percentage +50%
- Rebounds always controlled
- Can see shots in slow motion (brief)
- Shutouts earn double XP
- Title: "The Brick Wall"

**Supreme Goalie Abilities:**
| Ability | Effect | Cooldown |
|---------|--------|----------|
| Flash Glove | Guaranteed save on next shot | 3 min |
| Fortress Mode | +75% save chance for 10s | 5 min |
| Psyche Out | Shooter -50% accuracy | 2 min |

---

## Tier 10: Legendary Mastery

### Frozen Legend
**Unlock:** Complete BOTH Ice General AND Netminder Supreme
**Description:** Songs are sung of your pond hockey exploits. You have transcended the game.

**Effects:**
- All hockey skills at maximum effectiveness
- Can play any position at elite level
- Attract crowds to watch your games
- Legendary matches can spawn (challenge games)
- Title: "Frozen Legend" visible to all

**Spirit Bonuses:**
| Bonus | Effect |
|-------|--------|
| Legend's Presence | All teammates +30% when you play |
| Highlight Reel | Spectacular plays shown in slow-mo |
| Ice Memory | Frozen ponds remember you (permanent stat boost there) |
| Winter's Favor | Weather never degrades your play |

**Legendary Companion: Rink Dog**
- A loyal hound that retrieves stray pucks
- Barks to distract opponents (brief confusion)
- Passive: +10% stamina regen on ice
- Keeps you warm between games

**Ultimate Ability: Frozen Moment**
- Time slows for 3 seconds during critical plays
- Can line up perfect shot or save
- 1 use per game
- Recharges after scoring or saving a goal

---

## Point Acquisition

| Action | Points |
|--------|--------|
| First time on ice | 25 |
| Complete a game | 20 |
| Goal scored | 15 |
| Assist | 10 |
| Save made (goalie) | 8 |
| Clean steal | 5 |
| Shot blocked | 6 |
| Game won | 25 |
| Hat trick (3+ goals) | 40 |
| Shutout (goalie) | 50 |
| Perfect game (win 10-0+) | 100 |
| Play during blizzard | 20 |
| Overtime winner | 35 |
| Comeback victory (down 3+) | 60 |

**Points Required Per Tier:**
- Tier 1 → Tier 2: 50 points
- Tier 2 → Tier 3: 100 points
- Tier 3 → Tier 4: 175 points
- Tier 4 → Tier 5: 275 points
- Tier 5 → Tier 6: 400 points
- Tier 6 → Tier 7: 550 points
- Tier 7 → Tier 8: 750 points
- Tier 8 → Tier 9: 1000 points
- Tier 9 → Tier 10: 1400 points

---

## Stick Handling Skills

Separate from the main tree, stick handling develops through use:

### Puck Control Progression

| Level | Ability | Unlock Condition |
|-------|---------|------------------|
| 1 | Basic Carry | Automatic |
| 2 | Forehand Control | Carry 500m |
| 3 | Backhand Control | 100 backhand touches |
| 4 | Toe Drag | Evade 20 defenders |
| 5 | Dangle | Deke past 30 defenders |
| 6 | Spin-o-rama | Complete 20 spins with puck |
| 7 | The Magician | Score 10 deke goals |

**Deke Moves:**
| Move | Difficulty | Effect |
|------|------------|--------|
| Forehand-Backhand | Easy | Shift puck side to side |
| Toe Drag | Medium | Pull puck back, evade |
| Between-the-Legs | Hard | Puck goes between legs |
| Spin Move | Medium | 360° while keeping puck |
| The Michigan | Very Hard | Lacrosse-style scoop goal |

---

## Winter Availability System

### Ice Formation Conditions

**Temperature Requirements:**
| Condition | Effect |
|-----------|--------|
| 32°F (0°C) | Ponds begin to freeze (surface only) |
| 28°F (-2°C) for 3 days | Ice safe for skating |
| 20°F (-7°C) for 5 days | Optimal ice conditions |
| Below 10°F (-12°C) | Ice becomes very fast |

**Ice Quality States:**
| Quality | Speed Modifier | Danger |
|---------|----------------|--------|
| Fresh Freeze | 0.9x | Thin ice cracks |
| Standard | 1.0x | None |
| Cold Snap | 1.15x | None |
| Deep Freeze | 1.25x | Frostbite risk |
| Thawing | 0.75x | Ice holes, drowning |

### Ice Degradation

- Above 32°F: Ice softens, speed -25%
- Above 35°F: Cracks appear, danger zones spawn
- Above 38°F: Holes form, pond unsafe
- Above 40°F: Pond unplayable until next freeze

### Pond Locations

Procedurally spawned near water bodies:
- Small ponds: 2v2 max
- Medium ponds: 3v3 max
- Large ponds: 5v5 max
- Frozen lake sections: 6v6 max

---

## Game Modes

### Pickup Game (Default)
- Join or start a game at any frozen pond
- NPCs can be teammates or opponents
- First to 5 goals wins
- No goalies required

### Organized Match
- 3v3 or larger
- Dedicated goalies
- Three 5-minute periods
- Penalties for fouls

### Challenge Match
- 1v1 shootout
- Best of 5 attempts
- Wager items or reputation
- Unlocks at Tier 4

### Legendary Match
- Special opponents spawn at max skill
- Historic players from the past
- Unique rewards for victory
- Extremely difficult

---

## Cross-System Integration

### Hunting Skill Synergies
| Hunting Skill | Hockey Bonus |
|---------------|--------------|
| Shadow Hunter | +15% ice stealth |
| Predator Sense | Better awareness of opponents behind you |
| Wolf Tracker | Team coordination bonus |

### Horse Training Synergies
| Horse Skill | Hockey Bonus |
|-------------|--------------|
| Agility 5+ | +10% turn speed on ice |
| Speed training | +5% skating speed |
| Balance training | -10% fall chance (early tiers) |

### Weather System Integration
| Weather | Hockey Effect |
|---------|---------------|
| Clear Cold | Optimal conditions |
| Snow | Reduced visibility, slower puck |
| Blizzard | Extreme conditions, +50% XP |
| Sleet | Dangerous ice, +25% fall chance |

### Faction Integration
| Faction Standing | Effect |
|------------------|--------|
| Friendly with natives | Learn traditional shinny variations |
| Colonial friendships | Access to better stick crafting |
| Trading post access | Buy/sell hockey equipment |

### NPC Integration
- NPCs have varying skill levels
- Some NPCs are ringers (very good)
- Betting NPCs offer wagers
- Champion NPCs guard legendary matches

---

## Equipment

### Sticks
| Stick Type | Speed | Power | Control | Durability |
|------------|-------|-------|---------|------------|
| Crude Branch | 0.8x | 0.7x | 0.6x | Low |
| Carved Stick | 1.0x | 1.0x | 1.0x | Medium |
| Ash Stick | 1.1x | 1.0x | 1.15x | High |
| Hickory Stick | 1.0x | 1.2x | 1.0x | Very High |
| Legendary Stick | 1.2x | 1.2x | 1.2x | Unbreakable |

### Pucks/Balls
| Type | Speed | Bounce | Visibility |
|------|-------|--------|------------|
| Leather Ball | 0.9x | High | Medium |
| Frozen Dung | 0.8x | Low | Low |
| Carved Wood Puck | 1.0x | Medium | High |
| Stone Puck | 1.1x | None | High |

### Protective Gear
| Gear | Effect |
|------|--------|
| Leather Gloves | -25% frostbite chance |
| Fur-Lined Boots | +10% grip on ice |
| Padded Leggings | -30% damage from checks |
| Knit Cap | +5 min cold tolerance |

---

## Crafting Recipes

### Basic Equipment
| Item | Materials | Effect |
|------|-----------|--------|
| Carved Stick | Hardwood + Knife | Standard hockey stick |
| Leather Ball | Leather + Twine | Standard puck |
| Practice Goal | 4 Branches + Rope | Set up anywhere |

### Advanced Equipment
| Item | Materials | Required Skill | Effect |
|------|-----------|----------------|--------|
| Ash Stick | Ash Wood + Carving Tools | Tier 4 | +15% control |
| Hickory Stick | Hickory + Carving Tools | Tier 6 | +20% power |
| Reinforced Gloves | Leather + Padding | Tier 3 | Cold protection |
| Champion's Stick | Legendary Wood + Master Crafting | Tier 9 | Best stick |

---

## Data Structures (Rust)

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PondHockeySkills {
    pub points: u32,

    // Tier 1
    pub ice_legs: bool,

    // Tier 2
    pub steady_stride: bool,
    pub quick_stops: bool,

    // Tier 3
    pub crossover_turn: bool,
    pub backward_skate: bool,

    // Tier 4
    pub edge_master: bool,
    pub phantom_stride: bool,

    // Tier 5
    pub skating_foundation: bool,

    // Tier 6
    pub sharp_shooter: bool,
    pub solid_defender: bool,

    // Tier 7
    pub one_timer: bool,
    pub top_shelf: bool,
    pub stick_lift: bool,
    pub the_wall: bool,

    // Tier 8
    pub sniper_elite: bool,
    pub playmaker: bool,
    pub iron_curtain: bool,
    pub crease_master: bool,

    // Tier 9
    pub ice_general: bool,
    pub netminder_supreme: bool,

    // Tier 10
    pub frozen_legend: bool,

    // Companion
    pub rink_dog: Option<RinkDog>,

    // Tracking
    pub puck_control_level: u8,
    pub goals_scored: u32,
    pub assists: u32,
    pub saves_made: u32,
    pub games_won: u32,
    pub games_played: u32,
    pub shutouts: u32,
    pub hat_tricks: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RinkDog {
    pub name: String,
    pub loyalty: f32,
    pub tricks_known: Vec<DogTrick>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum DogTrick {
    FetchPuck,
    Bark,
    Celebrate,
    WarmPlayer,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ShotType {
    WristShot,
    SnapShot,
    SlapShot,
    OneTimer,
    Backhand,
    TheMichigan,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum DekeMove {
    ForehandBackhand,
    ToeDrag,
    BetweenTheLegs,
    SpinMove,
    ShoulderFake,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FrozenPond {
    pub position: Vec3,
    pub radius: f32,
    pub ice_quality: IceQuality,
    pub ice_thickness: f32,
    pub max_players: u8,
    pub current_game: Option<HockeyGame>,
    pub frozen_since: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum IceQuality {
    ThinIce,      // Dangerous
    FreshFreeze,  // Okay
    Standard,     // Good
    ColdSnap,     // Great
    DeepFreeze,   // Excellent but cold
    Thawing,      // Dangerous
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HockeyGame {
    pub game_id: u64,
    pub pond_id: u64,
    pub team_a: Vec<PlayerId>,
    pub team_b: Vec<PlayerId>,
    pub score_a: u8,
    pub score_b: u8,
    pub period: u8,
    pub time_remaining: f32,
    pub puck_position: Vec3,
    pub puck_holder: Option<PlayerId>,
    pub game_type: GameType,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum GameType {
    Pickup { first_to: u8 },
    Organized { periods: u8, period_length: f32 },
    Challenge { stakes: ChallengeStakes },
    Legendary { opponent_name: String },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChallengeStakes {
    pub wager_type: WagerType,
    pub amount: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum WagerType {
    Gold,
    Reputation,
    Item(ItemId),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HockeyStats {
    pub total_goals: u32,
    pub total_assists: u32,
    pub total_saves: u32,
    pub games_played: u32,
    pub games_won: u32,
    pub shutouts: u32,
    pub hat_tricks: u32,
    pub longest_win_streak: u32,
    pub current_win_streak: u32,
    pub overtime_winners: u32,
    pub comeback_victories: u32,
    pub blizzard_games: u32,
    pub legendary_victories: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HockeyEquipment {
    pub stick: Option<StickType>,
    pub puck: Option<PuckType>,
    pub gloves: Option<GloveType>,
    pub boots: Option<BootType>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum StickType {
    CrudeBranch,
    CarvedStick,
    AshStick,
    HickoryStick,
    LegendaryStick,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PuckType {
    LeatherBall,
    FrozenDung,
    WoodPuck,
    StonePuck,
}
```

---

## Audio/Visual Feedback

### On Ice
- Blade cutting ice sounds
- Puck sliding/bouncing sounds
- Stick-on-puck contact
- Crowd cheers (if spectators)
- Wind across frozen pond

### Skill Progression
- Level-up chime with winter bells
- New ability visual demonstration
- Title card for milestones
- Slow-motion on spectacular plays

### Weather Effects
- Blowing snow particles
- Reduced visibility in storms
- Ice cracking sounds near thin areas
- Breath vapor in cold

---

## Implementation Priority

### Phase 1 (Core)
- [ ] Pond freezing detection (temperature tracking)
- [ ] Basic skating movement on ice
- [ ] Puck physics on ice surface
- [ ] Simple goal detection

### Phase 2 (Gameplay)
- [ ] AI opponents and teammates
- [ ] Basic game rules (pickup mode)
- [ ] Shooting mechanics
- [ ] Saving mechanics

### Phase 3 (Progression)
- [ ] Skill point tracking
- [ ] Tier 1-5 skills
- [ ] Stick handling progression
- [ ] Equipment crafting

### Phase 4 (Advanced)
- [ ] Tier 6-8 skills
- [ ] Organized matches
- [ ] Challenge mode
- [ ] NPC skill variance

### Phase 5 (Mastery)
- [ ] Tier 9-10 skills
- [ ] Legendary matches
- [ ] Rink Dog companion
- [ ] Frozen Moment ability
- [ ] Cross-system integration

---

## Balance Considerations

### Skill Scaling
- Early tiers feel impactful (can't play without Ice Legs)
- Mid tiers differentiate playstyles
- Late tiers are powerful but not required for fun
- Legendary tier is aspirational long-term goal

### Seasonal Availability
- Winter is ~3 months of game time
- Not every winter has prolonged cold snaps
- Creates anticipation for pond hockey season
- Matches feel special, not routine

### AI Difficulty
| NPC Tier | Skill Level | Behavior |
|----------|-------------|----------|
| Beginner | Tier 1-2 | Slow, mistakes often |
| Average | Tier 3-4 | Competent, predictable |
| Skilled | Tier 5-6 | Fast, strategic |
| Elite | Tier 7-8 | Very challenging |
| Legendary | Tier 9-10 | Near perfect |

### Injury Risk
- Body checks can injure
- Thin ice can break
- Frostbite in extreme cold
- Encourages smart play, not reckless

---

## Historical Notes

While organized ice hockey didn't exist in the 1580s, stick-and-ball games on ice have ancient roots:

- **Shinny/Shinney**: Scottish/Irish game on ice with curved sticks
- **Bandy**: European ice game with a ball
- **Native American variants**: Various tribes played ball games adapted to frozen conditions
- **Colonial adaptation**: Settlers combined European and native traditions

This skill tree imagines the organic development of such games in the harsh winters of the colonial frontier — a plausible cultural evolution that would eventually birth the sport we know today.
