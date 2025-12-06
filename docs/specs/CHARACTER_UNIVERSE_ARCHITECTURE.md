# Character Universe Architecture

## Agent Relationship Diagram

```
                           +---------------------------+
                           |    CHARACTER UNIVERSE     |
                           |     (Unified Agent Bus)   |
                           +-------------+-------------+
                                         |
         +-------------------------------+-------------------------------+
         |                               |                               |
         v                               v                               v
+------------------+           +------------------+           +------------------+
|   HUMAN AGENTS   |           |  ANIMAL AGENTS   |           |  EVENT AGENTS    |
|   (NPC System)   |           | (Wildlife Sys)   |           | (World Events)   |
+--------+---------+           +--------+---------+           +--------+---------+
         |                               |                               |
    +----+----+                    +-----+-----+                   +-----+-----+
    |         |                    |           |                   |           |
    v         v                    v           v                   v           v
+-------+ +-------+           +-------+ +-------+           +-------+ +-------+
|Village| |Trader |           | Pack  | |Solitary|          |Weather| |Quest  |
| NPCs  | | NPCs  |           |Animals| |Predator|          |Events | |Events |
+-------+ +-------+           +-------+ +-------+           +-------+ +-------+
```

## Core Agent Trait Hierarchy

```
                      +------------------+
                      |  CharacterAgent  |
                      |    (Trait)       |
                      +--------+---------+
                               |
       +-----------------------+-----------------------+
       |                       |                       |
       v                       v                       v
+-------------+         +-------------+         +-------------+
| Sentient    |         |  Creature   |         | Phenomenon  |
| Agent       |         |  Agent      |         | Agent       |
+------+------+         +------+------+         +------+------+
       |                       |                       |
       v                       v                       v
  NpcInstance               Animal              WorldEvent

```

## Communication Channels

```
+----------------+          +----------------+          +----------------+
|   PERCEPTION   |  ------> |   AWARENESS    |  ------> |   REACTION     |
|    CHANNEL     |          |    CHANNEL     |          |    CHANNEL     |
+----------------+          +----------------+          +----------------+
      ^                           ^                           |
      |                           |                           v
+----------------+          +----------------+          +----------------+
|  SPATIAL HASH  |          |  RELATIONSHIP  |          |   BEHAVIOR     |
|   (Position)   |          |    MANAGER     |          |   STATE MACHINE|
+----------------+          +----------------+          +----------------+
```

## Agent State Flow

```
                    +------------------+
                    |      IDLE        |
                    +--------+---------+
                             |
            +----------------+----------------+
            |                                 |
            v                                 v
    +---------------+                 +---------------+
    |   PATROLLING  |                 |   SCHEDULED   |
    | (Territory)   |                 |  (Schedule)   |
    +-------+-------+                 +-------+-------+
            |                                 |
            +----------------+----------------+
                             |
                             v
                    +--------+---------+
                    |     AWARE        |
                    | (Player Detected)|
                    +--------+---------+
                             |
         +-------------------+-------------------+
         |                   |                   |
         v                   v                   v
  +-----------+       +-----------+       +-----------+
  | APPROACH  |       |  OBSERVE  |       |   FLEE    |
  | (Friendly)|       | (Neutral) |       | (Hostile) |
  +-----------+       +-----------+       +-----------+
         |                   |                   |
         v                   v                   v
  +-----------+       +-----------+       +-----------+
  | INTERACT  |       |  IGNORE   |       |  ATTACK   |
  | (Dialogue)|       |  (Resume) |       | (Combat)  |
  +-----------+       +-----------+       +-----------+
```

## Pathing System Architecture

```
+------------------------------------------------------------------+
|                        PATHING LAYER                              |
+------------------------------------------------------------------+
|                                                                   |
|  +-------------+     +-------------+     +-------------+          |
|  | WAYPOINT    |---->| PATH        |---->| MOVEMENT    |          |
|  | GENERATOR   |     | SMOOTHER    |     | EXECUTOR    |          |
|  +-------------+     +-------------+     +-------------+          |
|        ^                                        |                 |
|        |                                        v                 |
|  +-------------+                         +-------------+          |
|  | NAVIGATION  |                         | COLLISION   |          |
|  | MESH        |                         | AVOIDANCE   |          |
|  +-------------+                         +-------------+          |
|                                                                   |
+------------------------------------------------------------------+

Path Types:
  - SCHEDULE_PATH: NPC daily routines (home -> work -> social -> home)
  - PATROL_PATH:   Animal territory patrols (circular, random)
  - PURSUIT_PATH:  Direct chase with prediction
  - FLEE_PATH:     Away from threat with obstacle avoidance
  - WANDER_PATH:   Random exploration within bounds
```

## Visual Orb Dialogue System

```
+------------------------------------------------------------------+
|                     ORB DIALOGUE LAYER                            |
+------------------------------------------------------------------+
|                                                                   |
|   EMITTER ORB                        RECEIVER ORB                 |
|   +----------+                       +----------+                 |
|   |  Agent A |  ---- BEAM ---->      |  Agent B |                 |
|   | (Glowing)|  (Visual Link)        | (Pulsing)|                 |
|   +----------+                       +----------+                 |
|        |                                  |                       |
|        v                                  v                       |
|   +---------+                        +---------+                  |
|   | EMOTION |                        | EMOTION |                  |
|   | COLOR   |                        | COLOR   |                  |
|   +---------+                        +---------+                  |
|                                                                   |
|   Emotion Colors:                                                 |
|     - FRIENDLY:  Green glow, soft pulse                           |
|     - CURIOUS:   Blue glow, medium pulse                          |
|     - ALERT:     Yellow glow, fast pulse                          |
|     - HOSTILE:   Red glow, intense pulse                          |
|     - FEARFUL:   Purple glow, erratic pulse                       |
|                                                                   |
+------------------------------------------------------------------+
```

## Inter-Agent Communication Events

```
+------------------+          +------------------+
|   WORLD EVENT    |          |   NPC WITNESS    |
|   (Hunt Kill)    |   --->   |   (Saw Player)   |
+------------------+          +--------+---------+
                                       |
                                       v
                              +------------------+
                              |  REPUTATION      |
                              |  SPREAD          |
                              +--------+---------+
                                       |
                     +-----------------+-----------------+
                     |                                   |
                     v                                   v
            +------------------+              +------------------+
            |  NEARBY NPCS     |              |  ANIMAL PACKS    |
            | (Opinion Change) |              |  (Alert Level)   |
            +------------------+              +------------------+
```

## Unified Agent Manager

```rust
// Core trait for all character agents
pub trait CharacterAgent {
    fn id(&self) -> AgentId;
    fn position(&self) -> Vec3;
    fn set_position(&mut self, pos: Vec3);
    fn velocity(&self) -> Vec3;
    fn set_velocity(&mut self, vel: Vec3);
    fn awareness(&self) -> f32;
    fn set_awareness(&mut self, level: f32);
    fn agent_type(&self) -> AgentType;
    fn behavior_state(&self) -> UnifiedBehaviorState;
    fn set_behavior_state(&mut self, state: UnifiedBehaviorState);
    fn update(&mut self, ctx: &AgentContext, dt: f32);
    fn can_communicate_with(&self, other: &dyn CharacterAgent) -> bool;
}
```

## Campaign Integration

```
+------------------------------------------------------------------+
|                      CAMPAIGN SYSTEM                              |
+------------------------------------------------------------------+
|                                                                   |
|  WORLD PHASE          AGENT BEHAVIORS          EVENT TRIGGERS     |
|  +----------+         +--------------+         +--------------+   |
|  | Arrival  | ------> | Cautious NPCs| ------> | Tutorial     |   |
|  +----------+         +--------------+         | Events       |   |
|       |                                        +--------------+   |
|       v                                               |           |
|  +----------+         +--------------+                v           |
|  |Settlement| ------> | Trading Open | ------> +--------------+   |
|  +----------+         +--------------+         | Quest Events |   |
|       |                                        +--------------+   |
|       v                                               |           |
|  +----------+         +--------------+                v           |
|  | Conflict | ------> | Faction Wars | ------> +--------------+   |
|  +----------+         +--------------+         | Crisis Events|   |
|       |                                        +--------------+   |
|       v                                               |           |
|  +----------+         +--------------+                v           |
|  |Resolution| ------> | Peace/War    | ------> +--------------+   |
|  +----------+         +--------------+         | Ending Events|   |
|                                                +--------------+   |
+------------------------------------------------------------------+
```

## Data Flow Summary

1. **Spatial Query**: SpatialHash provides O(1) position queries
2. **Awareness**: Agents detect nearby entities via perception radius
3. **Relationship**: RelationshipManager tracks trust/fear/respect
4. **Behavior**: State machine determines actions
5. **Movement**: Pathing system executes locomotion
6. **Communication**: Visual orbs show inter-agent awareness
7. **Events**: World events modify agent behaviors globally
8. **Campaign**: World phase determines available interactions
