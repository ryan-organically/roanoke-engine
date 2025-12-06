-- ============================================================================
-- ROANOKE MARKETING DATA PLATFORM - PostgreSQL Schema
-- ============================================================================
-- Version: 1.0.0
-- Last Updated: 2025-12-05
-- Description: Core operational database schema for marketing analytics
-- ============================================================================

-- Enable required extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";
CREATE EXTENSION IF NOT EXISTS "pg_trgm";  -- For text search

-- Create schema
CREATE SCHEMA IF NOT EXISTS marketing;
SET search_path TO marketing, public;

-- ============================================================================
-- CORE DIMENSION TABLES
-- ============================================================================

-- Campaigns
CREATE TABLE campaigns (
    campaign_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    external_id VARCHAR(255) UNIQUE,
    campaign_name VARCHAR(500) NOT NULL,
    campaign_type VARCHAR(50) NOT NULL,
    status VARCHAR(20) DEFAULT 'DRAFT',
    start_date TIMESTAMPTZ,
    end_date TIMESTAMPTZ,
    budget_total DECIMAL(12, 2),
    budget_daily DECIMAL(10, 2),
    budget_currency CHAR(3) DEFAULT 'USD',
    target_audience JSONB,
    goals JSONB,
    tags VARCHAR(100)[],
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    created_by UUID,

    CONSTRAINT valid_status CHECK (status IN ('DRAFT', 'SCHEDULED', 'ACTIVE', 'PAUSED', 'COMPLETED', 'CANCELLED')),
    CONSTRAINT valid_dates CHECK (end_date IS NULL OR end_date > start_date)
);

CREATE INDEX idx_campaigns_status ON campaigns(status);
CREATE INDEX idx_campaigns_dates ON campaigns(start_date, end_date);
CREATE INDEX idx_campaigns_type ON campaigns(campaign_type);
CREATE INDEX idx_campaigns_tags ON campaigns USING GIN(tags);

-- Platforms
CREATE TABLE platforms (
    platform_id SERIAL PRIMARY KEY,
    platform_code VARCHAR(50) UNIQUE NOT NULL,
    platform_name VARCHAR(100) NOT NULL,
    platform_type VARCHAR(50) NOT NULL,  -- SOCIAL, AD, ANALYTICS
    api_enabled BOOLEAN DEFAULT true,
    rate_limit_per_minute INT,
    credentials_vault_key VARCHAR(255),
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Insert default platforms
INSERT INTO platforms (platform_code, platform_name, platform_type, rate_limit_per_minute) VALUES
('TWITTER', 'Twitter/X', 'SOCIAL', 300),
('TIKTOK', 'TikTok', 'SOCIAL', 100),
('DISCORD', 'Discord', 'SOCIAL', 50),
('REDDIT', 'Reddit', 'SOCIAL', 60),
('YOUTUBE', 'YouTube', 'SOCIAL', 100),
('TWITCH', 'Twitch', 'SOCIAL', 100),
('INSTAGRAM', 'Instagram', 'SOCIAL', 200),
('GOOGLE_ADS', 'Google Ads', 'AD', 100),
('META_ADS', 'Meta Ads', 'AD', 100),
('TIKTOK_ADS', 'TikTok Ads', 'AD', 50),
('STEAM', 'Steam', 'PLATFORM', 10),
('GOOGLE_ANALYTICS', 'Google Analytics', 'ANALYTICS', 100),
('MIXPANEL', 'Mixpanel', 'ANALYTICS', 100);

-- Creators
CREATE TABLE creators (
    creator_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    platform_user_ids JSONB NOT NULL,  -- {"TWITTER": "123", "YOUTUBE": "abc"}
    display_name VARCHAR(255),
    email VARCHAR(255),
    tier VARCHAR(20) DEFAULT 'NANO',
    follower_count_total BIGINT DEFAULT 0,
    verified_at TIMESTAMPTZ,
    partnership_status VARCHAR(20) DEFAULT 'PROSPECT',
    revenue_share_rate DECIMAL(5, 4) DEFAULT 0.88,
    notes TEXT,
    tags VARCHAR(100)[],
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),

    CONSTRAINT valid_tier CHECK (tier IN ('NANO', 'MICRO', 'MID', 'MACRO', 'MEGA')),
    CONSTRAINT valid_partnership CHECK (partnership_status IN ('PROSPECT', 'CONTACTED', 'NEGOTIATING', 'ACTIVE', 'PAUSED', 'ENDED'))
);

CREATE INDEX idx_creators_tier ON creators(tier);
CREATE INDEX idx_creators_partnership ON creators(partnership_status);
CREATE INDEX idx_creators_platform_ids ON creators USING GIN(platform_user_ids);

-- ============================================================================
-- METRICS TABLES
-- ============================================================================

-- Daily Summary Metrics
CREATE TABLE daily_summary (
    id BIGSERIAL PRIMARY KEY,
    date DATE NOT NULL,
    social_mentions BIGINT DEFAULT 0,
    social_engagement BIGINT DEFAULT 0,
    sentiment_score DECIMAL(4, 3),
    ad_impressions BIGINT DEFAULT 0,
    ad_clicks BIGINT DEFAULT 0,
    ad_conversions BIGINT DEFAULT 0,
    ad_spend DECIMAL(12, 2) DEFAULT 0,
    ctr DECIMAL(6, 4),
    cvr DECIMAL(6, 4),
    cpa DECIMAL(10, 2),
    web_sessions BIGINT DEFAULT 0,
    web_users BIGINT DEFAULT 0,
    web_conversions BIGINT DEFAULT 0,
    game_dau BIGINT DEFAULT 0,
    game_revenue DECIMAL(12, 2) DEFAULT 0,
    processing_timestamp TIMESTAMPTZ DEFAULT NOW(),

    CONSTRAINT unique_date UNIQUE (date)
);

CREATE INDEX idx_daily_summary_date ON daily_summary(date DESC);

-- Social Metrics by Platform
CREATE TABLE social_metrics_daily (
    id BIGSERIAL PRIMARY KEY,
    date DATE NOT NULL,
    platform VARCHAR(50) NOT NULL,
    total_events BIGINT DEFAULT 0,
    unique_authors BIGINT DEFAULT 0,
    total_likes BIGINT DEFAULT 0,
    total_shares BIGINT DEFAULT 0,
    total_comments BIGINT DEFAULT 0,
    total_views BIGINT DEFAULT 0,
    avg_sentiment DECIMAL(4, 3),
    sentiment_stddev DECIMAL(4, 3),
    positive_count BIGINT DEFAULT 0,
    negative_count BIGINT DEFAULT 0,
    neutral_count BIGINT DEFAULT 0,
    meme_count BIGINT DEFAULT 0,
    influencer_mentions BIGINT DEFAULT 0,
    creator_mentions BIGINT DEFAULT 0,
    requires_response_count BIGINT DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT NOW(),

    CONSTRAINT unique_date_platform UNIQUE (date, platform)
);

CREATE INDEX idx_social_metrics_date ON social_metrics_daily(date DESC);
CREATE INDEX idx_social_metrics_platform ON social_metrics_daily(platform);

-- Campaign Metrics Daily
CREATE TABLE campaign_metrics_daily (
    id BIGSERIAL PRIMARY KEY,
    date DATE NOT NULL,
    campaign_id UUID REFERENCES campaigns(campaign_id),
    platform VARCHAR(50) NOT NULL,
    impressions BIGINT DEFAULT 0,
    reach BIGINT DEFAULT 0,
    clicks BIGINT DEFAULT 0,
    conversions BIGINT DEFAULT 0,
    video_views BIGINT DEFAULT 0,
    video_completions BIGINT DEFAULT 0,
    installs BIGINT DEFAULT 0,
    wishlist_adds BIGINT DEFAULT 0,
    purchases BIGINT DEFAULT 0,
    spend DECIMAL(10, 4) DEFAULT 0,
    currency CHAR(3) DEFAULT 'USD',
    ctr DECIMAL(8, 6),
    cvr DECIMAL(8, 6),
    cpc DECIMAL(8, 4),
    cpm DECIMAL(8, 4),
    cpa DECIMAL(10, 4),
    roas DECIMAL(8, 4),
    created_at TIMESTAMPTZ DEFAULT NOW(),

    CONSTRAINT unique_campaign_date_platform UNIQUE (date, campaign_id, platform)
);

CREATE INDEX idx_campaign_metrics_date ON campaign_metrics_daily(date DESC);
CREATE INDEX idx_campaign_metrics_campaign ON campaign_metrics_daily(campaign_id);

-- Funnel Metrics Daily
CREATE TABLE daily_funnel (
    id BIGSERIAL PRIMARY KEY,
    date DATE NOT NULL,
    stage VARCHAR(50) NOT NULL,
    stage_order INT NOT NULL,
    users BIGINT DEFAULT 0,
    conversion_rate DECIMAL(6, 3),
    dropoff_rate DECIMAL(6, 3),
    created_at TIMESTAMPTZ DEFAULT NOW(),

    CONSTRAINT unique_funnel_stage UNIQUE (date, stage)
);

CREATE INDEX idx_funnel_date ON daily_funnel(date DESC);

-- ============================================================================
-- REAL-TIME TABLES (for streaming data)
-- ============================================================================

-- Sentiment Alerts
CREATE TABLE sentiment_alerts (
    alert_id VARCHAR(32) PRIMARY KEY,
    alert_type VARCHAR(50) NOT NULL,
    severity VARCHAR(20) NOT NULL,
    platform VARCHAR(50),
    timestamp TIMESTAMPTZ NOT NULL,
    message TEXT NOT NULL,
    current_value DECIMAL(10, 4),
    threshold DECIMAL(10, 4),
    context JSONB,
    acknowledged BOOLEAN DEFAULT false,
    acknowledged_by UUID,
    acknowledged_at TIMESTAMPTZ,
    resolved BOOLEAN DEFAULT false,
    resolved_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),

    CONSTRAINT valid_severity CHECK (severity IN ('INFO', 'LOW', 'MEDIUM', 'HIGH', 'CRITICAL'))
);

CREATE INDEX idx_alerts_timestamp ON sentiment_alerts(timestamp DESC);
CREATE INDEX idx_alerts_severity ON sentiment_alerts(severity);
CREATE INDEX idx_alerts_unresolved ON sentiment_alerts(resolved) WHERE resolved = false;

-- Response Queue (posts requiring response)
CREATE TABLE response_queue (
    id BIGSERIAL PRIMARY KEY,
    event_id VARCHAR(255) NOT NULL,
    platform VARCHAR(50) NOT NULL,
    platform_event_id VARCHAR(255),
    author_id VARCHAR(255),
    author_username VARCHAR(255),
    author_followers BIGINT,
    content TEXT,
    sentiment_score DECIMAL(4, 3),
    priority VARCHAR(20) DEFAULT 'LOW',
    topics VARCHAR(100)[],
    event_timestamp TIMESTAMPTZ NOT NULL,
    ingested_at TIMESTAMPTZ DEFAULT NOW(),
    status VARCHAR(20) DEFAULT 'PENDING',
    assigned_to UUID,
    assigned_at TIMESTAMPTZ,
    responded BOOLEAN DEFAULT false,
    responded_at TIMESTAMPTZ,
    response_text TEXT,

    CONSTRAINT valid_priority CHECK (priority IN ('LOW', 'MEDIUM', 'HIGH', 'URGENT')),
    CONSTRAINT valid_status CHECK (status IN ('PENDING', 'ASSIGNED', 'IN_PROGRESS', 'RESPONDED', 'SKIPPED'))
);

CREATE INDEX idx_response_queue_status ON response_queue(status);
CREATE INDEX idx_response_queue_priority ON response_queue(priority);
CREATE INDEX idx_response_queue_platform ON response_queue(platform);

-- ============================================================================
-- AUDIT & COMPLIANCE
-- ============================================================================

-- Audit Log
CREATE TABLE audit_log (
    id BIGSERIAL PRIMARY KEY,
    timestamp TIMESTAMPTZ DEFAULT NOW(),
    user_id UUID,
    action VARCHAR(50) NOT NULL,
    resource_type VARCHAR(50) NOT NULL,
    resource_id VARCHAR(255),
    old_values JSONB,
    new_values JSONB,
    ip_address INET,
    user_agent TEXT,
    success BOOLEAN DEFAULT true,
    error_message TEXT
);

CREATE INDEX idx_audit_timestamp ON audit_log(timestamp DESC);
CREATE INDEX idx_audit_user ON audit_log(user_id);
CREATE INDEX idx_audit_resource ON audit_log(resource_type, resource_id);

-- Data Retention Log
CREATE TABLE data_retention_log (
    id BIGSERIAL PRIMARY KEY,
    timestamp TIMESTAMPTZ DEFAULT NOW(),
    table_name VARCHAR(100) NOT NULL,
    records_deleted BIGINT,
    retention_days INT,
    oldest_record_date DATE,
    success BOOLEAN DEFAULT true,
    error_message TEXT
);

-- ============================================================================
-- VIEWS
-- ============================================================================

-- Combined Daily Overview
CREATE OR REPLACE VIEW daily_overview AS
SELECT
    ds.date,
    ds.social_mentions,
    ds.social_engagement,
    ds.sentiment_score,
    ds.ad_impressions,
    ds.ad_clicks,
    ds.ad_conversions,
    ds.ad_spend,
    ds.ctr,
    ds.cvr,
    ds.cpa,
    ds.web_sessions,
    ds.game_dau,
    ds.game_revenue,
    COALESCE(SUM(cmd.impressions), 0) AS total_campaign_impressions,
    COUNT(DISTINCT cmd.campaign_id) AS active_campaigns,
    LAG(ds.sentiment_score) OVER (ORDER BY ds.date) AS prev_day_sentiment,
    ds.sentiment_score - LAG(ds.sentiment_score) OVER (ORDER BY ds.date) AS sentiment_change
FROM daily_summary ds
LEFT JOIN campaign_metrics_daily cmd ON ds.date = cmd.date
GROUP BY ds.date, ds.social_mentions, ds.social_engagement, ds.sentiment_score,
         ds.ad_impressions, ds.ad_clicks, ds.ad_conversions, ds.ad_spend,
         ds.ctr, ds.cvr, ds.cpa, ds.web_sessions, ds.game_dau, ds.game_revenue;

-- Platform Performance Summary
CREATE OR REPLACE VIEW platform_performance_7d AS
SELECT
    platform,
    SUM(total_events) AS total_events,
    SUM(unique_authors) AS unique_authors,
    SUM(total_likes + total_shares + total_comments) AS total_engagement,
    AVG(avg_sentiment) AS avg_sentiment,
    SUM(meme_count) AS meme_count,
    SUM(influencer_mentions) AS influencer_mentions
FROM social_metrics_daily
WHERE date >= CURRENT_DATE - INTERVAL '7 days'
GROUP BY platform
ORDER BY total_engagement DESC;

-- Active Campaigns Summary
CREATE OR REPLACE VIEW active_campaigns_summary AS
SELECT
    c.campaign_id,
    c.campaign_name,
    c.campaign_type,
    c.status,
    c.start_date,
    c.budget_total,
    c.budget_currency,
    SUM(cmd.impressions) AS total_impressions,
    SUM(cmd.clicks) AS total_clicks,
    SUM(cmd.conversions) AS total_conversions,
    SUM(cmd.spend) AS total_spend,
    CASE WHEN SUM(cmd.impressions) > 0
         THEN SUM(cmd.clicks)::DECIMAL / SUM(cmd.impressions) * 100
         ELSE 0 END AS overall_ctr,
    CASE WHEN SUM(cmd.clicks) > 0
         THEN SUM(cmd.conversions)::DECIMAL / SUM(cmd.clicks) * 100
         ELSE 0 END AS overall_cvr,
    c.budget_total - SUM(cmd.spend) AS remaining_budget
FROM campaigns c
LEFT JOIN campaign_metrics_daily cmd ON c.campaign_id = cmd.campaign_id
WHERE c.status = 'ACTIVE'
GROUP BY c.campaign_id, c.campaign_name, c.campaign_type, c.status,
         c.start_date, c.budget_total, c.budget_currency;

-- ============================================================================
-- FUNCTIONS
-- ============================================================================

-- Function to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Apply to tables
CREATE TRIGGER update_campaigns_updated_at
    BEFORE UPDATE ON campaigns
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_creators_updated_at
    BEFORE UPDATE ON creators
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Function for data retention cleanup
CREATE OR REPLACE FUNCTION cleanup_old_data(retention_days INT DEFAULT 90)
RETURNS TABLE(table_name TEXT, deleted_count BIGINT) AS $$
DECLARE
    cutoff_date DATE := CURRENT_DATE - retention_days;
    deleted BIGINT;
BEGIN
    -- Clean social metrics (keep 2 years)
    DELETE FROM social_metrics_daily WHERE date < CURRENT_DATE - 730;
    GET DIAGNOSTICS deleted = ROW_COUNT;
    INSERT INTO data_retention_log (table_name, records_deleted, retention_days)
    VALUES ('social_metrics_daily', deleted, 730);

    -- Clean response queue (keep 90 days)
    DELETE FROM response_queue WHERE ingested_at < cutoff_date AND status IN ('RESPONDED', 'SKIPPED');
    GET DIAGNOSTICS deleted = ROW_COUNT;
    INSERT INTO data_retention_log (table_name, records_deleted, retention_days)
    VALUES ('response_queue', deleted, retention_days);

    -- Clean resolved alerts (keep 180 days)
    DELETE FROM sentiment_alerts WHERE resolved = true AND resolved_at < CURRENT_DATE - 180;
    GET DIAGNOSTICS deleted = ROW_COUNT;
    INSERT INTO data_retention_log (table_name, records_deleted, retention_days)
    VALUES ('sentiment_alerts', deleted, 180);

    -- Return summary
    RETURN QUERY SELECT * FROM data_retention_log
                 WHERE timestamp > NOW() - INTERVAL '1 minute';
END;
$$ LANGUAGE plpgsql;

-- ============================================================================
-- ROW LEVEL SECURITY
-- ============================================================================

-- Enable RLS on sensitive tables
ALTER TABLE campaigns ENABLE ROW LEVEL SECURITY;
ALTER TABLE creators ENABLE ROW LEVEL SECURITY;
ALTER TABLE audit_log ENABLE ROW LEVEL SECURITY;

-- Policies (example - would be customized per role)
CREATE POLICY campaigns_read_policy ON campaigns
    FOR SELECT USING (true);  -- All authenticated users can read

CREATE POLICY campaigns_write_policy ON campaigns
    FOR ALL USING (current_setting('app.user_role') IN ('admin', 'marketing_manager'));

-- ============================================================================
-- GRANTS (example - adjust for actual roles)
-- ============================================================================

-- Read-only role for dashboards
-- CREATE ROLE marketing_readonly;
-- GRANT USAGE ON SCHEMA marketing TO marketing_readonly;
-- GRANT SELECT ON ALL TABLES IN SCHEMA marketing TO marketing_readonly;

-- Read-write role for applications
-- CREATE ROLE marketing_app;
-- GRANT USAGE ON SCHEMA marketing TO marketing_app;
-- GRANT SELECT, INSERT, UPDATE ON ALL TABLES IN SCHEMA marketing TO marketing_app;
-- GRANT USAGE ON ALL SEQUENCES IN SCHEMA marketing TO marketing_app;

-- ============================================================================
-- END OF SCHEMA
-- ============================================================================
