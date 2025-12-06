#!/usr/bin/env python3
"""
Roanoke Marketing Data Pipeline - Daily Aggregation Job

This Spark job performs daily aggregation of marketing events across all sources,
computing key metrics and materializing them for dashboard consumption.

Schedule: Daily at 02:00 UTC
Runtime: ~30 minutes
"""

from pyspark.sql import SparkSession
from pyspark.sql import functions as F
from pyspark.sql.types import (
    StructType, StructField, StringType, LongType,
    DoubleType, TimestampType, ArrayType, MapType
)
from pyspark.sql.window import Window
from datetime import datetime, timedelta
import argparse
import logging
import sys

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger('daily_aggregation')


class DailyAggregationJob:
    """Main job class for daily marketing data aggregation."""

    def __init__(self, spark: SparkSession, config: dict):
        self.spark = spark
        self.config = config
        self.processing_date = config.get('processing_date',
            (datetime.utcnow() - timedelta(days=1)).strftime('%Y-%m-%d'))

    def run(self) -> bool:
        """Execute the full daily aggregation pipeline."""
        try:
            logger.info(f"Starting daily aggregation for {self.processing_date}")

            # Step 1: Load raw events
            social_events = self._load_social_events()
            campaign_events = self._load_campaign_events()
            web_events = self._load_web_analytics()

            # Step 2: Compute aggregations
            social_metrics = self._aggregate_social_metrics(social_events)
            campaign_metrics = self._aggregate_campaign_metrics(campaign_events)
            web_metrics = self._aggregate_web_metrics(web_events)
            funnel_metrics = self._compute_funnel_metrics(web_events, campaign_events)

            # Step 3: Compute derived metrics
            combined_metrics = self._combine_and_derive_metrics(
                social_metrics, campaign_metrics, web_metrics
            )

            # Step 4: Quality checks
            if not self._validate_output(combined_metrics):
                raise ValueError("Output validation failed")

            # Step 5: Write outputs
            self._write_outputs(combined_metrics, funnel_metrics)

            logger.info("Daily aggregation completed successfully")
            return True

        except Exception as e:
            logger.error(f"Daily aggregation failed: {str(e)}")
            raise

    def _load_social_events(self):
        """Load social events for the processing date."""
        path = f"{self.config['raw_data_path']}/social/date={self.processing_date}"

        logger.info(f"Loading social events from {path}")

        return (
            self.spark.read
            .format("avro")
            .load(path)
            .filter(F.col("event_type").isNotNull())
            .withColumn("hour", F.hour("timestamp"))
        )

    def _load_campaign_events(self):
        """Load campaign events for the processing date."""
        path = f"{self.config['raw_data_path']}/campaigns/date={self.processing_date}"

        logger.info(f"Loading campaign events from {path}")

        return (
            self.spark.read
            .format("avro")
            .load(path)
            .filter(F.col("campaign.campaign_id").isNotNull())
        )

    def _load_web_analytics(self):
        """Load web analytics events for the processing date."""
        path = f"{self.config['raw_data_path']}/web/date={self.processing_date}"

        logger.info(f"Loading web analytics from {path}")

        return (
            self.spark.read
            .format("parquet")
            .load(path)
        )

    def _aggregate_social_metrics(self, df):
        """Compute social media aggregations."""
        logger.info("Computing social metrics aggregations")

        # Hourly aggregations by platform
        hourly_by_platform = (
            df.groupBy("platform", "hour")
            .agg(
                F.count("*").alias("event_count"),
                F.countDistinct("author.platform_user_id").alias("unique_authors"),
                F.sum("engagement.likes").alias("total_likes"),
                F.sum("engagement.shares").alias("total_shares"),
                F.sum("engagement.comments").alias("total_comments"),
                F.sum("engagement.views").alias("total_views"),
                F.avg("sentiment.score").alias("avg_sentiment"),
                F.sum(F.when(F.col("sentiment.label") == "POSITIVE", 1).otherwise(0)).alias("positive_count"),
                F.sum(F.when(F.col("sentiment.label") == "NEGATIVE", 1).otherwise(0)).alias("negative_count"),
                F.sum(F.when(F.col("classification.is_meme"), 1).otherwise(0)).alias("meme_count"),
                F.sum(F.when(F.col("classification.requires_response"), 1).otherwise(0)).alias("requires_response_count"),
                F.sum(F.when(F.col("author.is_known_creator"), 1).otherwise(0)).alias("creator_mentions")
            )
            .withColumn("date", F.lit(self.processing_date))
        )

        # Daily aggregations by platform
        daily_by_platform = (
            df.groupBy("platform")
            .agg(
                F.count("*").alias("total_events"),
                F.countDistinct("author.platform_user_id").alias("unique_authors"),
                F.sum("engagement.likes").alias("total_likes"),
                F.sum("engagement.shares").alias("total_shares"),
                F.sum("engagement.comments").alias("total_comments"),
                F.sum("engagement.views").alias("total_views"),
                F.avg("sentiment.score").alias("avg_sentiment"),
                F.stddev("sentiment.score").alias("sentiment_stddev"),
                F.expr("percentile_approx(sentiment.score, 0.5)").alias("median_sentiment"),
                F.sum(F.when(F.col("author.follower_count") > 10000, 1).otherwise(0)).alias("influencer_mentions"),
                F.max("engagement.likes").alias("max_likes_single_post"),
                F.collect_set(F.when(F.col("classification.is_meme"), F.col("classification.meme_template"))).alias("meme_templates_used")
            )
            .withColumn("date", F.lit(self.processing_date))
        )

        # Topic analysis
        topics = (
            df.select(
                "platform",
                F.explode("classification.topics").alias("topic")
            )
            .groupBy("platform", "topic")
            .agg(F.count("*").alias("mention_count"))
            .withColumn("date", F.lit(self.processing_date))
        )

        # Hashtag analysis
        hashtags = (
            df.select(
                "platform",
                F.explode("content.hashtags").alias("hashtag")
            )
            .groupBy("platform", "hashtag")
            .agg(F.count("*").alias("usage_count"))
            .orderBy(F.desc("usage_count"))
            .limit(100)
            .withColumn("date", F.lit(self.processing_date))
        )

        return {
            "hourly": hourly_by_platform,
            "daily": daily_by_platform,
            "topics": topics,
            "hashtags": hashtags
        }

    def _aggregate_campaign_metrics(self, df):
        """Compute campaign performance aggregations."""
        logger.info("Computing campaign metrics aggregations")

        # Campaign-level daily metrics
        campaign_daily = (
            df.groupBy(
                "campaign.campaign_id",
                "campaign.campaign_name",
                "campaign.campaign_type",
                "platform"
            )
            .agg(
                F.sum("metrics.impressions").alias("impressions"),
                F.sum("metrics.reach").alias("reach"),
                F.sum("metrics.clicks").alias("clicks"),
                F.sum("metrics.conversions").alias("conversions"),
                F.sum("metrics.video_views").alias("video_views"),
                F.sum("metrics.installs").alias("installs"),
                F.sum("metrics.wishlist_adds").alias("wishlist_adds"),
                F.sum("metrics.purchases").alias("purchases"),
                F.sum(F.col("costs.spend").cast("double")).alias("total_spend"),
                F.first("costs.currency").alias("currency")
            )
            .withColumn("ctr", F.col("clicks") / F.col("impressions") * 100)
            .withColumn("cvr", F.col("conversions") / F.col("clicks") * 100)
            .withColumn("cpc", F.col("total_spend") / F.col("clicks"))
            .withColumn("cpm", F.col("total_spend") / F.col("impressions") * 1000)
            .withColumn("cpa", F.col("total_spend") / F.col("conversions"))
            .withColumn("date", F.lit(self.processing_date))
        )

        # Creative-level performance
        creative_daily = (
            df.filter(F.col("creative").isNotNull())
            .groupBy(
                "campaign.campaign_id",
                "creative.creative_id",
                "creative.creative_name",
                "creative.creative_type"
            )
            .agg(
                F.sum("metrics.impressions").alias("impressions"),
                F.sum("metrics.clicks").alias("clicks"),
                F.sum("metrics.conversions").alias("conversions"),
                F.sum("metrics.video_completions").alias("video_completions"),
                F.sum(F.col("costs.spend").cast("double")).alias("spend")
            )
            .withColumn("ctr", F.col("clicks") / F.col("impressions") * 100)
            .withColumn("video_completion_rate",
                F.col("video_completions") / F.col("impressions") * 100)
            .withColumn("date", F.lit(self.processing_date))
        )

        # Platform-level rollup
        platform_daily = (
            df.groupBy("platform")
            .agg(
                F.countDistinct("campaign.campaign_id").alias("active_campaigns"),
                F.sum("metrics.impressions").alias("total_impressions"),
                F.sum("metrics.clicks").alias("total_clicks"),
                F.sum("metrics.conversions").alias("total_conversions"),
                F.sum("metrics.installs").alias("total_installs"),
                F.sum(F.col("costs.spend").cast("double")).alias("total_spend")
            )
            .withColumn("date", F.lit(self.processing_date))
        )

        return {
            "campaign_daily": campaign_daily,
            "creative_daily": creative_daily,
            "platform_daily": platform_daily
        }

    def _aggregate_web_metrics(self, df):
        """Compute web analytics aggregations."""
        logger.info("Computing web analytics aggregations")

        # Page-level metrics
        page_metrics = (
            df.filter(F.col("event_type") == "pageview")
            .groupBy("page_path")
            .agg(
                F.count("*").alias("pageviews"),
                F.countDistinct("session_id").alias("unique_sessions"),
                F.countDistinct("user_id").alias("unique_users"),
                F.avg("time_on_page").alias("avg_time_on_page"),
                F.sum(F.when(F.col("is_bounce"), 1).otherwise(0)).alias("bounces")
            )
            .withColumn("bounce_rate", F.col("bounces") / F.col("unique_sessions") * 100)
            .withColumn("date", F.lit(self.processing_date))
        )

        # Source/Medium attribution
        traffic_sources = (
            df.filter(F.col("event_type") == "session_start")
            .groupBy("utm_source", "utm_medium", "utm_campaign")
            .agg(
                F.count("*").alias("sessions"),
                F.countDistinct("user_id").alias("users"),
                F.sum(F.when(F.col("converted"), 1).otherwise(0)).alias("conversions"),
                F.sum("revenue").alias("revenue")
            )
            .withColumn("conversion_rate", F.col("conversions") / F.col("sessions") * 100)
            .withColumn("date", F.lit(self.processing_date))
        )

        # Device breakdown
        device_metrics = (
            df.groupBy("device_category", "browser", "os")
            .agg(
                F.countDistinct("session_id").alias("sessions"),
                F.countDistinct("user_id").alias("users"),
                F.avg("session_duration").alias("avg_session_duration")
            )
            .withColumn("date", F.lit(self.processing_date))
        )

        # Geo breakdown
        geo_metrics = (
            df.groupBy("country", "region")
            .agg(
                F.countDistinct("session_id").alias("sessions"),
                F.countDistinct("user_id").alias("users"),
                F.sum(F.when(F.col("converted"), 1).otherwise(0)).alias("conversions")
            )
            .withColumn("date", F.lit(self.processing_date))
        )

        return {
            "pages": page_metrics,
            "traffic_sources": traffic_sources,
            "devices": device_metrics,
            "geo": geo_metrics
        }

    def _compute_funnel_metrics(self, web_events, campaign_events):
        """Compute conversion funnel metrics."""
        logger.info("Computing funnel metrics")

        # Define funnel stages
        funnel_stages = [
            ("awareness", ["impression", "view"]),
            ("interest", ["click", "pageview"]),
            ("consideration", ["wishlist_add", "time_on_site_5min"]),
            ("intent", ["add_to_cart", "pricing_page"]),
            ("purchase", ["purchase", "subscribe"])
        ]

        # Count users at each stage
        stage_counts = []
        for stage_name, events in funnel_stages:
            count = (
                web_events
                .filter(F.col("event_type").isin(events))
                .select("user_id")
                .distinct()
                .count()
            )
            stage_counts.append((stage_name, count))

        # Create funnel DataFrame
        funnel_df = self.spark.createDataFrame(
            [(self.processing_date, stage, count, idx)
             for idx, (stage, count) in enumerate(stage_counts)],
            ["date", "stage", "users", "stage_order"]
        )

        # Add conversion rates
        window = Window.orderBy("stage_order")
        funnel_df = (
            funnel_df
            .withColumn("prev_users", F.lag("users").over(window))
            .withColumn("conversion_rate",
                F.when(F.col("prev_users").isNotNull(),
                    F.col("users") / F.col("prev_users") * 100
                ).otherwise(100.0)
            )
            .withColumn("dropoff_rate", 100 - F.col("conversion_rate"))
        )

        return funnel_df

    def _combine_and_derive_metrics(self, social, campaign, web):
        """Combine metrics and compute derived KPIs."""
        logger.info("Computing derived metrics")

        # Calculate key marketing KPIs
        social_totals = social["daily"].agg(
            F.sum("total_events").alias("total_social_events"),
            F.sum("total_likes").alias("total_social_engagement"),
            F.avg("avg_sentiment").alias("overall_sentiment")
        ).collect()[0]

        campaign_totals = campaign["platform_daily"].agg(
            F.sum("total_impressions").alias("total_impressions"),
            F.sum("total_clicks").alias("total_clicks"),
            F.sum("total_conversions").alias("total_conversions"),
            F.sum("total_spend").alias("total_spend")
        ).collect()[0]

        # Create daily summary
        daily_summary = self.spark.createDataFrame([{
            "date": self.processing_date,
            "social_mentions": social_totals["total_social_events"],
            "social_engagement": social_totals["total_social_engagement"],
            "sentiment_score": social_totals["overall_sentiment"],
            "ad_impressions": campaign_totals["total_impressions"],
            "ad_clicks": campaign_totals["total_clicks"],
            "ad_conversions": campaign_totals["total_conversions"],
            "ad_spend": campaign_totals["total_spend"],
            "ctr": (campaign_totals["total_clicks"] / campaign_totals["total_impressions"] * 100)
                   if campaign_totals["total_impressions"] > 0 else 0,
            "cvr": (campaign_totals["total_conversions"] / campaign_totals["total_clicks"] * 100)
                   if campaign_totals["total_clicks"] > 0 else 0,
            "cpa": (campaign_totals["total_spend"] / campaign_totals["total_conversions"])
                   if campaign_totals["total_conversions"] > 0 else 0,
            "processing_timestamp": datetime.utcnow().isoformat()
        }])

        return {
            "daily_summary": daily_summary,
            "social": social,
            "campaign": campaign,
            "web": web
        }

    def _validate_output(self, metrics) -> bool:
        """Validate output data quality."""
        logger.info("Validating output data quality")

        validations = [
            # Check daily summary exists
            metrics["daily_summary"].count() == 1,

            # Check no negative metrics
            metrics["daily_summary"].filter(
                (F.col("social_mentions") < 0) |
                (F.col("ad_impressions") < 0)
            ).count() == 0,

            # Check sentiment in valid range
            metrics["social"]["daily"].filter(
                (F.col("avg_sentiment") < -1) |
                (F.col("avg_sentiment") > 1)
            ).count() == 0,
        ]

        return all(validations)

    def _write_outputs(self, metrics, funnel):
        """Write aggregated outputs to storage."""
        output_path = self.config["processed_data_path"]

        # Write daily summary
        logger.info("Writing daily summary")
        (
            metrics["daily_summary"]
            .write
            .format("parquet")
            .mode("overwrite")
            .save(f"{output_path}/daily_summary/date={self.processing_date}")
        )

        # Write social metrics
        logger.info("Writing social metrics")
        for name, df in metrics["social"].items():
            (
                df.write
                .format("parquet")
                .mode("overwrite")
                .save(f"{output_path}/social/{name}/date={self.processing_date}")
            )

        # Write campaign metrics
        logger.info("Writing campaign metrics")
        for name, df in metrics["campaign"].items():
            (
                df.write
                .format("parquet")
                .mode("overwrite")
                .save(f"{output_path}/campaigns/{name}/date={self.processing_date}")
            )

        # Write web metrics
        logger.info("Writing web metrics")
        for name, df in metrics["web"].items():
            (
                df.write
                .format("parquet")
                .mode("overwrite")
                .save(f"{output_path}/web/{name}/date={self.processing_date}")
            )

        # Write funnel metrics
        logger.info("Writing funnel metrics")
        (
            funnel
            .write
            .format("parquet")
            .mode("overwrite")
            .save(f"{output_path}/funnel/date={self.processing_date}")
        )

        # Also write to database for dashboards
        self._write_to_database(metrics, funnel)

    def _write_to_database(self, metrics, funnel):
        """Write key metrics to PostgreSQL for dashboard access."""
        jdbc_url = self.config["jdbc_url"]
        jdbc_props = {
            "user": self.config["db_user"],
            "password": self.config["db_password"],
            "driver": "org.postgresql.Driver"
        }

        # Write daily summary
        (
            metrics["daily_summary"]
            .write
            .jdbc(jdbc_url, "marketing.daily_summary", "append", jdbc_props)
        )

        # Write funnel
        (
            funnel
            .write
            .jdbc(jdbc_url, "marketing.daily_funnel", "append", jdbc_props)
        )


def main():
    parser = argparse.ArgumentParser(description='Daily Marketing Aggregation Job')
    parser.add_argument('--date', type=str, help='Processing date (YYYY-MM-DD)')
    parser.add_argument('--config', type=str, required=True, help='Path to config file')
    args = parser.parse_args()

    # Initialize Spark
    spark = (
        SparkSession.builder
        .appName("DailyMarketingAggregation")
        .config("spark.sql.shuffle.partitions", "200")
        .config("spark.sql.adaptive.enabled", "true")
        .config("spark.serializer", "org.apache.spark.serializer.KryoSerializer")
        .enableHiveSupport()
        .getOrCreate()
    )

    try:
        # Load config
        import json
        with open(args.config) as f:
            config = json.load(f)

        if args.date:
            config['processing_date'] = args.date

        # Run job
        job = DailyAggregationJob(spark, config)
        success = job.run()

        sys.exit(0 if success else 1)

    finally:
        spark.stop()


if __name__ == "__main__":
    main()
