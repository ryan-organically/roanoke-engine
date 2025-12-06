# DATA PIPELINE SECURITY HARDENING GUIDE

<!--
@document-metadata
doc_id: DATA-SEC-001
title: Security Hardening Guide
version: 1.0.0
status: ACTIVE
owner: Security
created: 2025-12-05
updated: 2025-12-05
review_date: 2026-03-05
classification: Internal - Security
changelog: See /marketing/CHANGELOG.md
-->

| Field | Value |
|-------|-------|
| **Document ID** | DATA-SEC-001 |
| **Version** | 1.0.0 |
| **Status** | ACTIVE |
| **Owner** | Security |
| **Last Updated** | 2025-12-05 |
| **Classification** | Internal - Security |

---

## 1. Security Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        SECURITY PERIMETER                                    │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌──────────────────────────────────────────────────────────────────────┐   │
│  │                         NETWORK LAYER                                 │   │
│  │  ┌────────────┐   ┌────────────┐   ┌────────────┐   ┌────────────┐  │   │
│  │  │   WAF      │   │   DDoS     │   │ Rate Limit │   │   mTLS     │  │   │
│  │  │ (CloudFlare)│   │ Protection │   │  Gateway   │   │ Termination│  │   │
│  │  └────────────┘   └────────────┘   └────────────┘   └────────────┘  │   │
│  └──────────────────────────────────────────────────────────────────────┘   │
│                                    │                                         │
│  ┌──────────────────────────────────────────────────────────────────────┐   │
│  │                        IDENTITY LAYER                                 │   │
│  │  ┌────────────┐   ┌────────────┐   ┌────────────┐   ┌────────────┐  │   │
│  │  │   OIDC     │   │   RBAC     │   │   MFA      │   │  Service   │  │   │
│  │  │  (Auth0)   │   │  Engine    │   │ Required   │   │  Accounts  │  │   │
│  │  └────────────┘   └────────────┘   └────────────┘   └────────────┘  │   │
│  └──────────────────────────────────────────────────────────────────────┘   │
│                                    │                                         │
│  ┌──────────────────────────────────────────────────────────────────────┐   │
│  │                          DATA LAYER                                   │   │
│  │  ┌────────────┐   ┌────────────┐   ┌────────────┐   ┌────────────┐  │   │
│  │  │ Encryption │   │    PII     │   │   Data     │   │   Field    │  │   │
│  │  │  at Rest   │   │ Tokenization│   │  Masking  │   │   Level    │  │   │
│  │  │  (AES-256) │   │  (Vault)   │   │ (Dynamic)  │   │  Encryption│  │   │
│  │  └────────────┘   └────────────┘   └────────────┘   └────────────┘  │   │
│  └──────────────────────────────────────────────────────────────────────┘   │
│                                    │                                         │
│  ┌──────────────────────────────────────────────────────────────────────┐   │
│  │                        AUDIT LAYER                                    │   │
│  │  ┌────────────┐   ┌────────────┐   ┌────────────┐   ┌────────────┐  │   │
│  │  │  Access    │   │   Query    │   │  Change    │   │  Anomaly   │  │   │
│  │  │   Logs     │   │   Logs     │   │   Logs     │   │ Detection  │  │   │
│  │  └────────────┘   └────────────┘   └────────────┘   └────────────┘  │   │
│  └──────────────────────────────────────────────────────────────────────┘   │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 2. Authentication & Authorization

### 2.1 Authentication Requirements

| User Type | Method | MFA Required | Session Duration |
|-----------|--------|--------------|------------------|
| Human Users | OIDC (Auth0) | Yes | 8 hours |
| Service Accounts | mTLS + JWT | N/A | 1 hour |
| API Clients | API Key + JWT | Recommended | Per request |
| ETL Jobs | Service Account | N/A | Job duration |

### 2.2 RBAC Configuration

```yaml
# roles.yaml
roles:
  # Read-only access for dashboards and reporting
  data_viewer:
    description: "View metrics and reports"
    permissions:
      - metrics:read
      - reports:read
      - dashboards:read
      - campaigns:read
      - alerts:read

  # Standard analyst access
  data_analyst:
    extends: data_viewer
    permissions:
      - reports:create
      - exports:create
      - campaigns:read:all
      - social:read:detailed

  # Campaign management
  campaign_manager:
    extends: data_analyst
    permissions:
      - campaigns:create
      - campaigns:update
      - campaigns:delete
      - budgets:update
      - creatives:manage

  # Platform administration
  data_admin:
    extends: campaign_manager
    permissions:
      - users:manage
      - roles:manage
      - integrations:manage
      - settings:manage
      - audit_logs:read

  # Service account for ETL
  etl_service:
    permissions:
      - data:ingest
      - data:transform
      - data:write
      - schemas:read

  # Service account for APIs
  api_service:
    permissions:
      - data:read
      - metrics:read
      - cache:write
```

### 2.3 Service Account Management

```yaml
# service_accounts.yaml
service_accounts:
  - name: etl-spark-daily
    description: "Daily Spark aggregation jobs"
    roles:
      - etl_service
    allowed_ips:
      - 10.0.0.0/8  # Internal VPC only
    key_rotation: 30d
    audit: true

  - name: etl-flink-streaming
    description: "Real-time Flink streaming jobs"
    roles:
      - etl_service
    allowed_ips:
      - 10.0.0.0/8
    key_rotation: 30d
    audit: true

  - name: api-marketing-service
    description: "Marketing API service"
    roles:
      - api_service
    rate_limit: 10000/min
    key_rotation: 7d
    audit: true

  - name: grafana-readonly
    description: "Grafana dashboard service"
    roles:
      - data_viewer
    allowed_ips:
      - 10.0.0.0/8
    key_rotation: 90d
    audit: true
```

---

## 3. Encryption Standards

### 3.1 Encryption at Rest

| Component | Encryption | Key Management |
|-----------|------------|----------------|
| S3 Data Lake | AES-256-GCM | AWS KMS (CMK) |
| PostgreSQL | AES-256 TDE | Vault Transit |
| ClickHouse | AES-256 | Vault Transit |
| Redis | AES-256 | Vault Transit |
| Kafka | AES-256 | Confluent KMS |
| Backups | AES-256-GCM | AWS KMS (separate CMK) |

### 3.2 Encryption in Transit

```yaml
# tls_config.yaml
tls:
  minimum_version: TLSv1.3
  cipher_suites:
    - TLS_AES_256_GCM_SHA384
    - TLS_CHACHA20_POLY1305_SHA256
    - TLS_AES_128_GCM_SHA256

  certificate_management:
    provider: "Let's Encrypt"
    auto_renewal: true
    renewal_threshold_days: 30

  internal_services:
    mtls_required: true
    ca: "internal-ca.crt"
    cert_validity_days: 90

  external_apis:
    hsts: true
    hsts_max_age: 31536000
    hsts_include_subdomains: true
```

### 3.3 Field-Level Encryption

```python
# field_encryption.py
"""
Field-level encryption for sensitive data fields.
"""

from cryptography.fernet import Fernet
from hashlib import sha256
import base64
import os

class FieldEncryption:
    """Encrypt/decrypt sensitive fields using Vault transit backend."""

    SENSITIVE_FIELDS = {
        'email': 'pii',
        'ip_address': 'pii',
        'user_agent': 'pii',
        'author_id': 'pseudonymous',
        'content_text': 'content'
    }

    def __init__(self, vault_client):
        self.vault = vault_client

    def encrypt_field(self, field_name: str, value: str) -> str:
        """Encrypt a field value."""
        if field_name not in self.SENSITIVE_FIELDS:
            return value

        key_name = f"data-pipeline-{self.SENSITIVE_FIELDS[field_name]}"
        encrypted = self.vault.secrets.transit.encrypt_data(
            name=key_name,
            plaintext=base64.b64encode(value.encode()).decode()
        )
        return encrypted['data']['ciphertext']

    def decrypt_field(self, field_name: str, ciphertext: str) -> str:
        """Decrypt a field value."""
        if field_name not in self.SENSITIVE_FIELDS:
            return ciphertext

        key_name = f"data-pipeline-{self.SENSITIVE_FIELDS[field_name]}"
        decrypted = self.vault.secrets.transit.decrypt_data(
            name=key_name,
            ciphertext=ciphertext
        )
        return base64.b64decode(decrypted['data']['plaintext']).decode()

    def hash_for_analytics(self, field_name: str, value: str) -> str:
        """Create deterministic hash for analytics without exposing PII."""
        salt = os.environ.get('ANALYTICS_SALT', 'default-salt')
        return sha256(f"{salt}:{field_name}:{value}".encode()).hexdigest()
```

---

## 4. PII Handling & Data Masking

### 4.1 PII Classification

| Field | Classification | Retention | Masking Rule |
|-------|---------------|-----------|--------------|
| Email | PII-High | 30 days | Hash + domain |
| IP Address | PII-Medium | 7 days | Truncate to /24 |
| Username | PII-Low | 90 days | Partial mask |
| User Agent | PII-Low | 30 days | Parse to categories |
| Geo Location | PII-Low | Indefinite | Round to region |
| Content Text | Content | 2 years | No masking |
| Author ID | Pseudonymous | Indefinite | Tokenize |

### 4.2 Dynamic Data Masking Policy

```sql
-- PostgreSQL Dynamic Masking
-- Apply different masking based on user role

-- Create masking function
CREATE OR REPLACE FUNCTION mask_email(email TEXT, user_role TEXT)
RETURNS TEXT AS $$
BEGIN
    IF user_role IN ('data_admin', 'campaign_manager') THEN
        RETURN email;
    ELSIF user_role = 'data_analyst' THEN
        RETURN CONCAT(
            LEFT(SPLIT_PART(email, '@', 1), 2),
            '***@',
            SPLIT_PART(email, '@', 2)
        );
    ELSE
        RETURN CONCAT('***@', SPLIT_PART(email, '@', 2));
    END IF;
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

-- Create masking view
CREATE OR REPLACE VIEW masked_creators AS
SELECT
    creator_id,
    mask_email(email, current_setting('app.user_role')) as email,
    tier,
    partnership_status,
    created_at
FROM creators;
```

### 4.3 Data Retention Automation

```python
# data_retention.py
"""
Automated data retention enforcement.
"""

from datetime import datetime, timedelta
import logging

logger = logging.getLogger(__name__)

RETENTION_POLICIES = {
    'social_events_raw': {
        'retention_days': 90,
        'archive_to_cold': True,
        'delete_pii_fields': ['author.email', 'content.raw_text']
    },
    'campaign_events': {
        'retention_days': 730,  # 2 years
        'archive_to_cold': True,
        'delete_pii_fields': ['user.ip_address']
    },
    'web_analytics': {
        'retention_days': 365,
        'archive_to_cold': True,
        'anonymize_after_days': 30
    },
    'sentiment_alerts': {
        'retention_days': 180,
        'archive_to_cold': False,
        'delete_pii_fields': []
    }
}

class DataRetentionEnforcer:
    """Enforce data retention policies."""

    def __init__(self, spark, s3_client, db_client):
        self.spark = spark
        self.s3 = s3_client
        self.db = db_client

    def run_retention_job(self):
        """Execute all retention policies."""
        results = []

        for table, policy in RETENTION_POLICIES.items():
            try:
                result = self._enforce_policy(table, policy)
                results.append({'table': table, 'status': 'success', **result})
            except Exception as e:
                logger.error(f"Retention failed for {table}: {e}")
                results.append({'table': table, 'status': 'failed', 'error': str(e)})

        return results

    def _enforce_policy(self, table: str, policy: dict) -> dict:
        """Enforce a single retention policy."""
        cutoff_date = datetime.utcnow() - timedelta(days=policy['retention_days'])

        # Archive to cold storage if configured
        if policy.get('archive_to_cold'):
            archived = self._archive_old_data(table, cutoff_date)
        else:
            archived = 0

        # Delete old data
        deleted = self._delete_old_data(table, cutoff_date)

        # Anonymize if configured
        anonymized = 0
        if policy.get('anonymize_after_days'):
            anon_cutoff = datetime.utcnow() - timedelta(days=policy['anonymize_after_days'])
            anonymized = self._anonymize_data(table, anon_cutoff, policy['delete_pii_fields'])

        return {
            'cutoff_date': cutoff_date.isoformat(),
            'archived_records': archived,
            'deleted_records': deleted,
            'anonymized_records': anonymized
        }

    def _archive_old_data(self, table: str, cutoff: datetime) -> int:
        """Archive old data to S3 Glacier."""
        # Implementation would move data to cold storage
        pass

    def _delete_old_data(self, table: str, cutoff: datetime) -> int:
        """Delete data older than cutoff."""
        # Implementation would delete from hot/warm storage
        pass

    def _anonymize_data(self, table: str, cutoff: datetime, fields: list) -> int:
        """Anonymize PII fields in older data."""
        # Implementation would hash/remove PII fields
        pass
```

---

## 5. Network Security

### 5.1 Network Segmentation

```yaml
# network_topology.yaml
vpcs:
  production:
    cidr: "10.0.0.0/16"
    subnets:
      public:
        - cidr: "10.0.1.0/24"
          az: "us-east-1a"
          purpose: "Load balancers, NAT gateways"
      private_app:
        - cidr: "10.0.10.0/24"
          az: "us-east-1a"
          purpose: "API servers, workers"
        - cidr: "10.0.11.0/24"
          az: "us-east-1b"
          purpose: "API servers, workers"
      private_data:
        - cidr: "10.0.20.0/24"
          az: "us-east-1a"
          purpose: "Databases, Kafka"
        - cidr: "10.0.21.0/24"
          az: "us-east-1b"
          purpose: "Databases, Kafka"
      private_processing:
        - cidr: "10.0.30.0/24"
          az: "us-east-1a"
          purpose: "Spark, Flink clusters"

security_groups:
  alb:
    ingress:
      - port: 443
        source: "0.0.0.0/0"
    egress:
      - port: 8080
        destination: "sg-api"

  api:
    ingress:
      - port: 8080
        source: "sg-alb"
    egress:
      - port: 5432
        destination: "sg-database"
      - port: 9092
        destination: "sg-kafka"
      - port: 6379
        destination: "sg-redis"

  database:
    ingress:
      - port: 5432
        source: "sg-api"
      - port: 5432
        source: "sg-processing"
    egress: []

  kafka:
    ingress:
      - port: 9092
        source: "sg-api"
      - port: 9092
        source: "sg-processing"
    egress: []

  processing:
    ingress:
      - port: 7077  # Spark
        source: "sg-processing"
    egress:
      - port: 5432
        destination: "sg-database"
      - port: 9092
        destination: "sg-kafka"
      - port: 443
        destination: "0.0.0.0/0"  # S3, external APIs
```

### 5.2 WAF Rules

```yaml
# waf_rules.yaml
waf:
  provider: cloudflare

  rate_limiting:
    - name: api_rate_limit
      path: "/marketing/v1/*"
      requests: 100
      period: 60
      action: block

    - name: auth_rate_limit
      path: "/auth/*"
      requests: 10
      period: 60
      action: challenge

  managed_rules:
    - cloudflare_managed
    - owasp_core
    - known_bots

  custom_rules:
    - name: block_bad_user_agents
      expression: |
        (http.user_agent contains "sqlmap") or
        (http.user_agent contains "nikto") or
        (http.user_agent contains "nmap")
      action: block

    - name: require_auth_header
      expression: |
        (http.request.uri.path matches "^/marketing/v1/") and
        not (http.request.headers["authorization"] matches "^Bearer ")
      action: block

    - name: geo_restrict_admin
      expression: |
        (http.request.uri.path matches "^/admin/") and
        not (ip.geoip.country in {"US" "CA" "GB" "DE"})
      action: challenge
```

---

## 6. Audit Logging

### 6.1 Audit Log Schema

```json
{
  "timestamp": "2025-01-15T14:30:00.000Z",
  "event_id": "uuid-v4",
  "event_type": "data_access",
  "actor": {
    "user_id": "uuid",
    "username": "analyst@roanoke.com",
    "role": "data_analyst",
    "ip_address": "10.0.10.15",
    "user_agent": "Mozilla/5.0...",
    "session_id": "sess_abc123"
  },
  "resource": {
    "type": "table",
    "name": "campaign_metrics_daily",
    "schema": "marketing"
  },
  "action": {
    "type": "SELECT",
    "query_hash": "sha256...",
    "rows_returned": 1500,
    "columns_accessed": ["campaign_id", "impressions", "spend"]
  },
  "context": {
    "application": "grafana",
    "dashboard_id": "dash_123",
    "request_id": "req_xyz"
  },
  "outcome": {
    "success": true,
    "duration_ms": 45
  }
}
```

### 6.2 Audit Configuration

```yaml
# audit_config.yaml
audit:
  enabled: true
  log_destination: "s3://roanoke-audit-logs/data-platform/"

  events:
    # Always log
    always:
      - authentication
      - authorization_failure
      - data_export
      - schema_change
      - user_management
      - role_change
      - configuration_change

    # Log based on data classification
    by_classification:
      pii_high:
        - data_access
        - data_modify
      pii_medium:
        - data_modify
      pii_low: []

    # Sample for high-volume operations
    sampled:
      data_access:
        rate: 0.1  # 10% sampling
        exclude_roles: ["etl_service"]

  retention:
    hot_days: 30
    warm_days: 90
    cold_days: 2555  # 7 years for compliance

  alerting:
    - condition: "5+ failed auth in 5 minutes"
      severity: high
      channel: security-alerts

    - condition: "bulk data export > 100k rows"
      severity: medium
      channel: data-alerts

    - condition: "access from new IP"
      severity: low
      channel: security-info
```

---

## 7. Secrets Management

### 7.1 Vault Configuration

```hcl
# vault_policy.hcl
# Data pipeline secrets access

path "secret/data/marketing/database/*" {
  capabilities = ["read"]
}

path "secret/data/marketing/api-keys/*" {
  capabilities = ["read"]
}

path "transit/encrypt/data-pipeline-pii" {
  capabilities = ["update"]
}

path "transit/decrypt/data-pipeline-pii" {
  capabilities = ["update"]
}

path "auth/token/renew-self" {
  capabilities = ["update"]
}
```

### 7.2 Secret Rotation

```yaml
# secret_rotation.yaml
rotation:
  database_credentials:
    frequency: 30d
    method: rotate_credentials
    notification_before: 7d

  api_keys:
    frequency: 90d
    method: rotate_and_invalidate
    grace_period: 24h

  encryption_keys:
    frequency: 365d
    method: key_rotation
    keep_old_versions: 3

  service_account_tokens:
    frequency: 7d
    method: auto_rotate
    notification: none
```

---

## 8. Incident Response

### 8.1 Security Incident Runbook

```yaml
# incident_response.yaml
incidents:
  data_breach_suspected:
    severity: critical
    response_time: 15m
    steps:
      1: "Alert security team via PagerDuty"
      2: "Isolate affected systems"
      3: "Capture forensic evidence (logs, snapshots)"
      4: "Identify scope of exposure"
      5: "Notify legal and DPO"
      6: "Prepare customer notification if required"

  unauthorized_access:
    severity: high
    response_time: 30m
    steps:
      1: "Revoke compromised credentials"
      2: "Block source IP"
      3: "Review audit logs for scope"
      4: "Reset affected user sessions"
      5: "Root cause analysis"

  anomalous_data_access:
    severity: medium
    response_time: 2h
    steps:
      1: "Verify with user if known"
      2: "Review access patterns"
      3: "Temporary access suspension if suspicious"
      4: "Escalate if confirmed unauthorized"

escalation:
  critical:
    - security-oncall
    - cto
    - legal
  high:
    - security-oncall
    - engineering-lead
  medium:
    - security-team
```

---

## 9. Compliance Checklist

### 9.1 GDPR Requirements

| Requirement | Implementation | Status |
|-------------|---------------|--------|
| Right to Access | Export API endpoint | ✓ |
| Right to Erasure | Deletion workflow | ✓ |
| Data Portability | JSON/CSV export | ✓ |
| Consent Tracking | Consent service integration | ✓ |
| Data Minimization | Retention policies | ✓ |
| Encryption | At rest and in transit | ✓ |
| Breach Notification | Incident response plan | ✓ |
| DPO Contact | privacy@playroanoke.com | ✓ |

### 9.2 SOC 2 Controls

| Control | Description | Evidence |
|---------|-------------|----------|
| CC6.1 | Logical access controls | RBAC, MFA |
| CC6.2 | Access provisioning | Approval workflow |
| CC6.3 | Access removal | Offboarding automation |
| CC6.6 | Encryption in transit | TLS 1.3 |
| CC6.7 | Encryption at rest | AES-256 |
| CC7.2 | Monitoring | Audit logs, SIEM |
| CC8.1 | Change management | GitOps, approval gates |

---

*© 2025 Roanoke Interactive, Inc. | Security Hardening Guide v1.0.0*
