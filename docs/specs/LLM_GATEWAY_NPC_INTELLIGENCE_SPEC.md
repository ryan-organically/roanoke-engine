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
