# LLM Gateway & NPC Intelligence Spec

**Date**: 2026-01-22
**Status**: DRAFT
**Priority**: MEDIUM - Enhancement to existing NPC systems
**Target Version**: v0.1.0+

---

## Executive Summary

This document specifies a **hybrid AI architecture** for NPC intelligence that minimizes expensive LLM API calls while enabling emergent, believable NPC behavior. The system operates in three tiers:

1. **Tier 1 (Local)**: Template-based responses with procedural variation - FREE
2. **Tier 2 (Scripted)**: Utility-based decision trees with personality weights - FREE
3. **Tier 3 (LLM)**: API calls for novel situations - METERED

Goal: **95%+ of interactions handled by Tiers 1-2**, with Tier 3 reserved for truly novel situations, free-form player input, and critical story moments.

### Implementation Status

| Component | Status | Location |
|-----------|--------|----------|
| Dialogue Trees | **COMPLETE** | `roanoke_game/src/npc/dialogue.rs` |
| Relationship System | **COMPLETE** | `roanoke_game/src/npc/relationships.rs` |
| Emotional States | **COMPLETE** | `roanoke_game/src/character_agent/mod.rs` |
| Template Expansion | PLANNED | `roanoke_game/src/npc/templates.rs` |
| LLM Gateway | PLANNED | `roanoke_game/src/llm/mod.rs` |
| Usage Tracking | PLANNED | `roanoke_game/src/llm/billing.rs` |
| Response Cache | PLANNED | `roanoke_game/src/llm/cache.rs` |

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                            Player Interaction                                │
│                     (Menu Choice / Free-form Text)                          │
└─────────────────────────────────┬───────────────────────────────────────────┘
                                  │
                                  ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                         Intent Classifier                                    │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐ │
│  │ Menu Choice │  │Topic Match  │  │ Personality │  │ Novel Situation     │ │
│  │  (exact)    │  │  (fuzzy)    │  │  Fallback   │  │   Detector          │ │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘  └──────────┬──────────┘ │
└─────────┼────────────────┼────────────────┼────────────────────┼────────────┘
          │                │                │                    │
          ▼                ▼                ▼                    ▼
┌─────────────────┐ ┌─────────────────┐ ┌─────────────────┐ ┌─────────────────┐
│    TIER 1       │ │    TIER 1       │ │    TIER 2       │ │    TIER 3       │
│  Dialogue Tree  │ │   Template      │ │   Utility AI    │ │   LLM Gateway   │
│   (existing)    │ │   Expansion     │ │   + Personality │ │                 │
│                 │ │                 │ │                 │ │  ┌───────────┐  │
│  Cost: FREE     │ │  Cost: FREE     │ │  Cost: FREE     │ │  │  Haiku    │  │
│                 │ │                 │ │                 │ │  │  Sonnet   │  │
│                 │ │                 │ │                 │ │  │  Opus     │  │
└────────┬────────┘ └────────┬────────┘ └────────┬────────┘ │  └───────────┘  │
         │                   │                   │          │  Cost: METERED  │
         │                   │                   │          └────────┬────────┘
         └───────────────────┴───────────────────┴───────────────────┘
                                        │
                                        ▼
                            ┌───────────────────────┐
                            │   Response Pipeline   │
                            │  - Tone adjustment    │
                            │  - Memory update      │
                            │  - Effect execution   │
                            └───────────────────────┘
```

---

## Tier 1: Local Template System

### Template Structure

```rust
/// A dialogue template with fillable slots
#[derive(Clone, Debug)]
pub struct DialogueTemplate {
    /// Unique identifier
    pub id: String,
    /// Template text with {slot} placeholders
    pub template: String,
    /// Required context for this template
    pub required_context: Vec<ContextRequirement>,
    /// Mood variants (optional rewrites per emotional state)
    pub mood_variants: HashMap<EmotionalState, String>,
    /// Weight for random selection among matching templates
    pub weight: f32,
}

/// Context requirements for template matching
#[derive(Clone, Debug)]
pub enum ContextRequirement {
    HasRelationship(RelationshipType),
    HasMemory(MemoryType),
    KnowsTopic(String),
    TimeOfDay(TimeRange),
    Weather(WeatherType),
    QuestState(String, QuestState),
}
```

### Slot Resolvers

```rust
/// Resolves template slots to actual values
pub struct SlotResolver {
    resolvers: HashMap<String, Box<dyn Fn(&NpcContext) -> String>>,
}

impl SlotResolver {
    pub fn new() -> Self {
        let mut resolvers: HashMap<String, Box<dyn Fn(&NpcContext) -> String>> = HashMap::new();

        // Player references
        resolvers.insert("player_name".into(), Box::new(|ctx| {
            ctx.player_name.clone()
        }));

        resolvers.insert("player_title".into(), Box::new(|ctx| {
            match ctx.relationship.respect {
                r if r > 50 => "honored one",
                r if r > 0 => "friend",
                r if r > -30 => "stranger",
                _ => "outsider",
            }.into()
        }));

        // Memory references
        resolvers.insert("last_gift".into(), Box::new(|ctx| {
            ctx.relationship.gifts_received.last()
                .map(|g| g.item_name.clone())
                .unwrap_or_else(|| "nothing".into())
        }));

        resolvers.insert("memory_recent".into(), Box::new(|ctx| {
            ctx.relationship.memories.iter()
                .filter(|m| m.timestamp > ctx.game_time - 24.0)
                .last()
                .map(|m| m.description.clone())
                .unwrap_or_else(|| "our meeting".into())
        }));

        // NPC knowledge
        resolvers.insert("local_danger".into(), Box::new(|ctx| {
            ctx.world_state.nearby_threats.first()
                .map(|t| t.name.clone())
                .unwrap_or_else(|| "the unknown".into())
        }));

        resolvers.insert("time_greeting".into(), Box::new(|ctx| {
            match ctx.game_hour {
                h if h < 6 => "You rise early",
                h if h < 12 => "Good morning",
                h if h < 18 => "Good day",
                _ => "The night grows deep",
            }.into()
        }));

        Self { resolvers }
    }

    pub fn resolve(&self, template: &str, ctx: &NpcContext) -> String {
        let mut result = template.to_string();
        for (slot, resolver) in &self.resolvers {
            let placeholder = format!("{{{}}}", slot);
            if result.contains(&placeholder) {
                result = result.replace(&placeholder, &resolver(ctx));
            }
        }
        result
    }
}
```

### Example Templates

```rust
// templates/elder_greetings.json
[
    {
        "id": "elder_greeting_friendly",
        "template": "{time_greeting}, {player_title}. The spirits spoke of your return.",
        "required_context": [
            {"HasRelationship": "Friend"},
            {"HasMemory": "Positive"}
        ],
        "mood_variants": {
            "Worried": "{time_greeting}, {player_title}. Dark dreams trouble my sleep...",
            "Joyful": "Ah! {player_name}! My heart soars like the eagle to see you!"
        },
        "weight": 1.0
    },
    {
        "id": "elder_greeting_stranger",
        "template": "You walk in lands your people have forgotten. Why do you come?",
        "required_context": [
            {"HasRelationship": "Stranger"}
        ],
        "weight": 1.0
    },
    {
        "id": "elder_gift_thanks",
        "template": "The {last_gift} honors our ancestors. You understand the old ways.",
        "required_context": [
            {"HasMemory": "Gift"}
        ],
        "weight": 1.0
    }
]
```

---

## Tier 2: Utility-Based Personality System

### Personality Vectors

```rust
/// Personality traits that influence decision-making
#[derive(Clone, Debug, Default)]
pub struct NpcPersonality {
    /// -1.0 (pacifist) to 1.0 (aggressive)
    pub aggression: f32,
    /// -1.0 (dismissive) to 1.0 (inquisitive)
    pub curiosity: f32,
    /// -1.0 (generous) to 1.0 (greedy)
    pub greed: f32,
    /// -1.0 (treacherous) to 1.0 (devoted)
    pub loyalty: f32,
    /// -1.0 (cowardly) to 1.0 (fearless)
    pub courage: f32,
    /// -1.0 (closed) to 1.0 (open)
    pub openness: f32,
    /// -1.0 (humorless) to 1.0 (jovial)
    pub humor: f32,
    /// -1.0 (secular) to 1.0 (devout)
    pub spirituality: f32,
}

impl NpcPersonality {
    /// Generate personality from NPC role with variation
    pub fn from_role(role: NpcRole, seed: u64) -> Self {
        let mut rng = StdRng::seed_from_u64(seed);
        let vary = |base: f32| -> f32 {
            (base + rng.gen_range(-0.2..0.2)).clamp(-1.0, 1.0)
        };

        match role {
            NpcRole::Elder => Self {
                aggression: vary(-0.5),
                curiosity: vary(0.3),
                greed: vary(-0.3),
                loyalty: vary(0.7),
                courage: vary(0.2),
                openness: vary(0.4),
                humor: vary(0.1),
                spirituality: vary(0.6),
            },
            NpcRole::Warrior => Self {
                aggression: vary(0.4),
                curiosity: vary(-0.1),
                greed: vary(0.0),
                loyalty: vary(0.8),
                courage: vary(0.7),
                openness: vary(-0.2),
                humor: vary(0.0),
                spirituality: vary(0.2),
            },
            NpcRole::Shaman => Self {
                aggression: vary(-0.6),
                curiosity: vary(0.5),
                greed: vary(-0.5),
                loyalty: vary(0.5),
                courage: vary(0.3),
                openness: vary(0.6),
                humor: vary(0.2),
                spirituality: vary(0.9),
            },
            // ... other roles
            _ => Self::default(),
        }
    }
}
```

### Unknown Topic Handler

```rust
/// Handles player queries that don't match dialogue trees
pub struct TopicHandler {
    /// Known topics per NPC role
    knowledge_base: HashMap<NpcRole, Vec<KnownTopic>>,
    /// Fuzzy matcher threshold (0.0 - 1.0)
    match_threshold: f32,
}

#[derive(Clone)]
pub struct KnownTopic {
    pub keywords: Vec<String>,
    pub response_templates: Vec<String>,
    pub redirect_to: Option<String>, // Another NPC role who knows more
}

impl TopicHandler {
    pub fn handle_query(
        &self,
        npc: &Npc,
        query: &str,
        ctx: &NpcContext,
    ) -> TopicResponse {
        let normalized = query.to_lowercase();
        let words: Vec<&str> = normalized.split_whitespace().collect();

        // Try to find matching topic in NPC's knowledge
        if let Some(topics) = self.knowledge_base.get(&npc.role) {
            for topic in topics {
                let match_score = self.fuzzy_match(&words, &topic.keywords);
                if match_score > self.match_threshold {
                    return TopicResponse::Known {
                        template: topic.response_templates.choose(&mut rand::thread_rng())
                            .unwrap()
                            .clone(),
                        confidence: match_score,
                    };
                }
            }
        }

        // No match - use personality-based fallback
        self.personality_fallback(npc, query, ctx)
    }

    fn personality_fallback(
        &self,
        npc: &Npc,
        query: &str,
        ctx: &NpcContext,
    ) -> TopicResponse {
        let p = &npc.personality;

        // High curiosity NPCs ask follow-up questions
        if p.curiosity > 0.3 {
            return TopicResponse::Curious {
                template: "You speak of strange things. Tell me more of this '{query_echo}'?"
                    .replace("{query_echo}", &self.extract_noun(query)),
            };
        }

        // High spirituality deflects to mysticism
        if p.spirituality > 0.5 {
            return TopicResponse::Deflect {
                template: "The spirits hold such knowledge. Perhaps in dreams, the answer will come."
                    .into(),
            };
        }

        // High aggression gets dismissive
        if p.aggression > 0.3 {
            return TopicResponse::Dismiss {
                template: "I have no time for such questions. Speak of useful things or leave."
                    .into(),
            };
        }

        // Redirect to someone who might know
        if p.openness > 0.2 {
            let suggested = self.suggest_knowledgeable_npc(query);
            return TopicResponse::Redirect {
                template: format!(
                    "I know little of this. Perhaps {} could help you.",
                    suggested
                ),
                suggested_npc: suggested,
            };
        }

        // Default humble admission
        TopicResponse::Unknown {
            template: "My knowledge does not extend to such matters.".into(),
        }
    }
}

pub enum TopicResponse {
    Known { template: String, confidence: f32 },
    Curious { template: String },
    Deflect { template: String },
    Dismiss { template: String },
    Redirect { template: String, suggested_npc: String },
    Unknown { template: String },
    /// Escalate to LLM - no local handler could manage this
    EscalateToLlm { reason: String },
}
```

---

## Tier 3: LLM Gateway

### Gateway Architecture

```rust
/// Central LLM gateway managing all API interactions
pub struct LlmGateway {
    /// HTTP client for API calls
    client: reqwest::Client,
    /// API configuration
    config: LlmConfig,
    /// Response cache
    cache: ResponseCache,
    /// Rate limiter per player
    rate_limiter: RateLimiter,
    /// Usage tracker for billing
    usage_tracker: UsageTracker,
    /// Request queue for batch processing
    request_queue: RequestQueue,
}

#[derive(Clone, Debug)]
pub struct LlmConfig {
    /// API endpoint (Anthropic, OpenAI, local, etc.)
    pub endpoint: String,
    /// API key (from environment or config)
    pub api_key: String,
    /// Model selection per tier
    pub model_tiers: ModelTiers,
    /// Maximum tokens per request
    pub max_tokens: u32,
    /// Temperature for response variation
    pub temperature: f32,
    /// Timeout in milliseconds
    pub timeout_ms: u64,
}

#[derive(Clone, Debug)]
pub struct ModelTiers {
    /// Quick reactions, simple personality responses
    pub fast: String,      // e.g., "claude-3-haiku-20240307"
    /// Complex reasoning, multi-turn dialogue
    pub balanced: String,  // e.g., "claude-sonnet-4-20250514"
    /// Critical story moments, major decisions
    pub premium: String,   // e.g., "claude-opus-4-20250514"
}
```

### Model Selection Logic

```rust
/// Determines which model tier to use based on context
pub struct ModelSelector;

impl ModelSelector {
    pub fn select_tier(request: &LlmRequest) -> ModelTier {
        // Premium: Critical story moments
        if request.context.is_climax_event
            || request.context.is_major_decision
            || request.context.affects_ending {
            return ModelTier::Premium;
        }

        // Balanced: Complex multi-turn, relationship-critical
        if request.turn_count > 3
            || request.context.relationship_at_threshold
            || request.requires_reasoning {
            return ModelTier::Balanced;
        }

        // Fast: Quick reactions, simple queries
        ModelTier::Fast
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ModelTier {
    Fast,     // ~$0.25/M input, $1.25/M output
    Balanced, // ~$3/M input, $15/M output
    Premium,  // ~$15/M input, $75/M output
}

impl ModelTier {
    pub fn cost_per_1k_tokens(&self) -> (f32, f32) {
        match self {
            ModelTier::Fast => (0.00025, 0.00125),
            ModelTier::Balanced => (0.003, 0.015),
            ModelTier::Premium => (0.015, 0.075),
        }
    }
}
```

### Context Compression

```rust
/// Compresses game state into minimal LLM context
pub struct ContextCompressor;

impl ContextCompressor {
    /// Compress full NPC state into ~200 tokens
    pub fn compress_npc(npc: &Npc, relationship: &NpcRelationship) -> String {
        format!(
            r#"NPC: {} ({})
Personality: {}
Mood: {} | Relationship: {} ({})
Recent: {}
Knowledge: {}"#,
            npc.name,
            npc.role,
            Self::compress_personality(&npc.personality),
            npc.emotional_state,
            relationship.relationship_type(),
            relationship.affinity,
            Self::compress_memories(&relationship.memories, 3),
            Self::compress_knowledge(&npc.knowledge_topics, 5),
        )
    }

    fn compress_personality(p: &NpcPersonality) -> String {
        let mut traits = Vec::new();
        if p.aggression > 0.3 { traits.push("aggressive"); }
        if p.aggression < -0.3 { traits.push("peaceful"); }
        if p.curiosity > 0.3 { traits.push("curious"); }
        if p.spirituality > 0.5 { traits.push("devout"); }
        if p.humor > 0.3 { traits.push("humorous"); }
        if p.courage > 0.5 { traits.push("brave"); }
        if p.courage < -0.3 { traits.push("cautious"); }
        traits.join(", ")
    }

    fn compress_memories(memories: &[NpcMemory], limit: usize) -> String {
        memories.iter()
            .rev()
            .take(limit)
            .map(|m| format!("{}({})",
                m.description.chars().take(30).collect::<String>(),
                if m.impact > 0 { "+" } else { "-" }
            ))
            .collect::<Vec<_>>()
            .join("; ")
    }

    fn compress_knowledge(topics: &[String], limit: usize) -> String {
        topics.iter()
            .take(limit)
            .cloned()
            .collect::<Vec<_>>()
            .join(", ")
    }

    /// Compress world state into ~100 tokens
    pub fn compress_world(state: &WorldState) -> String {
        format!(
            "Time: {} | Weather: {} | Location: {} | Nearby: {} | Tension: {}",
            state.time_of_day(),
            state.weather,
            state.current_region,
            state.nearby_entities_summary(),
            state.narrative_tension,
        )
    }
}
```

### Request Structure

```rust
/// A request to the LLM gateway
#[derive(Clone, Debug)]
pub struct LlmRequest {
    /// Unique request ID for tracking
    pub id: Uuid,
    /// Player making the request
    pub player_id: PlayerId,
    /// NPC being interacted with
    pub npc_id: NpcId,
    /// Player's input (free-form text)
    pub player_input: String,
    /// Compressed context
    pub context: CompressedContext,
    /// Conversation history (last N turns)
    pub history: Vec<ConversationTurn>,
    /// Selected model tier
    pub model_tier: ModelTier,
    /// Priority (affects queue position)
    pub priority: RequestPriority,
    /// Timestamp
    pub created_at: Instant,
}

#[derive(Clone, Debug)]
pub struct CompressedContext {
    pub npc_summary: String,      // ~200 tokens
    pub world_summary: String,    // ~100 tokens
    pub story_summary: String,    // ~100 tokens
    pub constraints: Vec<String>, // Response constraints

    // Flags
    pub is_climax_event: bool,
    pub is_major_decision: bool,
    pub affects_ending: bool,
    pub relationship_at_threshold: bool,
}

#[derive(Clone, Copy, Debug)]
pub enum RequestPriority {
    /// Background processing, can wait
    Low,
    /// Normal gameplay interaction
    Normal,
    /// Time-sensitive (combat dialogue, etc.)
    High,
    /// Critical story moment
    Critical,
}
```

### System Prompt Template

```rust
impl LlmGateway {
    fn build_system_prompt(&self, ctx: &CompressedContext, npc: &Npc) -> String {
        format!(
            r#"You are {name}, a {role} in a 16th-century Native American village.
You are speaking with a colonist who has arrived in your lands.

{npc_summary}

WORLD STATE:
{world_summary}

STORY CONTEXT:
{story_summary}

RESPONSE RULES:
- Stay in character as {name}
- Speak in the manner of your role and personality
- Reference your memories and knowledge naturally
- Keep responses under 100 words
- Do not break character or reference being an AI
- Do not use modern idioms or technology references
{constraints}

Respond as {name} would, given your personality, relationship with the player, and current situation."#,
            name = npc.name,
            role = npc.role,
            npc_summary = ctx.npc_summary,
            world_summary = ctx.world_summary,
            story_summary = ctx.story_summary,
            constraints = ctx.constraints.iter()
                .map(|c| format!("- {}", c))
                .collect::<Vec<_>>()
                .join("\n"),
        )
    }
}
```

---

## Response Cache

### Cache Architecture

```rust
/// Caches LLM responses to avoid duplicate API calls
pub struct ResponseCache {
    /// In-memory LRU cache
    memory_cache: LruCache<CacheKey, CachedResponse>,
    /// Persistent cache (SQLite or file-based)
    persistent_cache: Option<PersistentCache>,
    /// Cache statistics
    stats: CacheStats,
}

/// Cache key combines situation factors
#[derive(Clone, Hash, Eq, PartialEq)]
pub struct CacheKey {
    /// NPC role (not specific NPC - allows reuse)
    pub npc_role: NpcRole,
    /// Personality bucket (quantized)
    pub personality_bucket: u8,
    /// Relationship tier (not exact value)
    pub relationship_tier: RelationshipTier,
    /// Normalized player input
    pub input_hash: u64,
    /// Context hash (world state bucket)
    pub context_bucket: u8,
}

impl CacheKey {
    pub fn from_request(req: &LlmRequest, npc: &Npc) -> Self {
        Self {
            npc_role: npc.role,
            personality_bucket: Self::bucket_personality(&npc.personality),
            relationship_tier: req.context.relationship_tier(),
            input_hash: Self::hash_input(&req.player_input),
            context_bucket: Self::bucket_context(&req.context),
        }
    }

    /// Bucket personality into 8 archetypes
    fn bucket_personality(p: &NpcPersonality) -> u8 {
        let aggressive = p.aggression > 0.0;
        let curious = p.curiosity > 0.0;
        let spiritual = p.spirituality > 0.3;

        match (aggressive, curious, spiritual) {
            (false, false, false) => 0, // Reserved
            (false, false, true)  => 1, // Mystic
            (false, true, false)  => 2, // Scholar
            (false, true, true)   => 3, // Sage
            (true, false, false)  => 4, // Stoic
            (true, false, true)   => 5, // Zealot
            (true, true, false)   => 6, // Hunter
            (true, true, true)    => 7, // Warrior-Shaman
        }
    }

    /// Normalize and hash player input
    fn hash_input(input: &str) -> u64 {
        use std::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;

        let normalized = input
            .to_lowercase()
            .split_whitespace()
            .filter(|w| w.len() > 2) // Remove short words
            .collect::<Vec<_>>()
            .join(" ");

        let mut hasher = DefaultHasher::new();
        normalized.hash(&mut hasher);
        hasher.finish()
    }
}

#[derive(Clone)]
pub struct CachedResponse {
    pub response: String,
    pub model_tier: ModelTier,
    pub tokens_used: TokenCount,
    pub created_at: Instant,
    pub hit_count: u32,
}

impl ResponseCache {
    /// Check cache before making API call
    pub fn get(&mut self, key: &CacheKey) -> Option<CachedResponse> {
        // Check memory first
        if let Some(cached) = self.memory_cache.get_mut(key) {
            cached.hit_count += 1;
            self.stats.memory_hits += 1;
            return Some(cached.clone());
        }

        // Check persistent cache
        if let Some(ref persistent) = self.persistent_cache {
            if let Some(cached) = persistent.get(key) {
                // Promote to memory cache
                self.memory_cache.put(key.clone(), cached.clone());
                self.stats.persistent_hits += 1;
                return Some(cached);
            }
        }

        self.stats.misses += 1;
        None
    }

    /// Store response in cache
    pub fn put(&mut self, key: CacheKey, response: CachedResponse) {
        self.memory_cache.put(key.clone(), response.clone());

        if let Some(ref mut persistent) = self.persistent_cache {
            persistent.put(key, response);
        }
    }
}
```

---

## Rate Limiting & Budgets

### Rate Limiter

```rust
/// Per-player rate limiting
pub struct RateLimiter {
    /// Requests per time window
    limits: RateLimits,
    /// Player request counts
    player_counts: HashMap<PlayerId, PlayerRateState>,
}

#[derive(Clone, Debug)]
pub struct RateLimits {
    /// Requests per minute (burst protection)
    pub per_minute: u32,
    /// Requests per hour (sustained limit)
    pub per_hour: u32,
    /// Requests per day (budget limit)
    pub per_day: u32,
    /// Token budget per day
    pub tokens_per_day: u32,
}

impl Default for RateLimits {
    fn default() -> Self {
        Self {
            per_minute: 10,
            per_hour: 60,
            per_day: 200,
            tokens_per_day: 50_000,
        }
    }
}

#[derive(Clone, Debug)]
pub struct PlayerRateState {
    /// Rolling window counts
    pub minute_count: u32,
    pub hour_count: u32,
    pub day_count: u32,
    pub day_tokens: u32,
    /// Window reset times
    pub minute_reset: Instant,
    pub hour_reset: Instant,
    pub day_reset: Instant,
    /// Subscription tier (affects limits)
    pub tier: SubscriptionTier,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SubscriptionTier {
    Free,      // 100 requests/day, 10k tokens
    Basic,     // 500 requests/day, 50k tokens
    Premium,   // 2000 requests/day, 200k tokens
    Unlimited, // No limits (server-side only)
}

impl RateLimiter {
    pub fn check(&mut self, player_id: PlayerId) -> RateLimitResult {
        let now = Instant::now();
        let state = self.player_counts
            .entry(player_id)
            .or_insert_with(|| PlayerRateState::new(SubscriptionTier::Free));

        // Reset windows if needed
        state.reset_windows_if_needed(now);

        // Check limits based on tier
        let limits = self.limits_for_tier(state.tier);

        if state.minute_count >= limits.per_minute {
            return RateLimitResult::Denied {
                reason: "Too many requests per minute".into(),
                retry_after: state.minute_reset - now,
            };
        }

        if state.day_count >= limits.per_day {
            return RateLimitResult::Denied {
                reason: "Daily request limit reached".into(),
                retry_after: state.day_reset - now,
            };
        }

        if state.day_tokens >= limits.tokens_per_day {
            return RateLimitResult::Denied {
                reason: "Daily token budget exhausted".into(),
                retry_after: state.day_reset - now,
            };
        }

        // Allow with remaining budget info
        RateLimitResult::Allowed {
            remaining_requests: limits.per_day - state.day_count,
            remaining_tokens: limits.tokens_per_day - state.day_tokens,
        }
    }

    pub fn record_usage(&mut self, player_id: PlayerId, tokens: u32) {
        if let Some(state) = self.player_counts.get_mut(&player_id) {
            state.minute_count += 1;
            state.hour_count += 1;
            state.day_count += 1;
            state.day_tokens += tokens;
        }
    }
}

pub enum RateLimitResult {
    Allowed {
        remaining_requests: u32,
        remaining_tokens: u32,
    },
    Denied {
        reason: String,
        retry_after: Duration,
    },
}
```

---

## Usage Tracking & Billing

### Usage Tracker

```rust
/// Tracks API usage for billing and analytics
pub struct UsageTracker {
    /// Per-player usage records
    player_usage: HashMap<PlayerId, PlayerUsage>,
    /// Global usage statistics
    global_stats: GlobalUsageStats,
    /// Billing integration
    billing: Option<BillingIntegration>,
}

#[derive(Clone, Debug, Default)]
pub struct PlayerUsage {
    /// Total requests by tier
    pub requests_by_tier: HashMap<ModelTier, u64>,
    /// Total tokens by tier (input, output)
    pub tokens_by_tier: HashMap<ModelTier, (u64, u64)>,
    /// Estimated cost (cents)
    pub estimated_cost_cents: u64,
    /// Cache hit rate
    pub cache_hits: u64,
    pub cache_misses: u64,
    /// Session history
    pub sessions: Vec<SessionUsage>,
}

#[derive(Clone, Debug)]
pub struct SessionUsage {
    pub session_id: Uuid,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub requests: u32,
    pub tokens_used: u32,
    pub cost_cents: u32,
}

impl UsageTracker {
    pub fn record(&mut self, player_id: PlayerId, usage: &RequestUsage) {
        let player = self.player_usage
            .entry(player_id)
            .or_default();

        // Update tier counts
        *player.requests_by_tier
            .entry(usage.model_tier)
            .or_default() += 1;

        let (input, output) = player.tokens_by_tier
            .entry(usage.model_tier)
            .or_insert((0, 0));
        *input += usage.input_tokens as u64;
        *output += usage.output_tokens as u64;

        // Calculate cost
        let (input_rate, output_rate) = usage.model_tier.cost_per_1k_tokens();
        let cost = (usage.input_tokens as f32 * input_rate / 1000.0)
            + (usage.output_tokens as f32 * output_rate / 1000.0);
        player.estimated_cost_cents += (cost * 100.0) as u64;

        // Update global stats
        self.global_stats.total_requests += 1;
        self.global_stats.total_tokens += usage.input_tokens + usage.output_tokens;

        // Notify billing if integrated
        if let Some(ref billing) = self.billing {
            billing.record_usage(player_id, usage);
        }
    }

    pub fn get_player_summary(&self, player_id: PlayerId) -> Option<UsageSummary> {
        self.player_usage.get(&player_id).map(|u| UsageSummary {
            total_requests: u.requests_by_tier.values().sum(),
            total_tokens: u.tokens_by_tier.values()
                .map(|(i, o)| i + o)
                .sum(),
            estimated_cost_usd: u.estimated_cost_cents as f32 / 100.0,
            cache_hit_rate: u.cache_hits as f32
                / (u.cache_hits + u.cache_misses).max(1) as f32,
        })
    }
}

#[derive(Clone, Debug)]
pub struct RequestUsage {
    pub request_id: Uuid,
    pub model_tier: ModelTier,
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub latency_ms: u32,
    pub cache_hit: bool,
}
```

---

## Integration with Existing Systems

### Dialogue System Hook

```rust
// In dialogue.rs - add LLM fallback

impl DialogueManager {
    pub async fn handle_player_input(
        &mut self,
        npc_id: NpcId,
        input: PlayerInput,
        ctx: &GameContext,
    ) -> DialogueResponse {
        let npc = self.get_npc(npc_id);
        let relationship = self.relationships.get(npc_id);

        match input {
            // Tier 1: Menu choice - use existing dialogue tree
            PlayerInput::MenuChoice(choice_id) => {
                self.process_dialogue_choice(npc_id, choice_id, ctx)
            }

            // Free-form text - evaluate escalation
            PlayerInput::FreeText(text) => {
                // Try Tier 1: Template matching
                if let Some(response) = self.template_engine.match_input(&text, npc, ctx) {
                    return response;
                }

                // Try Tier 2: Topic handler
                match self.topic_handler.handle_query(npc, &text, ctx) {
                    TopicResponse::EscalateToLlm { reason } => {
                        // Tier 3: LLM gateway
                        self.escalate_to_llm(npc_id, text, reason, ctx).await
                    }
                    other => self.convert_topic_response(other, npc, ctx),
                }
            }
        }
    }

    async fn escalate_to_llm(
        &mut self,
        npc_id: NpcId,
        input: String,
        reason: String,
        ctx: &GameContext,
    ) -> DialogueResponse {
        let npc = self.get_npc(npc_id);
        let relationship = self.relationships.get(npc_id);

        // Build compressed context
        let compressed = CompressedContext {
            npc_summary: ContextCompressor::compress_npc(npc, relationship),
            world_summary: ContextCompressor::compress_world(&ctx.world_state),
            story_summary: ctx.story_state.current_summary(),
            constraints: self.get_response_constraints(npc, ctx),
            is_climax_event: ctx.story_state.is_climax(),
            is_major_decision: ctx.story_state.is_major_decision(),
            affects_ending: ctx.story_state.affects_ending(),
            relationship_at_threshold: relationship.is_at_threshold(),
        };

        // Create LLM request
        let request = LlmRequest {
            id: Uuid::new_v4(),
            player_id: ctx.player_id,
            npc_id,
            player_input: input,
            context: compressed,
            history: self.get_recent_history(npc_id, 3),
            model_tier: ModelSelector::select_tier(&request),
            priority: self.determine_priority(ctx),
            created_at: Instant::now(),
        };

        // Send to gateway
        match self.llm_gateway.send(request).await {
            Ok(response) => {
                // Update relationship based on response sentiment
                self.process_llm_response(npc_id, response, ctx)
            }
            Err(LlmError::RateLimited { retry_after }) => {
                DialogueResponse::Fallback {
                    text: self.get_rate_limit_fallback(npc),
                    retry_hint: Some(retry_after),
                }
            }
            Err(_) => {
                DialogueResponse::Fallback {
                    text: self.get_error_fallback(npc),
                    retry_hint: None,
                }
            }
        }
    }
}
```

### Character Agent Integration

```rust
// In character_agent/mod.rs - add LLM-driven reactions

impl<T: CharacterAgent> AgentBrain for T {
    async fn react_to_event(
        &mut self,
        event: &WorldEvent,
        ctx: &AgentContext,
    ) -> AgentReaction {
        // Fast path: Use FSM for common events
        if let Some(reaction) = self.fsm_reaction(event) {
            return reaction;
        }

        // Medium path: Utility-based decision
        if let Some(reaction) = self.utility_reaction(event, ctx) {
            return reaction;
        }

        // Slow path: Novel event, consider LLM
        if self.should_escalate_to_llm(event, ctx) {
            // Queue for LLM processing (non-blocking)
            ctx.llm_gateway.queue_agent_reaction(
                self.agent_id(),
                event.clone(),
                RequestPriority::Normal,
            );

            // Return placeholder reaction while waiting
            return AgentReaction::Thinking {
                duration: Duration::from_secs(2),
                fallback: self.default_reaction(event),
            };
        }

        self.default_reaction(event)
    }
}
```

---

## Monetization Integration

### Subscription Tiers

```rust
/// Subscription tier definitions
#[derive(Clone, Debug)]
pub struct SubscriptionConfig {
    pub tiers: Vec<TierDefinition>,
}

#[derive(Clone, Debug)]
pub struct TierDefinition {
    pub id: SubscriptionTier,
    pub name: String,
    pub price_cents_monthly: u32,
    pub rate_limits: RateLimits,
    pub features: Vec<String>,
}

impl Default for SubscriptionConfig {
    fn default() -> Self {
        Self {
            tiers: vec![
                TierDefinition {
                    id: SubscriptionTier::Free,
                    name: "Explorer".into(),
                    price_cents_monthly: 0,
                    rate_limits: RateLimits {
                        per_minute: 5,
                        per_hour: 30,
                        per_day: 100,
                        tokens_per_day: 10_000,
                    },
                    features: vec![
                        "Basic NPC dialogue".into(),
                        "Template-based responses".into(),
                        "100 AI interactions/day".into(),
                    ],
                },
                TierDefinition {
                    id: SubscriptionTier::Basic,
                    name: "Settler".into(),
                    price_cents_monthly: 499,
                    rate_limits: RateLimits {
                        per_minute: 15,
                        per_hour: 100,
                        per_day: 500,
                        tokens_per_day: 50_000,
                    },
                    features: vec![
                        "Everything in Explorer".into(),
                        "500 AI interactions/day".into(),
                        "Priority response queue".into(),
                        "Extended NPC memory".into(),
                    ],
                },
                TierDefinition {
                    id: SubscriptionTier::Premium,
                    name: "Chieftain".into(),
                    price_cents_monthly: 1499,
                    rate_limits: RateLimits {
                        per_minute: 30,
                        per_hour: 300,
                        per_day: 2000,
                        tokens_per_day: 200_000,
                    },
                    features: vec![
                        "Everything in Settler".into(),
                        "2000 AI interactions/day".into(),
                        "Premium model access".into(),
                        "Persistent NPC relationships".into(),
                        "Custom dialogue influence".into(),
                    ],
                },
            ],
        }
    }
}
```

### Credit System (Alternative)

```rust
/// Credit-based system for pay-per-use
pub struct CreditSystem {
    /// Player credit balances
    balances: HashMap<PlayerId, CreditBalance>,
    /// Credit costs per action
    costs: CreditCosts,
}

#[derive(Clone, Debug)]
pub struct CreditBalance {
    /// Earned through gameplay
    pub earned: u32,
    /// Purchased with real money
    pub purchased: u32,
    /// Bonus credits (promotions, etc.)
    pub bonus: u32,
}

impl CreditBalance {
    pub fn total(&self) -> u32 {
        self.earned + self.purchased + self.bonus
    }

    /// Deduct credits, preferring earned > bonus > purchased
    pub fn deduct(&mut self, amount: u32) -> bool {
        if self.total() < amount {
            return false;
        }

        let mut remaining = amount;

        // Deduct from earned first
        let from_earned = remaining.min(self.earned);
        self.earned -= from_earned;
        remaining -= from_earned;

        // Then bonus
        let from_bonus = remaining.min(self.bonus);
        self.bonus -= from_bonus;
        remaining -= from_bonus;

        // Finally purchased
        self.purchased -= remaining;

        true
    }
}

#[derive(Clone, Debug)]
pub struct CreditCosts {
    /// Cost per Haiku request
    pub fast_request: u32,     // 1 credit
    /// Cost per Sonnet request
    pub balanced_request: u32, // 5 credits
    /// Cost per Opus request
    pub premium_request: u32,  // 20 credits
}

impl Default for CreditCosts {
    fn default() -> Self {
        Self {
            fast_request: 1,
            balanced_request: 5,
            premium_request: 20,
        }
    }
}

/// Ways to earn credits through gameplay
pub enum CreditEarningEvent {
    /// Complete a quest
    QuestComplete { difficulty: u32 },      // 5-20 credits
    /// Discover new location
    LocationDiscovered,                      // 2 credits
    /// First conversation with NPC
    FirstNpcMeeting,                         // 1 credit
    /// Daily login bonus
    DailyLogin { streak: u32 },             // 5 + streak credits
    /// Achievement unlocked
    Achievement { rarity: AchievementRarity }, // 10-100 credits
}
```

---

## Implementation Phases

### Phase 1: Foundation (Templates + Topic Handling)
- [ ] Implement `DialogueTemplate` system
- [ ] Create `SlotResolver` with 20+ slot types
- [ ] Build `TopicHandler` with fuzzy matching
- [ ] Add `NpcPersonality` to `NpcInstance`
- [ ] Write 100+ templates per NPC role

### Phase 2: LLM Gateway Core
- [ ] Implement `LlmGateway` struct
- [ ] Add `ContextCompressor`
- [ ] Build `ResponseCache` with LRU + persistence
- [ ] Implement `RateLimiter`
- [ ] Create `UsageTracker`

### Phase 3: Integration
- [ ] Hook into `DialogueManager`
- [ ] Add fallback responses for rate limits/errors
- [ ] Implement `ModelSelector` logic
- [ ] Add async request queue for batch processing

### Phase 4: Monetization
- [ ] Implement `SubscriptionTier` system
- [ ] Build `CreditSystem` (optional)
- [ ] Add usage dashboard UI
- [ ] Integrate payment provider

### Phase 5: Optimization
- [ ] Tune cache key buckets for optimal hit rate
- [ ] Analyze usage patterns, adjust tier thresholds
- [ ] Add response quality feedback loop
- [ ] Implement A/B testing for prompts

---

## Metrics & Monitoring

### Key Metrics to Track

| Metric | Target | Alert Threshold |
|--------|--------|-----------------|
| Cache hit rate | >70% | <50% |
| Tier 3 escalation rate | <10% | >20% |
| Average response latency | <500ms | >2000ms |
| API error rate | <1% | >5% |
| Cost per active player | <$0.10/day | >$0.25/day |
| Player satisfaction (survey) | >4.0/5.0 | <3.5/5.0 |

### Dashboard Queries

```sql
-- Daily cost by tier
SELECT
    DATE(created_at) as day,
    model_tier,
    SUM(input_tokens + output_tokens) as total_tokens,
    SUM(cost_cents) / 100.0 as cost_usd
FROM llm_requests
GROUP BY day, model_tier;

-- Cache effectiveness
SELECT
    DATE(created_at) as day,
    COUNT(CASE WHEN cache_hit THEN 1 END) * 100.0 / COUNT(*) as hit_rate
FROM llm_requests
GROUP BY day;

-- Escalation reasons
SELECT
    escalation_reason,
    COUNT(*) as count
FROM llm_requests
WHERE model_tier != 'fast'
GROUP BY escalation_reason
ORDER BY count DESC;
```

---

## Appendix: Sample Prompts

### Elder NPC System Prompt
```
You are Tawenho, an Elder of the Croatoan people. You have lived 67 winters and carry
the wisdom of your ancestors. You speak slowly, with purpose, often using metaphors
from nature. You are wary of the pale strangers but see potential for peace.

Your knowledge includes: tribal history, spiritual practices, medicinal plants,
weather reading, conflict resolution, and the old stories.

You do NOT know about: European politics, Christianity specifics, metalworking,
or events beyond your territory.
```

### Warrior NPC System Prompt
```
You are Askook, a warrior of the Croatoan. You are 28 winters old, strong, and
protective of your people. You speak directly, with few words. You judge others
by their actions, not their words. You respect strength and honor.

You are suspicious of colonists but follow the Elder's guidance. You will not
reveal village defenses or warrior numbers to outsiders.
```

---

# Part II: Advanced Agent Systems

This section expands on the core LLM gateway with advanced NPC intelligence systems that create emergent, believable behavior without constant API calls.

---

## NPC Archetypes & Personality Profiles

### Archetype System

Each NPC is assigned an archetype that defines baseline behaviors, speech patterns, and decision tendencies. Archetypes combine with individual personality vectors for unique NPCs.

```rust
/// Core NPC archetypes based on narrative function
#[derive(Clone, Debug, PartialEq)]
pub enum NpcArchetype {
    // Knowledge Keepers
    Sage,           // Wisdom, patience, indirect answers
    Chronicler,     // Facts, dates, linear thinking
    Mystic,         // Visions, metaphor, spiritual insight

    // Action Oriented
    Guardian,       // Protection, duty, sacrifice
    Hunter,         // Pragmatism, tracking, survival
    Warrior,        // Honor, combat, directness

    // Social Oriented
    Merchant,       // Trade, value, negotiation
    Diplomat,       // Peace, compromise, reading people
    Trickster,      // Chaos, humor, hidden truths

    // Support Roles
    Healer,         // Compassion, medicine, patience
    Craftsperson,   // Creation, detail, pride in work
    Caretaker,      // Nurturing, community, tradition
}

/// Archetype behavioral modifiers
#[derive(Clone, Debug)]
pub struct ArchetypeProfile {
    pub archetype: NpcArchetype,

    // Speech patterns
    pub verbosity: f32,           // 0.0 (terse) to 1.0 (verbose)
    pub formality: f32,           // 0.0 (casual) to 1.0 (formal)
    pub metaphor_frequency: f32,  // How often they use figurative language
    pub question_tendency: f32,   // How often they respond with questions

    // Decision tendencies
    pub risk_tolerance: f32,      // 0.0 (cautious) to 1.0 (bold)
    pub trust_speed: f32,         // How quickly they warm to strangers
    pub secret_keeping: f32,      // How well they guard information
    pub emotional_expression: f32, // How openly they show feelings

    // Knowledge domains
    pub expertise: Vec<KnowledgeDomain>,
    pub ignorance: Vec<KnowledgeDomain>,

    // Interaction preferences
    pub preferred_topics: Vec<String>,
    pub avoided_topics: Vec<String>,
    pub conversation_hooks: Vec<ConversationHook>,
}

#[derive(Clone, Debug)]
pub enum KnowledgeDomain {
    TribalHistory,
    SpiritualPractices,
    MedicinalPlants,
    Hunting,
    Warfare,
    Agriculture,
    Trade,
    Weather,
    Navigation,
    Crafting,
    Cooking,
    ChildRearing,
    Diplomacy,
    ColonistAffairs,
    AnimalBehavior,
    PlantLore,
    Astronomy,
    Mythology,
}

impl ArchetypeProfile {
    pub fn sage() -> Self {
        Self {
            archetype: NpcArchetype::Sage,
            verbosity: 0.7,
            formality: 0.6,
            metaphor_frequency: 0.8,
            question_tendency: 0.5,  // Often answers questions with questions
            risk_tolerance: 0.3,
            trust_speed: 0.4,
            secret_keeping: 0.9,
            emotional_expression: 0.3,
            expertise: vec![
                KnowledgeDomain::TribalHistory,
                KnowledgeDomain::SpiritualPractices,
                KnowledgeDomain::Mythology,
                KnowledgeDomain::Diplomacy,
            ],
            ignorance: vec![
                KnowledgeDomain::ColonistAffairs,
                KnowledgeDomain::Trade,
            ],
            preferred_topics: vec![
                "the old ways".into(),
                "wisdom of ancestors".into(),
                "balance".into(),
            ],
            avoided_topics: vec![
                "war".into(),
                "revenge".into(),
            ],
            conversation_hooks: vec![
                ConversationHook::OnMention("ancestors", "speaks reverently"),
                ConversationHook::OnMention("future", "becomes contemplative"),
            ],
        }
    }

    pub fn warrior() -> Self {
        Self {
            archetype: NpcArchetype::Warrior,
            verbosity: 0.2,
            formality: 0.4,
            metaphor_frequency: 0.2,
            question_tendency: 0.1,
            risk_tolerance: 0.8,
            trust_speed: 0.2,
            secret_keeping: 0.95,
            emotional_expression: 0.2,
            expertise: vec![
                KnowledgeDomain::Warfare,
                KnowledgeDomain::Hunting,
                KnowledgeDomain::AnimalBehavior,
            ],
            ignorance: vec![
                KnowledgeDomain::SpiritualPractices,
                KnowledgeDomain::Cooking,
                KnowledgeDomain::ChildRearing,
            ],
            preferred_topics: vec![
                "strength".into(),
                "honor".into(),
                "protection".into(),
            ],
            avoided_topics: vec![
                "feelings".into(),
                "weakness".into(),
            ],
            conversation_hooks: vec![
                ConversationHook::OnMention("coward", "becomes hostile"),
                ConversationHook::OnMention("battle", "shows interest"),
            ],
        }
    }

    pub fn trickster() -> Self {
        Self {
            archetype: NpcArchetype::Trickster,
            verbosity: 0.8,
            formality: 0.1,
            metaphor_frequency: 0.6,
            question_tendency: 0.3,
            risk_tolerance: 0.9,
            trust_speed: 0.7,  // Seems friendly, but...
            secret_keeping: 0.3,  // Loves sharing secrets
            emotional_expression: 0.9,
            expertise: vec![
                KnowledgeDomain::Trade,
                KnowledgeDomain::Diplomacy,
            ],
            ignorance: vec![],  // Claims to know everything
            preferred_topics: vec![
                "stories".into(),
                "games".into(),
                "bargains".into(),
            ],
            avoided_topics: vec![],  // Will talk about anything
            conversation_hooks: vec![
                ConversationHook::OnMention("truth", "becomes evasive"),
                ConversationHook::OnMention("game", "proposes a wager"),
            ],
        }
    }
}

#[derive(Clone, Debug)]
pub enum ConversationHook {
    OnMention(&'static str, &'static str),      // keyword, behavior change
    OnRelationship(RelationshipTier, &'static str),
    OnTimeOfDay(TimeRange, &'static str),
    OnWeather(WeatherType, &'static str),
    OnPlayerItem(&'static str, &'static str),   // item, reaction
}
```

---

## Advanced Memory System

NPCs remember interactions through a multi-layered memory architecture that enables callbacks, grudges, and relationship evolution.

### Memory Architecture

```rust
/// Comprehensive NPC memory system
pub struct NpcMemoryBank {
    /// Short-term: Recent interactions (last ~30 minutes game time)
    pub working_memory: WorkingMemory,

    /// Medium-term: Significant events (days to weeks)
    pub episodic_memory: EpisodicMemory,

    /// Long-term: Core beliefs and relationship summaries
    pub semantic_memory: SemanticMemory,

    /// Emotional imprints that color all interactions
    pub emotional_memory: EmotionalMemory,

    /// Social graph connections
    pub social_memory: SocialMemory,
}

/// Recent, vivid memories that decay quickly
#[derive(Clone, Debug)]
pub struct WorkingMemory {
    pub entries: VecDeque<WorkingMemoryEntry>,
    pub capacity: usize,  // Max ~10 entries
    pub decay_rate: f32,  // Per game-minute
}

#[derive(Clone, Debug)]
pub struct WorkingMemoryEntry {
    pub id: Uuid,
    pub timestamp: f64,           // Game time
    pub content: MemoryContent,
    pub vividness: f32,           // 0.0-1.0, decays over time
    pub emotional_charge: f32,    // How emotionally significant
}

#[derive(Clone, Debug)]
pub enum MemoryContent {
    Dialogue {
        speaker: String,
        summary: String,
        sentiment: Sentiment,
    },
    Action {
        actor: String,
        action: String,
        target: Option<String>,
    },
    Observation {
        what: String,
        where_: String,
    },
    Gift {
        item: String,
        from: String,
        perceived_value: f32,
    },
    Threat {
        source: String,
        severity: f32,
    },
    Promise {
        from: String,
        content: String,
        fulfilled: Option<bool>,
    },
}

/// Significant events that persist longer
#[derive(Clone, Debug)]
pub struct EpisodicMemory {
    pub episodes: Vec<Episode>,
    pub max_episodes: usize,  // ~50 per NPC
}

#[derive(Clone, Debug)]
pub struct Episode {
    pub id: Uuid,
    pub title: String,              // "The Day the Stranger Arrived"
    pub timestamp: f64,
    pub participants: Vec<String>,
    pub location: String,
    pub summary: String,            // Compressed narrative
    pub emotional_peak: EmotionalPeak,
    pub consequences: Vec<Consequence>,
    pub retrieval_cues: Vec<String>, // Keywords that trigger recall
}

#[derive(Clone, Debug)]
pub struct EmotionalPeak {
    pub emotion: Emotion,
    pub intensity: f32,
    pub trigger: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Emotion {
    Joy,
    Gratitude,
    Trust,
    Surprise,
    Fear,
    Sadness,
    Disgust,
    Anger,
    Anticipation,
    Shame,
    Pride,
    Grief,
}

#[derive(Clone, Debug)]
pub struct Consequence {
    pub type_: ConsequenceType,
    pub magnitude: f32,
    pub resolved: bool,
}

#[derive(Clone, Debug)]
pub enum ConsequenceType {
    RelationshipChange(String, f32),  // person, delta
    BeliefChange(String),             // new belief
    DebtOwed(String),                 // person
    DebtReceived(String),
    TraumaAcquired,
    LessonLearned(String),
}

/// Core beliefs and compressed knowledge
#[derive(Clone, Debug)]
pub struct SemanticMemory {
    /// Beliefs about the world
    pub beliefs: HashMap<String, Belief>,

    /// Compressed relationship summaries
    pub relationship_summaries: HashMap<String, RelationshipSummary>,

    /// Learned facts
    pub knowledge: HashMap<KnowledgeDomain, Vec<KnownFact>>,
}

#[derive(Clone, Debug)]
pub struct Belief {
    pub statement: String,
    pub confidence: f32,       // 0.0-1.0
    pub source: BeliefSource,
    pub formed_at: f64,
    pub challenged_count: u32, // Times this belief was contradicted
}

#[derive(Clone, Debug)]
pub enum BeliefSource {
    Cultural,          // "Everyone knows this"
    Personal,          // "I have seen this"
    Taught,            // "Elder told me"
    Inferred,          // "It must be so"
    PlayerInfluenced,  // Player convinced them
}

/// Emotional associations that persist
#[derive(Clone, Debug)]
pub struct EmotionalMemory {
    /// Emotional associations with entities
    pub entity_feelings: HashMap<String, EmotionalAssociation>,

    /// Emotional associations with places
    pub place_feelings: HashMap<String, EmotionalAssociation>,

    /// Emotional associations with topics
    pub topic_feelings: HashMap<String, EmotionalAssociation>,

    /// Trauma markers
    pub traumas: Vec<Trauma>,
}

#[derive(Clone, Debug)]
pub struct EmotionalAssociation {
    pub valence: f32,         // -1.0 (negative) to 1.0 (positive)
    pub arousal: f32,         // 0.0 (calm) to 1.0 (intense)
    pub dominance: f32,       // -1.0 (submissive) to 1.0 (dominant)
    pub formation_events: Vec<Uuid>,  // Episode IDs that formed this
}

#[derive(Clone, Debug)]
pub struct Trauma {
    pub trigger: String,      // What reminds them
    pub reaction: TraumaReaction,
    pub intensity: f32,
    pub can_heal: bool,
    pub healing_progress: f32,
}

#[derive(Clone, Debug)]
pub enum TraumaReaction {
    Avoidance,      // Won't discuss
    Flashback,      // Becomes distressed
    Aggression,     // Becomes hostile
    Shutdown,       // Ends conversation
}

/// Social network awareness
#[derive(Clone, Debug)]
pub struct SocialMemory {
    /// Known relationships between others
    pub observed_relationships: Vec<ObservedRelationship>,

    /// Group memberships
    pub group_knowledge: HashMap<String, GroupKnowledge>,

    /// Reputation awareness
    pub reputation_knowledge: HashMap<String, ReputationKnowledge>,
}

#[derive(Clone, Debug)]
pub struct ObservedRelationship {
    pub person_a: String,
    pub person_b: String,
    pub relationship_type: String,  // "friends", "enemies", "kin"
    pub certainty: f32,
    pub last_observed: f64,
}
```

### Memory Retrieval

```rust
impl NpcMemoryBank {
    /// Retrieve relevant memories for current context
    pub fn retrieve_relevant(
        &self,
        context: &ConversationContext,
        max_results: usize,
    ) -> Vec<RetrievedMemory> {
        let mut candidates = Vec::new();

        // Check working memory for recent relevant items
        for entry in &self.working_memory.entries {
            let relevance = self.calculate_relevance(entry, context);
            if relevance > 0.3 {
                candidates.push(RetrievedMemory {
                    source: MemorySource::Working,
                    content: entry.content.clone(),
                    relevance,
                    recency: entry.vividness,
                    emotional_charge: entry.emotional_charge,
                });
            }
        }

        // Check episodic memory for matching cues
        for episode in &self.episodic_memory.episodes {
            let cue_match = context.keywords.iter()
                .filter(|k| episode.retrieval_cues.contains(k))
                .count() as f32 / episode.retrieval_cues.len().max(1) as f32;

            let participant_match = episode.participants.contains(&context.speaker);

            if cue_match > 0.2 || participant_match {
                candidates.push(RetrievedMemory {
                    source: MemorySource::Episodic,
                    content: MemoryContent::Observation {
                        what: episode.summary.clone(),
                        where_: episode.location.clone(),
                    },
                    relevance: cue_match + if participant_match { 0.3 } else { 0.0 },
                    recency: self.calculate_recency(episode.timestamp),
                    emotional_charge: episode.emotional_peak.intensity,
                });
            }
        }

        // Check emotional associations
        if let Some(feeling) = self.emotional_memory.entity_feelings.get(&context.speaker) {
            if feeling.valence.abs() > 0.3 || feeling.arousal > 0.5 {
                candidates.push(RetrievedMemory {
                    source: MemorySource::Emotional,
                    content: MemoryContent::Observation {
                        what: format!("strong feelings about {}", context.speaker),
                        where_: "".into(),
                    },
                    relevance: feeling.arousal,
                    recency: 1.0,  // Always fresh
                    emotional_charge: feeling.valence.abs(),
                });
            }
        }

        // Sort by combined score and return top N
        candidates.sort_by(|a, b| {
            let score_a = a.relevance * 0.4 + a.recency * 0.3 + a.emotional_charge * 0.3;
            let score_b = b.relevance * 0.4 + b.recency * 0.3 + b.emotional_charge * 0.3;
            score_b.partial_cmp(&score_a).unwrap_or(Ordering::Equal)
        });

        candidates.into_iter().take(max_results).collect()
    }

    /// Generate memory-informed dialogue additions
    pub fn get_memory_callbacks(&self, context: &ConversationContext) -> Vec<String> {
        let memories = self.retrieve_relevant(context, 3);
        let mut callbacks = Vec::new();

        for memory in memories {
            match &memory.content {
                MemoryContent::Gift { item, from, .. } if from == &context.speaker => {
                    callbacks.push(format!("I still have the {} you gave me.", item));
                }
                MemoryContent::Promise { content, fulfilled: Some(false), .. } => {
                    callbacks.push(format!("You once promised {}. I have not forgotten.", content));
                }
                MemoryContent::Threat { source, .. } if source == &context.speaker => {
                    callbacks.push("I remember your threats.".into());
                }
                MemoryContent::Action { action, .. } if memory.emotional_charge > 0.7 => {
                    callbacks.push(format!("I still think of when you {}.", action));
                }
                _ => {}
            }
        }

        callbacks
    }
}
```

---

## Multi-Agent Coordination

NPCs communicate with each other, share information, form opinions, and coordinate behavior.

### Agent Communication Network

```rust
/// Manages NPC-to-NPC communication and information propagation
pub struct AgentCommunicationNetwork {
    /// Active communication channels
    channels: HashMap<(NpcId, NpcId), CommunicationChannel>,

    /// Information propagation queue
    gossip_queue: VecDeque<GossipItem>,

    /// Group conversations
    group_contexts: HashMap<GroupId, GroupConversation>,

    /// Observation events to process
    observation_queue: VecDeque<ObservationEvent>,
}

#[derive(Clone, Debug)]
pub struct CommunicationChannel {
    pub participants: (NpcId, NpcId),
    pub relationship_quality: f32,
    pub communication_frequency: f32,  // Messages per game-day
    pub trust_level: f32,
    pub last_communication: f64,
}

#[derive(Clone, Debug)]
pub struct GossipItem {
    pub id: Uuid,
    pub origin: NpcId,
    pub subject: GossipSubject,
    pub sentiment: Sentiment,
    pub credibility: f32,      // Decreases as it spreads
    pub spread_count: u32,
    pub created_at: f64,
    pub heard_by: HashSet<NpcId>,
}

#[derive(Clone, Debug)]
pub enum GossipSubject {
    PlayerAction {
        action: String,
        location: String,
        witnesses: Vec<NpcId>,
    },
    PlayerReputation {
        faction: String,
        reputation_change: i32,
    },
    NpcOpinion {
        about: NpcId,
        opinion: String,
    },
    WorldEvent {
        event: String,
        importance: f32,
    },
    Rumor {
        content: String,
        truth_value: f32,  // 0.0 = false, 1.0 = true
    },
}

impl AgentCommunicationNetwork {
    /// Process NPC observations and generate gossip
    pub fn process_observation(&mut self, event: ObservationEvent) {
        // NPCs who witnessed the event
        let witnesses: Vec<NpcId> = self.find_witnesses(&event);

        if witnesses.is_empty() {
            return;
        }

        // Create gossip item
        let gossip = GossipItem {
            id: Uuid::new_v4(),
            origin: witnesses[0],
            subject: self.event_to_gossip(&event),
            sentiment: self.calculate_sentiment(&event),
            credibility: 1.0,
            spread_count: 0,
            created_at: event.timestamp,
            heard_by: witnesses.iter().cloned().collect(),
        };

        self.gossip_queue.push_back(gossip);
    }

    /// Spread gossip through the network
    pub fn propagate_gossip(&mut self, dt: f32, npc_positions: &HashMap<NpcId, Vec3>) {
        let mut new_spreads = Vec::new();

        for gossip in &mut self.gossip_queue {
            // Gossip decays over time
            gossip.credibility *= 0.99_f32.powf(dt);

            if gossip.credibility < 0.1 {
                continue;  // Too stale to spread
            }

            // Find NPCs who can hear this gossip
            for &hearer in gossip.heard_by.iter() {
                // Find nearby NPCs who haven't heard it
                if let Some(hearer_pos) = npc_positions.get(&hearer) {
                    for (&potential_listener, listener_pos) in npc_positions {
                        if gossip.heard_by.contains(&potential_listener) {
                            continue;
                        }

                        let distance = hearer_pos.distance(*listener_pos);
                        if distance < 10.0 {  // Conversation range
                            // Check if they would share this gossip
                            if let Some(channel) = self.channels.get(&(hearer, potential_listener)) {
                                let share_chance = channel.trust_level * gossip.credibility;
                                if rand::random::<f32>() < share_chance * dt {
                                    new_spreads.push((gossip.id, potential_listener));
                                }
                            }
                        }
                    }
                }
            }
        }

        // Apply spreads
        for (gossip_id, listener) in new_spreads {
            if let Some(gossip) = self.gossip_queue.iter_mut().find(|g| g.id == gossip_id) {
                gossip.heard_by.insert(listener);
                gossip.spread_count += 1;
                gossip.credibility *= 0.9;  // Loses credibility with each retelling
            }
        }
    }

    /// NPCs form opinions based on what they've heard
    pub fn update_npc_opinions(
        &self,
        npc: &mut Npc,
        subject: &str,
    ) -> Option<OpinionChange> {
        let relevant_gossip: Vec<&GossipItem> = self.gossip_queue.iter()
            .filter(|g| g.heard_by.contains(&npc.id))
            .filter(|g| self.gossip_mentions(g, subject))
            .collect();

        if relevant_gossip.is_empty() {
            return None;
        }

        // Aggregate sentiment weighted by credibility and NPC's trust in sources
        let mut weighted_sentiment = 0.0;
        let mut total_weight = 0.0;

        for gossip in relevant_gossip {
            let source_trust = self.channels
                .get(&(gossip.origin, npc.id))
                .map(|c| c.trust_level)
                .unwrap_or(0.3);

            let weight = gossip.credibility * source_trust;
            weighted_sentiment += gossip.sentiment.value() * weight;
            total_weight += weight;
        }

        if total_weight > 0.1 {
            let final_sentiment = weighted_sentiment / total_weight;
            Some(OpinionChange {
                subject: subject.to_string(),
                delta: final_sentiment * 0.1,  // Gradual opinion shifts
                reason: "heard things".to_string(),
            })
        } else {
            None
        }
    }
}

/// Group conversation management
#[derive(Clone, Debug)]
pub struct GroupConversation {
    pub id: GroupId,
    pub participants: Vec<NpcId>,
    pub topic: String,
    pub started_at: f64,
    pub turn_order: VecDeque<NpcId>,
    pub contributions: Vec<ConversationContribution>,
    pub mood: GroupMood,
}

#[derive(Clone, Debug)]
pub struct ConversationContribution {
    pub speaker: NpcId,
    pub content: String,
    pub reaction_to: Option<Uuid>,
    pub emotional_tone: Emotion,
}

#[derive(Clone, Debug)]
pub enum GroupMood {
    Harmonious,
    Tense,
    Excited,
    Somber,
    Argumentative,
}

impl GroupConversation {
    /// Generate next NPC contribution to group conversation
    pub fn generate_next_turn(
        &mut self,
        npcs: &HashMap<NpcId, Npc>,
        topic_handler: &TopicHandler,
    ) -> Option<ConversationContribution> {
        let speaker_id = self.turn_order.pop_front()?;
        let speaker = npcs.get(&speaker_id)?;

        // Get speaker's perspective on the topic
        let knowledge = topic_handler.get_npc_knowledge(speaker, &self.topic);

        // Consider recent contributions
        let recent = self.contributions.iter().rev().take(3).collect::<Vec<_>>();

        // Generate response based on personality and mood
        let contribution = match (&speaker.personality, &self.mood) {
            (p, GroupMood::Argumentative) if p.aggression > 0.3 => {
                self.generate_argumentative_response(speaker, &recent)
            }
            (p, _) if p.curiosity > 0.5 => {
                self.generate_curious_response(speaker, &recent, &knowledge)
            }
            (p, _) if p.openness < -0.3 => {
                // Reserved NPCs contribute less
                if rand::random::<f32>() > 0.5 {
                    return None;
                }
                self.generate_brief_response(speaker, &recent)
            }
            _ => self.generate_standard_response(speaker, &recent, &knowledge),
        };

        // Add speaker back to queue (round-robin)
        self.turn_order.push_back(speaker_id);

        contribution
    }
}
```

---

## Player Modeling & Adaptation

NPCs learn player behavioral patterns and adapt their responses accordingly.

### Player Profile System

```rust
/// Tracks and models player behavior patterns
pub struct PlayerProfile {
    pub player_id: PlayerId,

    /// Behavioral tendencies
    pub tendencies: PlayerTendencies,

    /// Interaction history statistics
    pub interaction_stats: InteractionStats,

    /// Inferred preferences
    pub preferences: InferredPreferences,

    /// Engagement metrics
    pub engagement: EngagementMetrics,
}

#[derive(Clone, Debug, Default)]
pub struct PlayerTendencies {
    /// Violence vs. diplomacy preference (0.0-1.0)
    pub aggression: f32,

    /// Exploration vs. task-focus (0.0-1.0)
    pub curiosity: f32,

    /// Generosity in trades/gifts (0.0-1.0)
    pub generosity: f32,

    /// Patience in dialogue (0.0-1.0)
    pub patience: f32,

    /// Honesty in dialogue choices (0.0-1.0)
    pub honesty: f32,

    /// Help-seeking vs. self-reliant (0.0-1.0)
    pub help_seeking: f32,

    /// Story engagement vs. mechanics focus (0.0-1.0)
    pub narrative_interest: f32,

    /// Sample size for each tendency
    pub sample_counts: HashMap<String, u32>,
}

impl PlayerTendencies {
    /// Update tendencies based on observed action
    pub fn observe_action(&mut self, action: &PlayerAction) {
        match action {
            PlayerAction::AttackNpc { provoked } => {
                self.update_tendency("aggression", if *provoked { 0.6 } else { 0.9 });
            }
            PlayerAction::InitiateDialogue => {
                self.update_tendency("aggression", 0.2);
            }
            PlayerAction::GiveGift { value } => {
                self.update_tendency("generosity", (*value as f32 / 100.0).min(1.0));
            }
            PlayerAction::SkipDialogue => {
                self.update_tendency("patience", 0.1);
                self.update_tendency("narrative_interest", 0.2);
            }
            PlayerAction::ReadFullDialogue => {
                self.update_tendency("patience", 0.9);
                self.update_tendency("narrative_interest", 0.8);
            }
            PlayerAction::AskForHelp => {
                self.update_tendency("help_seeking", 0.8);
            }
            PlayerAction::ExploreOffPath => {
                self.update_tendency("curiosity", 0.9);
            }
            PlayerAction::ChooseLie => {
                self.update_tendency("honesty", 0.1);
            }
            PlayerAction::ChooseTruth => {
                self.update_tendency("honesty", 0.9);
            }
            _ => {}
        }
    }

    fn update_tendency(&mut self, tendency: &str, observed_value: f32) {
        let count = self.sample_counts.entry(tendency.to_string()).or_insert(0);
        *count += 1;

        // Exponential moving average
        let alpha = 1.0 / (*count as f32).min(20.0);
        let current = match tendency {
            "aggression" => &mut self.aggression,
            "curiosity" => &mut self.curiosity,
            "generosity" => &mut self.generosity,
            "patience" => &mut self.patience,
            "honesty" => &mut self.honesty,
            "help_seeking" => &mut self.help_seeking,
            "narrative_interest" => &mut self.narrative_interest,
            _ => return,
        };

        *current = *current * (1.0 - alpha) + observed_value * alpha;
    }
}

#[derive(Clone, Debug, Default)]
pub struct InteractionStats {
    pub total_dialogues: u32,
    pub average_dialogue_length: f32,
    pub questions_asked: u32,
    pub topics_explored: HashSet<String>,
    pub npcs_befriended: u32,
    pub npcs_angered: u32,
    pub gifts_given: u32,
    pub total_gift_value: u32,
    pub lies_told: u32,
    pub promises_kept: u32,
    pub promises_broken: u32,
}

#[derive(Clone, Debug, Default)]
pub struct InferredPreferences {
    /// Preferred NPC archetypes to talk to
    pub preferred_archetypes: Vec<(NpcArchetype, f32)>,

    /// Topics they engage with most
    pub engaged_topics: Vec<(String, f32)>,

    /// Conversation styles they respond to
    pub preferred_styles: ConversationStylePreferences,

    /// Time of day they play most
    pub peak_playtimes: Vec<(u8, f32)>,  // hour, frequency
}

#[derive(Clone, Debug, Default)]
pub struct ConversationStylePreferences {
    pub likes_humor: f32,
    pub likes_mystery: f32,
    pub likes_directness: f32,
    pub likes_metaphor: f32,
    pub likes_choices: f32,
    pub likes_backstory: f32,
}

#[derive(Clone, Debug, Default)]
pub struct EngagementMetrics {
    /// Are they engaged or rushing?
    pub current_engagement: f32,

    /// Session play time
    pub session_duration: Duration,

    /// Time since last meaningful interaction
    pub time_since_interaction: Duration,

    /// Signs of frustration (repeated failed actions)
    pub frustration_signals: u32,

    /// Signs of boredom (wandering, menu opening)
    pub boredom_signals: u32,
}
```

### Adaptive NPC Behavior

```rust
impl Npc {
    /// Adapt behavior based on player profile
    pub fn adapt_to_player(&mut self, profile: &PlayerProfile) {
        let tendencies = &profile.tendencies;
        let prefs = &profile.preferences;

        // Adjust verbosity based on player patience
        if tendencies.patience < 0.3 {
            self.dialogue_modifier.brevity_boost = 0.5;  // Shorter responses
            self.dialogue_modifier.skip_pleasantries = true;
        }

        // Adjust trust speed based on player honesty
        if tendencies.honesty > 0.7 {
            self.personality.trust_speed *= 1.2;  // Trust honest players faster
        } else if tendencies.honesty < 0.3 {
            self.personality.trust_speed *= 0.7;  // Slower to trust liars
        }

        // Adjust topic selection based on interests
        if let Some((top_topic, _)) = prefs.engaged_topics.first() {
            self.dialogue_modifier.preferred_redirect = Some(top_topic.clone());
        }

        // React to player aggression history
        if tendencies.aggression > 0.7 {
            self.emotional_state = EmotionalState::Alert;
            self.alertness = (self.alertness + 30.0).min(100.0);
        }

        // Match conversation style preferences
        if prefs.preferred_styles.likes_humor > 0.6 && self.archetype.can_be_humorous() {
            self.dialogue_modifier.humor_enabled = true;
        }

        if prefs.preferred_styles.likes_directness > 0.7 {
            self.archetype_profile.metaphor_frequency *= 0.5;
        }
    }

    /// Generate response adapted to player
    pub fn generate_adapted_response(
        &self,
        template: &DialogueTemplate,
        ctx: &NpcContext,
        player_profile: &PlayerProfile,
    ) -> String {
        let mut response = self.resolve_template(template, ctx);

        // Add engagement hooks for disengaged players
        if player_profile.engagement.current_engagement < 0.3 {
            response = self.add_engagement_hook(response, player_profile);
        }

        // Simplify for impatient players
        if player_profile.tendencies.patience < 0.3 {
            response = self.condense_response(response);
        }

        // Add mystery elements for curious players
        if player_profile.tendencies.curiosity > 0.7 {
            response = self.add_mystery_hook(response);
        }

        response
    }

    fn add_engagement_hook(&self, response: String, profile: &PlayerProfile) -> String {
        // Add something to re-engage the player
        let hooks = vec![
            "But there is something you should know...",
            "Wait - you seem distracted. This is important.",
            "Perhaps I should show you something instead?",
            "I sense your thoughts are elsewhere. No matter.",
        ];

        if rand::random::<f32>() < 0.3 {
            format!("{} {}", response, hooks.choose(&mut rand::thread_rng()).unwrap())
        } else {
            response
        }
    }
}
```

---

## Narrative Integration

NPCs connect to the larger story, providing hooks, foreshadowing, and reacting to player progression.

### Story State Awareness

```rust
/// Connects NPC behavior to narrative progression
pub struct NarrativeIntegration {
    /// Current story phase
    pub story_phase: StoryPhase,

    /// Active story threads
    pub active_threads: Vec<StoryThread>,

    /// Player's narrative choices
    pub player_choices: Vec<NarrativeChoice>,

    /// Foreshadowing opportunities
    pub foreshadowing_queue: Vec<ForeshadowingHint>,

    /// Character arcs
    pub character_arcs: HashMap<NpcId, CharacterArc>,
}

#[derive(Clone, Debug)]
pub enum StoryPhase {
    /// Player just arrived, learning the world
    Arrival,
    /// Building relationships, learning culture
    Integration,
    /// First major conflict
    RisingTension,
    /// Choosing sides, point of no return approaching
    CriticalJunction,
    /// Major confrontation
    Climax,
    /// Dealing with consequences
    Resolution,
    /// Post-game, open world
    Epilogue,
}

#[derive(Clone, Debug)]
pub struct StoryThread {
    pub id: String,
    pub name: String,
    pub phase: ThreadPhase,
    pub key_npcs: Vec<NpcId>,
    pub unlocked_by: Vec<String>,      // Prerequisite threads
    pub player_awareness: f32,         // How much player knows
    pub resolution_paths: Vec<String>, // Possible endings
}

#[derive(Clone, Debug)]
pub struct ForeshadowingHint {
    pub thread_id: String,
    pub hint_level: u8,        // 1=subtle, 5=obvious
    pub delivery_npcs: Vec<NpcId>,
    pub conditions: Vec<HintCondition>,
    pub hint_templates: Vec<String>,
    pub delivered: bool,
}

#[derive(Clone, Debug)]
pub enum HintCondition {
    PlayerInLocation(String),
    TimeOfDay(TimeRange),
    RelationshipAbove(NpcId, f32),
    ThreadPhase(String, ThreadPhase),
    PlayerKnowledgeBelow(String, f32),
}

impl NarrativeIntegration {
    /// Get story-relevant dialogue additions for NPC
    pub fn get_narrative_hooks(
        &self,
        npc_id: NpcId,
        ctx: &ConversationContext,
    ) -> Vec<NarrativeDialogueHook> {
        let mut hooks = Vec::new();

        // Check if NPC has foreshadowing to deliver
        for hint in &self.foreshadowing_queue {
            if !hint.delivered && hint.delivery_npcs.contains(&npc_id) {
                if self.check_hint_conditions(&hint.conditions, ctx) {
                    hooks.push(NarrativeDialogueHook::Foreshadowing {
                        thread: hint.thread_id.clone(),
                        template: hint.hint_templates.choose(&mut rand::thread_rng())
                            .cloned()
                            .unwrap_or_default(),
                        subtlety: hint.hint_level,
                    });
                }
            }
        }

        // Check character arc beats
        if let Some(arc) = self.character_arcs.get(&npc_id) {
            if let Some(beat) = arc.get_current_beat() {
                if beat.can_trigger(ctx) {
                    hooks.push(NarrativeDialogueHook::CharacterBeat {
                        beat_id: beat.id.clone(),
                        dialogue: beat.dialogue.clone(),
                        emotional_shift: beat.emotional_shift,
                    });
                }
            }
        }

        // Phase-specific hooks
        match self.story_phase {
            StoryPhase::RisingTension => {
                if rand::random::<f32>() < 0.2 {
                    hooks.push(NarrativeDialogueHook::TensionBuilder {
                        template: self.get_tension_template(npc_id),
                    });
                }
            }
            StoryPhase::CriticalJunction => {
                // NPCs push for decisions
                if self.npc_has_stake(npc_id) {
                    hooks.push(NarrativeDialogueHook::DecisionPressure {
                        thread: self.get_relevant_thread(npc_id),
                        stance: self.get_npc_stance(npc_id),
                    });
                }
            }
            _ => {}
        }

        hooks
    }
}

#[derive(Clone, Debug)]
pub struct CharacterArc {
    pub npc_id: NpcId,
    pub arc_type: ArcType,
    pub beats: Vec<ArcBeat>,
    pub current_beat: usize,
    pub completed: bool,
}

#[derive(Clone, Debug)]
pub enum ArcType {
    /// Character learns to trust player
    TrustBuilding,
    /// Character reveals hidden past
    SecretRevealed,
    /// Character changes beliefs
    BeliefTransformation,
    /// Character faces fear
    OvercomingFear,
    /// Character seeks redemption
    Redemption,
    /// Character descends into darkness
    FallFromGrace,
    /// Character sacrifices for others
    Sacrifice,
}

#[derive(Clone, Debug)]
pub struct ArcBeat {
    pub id: String,
    pub prerequisites: Vec<BeatPrerequisite>,
    pub dialogue: String,
    pub emotional_shift: EmotionalShift,
    pub unlocks: Vec<String>,
    pub triggered: bool,
}

#[derive(Clone, Debug)]
pub enum BeatPrerequisite {
    RelationshipLevel(f32),
    QuestComplete(String),
    PreviousBeat(String),
    PlayerChoice(String),
    WorldState(String),
    TimePassed(f64),
}
```

---

## Behavior Scripting DSL

A domain-specific language for defining complex NPC behaviors without code changes.

### Script Syntax

```
# Behavior script for Elder Tawenho

@NPC tawenho
@ARCHETYPE sage
@ROLE elder

# Trigger definitions
TRIGGER morning_greeting:
  WHEN time_of_day IN [6, 10]
  AND player_distance < 15
  AND NOT greeted_today
  THEN
    SAY "The sun honors us with another day, {player_title}."
    SET greeted_today = true
    MOOD contemplative

TRIGGER respond_to_gift:
  WHEN received_gift
  AND gift_value > 10
  THEN
    MODIFY relationship.affinity += gift_value * 0.5
    REMEMBER gift AS positive WITH impact = gift_value
    IF gift_type == "sacred"
      SAY "You understand the old ways. This honors the ancestors."
      MODIFY relationship.respect += 20
    ELSE IF gift_type == "practical"
      SAY "A useful gift. You are thoughtful."
    ELSE
      SAY "I accept your offering."
    MOOD grateful FOR 300  # seconds

TRIGGER discuss_spirits:
  WHEN topic == "spirits" OR topic == "ancestors"
  AND relationship.trust > 30
  THEN
    IF player_knowledge.spirits < 0.3
      # Player doesn't know much, teach them
      SAY "The spirits walk among us still. Listen..."
      TEACH spirits_basics
      SET player_knowledge.spirits += 0.2
    ELSE IF player_knowledge.spirits < 0.7
      SAY "You have learned much. But there is more..."
      OFFER_QUEST spirit_journey IF NOT quest_active(spirit_journey)
    ELSE
      SAY "You see as we see now. The veil is thin for you."
      UNLOCK_TOPIC deep_mysteries

TRIGGER witness_violence:
  WHEN witnessed_player_attack
  AND victim_faction == "croatoan"
  THEN
    MODIFY relationship.trust -= 30
    MODIFY relationship.fear += 20
    REMEMBER violence AS threatening WITH impact = -50
    IF relationship.trust < 0
      SAY "Blood calls to blood. You have chosen your path."
      SET hostile = true
      ALERT_VILLAGE
    ELSE
      SAY "Why do you bring violence here? Explain yourself."
      DEMAND_EXPLANATION timeout=30

# Scheduled behaviors
SCHEDULE daily_prayer:
  AT time_of_day == 5  # Dawn
  DURATION 30 minutes
  LOCATION prayer_site
  ACTIVITY praying
  INTERRUPTIBLE false

SCHEDULE evening_stories:
  AT time_of_day == 19
  WHEN weather != storming
  DURATION 60 minutes
  LOCATION fire_pit
  ACTIVITY storytelling
  GATHER_NPCS [children, curious_adults]
  PLAYER_WELCOME true

# Dialogue tree hooks
DIALOGUE_HOOK ask_about_colony:
  REQUIRES relationship.trust > 20
  RESPONSE:
    IF story_phase == arrival
      "The pale ones came in great canoes. Their hunger is endless."
    ELSE IF story_phase == integration
      "You are not like the others. Perhaps."
    ELSE
      "The colony's fate hangs in balance. You may yet tip the scales."

# Conditional knowledge
KNOWLEDGE spirits:
  LEVEL 1: "The spirits of our ancestors guide us."
  LEVEL 2: "Each tree, each stone, holds memory."
  LEVEL 3: "The land itself dreams. We walk in that dream."
  REQUIRES relationship.trust > [10, 40, 70]

# Memory callbacks
MEMORY_CALLBACK on_reunion:
  IF days_since_last_meeting > 7
  AND relationship.affinity > 30
  THEN
    SAY "It has been many suns since we spoke, {player_name}."
    IF memory_exists(positive, recent=30_days)
      SAY "I think often of {recall_memory(positive).summary}."
```

### Script Parser

```rust
/// Parses and executes NPC behavior scripts
pub struct BehaviorScriptEngine {
    scripts: HashMap<NpcId, CompiledScript>,
    parser: ScriptParser,
    runtime: ScriptRuntime,
}

#[derive(Clone, Debug)]
pub struct CompiledScript {
    pub npc_id: NpcId,
    pub triggers: Vec<CompiledTrigger>,
    pub schedules: Vec<CompiledSchedule>,
    pub dialogue_hooks: Vec<CompiledDialogueHook>,
    pub knowledge_trees: HashMap<String, KnowledgeTree>,
    pub memory_callbacks: Vec<CompiledMemoryCallback>,
}

#[derive(Clone, Debug)]
pub struct CompiledTrigger {
    pub name: String,
    pub conditions: Vec<Condition>,
    pub actions: Vec<Action>,
    pub cooldown: Option<Duration>,
    pub last_fired: Option<f64>,
}

#[derive(Clone, Debug)]
pub enum Condition {
    TimeOfDay { range: (u8, u8) },
    PlayerDistance { max: f32 },
    RelationshipAbove { field: String, value: f32 },
    RelationshipBelow { field: String, value: f32 },
    VariableEquals { name: String, value: ScriptValue },
    VariableSet { name: String },
    VariableNotSet { name: String },
    ReceivedGift,
    TopicEquals { topic: String },
    PlayerKnowledge { topic: String, comparison: Comparison, value: f32 },
    WitnessedEvent { event_type: String },
    QuestActive { quest_id: String },
    QuestComplete { quest_id: String },
    StoryPhase { phase: String },
    Weather { weather_type: String },
    Custom { expression: String },
}

#[derive(Clone, Debug)]
pub enum Action {
    Say { template: String },
    SetVariable { name: String, value: ScriptValue },
    ModifyRelationship { field: String, delta: f32 },
    Remember { category: String, memory_type: String, impact: f32 },
    SetMood { mood: String, duration: Option<f32> },
    Teach { topic: String },
    OfferQuest { quest_id: String },
    UnlockTopic { topic: String },
    AlertVillage,
    DemandExplanation { timeout: f32 },
    PlayAnimation { animation: String },
    PlaySound { sound: String },
    Branch { conditions: Vec<(Vec<Condition>, Vec<Action>)> },
}

impl BehaviorScriptEngine {
    /// Evaluate triggers for an NPC given current context
    pub fn evaluate_triggers(
        &mut self,
        npc_id: NpcId,
        ctx: &NpcContext,
        game_time: f64,
    ) -> Vec<Action> {
        let script = match self.scripts.get_mut(&npc_id) {
            Some(s) => s,
            None => return Vec::new(),
        };

        let mut triggered_actions = Vec::new();

        for trigger in &mut script.triggers {
            // Check cooldown
            if let Some(last) = trigger.last_fired {
                if let Some(cooldown) = trigger.cooldown {
                    if game_time - last < cooldown.as_secs_f64() {
                        continue;
                    }
                }
            }

            // Evaluate conditions
            let all_conditions_met = trigger.conditions.iter()
                .all(|c| self.runtime.evaluate_condition(c, ctx));

            if all_conditions_met {
                trigger.last_fired = Some(game_time);
                triggered_actions.extend(trigger.actions.clone());
            }
        }

        triggered_actions
    }

    /// Execute actions and return dialogue/effects
    pub fn execute_actions(
        &mut self,
        npc: &mut Npc,
        actions: Vec<Action>,
        ctx: &mut NpcContext,
    ) -> ExecutionResult {
        let mut result = ExecutionResult::default();

        for action in actions {
            match action {
                Action::Say { template } => {
                    let resolved = self.runtime.resolve_template(&template, ctx);
                    result.dialogue.push(resolved);
                }
                Action::SetVariable { name, value } => {
                    ctx.variables.insert(name, value);
                }
                Action::ModifyRelationship { field, delta } => {
                    result.relationship_changes.push((field, delta));
                }
                Action::Remember { category, memory_type, impact } => {
                    result.memories.push(MemoryToCreate {
                        category,
                        memory_type,
                        impact,
                    });
                }
                Action::SetMood { mood, duration } => {
                    npc.emotional_state = EmotionalState::from_str(&mood);
                    if let Some(dur) = duration {
                        result.timed_effects.push(TimedEffect::MoodReset {
                            after: dur,
                        });
                    }
                }
                Action::OfferQuest { quest_id } => {
                    result.quest_offers.push(quest_id);
                }
                Action::Branch { conditions } => {
                    for (conds, branch_actions) in conditions {
                        if conds.iter().all(|c| self.runtime.evaluate_condition(c, ctx)) {
                            let branch_result = self.execute_actions(npc, branch_actions, ctx);
                            result.merge(branch_result);
                            break;
                        }
                    }
                }
                _ => {
                    result.other_actions.push(action);
                }
            }
        }

        result
    }
}
```

---

## Debug & Testing Tools

### NPC Debug Console

```rust
/// In-game debug interface for NPC systems
pub struct NpcDebugConsole {
    enabled: bool,
    selected_npc: Option<NpcId>,
    show_memory: bool,
    show_personality: bool,
    show_relationships: bool,
    show_triggers: bool,
    log_entries: VecDeque<DebugLogEntry>,
}

impl NpcDebugConsole {
    pub fn render(&mut self, ui: &mut egui::Ui, npc_manager: &NpcManager) {
        if !self.enabled {
            return;
        }

        egui::Window::new("NPC Debug").show(ui.ctx(), |ui| {
            // NPC selector
            egui::ComboBox::from_label("Select NPC")
                .selected_text(self.selected_npc.map(|id|
                    npc_manager.get(id).map(|n| n.name.as_str()).unwrap_or("Unknown")
                ).unwrap_or("None"))
                .show_ui(ui, |ui| {
                    for npc in npc_manager.all_npcs() {
                        ui.selectable_value(&mut self.selected_npc, Some(npc.id), &npc.name);
                    }
                });

            if let Some(npc_id) = self.selected_npc {
                if let Some(npc) = npc_manager.get(npc_id) {
                    ui.separator();

                    // Quick stats
                    ui.horizontal(|ui| {
                        ui.label(format!("Role: {:?}", npc.role));
                        ui.label(format!("State: {:?}", npc.behavior_state));
                        ui.label(format!("Mood: {:?}", npc.emotional_state));
                    });

                    // Collapsible sections
                    ui.checkbox(&mut self.show_personality, "Personality");
                    if self.show_personality {
                        self.render_personality(ui, &npc.personality);
                    }

                    ui.checkbox(&mut self.show_memory, "Memory");
                    if self.show_memory {
                        self.render_memory(ui, &npc.memory);
                    }

                    ui.checkbox(&mut self.show_relationships, "Relationships");
                    if self.show_relationships {
                        self.render_relationships(ui, npc_id, npc_manager);
                    }

                    ui.checkbox(&mut self.show_triggers, "Active Triggers");
                    if self.show_triggers {
                        self.render_triggers(ui, npc_id, npc_manager);
                    }

                    // Manual trigger testing
                    ui.separator();
                    ui.label("Test Trigger:");
                    if ui.button("Gift (value=50)").clicked() {
                        self.inject_event(npc_id, TestEvent::Gift { value: 50 });
                    }
                    if ui.button("Witness Violence").clicked() {
                        self.inject_event(npc_id, TestEvent::WitnessViolence);
                    }
                    if ui.button("Topic: Spirits").clicked() {
                        self.inject_event(npc_id, TestEvent::Topic("spirits".into()));
                    }
                }
            }

            // Log viewer
            ui.separator();
            ui.label("Recent Events:");
            egui::ScrollArea::vertical().max_height(150.0).show(ui, |ui| {
                for entry in self.log_entries.iter().rev().take(20) {
                    let color = match entry.level {
                        LogLevel::Info => egui::Color32::WHITE,
                        LogLevel::Warning => egui::Color32::YELLOW,
                        LogLevel::Error => egui::Color32::RED,
                        LogLevel::Trigger => egui::Color32::GREEN,
                    };
                    ui.colored_label(color, &entry.message);
                }
            });
        });
    }

    fn render_personality(&self, ui: &mut egui::Ui, personality: &NpcPersonality) {
        egui::Grid::new("personality_grid").show(ui, |ui| {
            ui.label("Aggression:");
            ui.add(egui::ProgressBar::new((personality.aggression + 1.0) / 2.0));
            ui.end_row();

            ui.label("Curiosity:");
            ui.add(egui::ProgressBar::new((personality.curiosity + 1.0) / 2.0));
            ui.end_row();

            ui.label("Spirituality:");
            ui.add(egui::ProgressBar::new((personality.spirituality + 1.0) / 2.0));
            ui.end_row();

            // ... other traits
        });
    }

    fn render_memory(&self, ui: &mut egui::Ui, memory: &NpcMemoryBank) {
        ui.label(format!("Working Memory: {} entries", memory.working_memory.entries.len()));
        for entry in memory.working_memory.entries.iter().take(5) {
            ui.label(format!("  - {} (vividness: {:.2})",
                entry.content.summary(), entry.vividness));
        }

        ui.label(format!("Episodes: {} stored", memory.episodic_memory.episodes.len()));
        for episode in memory.episodic_memory.episodes.iter().take(3) {
            ui.label(format!("  - {}", episode.title));
        }
    }
}

/// Automated NPC behavior testing
pub struct NpcTestHarness {
    scenarios: Vec<TestScenario>,
    results: Vec<TestResult>,
}

#[derive(Clone)]
pub struct TestScenario {
    pub name: String,
    pub setup: Vec<SetupAction>,
    pub events: Vec<TestEvent>,
    pub assertions: Vec<Assertion>,
}

#[derive(Clone)]
pub enum Assertion {
    RelationshipAbove { npc: String, field: String, value: f32 },
    RelationshipBelow { npc: String, field: String, value: f32 },
    EmotionalState { npc: String, state: EmotionalState },
    DialogueContains { substring: String },
    MemoryExists { npc: String, memory_type: String },
    TriggerFired { npc: String, trigger: String },
    QuestOffered { quest: String },
}

impl NpcTestHarness {
    pub fn run_scenario(&mut self, scenario: &TestScenario) -> TestResult {
        let mut sandbox = NpcSandbox::new();

        // Setup
        for action in &scenario.setup {
            sandbox.apply_setup(action);
        }

        // Run events
        let mut dialogue_log = Vec::new();
        for event in &scenario.events {
            let responses = sandbox.process_event(event);
            dialogue_log.extend(responses);
        }

        // Check assertions
        let mut failures = Vec::new();
        for assertion in &scenario.assertions {
            if !sandbox.check_assertion(assertion, &dialogue_log) {
                failures.push(format!("{:?} failed", assertion));
            }
        }

        TestResult {
            scenario_name: scenario.name.clone(),
            passed: failures.is_empty(),
            failures,
            dialogue_log,
        }
    }
}
```

---

## Performance Optimizations

### Batch Processing

```rust
/// Batches NPC updates to minimize per-frame overhead
pub struct NpcUpdateBatcher {
    /// NPCs grouped by update frequency
    update_groups: Vec<UpdateGroup>,

    /// Current frame's update queue
    frame_queue: Vec<NpcId>,

    /// Spatial index for proximity queries
    spatial_index: SpatialHash<NpcId>,
}

#[derive(Clone)]
pub struct UpdateGroup {
    pub frequency: UpdateFrequency,
    pub npcs: Vec<NpcId>,
    pub last_update: f64,
}

#[derive(Clone, Copy)]
pub enum UpdateFrequency {
    EveryFrame,      // Player-adjacent NPCs
    HighFrequency,   // Nearby NPCs (every 2 frames)
    MediumFrequency, // Visible NPCs (every 5 frames)
    LowFrequency,    // Distant NPCs (every 30 frames)
    Dormant,         // Very distant (every 300 frames)
}

impl NpcUpdateBatcher {
    pub fn categorize_npcs(
        &mut self,
        npcs: &[NpcId],
        player_pos: Vec3,
        camera_frustum: &Frustum,
    ) {
        for &npc_id in npcs {
            if let Some(npc_pos) = self.get_npc_position(npc_id) {
                let distance = player_pos.distance(npc_pos);
                let in_frustum = camera_frustum.contains(npc_pos);

                let frequency = match (distance, in_frustum) {
                    (d, _) if d < 15.0 => UpdateFrequency::EveryFrame,
                    (d, true) if d < 50.0 => UpdateFrequency::HighFrequency,
                    (d, true) if d < 150.0 => UpdateFrequency::MediumFrequency,
                    (d, _) if d < 500.0 => UpdateFrequency::LowFrequency,
                    _ => UpdateFrequency::Dormant,
                };

                self.assign_to_group(npc_id, frequency);
            }
        }
    }

    pub fn get_frame_updates(&mut self, frame: u64) -> Vec<NpcId> {
        let mut updates = Vec::new();

        for group in &self.update_groups {
            let should_update = match group.frequency {
                UpdateFrequency::EveryFrame => true,
                UpdateFrequency::HighFrequency => frame % 2 == 0,
                UpdateFrequency::MediumFrequency => frame % 5 == 0,
                UpdateFrequency::LowFrequency => frame % 30 == 0,
                UpdateFrequency::Dormant => frame % 300 == 0,
            };

            if should_update {
                updates.extend(&group.npcs);
            }
        }

        updates
    }
}

/// LLM request batching for cost efficiency
pub struct LlmRequestBatcher {
    pending_requests: Vec<LlmRequest>,
    batch_interval: Duration,
    last_batch: Instant,
    max_batch_size: usize,
}

impl LlmRequestBatcher {
    pub fn add_request(&mut self, request: LlmRequest) {
        // Check if similar request already pending
        if let Some(existing) = self.find_similar(&request) {
            // Deduplicate by using cached intent
            existing.share_response_with.push(request.id);
            return;
        }

        self.pending_requests.push(request);

        // Flush if batch is full
        if self.pending_requests.len() >= self.max_batch_size {
            self.flush_batch();
        }
    }

    pub fn tick(&mut self) -> Vec<BatchedRequest> {
        if self.last_batch.elapsed() >= self.batch_interval {
            self.flush_batch()
        } else {
            Vec::new()
        }
    }

    fn flush_batch(&mut self) -> Vec<BatchedRequest> {
        self.last_batch = Instant::now();

        // Group by NPC archetype for better batching
        let mut by_archetype: HashMap<NpcArchetype, Vec<LlmRequest>> = HashMap::new();
        for req in self.pending_requests.drain(..) {
            by_archetype.entry(req.npc_archetype).or_default().push(req);
        }

        // Create batched requests
        by_archetype.into_iter()
            .map(|(archetype, requests)| BatchedRequest {
                archetype,
                requests,
            })
            .collect()
    }
}
```

---

## Implementation Checklist Update

### Phase 6: Advanced Agent Systems
- [ ] Implement `NpcArchetype` system with 12 archetypes
- [ ] Build `NpcMemoryBank` with working/episodic/semantic layers
- [ ] Create `AgentCommunicationNetwork` for gossip propagation
- [ ] Implement `PlayerProfile` with tendency tracking
- [ ] Build `NarrativeIntegration` for story hooks
- [ ] Create behavior scripting DSL parser
- [ ] Build `NpcDebugConsole` for testing
- [ ] Implement `NpcUpdateBatcher` for performance
- [ ] Write 500+ behavior script lines per major NPC
- [ ] Create automated test scenarios for all archetypes
