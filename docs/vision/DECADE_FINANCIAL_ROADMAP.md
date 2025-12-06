# Roanoke: Decade Financial Roadmap & Economic Architecture

**Strategic Document Series:**
- `MARKETPLACE_LOOT_SYSTEM_SPEC.md` - Item drop mechanics and marketplace design
- `DECADE_FINANCIAL_ROADMAP.md` - Years 1-10 economic foundation (this document)
- `TRILLION_DOLLAR_VISION.md` - Years 11-20 trillion-dollar infrastructure thesis

---

## Executive Summary

Roanoke represents a paradigm shift in game economics—a living economy where digital scarcity creates real value, player labor generates measurable wealth, and institutional-grade infrastructure enables capital formation at scale. This document outlines a 10-year roadmap to build the first **institutionally investable game economy** that rivals traditional financial instruments in transparency, liquidity, and returns.

**Core Thesis:** Games generate $200B+ annually, but 99% of that value evaporates at logout. Roanoke captures, crystallizes, and makes tradeable the value that players create—transforming playtime into portable wealth.

---

## Part I: Currency Architecture

### 1.1 Dual-Currency System

Roanoke operates on a **dual-currency model** designed to separate speculative value from utilitarian function:

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         ROANOKE CURRENCY SYSTEM                         │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│   ┌─────────────────────────┐         ┌─────────────────────────┐      │
│   │      WAMPUM (WPM)       │         │     TOBACCO (TBC)       │      │
│   │    Utility Currency     │◄───────►│   Premium Currency      │      │
│   │                         │  Bridge  │                         │      │
│   └─────────────────────────┘         └─────────────────────────┘      │
│              │                                   │                      │
│              ▼                                   ▼                      │
│   ┌─────────────────────────┐         ┌─────────────────────────┐      │
│   │ • Earned through play   │         │ • Purchased with fiat   │      │
│   │ • NPC transactions      │         │ • Earned via events     │      │
│   │ • Crafting costs        │         │ • Premium marketplace   │      │
│   │ • Repair/maintenance    │         │ • Cosmetic purchases    │      │
│   │ • Soft cap: time-gated  │         │ • Hard cap: 100M total  │      │
│   └─────────────────────────┘         └─────────────────────────┘      │
│                                                                         │
│   ┌─────────────────────────────────────────────────────────────┐      │
│   │                    CONVERSION MECHANICS                      │      │
│   │                                                              │      │
│   │  WPM → TBC: Dynamic rate based on supply/demand              │      │
│   │  TBC → WPM: Fixed rate (1 TBC = 1000 WPM minimum)            │      │
│   │  Tax on conversion: 5% burned (deflationary)                 │      │
│   └─────────────────────────────────────────────────────────────┘      │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### 1.2 Wampum (WPM) - The Labor Token

Wampum represents **player effort crystallized into tradeable value**:

```rust
struct WampumEconomy {
    // Supply mechanics
    total_supply: u64,                  // No hard cap
    daily_emission_rate: f64,           // Target: 0.5% of supply per day
    emission_distribution: EmissionDistribution,

    // Sink mechanics
    daily_sink_rate: f64,               // Target: 0.45% of supply per day
    sink_categories: Vec<SinkCategory>,

    // Equilibrium target
    inflation_target: f64,              // 0.05% daily = ~20% annual
    rebalance_window: u32,              // Check every 24 hours
}

struct EmissionDistribution {
    // How new Wampum enters the economy
    hunting_drops: f64,                 // 25% - Animal harvesting
    archaeology_finds: f64,             // 15% - Fossil/artifact discovery
    crafting_sales: f64,                // 20% - NPC purchases
    quest_rewards: f64,                 // 15% - Quest completion
    trading_post_arbitrage: f64,        // 10% - NPC trade routes
    event_participation: f64,           // 10% - Live events
    exploration_bonus: f64,             // 5%  - New area discovery
}

struct SinkCategory {
    category: String,
    daily_target_percent: f64,
    actual_last_24h: u64,
    adjustment_needed: f64,
}

// Primary sinks (value destruction)
const WAMPUM_SINKS: &[(&str, f64)] = &[
    ("Crafting material costs", 0.15),      // Materials consumed
    ("Repair costs", 0.10),                  // Durability restoration
    ("Fast travel", 0.05),                   // Convenience fee
    ("NPC vendor purchases", 0.08),          // Buying from NPCs
    ("Marketplace listing fees", 0.03),      // Auction house cuts
    ("Transaction taxes", 0.02),             // P2P trade tax
    ("Death penalties", 0.02),               // Lost on death
];
```

### 1.3 Tobacco (TBC) - The Store of Value

Tobacco is the **deflationary premium currency** designed to appreciate:

```rust
struct TobaccoEconomy {
    // Fixed supply - creates scarcity
    max_supply: u64,                    // 100,000,000 TBC forever
    circulating_supply: u64,            // Currently in player hands
    treasury_reserve: u64,              // Developer-held for rewards
    burned_total: u64,                  // Permanently destroyed

    // Distribution schedule (10-year vesting)
    initial_sale: u64,                  // 20M - Public sale at launch
    player_rewards: u64,                // 40M - Earned through play
    team_reserve: u64,                  // 15M - 4-year vest
    ecosystem_fund: u64,                // 15M - Partnerships, grants
    liquidity_provision: u64,           // 10M - Exchange liquidity

    // Burn mechanics
    burn_rate: f64,                     // % of TBC transactions burned
    annual_burn_target: f64,            // Target 3-5% annual deflation
}

struct TobaccoDistributionSchedule {
    year: u32,
    player_emission: u64,               // TBC available to earn
    team_unlock: u64,                   // Team vesting release
    ecosystem_grants: u64,              // Partnership allocations
}

const TOBACCO_10_YEAR_SCHEDULE: &[TobaccoDistributionSchedule] = &[
    // Year 1: Heavy player incentives to bootstrap economy
    TobaccoDistributionSchedule { year: 1, player_emission: 8_000_000, team_unlock: 0, ecosystem_grants: 3_000_000 },
    // Year 2: Reduce emission, team begins vesting
    TobaccoDistributionSchedule { year: 2, player_emission: 6_000_000, team_unlock: 3_750_000, ecosystem_grants: 2_000_000 },
    // Year 3: Stabilization phase
    TobaccoDistributionSchedule { year: 3, player_emission: 5_000_000, team_unlock: 3_750_000, ecosystem_grants: 2_000_000 },
    // Year 4: Maturation
    TobaccoDistributionSchedule { year: 4, player_emission: 4_000_000, team_unlock: 3_750_000, ecosystem_grants: 2_000_000 },
    // Year 5: Full team vesting complete
    TobaccoDistributionSchedule { year: 5, player_emission: 4_000_000, team_unlock: 3_750_000, ecosystem_grants: 1_500_000 },
    // Years 6-10: Maintenance emission only
    TobaccoDistributionSchedule { year: 6, player_emission: 3_000_000, team_unlock: 0, ecosystem_grants: 1_000_000 },
    TobaccoDistributionSchedule { year: 7, player_emission: 3_000_000, team_unlock: 0, ecosystem_grants: 1_000_000 },
    TobaccoDistributionSchedule { year: 8, player_emission: 2_500_000, team_unlock: 0, ecosystem_grants: 750_000 },
    TobaccoDistributionSchedule { year: 9, player_emission: 2_500_000, team_unlock: 0, ecosystem_grants: 500_000 },
    TobaccoDistributionSchedule { year: 10, player_emission: 2_000_000, team_unlock: 0, ecosystem_grants: 250_000 },
];
```

### 1.4 Value Accrual Mechanisms

How value flows into the Roanoke economy:

```
┌─────────────────────────────────────────────────────────────────────────┐
│                       VALUE ACCRUAL FLYWHEEL                            │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│   EXTERNAL VALUE ENTRY                                                  │
│   ═════════════════════                                                 │
│                                                                         │
│   ┌─────────────┐    ┌─────────────┐    ┌─────────────┐                │
│   │  Premium    │    │  Battle     │    │ Cosmetic    │                │
│   │  Currency   │───►│   Pass      │───►│   Sales     │                │
│   │  Purchase   │    │ Seasons     │    │   (NFT)     │                │
│   └─────────────┘    └─────────────┘    └─────────────┘                │
│          │                  │                  │                        │
│          └──────────────────┼──────────────────┘                        │
│                             ▼                                           │
│   ┌─────────────────────────────────────────────────────────────┐      │
│   │                    TREASURY INFLOW                           │      │
│   │                                                              │      │
│   │   Revenue Split:                                             │      │
│   │   • 40% - Operating costs (servers, development)             │      │
│   │   • 30% - Treasury reserve (buybacks, stability)             │      │
│   │   • 20% - Player reward pool (events, competitions)          │      │
│   │   • 10% - Ecosystem fund (partnerships, content creators)    │      │
│   └─────────────────────────────────────────────────────────────┘      │
│                             │                                           │
│                             ▼                                           │
│   INTERNAL VALUE CIRCULATION                                            │
│   ═══════════════════════════                                           │
│                                                                         │
│   ┌─────────────┐    ┌─────────────┐    ┌─────────────┐                │
│   │   Player    │    │ Marketplace │    │   Item      │                │
│   │   Labor     │───►│   Trading   │───►│  Scarcity   │                │
│   │   (Time)    │    │  (Velocity) │    │  (Value)    │                │
│   └─────────────┘    └─────────────┘    └─────────────┘                │
│          │                  │                  │                        │
│          └──────────────────┼──────────────────┘                        │
│                             ▼                                           │
│   ┌─────────────────────────────────────────────────────────────┐      │
│   │                    VALUE CRYSTALLIZATION                     │      │
│   │                                                              │      │
│   │   Rare items become permanent stores of value:               │      │
│   │   • Primordial items: Fixed supply, appreciating assets      │      │
│   │   • Limited editions: Time-locked scarcity                   │      │
│   │   • Achievement items: Skill-gated collectibles              │      │
│   │   • Historical items: Provenance adds value over time        │      │
│   └─────────────────────────────────────────────────────────────┘      │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## Part II: Institutional Investment Thesis

### 2.1 Why Games Are Underinvested

Traditional institutional investors avoid games because:

| Problem | Traditional Games | Roanoke Solution |
|---------|-------------------|------------------|
| **Value Evaporation** | Player purchases have no secondary market | Full marketplace with price discovery |
| **Opaque Economics** | No visibility into economy health | Real-time economic dashboard (public) |
| **Platform Risk** | Game shuts down, everything lost | Portable asset standard (export/import) |
| **Manipulation** | Developers print currency at will | Hard-capped premium currency with transparent emission |
| **No Exit Liquidity** | Can't sell holdings to other investors | Secondary market for account assets |
| **Regulatory Uncertainty** | Gambling/securities concerns | Skill-based, deterministic drops |

### 2.2 Roanoke Investment Instruments

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    INSTITUTIONAL INVESTMENT PRODUCTS                    │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  TIER 1: DIRECT CURRENCY HOLDINGS                                       │
│  ════════════════════════════════                                       │
│                                                                         │
│  ┌─────────────────────────────────────────────────────────────┐       │
│  │  TOBACCO (TBC) TOKEN                                         │       │
│  │                                                              │       │
│  │  • Fixed 100M supply - deflationary by design                │       │
│  │  • Tradeable on licensed exchanges (post Year 2)             │       │
│  │  • Backed by treasury reserves + game revenue               │       │
│  │  • Transparent burn schedule creates appreciation            │       │
│  │                                                              │       │
│  │  Investment Profile:                                         │       │
│  │  - Risk: Medium (game adoption dependent)                    │       │
│  │  - Return: 15-50% annual (deflationary pressure)            │       │
│  │  - Liquidity: High (exchange-listed)                        │       │
│  │  - Minimum: $10,000                                         │       │
│  └─────────────────────────────────────────────────────────────┘       │
│                                                                         │
│  TIER 2: ASSET-BACKED INSTRUMENTS                                       │
│  ════════════════════════════════                                       │
│                                                                         │
│  ┌─────────────────────────────────────────────────────────────┐       │
│  │  PRIMORDIAL ASSET FUND (PAF)                                 │       │
│  │                                                              │       │
│  │  • Portfolio of server-unique Primordial items               │       │
│  │  • Each item exists in single-digit quantities               │       │
│  │  • Professional acquisition team (top players)               │       │
│  │  • Custody via multi-sig game accounts                       │       │
│  │                                                              │       │
│  │  Investment Profile:                                         │       │
│  │  - Risk: High (item-specific, illiquid)                      │       │
│  │  - Return: 50-200% annual (scarcity appreciation)            │       │
│  │  - Liquidity: Low (quarterly redemption windows)             │       │
│  │  - Minimum: $100,000                                         │       │
│  └─────────────────────────────────────────────────────────────┘       │
│                                                                         │
│  ┌─────────────────────────────────────────────────────────────┐       │
│  │  MARKETPLACE INDEX FUND (MIF)                                │       │
│  │                                                              │       │
│  │  • Tracks top 100 most-traded item categories                │       │
│  │  • Automated rebalancing based on volume                     │       │
│  │  • Diversified across weapon/armor/material/cosmetic         │       │
│  │  • Lower volatility than single-item holdings                │       │
│  │                                                              │       │
│  │  Investment Profile:                                         │       │
│  │  - Risk: Medium                                              │       │
│  │  - Return: 10-25% annual (market growth)                     │       │
│  │  - Liquidity: Medium (weekly redemption)                     │       │
│  │  - Minimum: $25,000                                          │       │
│  └─────────────────────────────────────────────────────────────┘       │
│                                                                         │
│  TIER 3: REVENUE PARTICIPATION                                          │
│  ══════════════════════════════                                         │
│                                                                         │
│  ┌─────────────────────────────────────────────────────────────┐       │
│  │  TREASURY REVENUE NOTES (TRN)                                │       │
│  │                                                              │       │
│  │  • Fixed-income instrument backed by game revenue            │       │
│  │  • Quarterly distributions from transaction fees             │       │
│  │  • Senior claim on marketplace revenue                       │       │
│  │  • Convertible to TBC at maturity                            │       │
│  │                                                              │       │
│  │  Investment Profile:                                         │       │
│  │  - Risk: Low-Medium                                          │       │
│  │  - Return: 8-12% annual (yield)                              │       │
│  │  - Liquidity: High (secondary market)                        │       │
│  │  - Minimum: $50,000                                          │       │
│  └─────────────────────────────────────────────────────────────┘       │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### 2.3 Economic Transparency Infrastructure

Institutional investors require data. Roanoke provides:

```rust
struct EconomicDashboard {
    // Real-time metrics (public API)
    pub struct LiveMetrics {
        // Currency supply
        wampum_total_supply: u64,
        wampum_24h_emission: u64,
        wampum_24h_burned: u64,
        wampum_velocity: f64,           // Transactions per token per day

        tobacco_circulating: u64,
        tobacco_treasury: u64,
        tobacco_burned_total: u64,
        tobacco_24h_volume: u64,

        // Market metrics
        marketplace_24h_volume_wpm: u64,
        marketplace_24h_volume_tbc: u64,
        marketplace_active_listings: u32,
        marketplace_avg_listing_duration: f64,

        // Item metrics
        total_items_existing: u64,
        items_24h_created: u32,
        items_24h_destroyed: u32,
        rarity_distribution: HashMap<Rarity, u32>,

        // Player metrics (anonymized)
        daily_active_users: u32,
        monthly_active_users: u32,
        average_session_duration: f64,
        average_daily_earnings_wpm: u64,

        // Price indices
        common_item_index: f64,         // Basket of 50 common items
        rare_item_index: f64,           // Basket of 20 rare items
        legendary_item_index: f64,      // Basket of 10 legendary items
        primordial_index: f64,          // All primordial items
    }

    // Historical data (queryable)
    pub struct HistoricalData {
        hourly_snapshots: Vec<Snapshot>,    // 30 days retention
        daily_snapshots: Vec<Snapshot>,     // 1 year retention
        weekly_snapshots: Vec<Snapshot>,    // Permanent
    }

    // Audit endpoints
    pub struct AuditEndpoints {
        verify_item_provenance: fn(ItemId) -> ProvenanceChain,
        verify_transaction: fn(TxId) -> TransactionDetails,
        verify_total_supply: fn() -> SupplyBreakdown,
        verify_treasury_holdings: fn() -> TreasuryReport,
    }
}

// Public API for institutional investors
impl EconomicDashboard {
    pub fn get_live_metrics(&self) -> LiveMetrics {
        // Real-time data, updated every 5 seconds
    }

    pub fn get_historical(&self, from: DateTime, to: DateTime) -> Vec<Snapshot> {
        // Historical queries for backtesting
    }

    pub fn export_audit_report(&self, period: Period) -> AuditReport {
        // Quarterly audit-ready reports
    }

    pub fn subscribe_webhooks(&self, events: Vec<EventType>) -> WebhookSubscription {
        // Real-time event notifications
    }
}
```

### 2.4 Risk Framework

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         RISK ASSESSMENT MATRIX                          │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  MARKET RISKS                                                           │
│  ════════════                                                           │
│                                                                         │
│  Risk: Player Base Decline                                              │
│  Probability: Medium    Impact: High                                    │
│  Mitigation:                                                            │
│  • Diversified gameplay loops (hunting, archaeology, crafting)          │
│  • Regular content updates (see roadmap)                                │
│  • Cross-platform accessibility                                         │
│  • Player-owned assets create switching costs                           │
│                                                                         │
│  Risk: Competing Game Launch                                            │
│  Probability: High      Impact: Medium                                  │
│  Mitigation:                                                            │
│  • First-mover advantage in economy transparency                        │
│  • Asset portability reduces platform lock-in fear                      │
│  • Network effects from marketplace liquidity                           │
│                                                                         │
│  ECONOMIC RISKS                                                         │
│  ══════════════                                                         │
│                                                                         │
│  Risk: Hyperinflation (WPM)                                             │
│  Probability: Medium    Impact: High                                    │
│  Mitigation:                                                            │
│  • Automated sink adjustment based on supply metrics                    │
│  • Emergency burn mechanisms (treasury intervention)                    │
│  • Hard rate limits on emission sources                                 │
│  • Player stake in stable economy (their assets)                        │
│                                                                         │
│  Risk: Deflationary Spiral (TBC)                                        │
│  Probability: Low       Impact: Medium                                  │
│  Mitigation:                                                            │
│  • Treasury reserve can inject liquidity                                │
│  • Burn rate adjustable based on velocity                               │
│  • Minimum emission from player rewards                                 │
│                                                                         │
│  Risk: Market Manipulation                                              │
│  Probability: Medium    Impact: Medium                                  │
│  Mitigation:                                                            │
│  • Wash trading detection (see spec)                                    │
│  • Position limits on single accounts                                   │
│  • Transparent order books                                              │
│  • Circuit breakers on extreme price moves                              │
│                                                                         │
│  OPERATIONAL RISKS                                                      │
│  ═════════════════                                                      │
│                                                                         │
│  Risk: Exploit/Duplication Bug                                          │
│  Probability: Low       Impact: Critical                                │
│  Mitigation:                                                            │
│  • Deterministic item generation (seed-based)                           │
│  • Server-authoritative for all transactions                            │
│  • Regular security audits                                              │
│  • Insurance fund (5% of treasury)                                      │
│  • Item rollback capability within 24h                                  │
│                                                                         │
│  Risk: Server Shutdown                                                  │
│  Probability: Very Low  Impact: Critical                                │
│  Mitigation:                                                            │
│  • Legal entity structured for asset protection                         │
│  • Player asset export functionality (JSON/portable)                    │
│  • Treasury reserve covers 2 years operations                           │
│  • Open-source game client (post Year 5)                                │
│                                                                         │
│  REGULATORY RISKS                                                       │
│  ════════════════                                                       │
│                                                                         │
│  Risk: Securities Classification (TBC)                                  │
│  Probability: Medium    Impact: High                                    │
│  Mitigation:                                                            │
│  • TBC designed as utility token (in-game use primary)                  │
│  • No profit-sharing or dividend features                               │
│  • Legal opinions from gaming + securities counsel                      │
│  • Geographic restrictions where required                               │
│  • Howey test analysis documented                                       │
│                                                                         │
│  Risk: Gambling Classification                                          │
│  Probability: Low       Impact: High                                    │
│  Mitigation:                                                            │
│  • All drops are skill-influenced (not pure chance)                     │
│  • No real-money wagering on outcomes                                   │
│  • Pity systems ensure eventual rewards                                 │
│  • Drop rates publicly disclosed                                        │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## Part III: Decade Roadmap

### Phase 1: Foundation (Year 1)

**Objective:** Launch with stable economy, prove core loop, establish trust.

```
YEAR 1 MILESTONES
═══════════════════════════════════════════════════════════════════════════

Q1: CLOSED BETA ECONOMY
────────────────────────────────────────────────────────────────────────────
• Launch with 1,000 invited players
• Wampum-only economy (no premium currency yet)
• Marketplace V1: Direct player-to-player trades
• Drop system V1: Full rarity tiers, no seasonal content
• Economic monitoring: Internal dashboards only
• Target: 500 DAU, 10,000 trades/day

Q2: OPEN BETA + TOBACCO INTRODUCTION
────────────────────────────────────────────────────────────────────────────
• Open registration (uncapped)
• Tobacco (TBC) introduced - earned only (no purchase)
• Marketplace V2: Auction house with order books
• First seasonal event (limited edition drops)
• Public economic dashboard (read-only API)
• Target: 10,000 DAU, 100,000 trades/day

Q3: COMMERCIAL LAUNCH
────────────────────────────────────────────────────────────────────────────
• Full release with marketing campaign
• TBC purchasable (controlled release, $2M cap)
• Battle Pass Season 1 (premium track with TBC)
• Provenance system V1 (item history tracking)
• First primordial drops (10 server-unique items)
• Target: 50,000 DAU, 500,000 trades/day

Q4: ECONOMY STABILIZATION
────────────────────────────────────────────────────────────────────────────
• Economic rebalancing based on 6 months data
• Wampum sink adjustments
• TBC burn rate calibration
• Marketplace V3: Advanced filtering, alerts
• Institutional API (licensed access)
• Target: 100,000 DAU, 1M trades/day

YEAR 1 FINANCIAL TARGETS
════════════════════════
Revenue:           $5-10M (conservative, mostly TBC sales)
Operating Costs:   $3-5M (team, servers, marketing)
Treasury Reserve:  $2-3M (from revenue split)
TBC Distributed:   8M (to players) + 3M (ecosystem)
WPM Total Supply:  ~500M (equilibrium target)
```

### Phase 2: Growth (Years 2-3)

**Objective:** Scale player base, introduce institutional products, expand content.

```
YEAR 2 MILESTONES
═══════════════════════════════════════════════════════════════════════════

Q1: EXPANSION CONTENT
────────────────────────────────────────────────────────────────────────────
• New biome: Coastal Region (fishing, shipwrecks)
• 50 new item templates, 20 new prefixes/suffixes
• Guild system with shared treasuries
• Cross-server trading (with transfer tax)
• Target: 200,000 DAU

Q2: INSTITUTIONAL LAUNCH
────────────────────────────────────────────────────────────────────────────
• TBC listed on licensed exchange (limited geography)
• Treasury Revenue Notes (TRN) first issuance: $5M
• Institutional custody solution (multi-sig accounts)
• Quarterly economic audits begin
• Target: $50M TBC market cap

Q3: CREATOR ECONOMY
────────────────────────────────────────────────────────────────────────────
• Player-created item skins (cosmetic only)
• Creator marketplace (revenue share: 70% creator)
• Streaming integration (Twitch drops)
• Content creator fund: $500K/quarter
• Target: 1,000 active creators

Q4: COMPETITIVE SEASON
────────────────────────────────────────────────────────────────────────────
• Ranked hunting seasons with leaderboards
• Tournament system with prize pools (TBC)
• Esports partnership exploration
• Professional player guild emerges
• Target: 300,000 DAU, $100M economy volume

YEAR 2 FINANCIAL TARGETS
════════════════════════
Revenue:           $25-40M
Operating Costs:   $10-15M
Treasury Reserve:  $15-20M (cumulative)
TBC Market Cap:    $50-100M
Player Earnings:   $10-20M (extracted via TBC sales)

───────────────────────────────────────────────────────────────────────────

YEAR 3 MILESTONES
═══════════════════════════════════════════════════════════════════════════

Q1: MOBILE LAUNCH
────────────────────────────────────────────────────────────────────────────
• iOS/Android companion app
• Limited gameplay (trading, inventory, some activities)
• Push notifications for marketplace
• Cross-platform account linking
• Target: 500,000 mobile DAU

Q2: PRIMORDIAL ASSET FUND
────────────────────────────────────────────────────────────────────────────
• Launch first institutional fund (PAF)
• Professional acquisition team
• Quarterly NAV reporting
• Minimum investment: $100K
• Target: $10M AUM

Q3: REGIONAL EXPANSION
────────────────────────────────────────────────────────────────────────────
• Asia server cluster (Japan, Korea, SEA)
• Localization: Japanese, Korean, Mandarin
• Regional compliance (currency restrictions)
• Local payment methods
• Target: 200,000 Asia DAU

Q4: LAND OWNERSHIP
────────────────────────────────────────────────────────────────────────────
• Player-purchasable land plots
• Building construction (housing)
• Land generates passive WPM (resources)
• Land scarcity: 10,000 plots per server
• Target: 80% land sold within Q4

YEAR 3 FINANCIAL TARGETS
════════════════════════
Revenue:           $75-120M
Operating Costs:   $25-35M
Treasury Reserve:  $40-60M (cumulative)
TBC Market Cap:    $200-400M
Institutional AUM: $25-50M
```

### Phase 3: Maturation (Years 4-5)

**Objective:** Establish as permanent institution, diversify revenue, prepare for longevity.

```
YEAR 4 MILESTONES
═══════════════════════════════════════════════════════════════════════════

Q1: ECONOMY GOVERNANCE
────────────────────────────────────────────────────────────────────────────
• Player council for economy decisions
• Proposal system for sink/emission changes
• Transparency reports (monthly)
• Independent economic advisory board
• Target: 50% player vote participation

Q2: LENDING MARKETS
────────────────────────────────────────────────────────────────────────────
• Item lending (rent equipment)
• WPM lending pools (interest-bearing)
• Collateralized loans (items as collateral)
• Default handling (item seizure)
• Target: $10M in active loans

Q3: INSURANCE PRODUCTS
────────────────────────────────────────────────────────────────────────────
• Item insurance (against loss/destruction)
• Premium based on item value
• Claims process for legitimate losses
• Reinsurance partnerships
• Target: $5M in premiums

Q4: DERIVATIVES (LIMITED)
────────────────────────────────────────────────────────────────────────────
• Price index futures (cash-settled)
• Options on TBC (regulated exchanges only)
• Institutional only (accredited investors)
• Risk management tools for large holders
• Target: $50M daily derivative volume

YEAR 4 FINANCIAL TARGETS
════════════════════════
Revenue:           $150-200M
Operating Costs:   $50-70M
Treasury Reserve:  $100M+ (cumulative)
TBC Market Cap:    $500M-1B
Institutional AUM: $100-200M

───────────────────────────────────────────────────────────────────────────

YEAR 5 MILESTONES
═══════════════════════════════════════════════════════════════════════════

Q1: OPEN SOURCE CLIENT
────────────────────────────────────────────────────────────────────────────
• Game client source code released
• Community modifications allowed
• Server remains proprietary
• Long-term preservation guaranteed
• Trust signal for institutional investors

Q2: SECOND GAME INTEGRATION
────────────────────────────────────────────────────────────────────────────
• Partner game uses TBC as currency
• Cross-game item portability (cosmetics)
• Shared marketplace infrastructure
• Network effects amplification
• Target: Partner game adds 200K DAU to economy

Q3: REAL ESTATE FUND
────────────────────────────────────────────────────────────────────────────
• Land-focused investment vehicle
• Rental yield from land resources
• Development rights trading
• Zoning and city planning system
• Target: $50M in land AUM

Q4: FIVE-YEAR RETROSPECTIVE
────────────────────────────────────────────────────────────────────────────
• Full economic report (5-year analysis)
• Player wealth distribution study
• Lessons learned documentation
• Roadmap update for Years 6-10
• Industry conference presentation

YEAR 5 FINANCIAL TARGETS
════════════════════════
Revenue:           $200-300M
Operating Costs:   $80-100M
Treasury Reserve:  $150M+ (cumulative)
TBC Market Cap:    $1-2B
Total Economy GMV: $500M+ annual
```

### Phase 4: Institution (Years 6-10)

**Objective:** Become permanent economic infrastructure, weather downturns, expand ecosystem.

```
YEARS 6-10 OVERVIEW
═══════════════════════════════════════════════════════════════════════════

YEAR 6: ECOSYSTEM EXPANSION
────────────────────────────────────────────────────────────────────────────
• 3+ games on Roanoke economy rails
• Developer SDK for economy integration
• Third-party marketplace applications
• Dedicated institutional trading desk
• Target: 2M combined DAU across ecosystem

YEAR 7: FINANCIAL SERVICES
────────────────────────────────────────────────────────────────────────────
• Roanoke Bank: Full lending/borrowing
• Credit scoring for players
• Automated market making
• Institutional prime brokerage
• Target: $500M in financial services volume

YEAR 8: SOVEREIGNTY
────────────────────────────────────────────────────────────────────────────
• Player-elected economic council
• Treasury management transferred to DAO
• Developer becomes service provider
• Constitutional economics (hard rules)
• Target: 80% decentralized operations

YEAR 9: LEGACY PLANNING
────────────────────────────────────────────────────────────────────────────
• 100-year sustainability audit
• Endowment fund establishment
• Academic partnerships (economics research)
• Cultural preservation initiatives
• Target: $500M endowment

YEAR 10: PERMANENCE
────────────────────────────────────────────────────────────────────────────
• Roanoke as permanent digital institution
• Economy operates independently of developer
• Multi-generational wealth preservation
• IPO or perpetual trust structure
• Target: $5B+ economy, self-sustaining

DECADE FINANCIAL SUMMARY
════════════════════════

                    YEAR 1    YEAR 5    YEAR 10
                    ──────    ──────    ───────
Revenue             $10M      $250M     $500M+
Operating Costs     $5M       $90M      $150M
Treasury Reserve    $3M       $150M     $500M+
TBC Market Cap      $10M      $1.5B     $5B+
Economy GMV         $50M      $500M     $2B+
Institutional AUM   $0        $200M     $1B+
Player Earnings     $5M       $100M     $300M+
DAU                 100K      1M        3M+
```

---

## Part IV: Item Economy Deep Dive

### 4.1 Scarcity Mechanics

```rust
struct ScarcityEngine {
    // Global scarcity tracking
    primordial_registry: HashMap<ItemVariantId, PrimordialRecord>,
    limited_edition_registry: HashMap<EditionId, LimitedEditionRecord>,
    seasonal_registry: HashMap<SeasonId, SeasonalRecord>,

    // Server-wide caps
    primordial_cap_per_type: u32,       // 1-9 per item type
    legendary_daily_cap: u32,           // Max legendaries per day server-wide
    mythic_weekly_cap: u32,             // Max mythics per week server-wide

    // Destruction tracking
    items_destroyed_by_rarity: HashMap<Rarity, u64>,
    net_supply_change_24h: HashMap<Rarity, i64>,
}

impl ScarcityEngine {
    fn on_item_created(&mut self, item: &Item) {
        match item.rarity {
            Rarity::Primordial => {
                // Register as one of max 9 in existence
                let count = self.primordial_registry
                    .entry(item.variant_id)
                    .or_insert(PrimordialRecord::new())
                    .increment();

                if count >= self.primordial_cap_per_type {
                    // This variant can NEVER drop again
                    self.seal_variant(item.variant_id);
                }
            },
            Rarity::Legendary => {
                // Check daily cap
                if self.legendary_today() >= self.legendary_daily_cap {
                    // Downgrade to Epic (preserves other properties)
                    // This creates artificial scarcity pressure
                }
            },
            _ => {}
        }
    }

    fn on_item_destroyed(&mut self, item: &Item) {
        // Permanently reduce supply
        self.items_destroyed_by_rarity
            .entry(item.rarity)
            .and_modify(|c| *c += 1);

        // For primordials: increase rarity of remaining
        if item.rarity == Rarity::Primordial {
            self.primordial_registry
                .get_mut(&item.variant_id)
                .map(|r| r.decrement());

            // Broadcast to market: one less exists
            self.emit_scarcity_event(ScarcityEvent::PrimordialDestroyed {
                variant_id: item.variant_id,
                remaining: self.primordial_count(item.variant_id),
            });
        }
    }
}

// Scarcity affects value perception
struct ScarcityMultiplier {
    fn calculate(&self, item: &Item) -> f64 {
        let base = match item.rarity {
            Rarity::Primordial => 100.0,
            Rarity::Mythic => 20.0,
            Rarity::Legendary => 5.0,
            _ => 1.0,
        };

        // Fewer existing = higher multiplier
        let supply_factor = if item.rarity >= Rarity::Legendary {
            let total_existing = self.count_existing(item.variant_id);
            10.0 / (total_existing as f64).sqrt()
        } else {
            1.0
        };

        // Older items worth more (provenance)
        let age_factor = {
            let age_days = item.provenance.age_days();
            1.0 + (age_days as f64 / 365.0) * 0.1  // +10% per year
        };

        // Famous owners increase value
        let provenance_factor = {
            let notable_owners = item.provenance.notable_owner_count();
            1.0 + (notable_owners as f64 * 0.05)  // +5% per notable owner
        };

        base * supply_factor * age_factor * provenance_factor
    }
}
```

### 4.2 Provenance Value System

```rust
struct ProvenanceValue {
    // Historical ownership adds value
    fn calculate_provenance_premium(&self, item: &Item) -> f64 {
        let mut premium = 0.0;

        // First owner bonus
        if item.provenance.first_owner_is_notable() {
            premium += 0.15;  // +15% if discovered by famous player
        }

        // Discovery circumstances
        premium += match &item.provenance.discovery_method {
            DropSource::LegendaryBeastDrop => 0.20,
            DropSource::WorldFirst => 0.50,        // First of its kind
            DropSource::EventExclusive(_) => 0.10,
            DropSource::PerfectKill => 0.05,
            _ => 0.0,
        };

        // Kill quality (for hunting drops)
        if let Some(kill) = &item.provenance.kill_record {
            premium += match kill.quality {
                KillQuality::Legendary => 0.25,
                KillQuality::Perfect => 0.15,
                KillQuality::Clean => 0.05,
                _ => 0.0,
            };
        }

        // Ownership chain value
        for owner in &item.provenance.ownership_history {
            if owner.is_notable {
                premium += 0.05;  // +5% for each notable previous owner
            }
        }

        // Historical significance
        if item.provenance.was_tournament_prize {
            premium += 0.30;  // Tournament-winning items
        }
        if item.provenance.was_world_record_item {
            premium += 0.40;  // Used in world record achievement
        }

        premium
    }
}

// Notable player registry
struct NotablePlayerRegistry {
    // Automatically tracks notable players
    criteria: Vec<NotabilityCriteria>,
}

enum NotabilityCriteria {
    TopLeaderboard { category: String, top_n: u32 },
    TournamentWinner { tier: TournamentTier },
    ContentCreator { min_followers: u32 },
    EarlyAdopter { joined_before: DateTime },
    WealthyPlayer { min_net_worth: u64 },
    TradeVolume { min_volume: u64 },
}
```

### 4.3 Market Making Infrastructure

```rust
struct MarketMaker {
    // Automated liquidity provision
    inventory: HashMap<ItemTemplateId, Vec<Item>>,
    wampum_reserves: u64,
    tobacco_reserves: u64,

    // Spread configuration
    base_spread: f64,           // 2% default
    volatility_adjustment: f64,  // Widen in volatile markets
    inventory_skew: f64,        // Adjust prices based on inventory
}

impl MarketMaker {
    fn calculate_bid_ask(&self, item_template: ItemTemplateId) -> (u64, u64) {
        let mid_price = self.estimate_fair_value(item_template);

        // Inventory-adjusted spread
        let inventory_count = self.inventory.get(&item_template)
            .map(|v| v.len())
            .unwrap_or(0);

        let inventory_factor = if inventory_count > 100 {
            1.5  // Wide spread if overstocked
        } else if inventory_count < 10 {
            0.7  // Tight spread if understocked
        } else {
            1.0
        };

        // Volatility adjustment
        let volatility = self.get_volatility(item_template);
        let vol_factor = 1.0 + volatility * 2.0;  // +200% spread at max volatility

        let spread = self.base_spread * inventory_factor * vol_factor;

        let bid = (mid_price as f64 * (1.0 - spread / 2.0)) as u64;
        let ask = (mid_price as f64 * (1.0 + spread / 2.0)) as u64;

        (bid, ask)
    }

    fn provide_liquidity(&mut self, book: &mut OrderBook) {
        let (bid, ask) = self.calculate_bid_ask(book.item_template);

        // Place orders on both sides
        book.add_bid(BuyOrder {
            buyer: MARKET_MAKER_ID,
            price: bid,
            quantity: self.calculate_order_size(book.item_template),
            min_quality: 50,  // Only average+ quality
            expires_at: now() + Duration::hours(1),
        });

        book.add_ask(SellOrder {
            seller: MARKET_MAKER_ID,
            item: self.get_inventory_item(book.item_template),
            price: ask,
            expires_at: now() + Duration::hours(1),
        });
    }
}

// Price oracle for external integrations
struct PriceOracle {
    // TWAP (Time-Weighted Average Price)
    fn get_twap(&self, item_template: ItemTemplateId, window: Duration) -> u64 {
        let trades = self.get_trades_in_window(item_template, window);

        if trades.is_empty() {
            return self.get_last_trade_price(item_template);
        }

        let total_time = window.as_secs() as f64;
        let mut weighted_sum = 0.0;

        for i in 0..trades.len() {
            let trade = &trades[i];
            let duration = if i + 1 < trades.len() {
                trades[i + 1].timestamp - trade.timestamp
            } else {
                now() - trade.timestamp
            };

            weighted_sum += trade.price as f64 * duration.as_secs() as f64;
        }

        (weighted_sum / total_time) as u64
    }

    // VWAP (Volume-Weighted Average Price)
    fn get_vwap(&self, item_template: ItemTemplateId, window: Duration) -> u64 {
        let trades = self.get_trades_in_window(item_template, window);

        let (price_volume_sum, total_volume) = trades.iter()
            .fold((0u128, 0u64), |(pv, v), trade| {
                (pv + trade.price as u128 * trade.quantity as u128, v + trade.quantity)
            });

        if total_volume == 0 {
            self.get_last_trade_price(item_template)
        } else {
            (price_volume_sum / total_volume as u128) as u64
        }
    }
}
```

---

## Part V: Player Wealth & Economic Mobility

### 5.1 Wealth Distribution Goals

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    TARGET WEALTH DISTRIBUTION (YEAR 5)                  │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  PLAYER SEGMENT              % OF PLAYERS    % OF WEALTH    MOBILITY   │
│  ═══════════════════════════════════════════════════════════════════   │
│                                                                         │
│  Casual (< 5 hrs/week)         60%             15%          Medium     │
│  ────────────────────────────────────────────────────────────────────  │
│  • Can participate in economy                                          │
│  • Owns 3-10 tradeable items                                          │
│  • Net worth: 10K-100K WPM                                            │
│  • Clear path to Regular tier                                          │
│                                                                         │
│  Regular (5-20 hrs/week)       30%             35%          High       │
│  ────────────────────────────────────────────────────────────────────  │
│  • Active marketplace participant                                      │
│  • Owns 20-100 tradeable items                                        │
│  • Net worth: 100K-1M WPM                                             │
│  • Can earn real value ($50-200/month extractable)                    │
│                                                                         │
│  Dedicated (20-40 hrs/week)    8%              30%          Medium     │
│  ────────────────────────────────────────────────────────────────────  │
│  • Market-maker / trader type                                          │
│  • Owns 100-500 tradeable items                                       │
│  • Net worth: 1M-10M WPM                                              │
│  • Can earn meaningful income ($200-1000/month)                       │
│                                                                         │
│  Professional (40+ hrs/week)   2%              20%          Low        │
│  ────────────────────────────────────────────────────────────────────  │
│  • Full-time player / content creator                                  │
│  • Owns 500+ items including Legendaries                              │
│  • Net worth: 10M+ WPM                                                │
│  • Earns living wage ($2000+/month)                                   │
│                                                                         │
│  GINI COEFFICIENT TARGET: 0.45-0.55 (less than real-world: 0.7+)      │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### 5.2 Anti-Concentration Mechanisms

```rust
struct WealthDistributionEngine {
    // Prevent extreme concentration
    gini_target: f64,               // 0.50 target
    gini_current: f64,
    adjustment_active: bool,

    // Progressive taxation
    tax_brackets: Vec<(u64, f64)>,  // (threshold, rate)

    // Redistribution mechanisms
    new_player_bonus_pool: u64,
    event_prize_pool: u64,
    skill_payout_pool: u64,
}

impl WealthDistributionEngine {
    const TAX_BRACKETS: &[(u64, f64)] = &[
        (0, 0.05),              // 5% on first 100K
        (100_000, 0.05),        // 5% on 100K-1M
        (1_000_000, 0.07),      // 7% on 1M-10M
        (10_000_000, 0.10),     // 10% on 10M-100M
        (100_000_000, 0.15),    // 15% on 100M+
    ];

    fn calculate_transaction_tax(&self, seller_wealth: u64, sale_price: u64) -> u64 {
        let bracket = Self::TAX_BRACKETS.iter()
            .rev()
            .find(|(threshold, _)| seller_wealth >= *threshold)
            .map(|(_, rate)| *rate)
            .unwrap_or(0.05);

        (sale_price as f64 * bracket) as u64
    }

    fn redistribute(&mut self) {
        // New player grants (first 30 days)
        let new_player_grant = self.new_player_bonus_pool / self.new_player_count();

        // Skill-based payouts (top performers in each activity)
        self.distribute_skill_rewards();

        // Event participation (everyone who shows up gets something)
        self.distribute_event_rewards();
    }

    // Catch-up mechanics for new players
    fn calculate_new_player_bonus(&self, days_played: u32) -> f64 {
        if days_played <= 7 {
            2.0     // 2x drop rates first week
        } else if days_played <= 30 {
            1.5     // 1.5x first month
        } else if days_played <= 90 {
            1.2     // 1.2x first quarter
        } else {
            1.0
        }
    }
}

// Social mobility paths
struct MobilityPaths {
    // Multiple ways to build wealth
    paths: Vec<MobilityPath>,
}

enum MobilityPath {
    // Skill-based: Get good at the game
    SkillMastery {
        skill: Skill,
        wealth_potential: WealthRange,
        time_to_proficiency: Duration,
    },

    // Trading: Arbitrage and market-making
    Trading {
        strategy: TradingStrategy,
        capital_requirement: u64,
        risk_level: RiskLevel,
    },

    // Crafting: Transform materials into value
    Crafting {
        specialization: CraftingSpec,
        startup_cost: u64,
        margin_potential: f64,
    },

    // Social: Content creation, guilds
    Social {
        activity: SocialActivity,
        audience_requirement: u32,
        monetization_method: MonetizationMethod,
    },

    // Discovery: Finding rare items/locations
    Discovery {
        exploration_type: ExplorationType,
        luck_factor: f64,
        persistence_requirement: Duration,
    },
}
```

### 5.3 Real-Money Extraction Mechanics

```rust
struct ExtractionSystem {
    // Players can convert TBC to real money
    // This is what makes the economy "real"

    // Extraction methods
    methods: Vec<ExtractionMethod>,

    // Limits and verification
    daily_extraction_limit: u64,        // Per player
    monthly_extraction_limit: u64,
    kyc_required_threshold: u64,        // KYC required above this

    // Fees and processing
    extraction_fee: f64,                // 5%
    processing_time: Duration,          // 1-3 business days
}

enum ExtractionMethod {
    // Direct TBC sale to exchange
    ExchangeSale {
        supported_exchanges: Vec<ExchangeId>,
        min_amount: u64,
        instant: bool,
    },

    // P2P sale to other player
    P2PSale {
        escrow_service: bool,
        buyer_verification: bool,
        dispute_resolution: bool,
    },

    // Gift card conversion
    GiftCard {
        supported_retailers: Vec<RetailerId>,
        discount_rate: f64,             // Slight discount vs cash
    },

    // Creator payout (for content creators)
    CreatorPayout {
        min_followers: u32,
        payment_methods: Vec<PaymentMethod>,
        payout_schedule: PayoutSchedule,
    },
}

// Injection (players bringing money in)
struct InjectionSystem {
    // Ways to buy TBC
    methods: Vec<InjectionMethod>,

    // Anti-whale measures
    daily_purchase_limit: u64,
    whale_threshold: u64,               // Flagged for review above this

    // Bonuses for new players
    first_purchase_bonus: f64,          // +10% TBC on first purchase
}

enum InjectionMethod {
    DirectPurchase {
        payment_methods: Vec<PaymentMethod>,
        min_amount: u64,
        bonus_tiers: Vec<(u64, f64)>,   // Volume bonuses
    },

    BattlePass {
        price: u64,
        tbc_value: u64,
        duration: Duration,
    },

    CosmesticPurchase {
        item: CosmeticItem,
        price_usd: f64,
        tbc_equivalent: u64,
    },
}

// Player earnings report
struct EarningsReport {
    player_id: PlayerId,
    period: Period,

    // Sources
    hunting_income: u64,
    crafting_income: u64,
    trading_profit: u64,
    event_rewards: u64,
    other_income: u64,

    // Costs
    repair_costs: u64,
    crafting_costs: u64,
    trading_losses: u64,
    taxes_paid: u64,

    // Net
    net_wpm_earned: i64,
    net_tbc_earned: i64,
    estimated_usd_value: f64,

    // Extractable
    tbc_available_to_extract: u64,
    extraction_limit_remaining: u64,
}
```

---

## Part VI: Governance & Long-Term Sustainability

### 6.1 Economic Governance Structure

```
┌─────────────────────────────────────────────────────────────────────────┐
│                       GOVERNANCE STRUCTURE (YEAR 5+)                    │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│                        ┌─────────────────────┐                          │
│                        │  ECONOMIC COUNCIL   │                          │
│                        │  (9 Members)        │                          │
│                        └──────────┬──────────┘                          │
│                                   │                                     │
│           ┌───────────────────────┼───────────────────────┐             │
│           │                       │                       │             │
│           ▼                       ▼                       ▼             │
│   ┌───────────────┐       ┌───────────────┐       ┌───────────────┐    │
│   │ PLAYER SEATS  │       │ DEVELOPER     │       │ INVESTOR      │    │
│   │ (5 members)   │       │ SEATS (2)     │       │ SEATS (2)     │    │
│   └───────────────┘       └───────────────┘       └───────────────┘    │
│                                                                         │
│   PLAYER SEATS                                                          │
│   ════════════                                                          │
│   • 3 elected by popular vote (all players)                            │
│   • 1 elected by top 1000 traders (economic expertise)                 │
│   • 1 elected by content creators (community voice)                    │
│   • Term: 6 months, max 3 consecutive terms                            │
│                                                                         │
│   DEVELOPER SEATS                                                       │
│   ═══════════════                                                       │
│   • Appointed by development team                                       │
│   • Veto power on game-breaking changes only                           │
│   • Technical expertise requirement                                     │
│                                                                         │
│   INVESTOR SEATS                                                        │
│   ══════════════                                                        │
│   • Elected by TRN holders (proportional to holdings)                  │
│   • Represent institutional interests                                   │
│   • Fiduciary duty to all stakeholders                                 │
│                                                                         │
│   COUNCIL POWERS                                                        │
│   ══════════════                                                        │
│   ✓ Adjust emission rates (within bounds)                              │
│   ✓ Modify sink parameters                                             │
│   ✓ Approve new seasonal content                                       │
│   ✓ Set marketplace fee structures                                     │
│   ✓ Manage treasury investments                                        │
│   ✓ Emergency interventions (7/9 supermajority)                        │
│                                                                         │
│   CONSTITUTIONAL LIMITS (Cannot Change)                                 │
│   ═════════════════════════════════════                                 │
│   ✗ TBC max supply (100M forever)                                      │
│   ✗ Primordial uniqueness guarantees                                   │
│   ✗ Item provenance immutability                                       │
│   ✗ Player ownership rights                                            │
│   ✗ Extraction/withdrawal rights                                       │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### 6.2 Treasury Management

```rust
struct Treasury {
    // Asset allocation
    allocations: TreasuryAllocations,

    // Investment policy
    policy: InvestmentPolicy,

    // Oversight
    auditor: ExternalAuditor,
    reporting: ReportingSchedule,
}

struct TreasuryAllocations {
    // Liquid reserves (immediate access)
    operating_cash: u64,            // 6 months runway
    emergency_fund: u64,            // 3 months expenses

    // Semi-liquid (1-week access)
    stability_reserve: u64,         // Market intervention fund
    insurance_fund: u64,            // Player loss coverage

    // Invested (long-term)
    growth_fund: u64,               // Ecosystem investments
    endowment: u64,                 // Perpetual sustainability
}

struct InvestmentPolicy {
    // Risk limits
    max_single_investment: f64,     // 5% of treasury
    max_sector_exposure: f64,       // 20% in any category
    liquidity_requirement: f64,     // 30% must be liquid

    // Allowed investments
    allowed_categories: Vec<InvestmentCategory>,
}

enum InvestmentCategory {
    // Low risk
    GovernmentBonds,
    MoneyMarket,

    // Medium risk
    GameIndustryEquity,
    PartnerGameRevShare,
    InfrastructureProjects,

    // Growth (limited allocation)
    EcosystemStartups,
    ContentCreatorFunding,
    TechnologyR_D,
}

// Transparency reporting
struct TreasuryReport {
    period: Period,

    // Holdings
    asset_breakdown: HashMap<AssetType, u64>,
    total_value_usd: f64,

    // Performance
    return_ytd: f64,
    return_since_inception: f64,

    // Flows
    inflows: Vec<FlowRecord>,
    outflows: Vec<FlowRecord>,

    // Audit
    auditor_opinion: AuditorOpinion,
    audit_date: DateTime,
}
```

### 6.3 Sustainability Mechanisms

```rust
struct SustainabilityEngine {
    // Revenue diversification
    revenue_streams: Vec<RevenueStream>,

    // Cost optimization
    cost_structure: CostStructure,

    // Endowment model
    endowment: EndowmentFund,
}

struct RevenueStream {
    source: RevenueSource,
    annual_revenue: u64,
    growth_rate: f64,
    reliability: Reliability,
}

enum RevenueSource {
    // Core game revenue
    TobaccoPurchases,           // Premium currency
    BattlePass,                 // Seasonal passes
    Cosmetics,                  // Skins, effects

    // Marketplace revenue
    TransactionFees,            // % of all trades
    ListingFees,                // Auction costs
    PremiumFeatures,            // Advanced trading tools

    // Institutional revenue
    APILicenses,                // Data access fees
    CustodyServices,            // Institutional custody
    FundManagement,             // AUM fees

    // Ecosystem revenue
    PartnerRevShare,            // Other games using TBC
    SDKLicenses,                // Developer tools
    WhiteLabelSolutions,        // Custom economy deploys

    // Passive income
    TreasuryYield,              // Investment returns
    LendingSpread,              // Loan interest
    MarketMaking,               // Trading desk profits
}

// Endowment for perpetual sustainability
struct EndowmentFund {
    principal: u64,             // Never touch
    target_size: u64,           // 10x annual operating costs

    // Spending policy
    annual_withdrawal_rate: f64,    // 4% rule
    inflation_adjustment: bool,

    // Growth policy
    contribution_rate: f64,         // % of revenue to endowment
    investment_horizon: Duration,   // Infinite
}

impl EndowmentFund {
    fn calculate_sustainable_spending(&self) -> u64 {
        // Can sustainably spend 4% annually forever
        (self.principal as f64 * 0.04) as u64
    }

    fn years_of_runway(&self, annual_costs: u64) -> f64 {
        // How long can we operate on endowment alone?
        self.principal as f64 / annual_costs as f64 / 0.04
    }
}
```

---

## Part VII: Integration with Existing Loot System

### 7.1 Currency Rewards from Drop System

Linking the drop mechanics (from MARKETPLACE_LOOT_SYSTEM_SPEC.md) to currency:

```rust
struct DropRewardIntegration {
    // Every drop has currency value
    fn calculate_drop_currency(&self, item: &Item, context: &DropContext) -> CurrencyReward {
        let base_wpm = match item.rarity {
            Rarity::Crude => 5,
            Rarity::Common => 20,
            Rarity::Uncommon => 100,
            Rarity::Rare => 500,
            Rarity::Epic => 2500,
            Rarity::Legendary => 15000,
            Rarity::Mythic => 100000,
            Rarity::Primordial => 1000000,
        };

        // Quality multiplier
        let quality_mult = 0.5 + (item.quality as f64 / 100.0);  // 0.5x - 1.5x

        // Provenance bonus (first kills, etc.)
        let provenance_mult = if context.is_first_kill { 2.0 }
            else if context.is_clean_kill { 1.25 }
            else { 1.0 };

        // Session pity bonus (reward long sessions)
        let session_mult = 1.0 + (context.session_minutes as f64 * 0.001);  // Up to 1.6x

        let wpm_reward = (base_wpm as f64 * quality_mult * provenance_mult * session_mult) as u64;

        // Rare chance for TBC bonus on high-rarity drops
        let tbc_reward = if item.rarity >= Rarity::Legendary {
            let tbc_chance = match item.rarity {
                Rarity::Legendary => 0.10,
                Rarity::Mythic => 0.25,
                Rarity::Primordial => 1.00,
                _ => 0.0,
            };

            if rand() < tbc_chance {
                match item.rarity {
                    Rarity::Legendary => 10,
                    Rarity::Mythic => 100,
                    Rarity::Primordial => 1000,
                    _ => 0,
                }
            } else {
                0
            }
        } else {
            0
        };

        CurrencyReward {
            wampum: wpm_reward,
            tobacco: tbc_reward,
        }
    }
}

// Item value estimation for marketplace
struct ValueEstimator {
    fn estimate_value(&self, item: &Item) -> ValueEstimate {
        // Base value from rarity
        let base = RARITY_BASE_VALUES[item.rarity as usize];

        // Quality adjustment
        let quality_adj = self.quality_curve(item.quality);

        // Prefix value
        let prefix_value = item.prefix
            .map(|p| PREFIT_VALUES[p as usize])
            .unwrap_or(0);

        // Suffix value
        let suffix_value = item.suffix
            .map(|s| SUFFIX_VALUES[s as usize])
            .unwrap_or(0);

        // Variant multiplier
        let variant_mult = match item.variant_class {
            VariantClass::Masterwork => 2.5,
            VariantClass::Ancient => 2.0,
            VariantClass::Blessed => 1.8,
            VariantClass::Perfected => 3.0,
            VariantClass::Singular => 10.0,
            _ => 1.0,
        };

        // Provenance premium
        let provenance_mult = self.calculate_provenance_premium(item);

        // Market data adjustment
        let market_adj = self.get_market_adjustment(item.template_id);

        let estimated_value = (base + prefix_value + suffix_value) as f64
            * quality_adj
            * variant_mult
            * provenance_mult
            * market_adj;

        ValueEstimate {
            low: (estimated_value * 0.7) as u64,
            mid: estimated_value as u64,
            high: (estimated_value * 1.4) as u64,
            confidence: self.calculate_confidence(item),
        }
    }
}
```

### 7.2 Seasonal Events Currency Integration

```rust
struct SeasonalCurrencyRewards {
    // Each season has currency rewards
    season: Season,

    // Bonus earning rates
    hunting_bonus: f64,         // +X% WPM from hunting
    archaeology_bonus: f64,     // +X% WPM from fossils
    trading_bonus: f64,         // Reduced fees

    // Season-exclusive currency sinks
    seasonal_vendor: SeasonalVendor,

    // TBC distribution for season
    tbc_pool: u64,              // Total TBC available
    distribution_method: DistributionMethod,
}

enum DistributionMethod {
    // Competitive: top performers get most
    Leaderboard {
        top_percent: f64,
        tiers: Vec<(u32, u64)>,  // (rank, tbc_reward)
    },

    // Participatory: everyone gets some
    Participation {
        threshold_hours: f64,   // Minimum playtime
        base_reward: u64,
        activity_multiplier: f64,
    },

    // Achievement: specific goals
    Milestones {
        milestones: Vec<(Milestone, u64)>,
    },

    // Lottery: random distribution to participants
    RandomDraw {
        entries_per_hour: f64,
        prizes: Vec<u64>,
    },
}

struct SeasonalVendor {
    // Exclusive items purchasable with seasonal currency
    items: Vec<SeasonalItem>,
    currency: SeasonalCurrency,

    // Scarcity controls
    item_limits: HashMap<ItemId, u32>,
    player_limits: HashMap<ItemId, u32>,
}

struct SeasonalItem {
    item: Item,
    price: u64,                 // In seasonal currency
    available_until: DateTime,
    quantity_remaining: u32,
}
```

---

## Part VIII: Success Metrics & KPIs

### 8.1 Economic Health Indicators

```
┌─────────────────────────────────────────────────────────────────────────┐
│                     KEY PERFORMANCE INDICATORS                          │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  CURRENCY HEALTH                                                        │
│  ═══════════════                                                        │
│                                                                         │
│  Metric                    Target          Warning         Critical     │
│  ────────────────────────────────────────────────────────────────────  │
│  WPM Daily Inflation       0.05%           0.10%           0.20%        │
│  WPM Velocity              2.0/day         1.0/day         0.5/day      │
│  TBC Burn Rate             3-5%/year       1%/year         0%/year      │
│  TBC/WPM Exchange Rate     Stable ±5%      ±15%            ±30%         │
│                                                                         │
│  MARKET HEALTH                                                          │
│  ═════════════                                                          │
│                                                                         │
│  Metric                    Target          Warning         Critical     │
│  ────────────────────────────────────────────────────────────────────  │
│  Daily Trade Volume        $1M+            $500K           $100K        │
│  Average Spread            <5%             10%             20%          │
│  Listing Duration          <24h            48h             72h          │
│  Wash Trading Rate         <1%             3%              5%           │
│                                                                         │
│  PLAYER ECONOMY                                                         │
│  ══════════════                                                         │
│                                                                         │
│  Metric                    Target          Warning         Critical     │
│  ────────────────────────────────────────────────────────────────────  │
│  Median Daily Earnings     500 WPM         200 WPM         50 WPM       │
│  Player Gini Coefficient   0.50            0.60            0.70         │
│  % Players with Trades     50%             30%             10%          │
│  New Player 30-Day Wealth  10K WPM         5K WPM          1K WPM       │
│                                                                         │
│  INSTITUTIONAL METRICS                                                  │
│  ═════════════════════                                                  │
│                                                                         │
│  Metric                    Target          Warning         Critical     │
│  ────────────────────────────────────────────────────────────────────  │
│  API Uptime                99.9%           99%             95%          │
│  Price Feed Latency        <1s             5s              30s          │
│  Audit Clean               100%            1 finding       3+ findings  │
│  Institutional AUM Growth  20%/year        10%/year        0%/year      │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### 8.2 Milestone Triggers

```rust
struct MilestoneTriggers {
    // Automatic actions based on metrics
    triggers: Vec<MilestoneTrigger>,
}

struct MilestoneTrigger {
    metric: Metric,
    threshold: ThresholdCondition,
    action: MilestoneAction,
    cooldown: Duration,
}

enum MilestoneAction {
    // Economy adjustments
    AdjustEmissionRate { change: f64 },
    AdjustSinkRate { change: f64 },
    ActivateCircuitBreaker,
    TreasuryIntervention { amount: u64 },

    // Governance
    AlertCouncil { severity: Severity },
    TriggerVote { proposal: Proposal },
    EmergencyPause { systems: Vec<System> },

    // Communication
    PublicAnnouncement { message: String },
    InstitutionalNotification { message: String },

    // Celebration
    AchievementUnlock { achievement: Achievement },
    BonusEvent { event: Event },
}

// Example triggers
const MILESTONE_TRIGGERS: &[MilestoneTrigger] = &[
    // Hit 1M DAU
    MilestoneTrigger {
        metric: Metric::DailyActiveUsers,
        threshold: ThresholdCondition::Above(1_000_000),
        action: MilestoneAction::BonusEvent {
            event: Event::MillionPlayerCelebration,
        },
        cooldown: Duration::days(365),  // Once per year
    },

    // Economy overheating
    MilestoneTrigger {
        metric: Metric::WpmInflation24h,
        threshold: ThresholdCondition::Above(0.002),  // 0.2%
        action: MilestoneAction::AdjustSinkRate { change: 0.10 },  // +10% sinks
        cooldown: Duration::hours(24),
    },

    // Market crash
    MilestoneTrigger {
        metric: Metric::LegendaryPriceIndex24hChange,
        threshold: ThresholdCondition::Below(-0.30),  // -30% crash
        action: MilestoneAction::ActivateCircuitBreaker,
        cooldown: Duration::hours(1),
    },

    // Gini too high
    MilestoneTrigger {
        metric: Metric::PlayerGiniCoefficient,
        threshold: ThresholdCondition::Above(0.65),
        action: MilestoneAction::AlertCouncil {
            severity: Severity::High,
        },
        cooldown: Duration::days(7),
    },
];
```

---

## Conclusion: The Investment Thesis

Roanoke offers institutional investors a unique opportunity:

**1. Asset Class Creation**
- First game economy with institutional-grade infrastructure
- Transparent, auditable, regulated (where applicable)
- Portable assets with real secondary markets

**2. Defensible Moat**
- Network effects: More players = more liquidity = more valuable
- Switching costs: Players own valuable, portable assets
- Data advantage: Real-time economic intelligence

**3. Multiple Revenue Streams**
- Direct: Currency sales, battle passes, cosmetics
- Marketplace: Transaction fees, listing fees, premium tools
- Institutional: API licenses, custody, fund management
- Ecosystem: Partner revenue share, SDK licensing

**4. Aligned Incentives**
- Players: Own their progress, can extract real value
- Developers: Revenue tied to player engagement and satisfaction
- Investors: Returns tied to economy growth, not exploitation

**5. Long-Term Sustainability**
- Endowment model ensures perpetual operation
- Player governance creates stakeholder alignment
- Open-source commitment protects player investments

**The ask:** $50M Series A to fund Years 1-3, achieve 500K DAU, and prove the institutional investment thesis. Targeting $500M economy GMV by Year 5.

---

*Document Version: 1.0*
*Last Updated: December 2024*
*Classification: Investor Confidential*
*Author: Roanoke Economic Architecture Team*
