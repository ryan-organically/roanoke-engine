#!/usr/bin/env python3
"""
Roanoke Marketing Data Pipeline - Real-time Social Sentiment Streaming

This Flink job processes social media events in real-time, computing
sentiment scores, detecting anomalies, and triggering alerts.

Runtime: Continuous streaming
Latency SLA: < 5 minutes end-to-end
"""

from pyflink.datastream import StreamExecutionEnvironment, TimeCharacteristic
from pyflink.datastream.connectors.kafka import (
    FlinkKafkaConsumer, FlinkKafkaProducer
)
from pyflink.datastream.functions import (
    MapFunction, FilterFunction, KeyedProcessFunction,
    RuntimeContext, ProcessWindowFunction
)
from pyflink.datastream.window import TumblingEventTimeWindows, SlidingEventTimeWindows
from pyflink.datastream.state import ValueStateDescriptor, ListStateDescriptor
from pyflink.common.typeinfo import Types
from pyflink.common.watermark_strategy import WatermarkStrategy
from pyflink.common.serialization import SimpleStringSchema
from pyflink.common import Time, Duration

import json
import logging
from datetime import datetime
from typing import Iterable, Optional
from dataclasses import dataclass
import hashlib

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger('social_sentiment_stream')


@dataclass
class SocialEvent:
    """Parsed social event."""
    event_id: str
    event_type: str
    timestamp: int
    platform: str
    author_id: str
    author_followers: int
    text: str
    sentiment_score: Optional[float]
    sentiment_label: Optional[str]
    is_meme: bool
    topics: list
    engagement_likes: int
    engagement_shares: int
    engagement_comments: int
    is_creator: bool
    requires_response: bool
    priority: str


@dataclass
class SentimentMetric:
    """Aggregated sentiment metric."""
    window_start: int
    window_end: int
    platform: str
    event_count: int
    avg_sentiment: float
    positive_count: int
    negative_count: int
    neutral_count: int
    total_engagement: int
    unique_authors: int
    influencer_mentions: int
    meme_count: int
    requires_response_count: int


@dataclass
class SentimentAlert:
    """Alert triggered by sentiment anomaly."""
    alert_id: str
    alert_type: str
    severity: str
    platform: str
    timestamp: int
    message: str
    current_value: float
    threshold: float
    context: dict


class ParseSocialEventFunction(MapFunction):
    """Parse raw JSON social events."""

    def map(self, value: str) -> Optional[SocialEvent]:
        try:
            data = json.loads(value)

            return SocialEvent(
                event_id=data.get('event_id', ''),
                event_type=data.get('event_type', ''),
                timestamp=data.get('timestamp', 0),
                platform=data.get('platform', 'UNKNOWN'),
                author_id=data.get('author', {}).get('platform_user_id', ''),
                author_followers=data.get('author', {}).get('follower_count', 0) or 0,
                text=data.get('content', {}).get('text', ''),
                sentiment_score=data.get('sentiment', {}).get('score'),
                sentiment_label=data.get('sentiment', {}).get('label'),
                is_meme=data.get('classification', {}).get('is_meme', False),
                topics=data.get('classification', {}).get('topics', []),
                engagement_likes=data.get('engagement', {}).get('likes', 0),
                engagement_shares=data.get('engagement', {}).get('shares', 0),
                engagement_comments=data.get('engagement', {}).get('comments', 0),
                is_creator=data.get('author', {}).get('is_known_creator', False),
                requires_response=data.get('classification', {}).get('requires_response', False),
                priority=data.get('classification', {}).get('priority', 'LOW')
            )
        except Exception as e:
            logger.error(f"Failed to parse event: {e}")
            return None


class FilterValidEventsFunction(FilterFunction):
    """Filter out invalid or irrelevant events."""

    def filter(self, event: Optional[SocialEvent]) -> bool:
        if event is None:
            return False
        if not event.event_id:
            return False
        if event.sentiment_score is None:
            return False
        return True


class SentimentAggregationFunction(ProcessWindowFunction):
    """Aggregate sentiment metrics over time windows."""

    def process(self,
                key: str,
                context: ProcessWindowFunction.Context,
                elements: Iterable[SocialEvent]) -> Iterable[SentimentMetric]:

        events = list(elements)
        if not events:
            return

        platform = key
        window = context.window()

        # Compute aggregations
        event_count = len(events)
        sentiments = [e.sentiment_score for e in events if e.sentiment_score is not None]
        avg_sentiment = sum(sentiments) / len(sentiments) if sentiments else 0.0

        positive_count = sum(1 for e in events if e.sentiment_label == 'POSITIVE' or e.sentiment_label == 'VERY_POSITIVE')
        negative_count = sum(1 for e in events if e.sentiment_label == 'NEGATIVE' or e.sentiment_label == 'VERY_NEGATIVE')
        neutral_count = event_count - positive_count - negative_count

        total_engagement = sum(
            e.engagement_likes + e.engagement_shares + e.engagement_comments
            for e in events
        )

        unique_authors = len(set(e.author_id for e in events))
        influencer_mentions = sum(1 for e in events if e.author_followers > 10000)
        meme_count = sum(1 for e in events if e.is_meme)
        requires_response_count = sum(1 for e in events if e.requires_response)

        yield SentimentMetric(
            window_start=window.start,
            window_end=window.end,
            platform=platform,
            event_count=event_count,
            avg_sentiment=avg_sentiment,
            positive_count=positive_count,
            negative_count=negative_count,
            neutral_count=neutral_count,
            total_engagement=total_engagement,
            unique_authors=unique_authors,
            influencer_mentions=influencer_mentions,
            meme_count=meme_count,
            requires_response_count=requires_response_count
        )


class AnomalyDetectionFunction(KeyedProcessFunction):
    """Detect sentiment anomalies using rolling statistics."""

    def open(self, runtime_context: RuntimeContext):
        # State for rolling average
        self.avg_state = runtime_context.get_state(
            ValueStateDescriptor("avg_sentiment", Types.FLOAT())
        )
        self.stddev_state = runtime_context.get_state(
            ValueStateDescriptor("stddev_sentiment", Types.FLOAT())
        )
        self.count_state = runtime_context.get_state(
            ValueStateDescriptor("window_count", Types.INT())
        )
        # Historical values for stddev calculation
        self.history_state = runtime_context.get_list_state(
            ListStateDescriptor("sentiment_history", Types.FLOAT())
        )

        self.alert_threshold_stddev = 2.0  # Alert if > 2 standard deviations
        self.min_windows_for_baseline = 10  # Need 10 windows to establish baseline

    def process_element(self,
                       metric: SentimentMetric,
                       ctx: KeyedProcessFunction.Context) -> Iterable[SentimentAlert]:

        # Get current state
        current_avg = self.avg_state.value() or 0.0
        current_stddev = self.stddev_state.value() or 0.5
        window_count = self.count_state.value() or 0

        # Update history
        history = list(self.history_state.get() or [])
        history.append(metric.avg_sentiment)

        # Keep last 100 windows
        if len(history) > 100:
            history = history[-100:]
        self.history_state.update(history)

        # Update rolling statistics
        new_avg = sum(history) / len(history)
        variance = sum((x - new_avg) ** 2 for x in history) / len(history)
        new_stddev = variance ** 0.5 if variance > 0 else 0.1

        self.avg_state.update(new_avg)
        self.stddev_state.update(new_stddev)
        self.count_state.update(window_count + 1)

        # Check for anomalies if we have enough baseline
        if window_count >= self.min_windows_for_baseline:
            deviation = abs(metric.avg_sentiment - current_avg) / current_stddev

            # Sentiment drop alert
            if metric.avg_sentiment < current_avg - (self.alert_threshold_stddev * current_stddev):
                yield self._create_alert(
                    alert_type="SENTIMENT_DROP",
                    severity="HIGH" if deviation > 3 else "MEDIUM",
                    platform=metric.platform,
                    message=f"Sentiment dropped significantly on {metric.platform}",
                    current_value=metric.avg_sentiment,
                    threshold=current_avg - (self.alert_threshold_stddev * current_stddev),
                    context={
                        "baseline_avg": current_avg,
                        "baseline_stddev": current_stddev,
                        "deviation_stddev": deviation,
                        "event_count": metric.event_count,
                        "negative_count": metric.negative_count
                    }
                )

            # Sentiment spike alert (could indicate viral positive moment)
            if metric.avg_sentiment > current_avg + (self.alert_threshold_stddev * current_stddev):
                yield self._create_alert(
                    alert_type="SENTIMENT_SPIKE",
                    severity="INFO",
                    platform=metric.platform,
                    message=f"Positive sentiment spike on {metric.platform}",
                    current_value=metric.avg_sentiment,
                    threshold=current_avg + (self.alert_threshold_stddev * current_stddev),
                    context={
                        "baseline_avg": current_avg,
                        "event_count": metric.event_count,
                        "positive_count": metric.positive_count
                    }
                )

            # Volume anomaly
            if metric.event_count > 0 and window_count > 20:
                # Would need historical event counts - simplified here
                if metric.requires_response_count > 10:
                    yield self._create_alert(
                        alert_type="HIGH_RESPONSE_VOLUME",
                        severity="HIGH",
                        platform=metric.platform,
                        message=f"High volume of posts requiring response on {metric.platform}",
                        current_value=float(metric.requires_response_count),
                        threshold=10.0,
                        context={
                            "event_count": metric.event_count,
                            "requires_response_count": metric.requires_response_count
                        }
                    )

    def _create_alert(self, alert_type: str, severity: str, platform: str,
                     message: str, current_value: float, threshold: float,
                     context: dict) -> SentimentAlert:
        timestamp = int(datetime.utcnow().timestamp() * 1000)
        alert_id = hashlib.sha256(
            f"{alert_type}:{platform}:{timestamp}".encode()
        ).hexdigest()[:16]

        return SentimentAlert(
            alert_id=alert_id,
            alert_type=alert_type,
            severity=severity,
            platform=platform,
            timestamp=timestamp,
            message=message,
            current_value=current_value,
            threshold=threshold,
            context=context
        )


class MetricToJsonFunction(MapFunction):
    """Convert metrics to JSON for output."""

    def map(self, metric: SentimentMetric) -> str:
        return json.dumps({
            "window_start": metric.window_start,
            "window_end": metric.window_end,
            "platform": metric.platform,
            "event_count": metric.event_count,
            "avg_sentiment": metric.avg_sentiment,
            "positive_count": metric.positive_count,
            "negative_count": metric.negative_count,
            "neutral_count": metric.neutral_count,
            "total_engagement": metric.total_engagement,
            "unique_authors": metric.unique_authors,
            "influencer_mentions": metric.influencer_mentions,
            "meme_count": metric.meme_count,
            "requires_response_count": metric.requires_response_count
        })


class AlertToJsonFunction(MapFunction):
    """Convert alerts to JSON for output."""

    def map(self, alert: SentimentAlert) -> str:
        return json.dumps({
            "alert_id": alert.alert_id,
            "alert_type": alert.alert_type,
            "severity": alert.severity,
            "platform": alert.platform,
            "timestamp": alert.timestamp,
            "message": alert.message,
            "current_value": alert.current_value,
            "threshold": alert.threshold,
            "context": alert.context
        })


def main():
    # Create execution environment
    env = StreamExecutionEnvironment.get_execution_environment()
    env.set_stream_time_characteristic(TimeCharacteristic.EventTime)
    env.set_parallelism(8)

    # Enable checkpointing for exactly-once semantics
    env.enable_checkpointing(60000)  # 60 second checkpoints
    env.get_checkpoint_config().set_min_pause_between_checkpoints(30000)
    env.get_checkpoint_config().set_checkpoint_timeout(120000)

    # Kafka configuration
    kafka_props = {
        'bootstrap.servers': 'kafka:9092',
        'group.id': 'social-sentiment-stream',
        'auto.offset.reset': 'latest'
    }

    # Create Kafka source
    kafka_source = FlinkKafkaConsumer(
        topics='social-events',
        deserialization_schema=SimpleStringSchema(),
        properties=kafka_props
    )

    # Assign timestamps and watermarks
    watermark_strategy = (
        WatermarkStrategy
        .for_bounded_out_of_orderness(Duration.of_minutes(5))
        .with_timestamp_assigner(
            lambda event, _: json.loads(event).get('timestamp', 0)
        )
    )

    # Build the pipeline
    social_events = (
        env
        .add_source(kafka_source)
        .assign_timestamps_and_watermarks(watermark_strategy)
        .map(ParseSocialEventFunction())
        .filter(FilterValidEventsFunction())
    )

    # Aggregate sentiment by platform in 5-minute windows
    sentiment_metrics = (
        social_events
        .key_by(lambda e: e.platform)
        .window(TumblingEventTimeWindows.of(Time.minutes(5)))
        .process(SentimentAggregationFunction())
    )

    # Detect anomalies
    alerts = (
        sentiment_metrics
        .key_by(lambda m: m.platform)
        .process(AnomalyDetectionFunction())
    )

    # Output metrics to Kafka
    metrics_sink = FlinkKafkaProducer(
        topic='sentiment-metrics',
        serialization_schema=SimpleStringSchema(),
        producer_config={'bootstrap.servers': 'kafka:9092'}
    )

    sentiment_metrics.map(MetricToJsonFunction()).add_sink(metrics_sink)

    # Output alerts to Kafka
    alerts_sink = FlinkKafkaProducer(
        topic='sentiment-alerts',
        serialization_schema=SimpleStringSchema(),
        producer_config={'bootstrap.servers': 'kafka:9092'}
    )

    alerts.map(AlertToJsonFunction()).add_sink(alerts_sink)

    # Execute
    env.execute("Social Sentiment Streaming Pipeline")


if __name__ == "__main__":
    main()
