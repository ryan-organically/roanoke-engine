# =============================================================================
# ROANOKE MARKETING DATA PIPELINE - TERRAFORM CONFIGURATION
# =============================================================================
# Version: 1.0.0
# Description: Infrastructure as Code for marketing data pipeline
# =============================================================================

terraform {
  required_version = ">= 1.6.0"

  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
    kubernetes = {
      source  = "hashicorp/kubernetes"
      version = "~> 2.23"
    }
    helm = {
      source  = "hashicorp/helm"
      version = "~> 2.11"
    }
    random = {
      source  = "hashicorp/random"
      version = "~> 3.5"
    }
  }

  backend "s3" {
    bucket         = "roanoke-terraform-state"
    key            = "marketing/data-pipeline/terraform.tfstate"
    region         = "us-east-1"
    encrypt        = true
    dynamodb_table = "terraform-state-lock"
  }
}

# =============================================================================
# VARIABLES
# =============================================================================

variable "environment" {
  description = "Deployment environment"
  type        = string
  validation {
    condition     = contains(["staging", "production"], var.environment)
    error_message = "Environment must be staging or production."
  }
}

variable "image_tag" {
  description = "Docker image tag to deploy"
  type        = string
}

variable "region" {
  description = "AWS region"
  type        = string
  default     = "us-east-1"
}

variable "deployment_strategy" {
  description = "Deployment strategy"
  type        = string
  default     = "rolling"
  validation {
    condition     = contains(["rolling", "blue-green", "canary"], var.deployment_strategy)
    error_message = "Deployment strategy must be rolling, blue-green, or canary."
  }
}

locals {
  name_prefix = "roanoke-marketing-${var.environment}"

  common_tags = {
    Project     = "roanoke-marketing"
    Environment = var.environment
    ManagedBy   = "terraform"
    Team        = "data-platform"
  }

  # Environment-specific configurations
  config = {
    staging = {
      api_replicas      = 2
      etl_workers       = 2
      db_instance_class = "db.r6g.large"
      kafka_brokers     = 3
      redis_node_type   = "cache.r6g.large"
    }
    production = {
      api_replicas      = 4
      etl_workers       = 6
      db_instance_class = "db.r6g.xlarge"
      kafka_brokers     = 6
      redis_node_type   = "cache.r6g.xlarge"
    }
  }
}

# =============================================================================
# PROVIDERS
# =============================================================================

provider "aws" {
  region = var.region

  default_tags {
    tags = local.common_tags
  }
}

provider "kubernetes" {
  host                   = data.aws_eks_cluster.main.endpoint
  cluster_ca_certificate = base64decode(data.aws_eks_cluster.main.certificate_authority[0].data)
  token                  = data.aws_eks_cluster_auth.main.token
}

provider "helm" {
  kubernetes {
    host                   = data.aws_eks_cluster.main.endpoint
    cluster_ca_certificate = base64decode(data.aws_eks_cluster.main.certificate_authority[0].data)
    token                  = data.aws_eks_cluster_auth.main.token
  }
}

# =============================================================================
# DATA SOURCES
# =============================================================================

data "aws_caller_identity" "current" {}

data "aws_eks_cluster" "main" {
  name = "${local.name_prefix}-eks"
}

data "aws_eks_cluster_auth" "main" {
  name = "${local.name_prefix}-eks"
}

data "aws_vpc" "main" {
  filter {
    name   = "tag:Name"
    values = ["${local.name_prefix}-vpc"]
  }
}

data "aws_subnets" "private" {
  filter {
    name   = "vpc-id"
    values = [data.aws_vpc.main.id]
  }
  filter {
    name   = "tag:Tier"
    values = ["private"]
  }
}

# =============================================================================
# KMS KEYS
# =============================================================================

resource "aws_kms_key" "data_pipeline" {
  description             = "KMS key for marketing data pipeline encryption"
  deletion_window_in_days = 30
  enable_key_rotation     = true

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Sid    = "Enable IAM policies"
        Effect = "Allow"
        Principal = {
          AWS = "arn:aws:iam::${data.aws_caller_identity.current.account_id}:root"
        }
        Action   = "kms:*"
        Resource = "*"
      },
      {
        Sid    = "Allow services"
        Effect = "Allow"
        Principal = {
          Service = [
            "rds.amazonaws.com",
            "s3.amazonaws.com",
            "kafka.amazonaws.com"
          ]
        }
        Action = [
          "kms:Encrypt",
          "kms:Decrypt",
          "kms:GenerateDataKey*"
        ]
        Resource = "*"
      }
    ]
  })

  tags = {
    Name = "${local.name_prefix}-kms"
  }
}

resource "aws_kms_alias" "data_pipeline" {
  name          = "alias/${local.name_prefix}"
  target_key_id = aws_kms_key.data_pipeline.key_id
}

# =============================================================================
# S3 BUCKETS
# =============================================================================

resource "aws_s3_bucket" "data_lake" {
  bucket = "${local.name_prefix}-data-lake"

  tags = {
    Name = "${local.name_prefix}-data-lake"
  }
}

resource "aws_s3_bucket_versioning" "data_lake" {
  bucket = aws_s3_bucket.data_lake.id
  versioning_configuration {
    status = "Enabled"
  }
}

resource "aws_s3_bucket_server_side_encryption_configuration" "data_lake" {
  bucket = aws_s3_bucket.data_lake.id

  rule {
    apply_server_side_encryption_by_default {
      kms_master_key_id = aws_kms_key.data_pipeline.arn
      sse_algorithm     = "aws:kms"
    }
    bucket_key_enabled = true
  }
}

resource "aws_s3_bucket_lifecycle_configuration" "data_lake" {
  bucket = aws_s3_bucket.data_lake.id

  rule {
    id     = "archive-old-data"
    status = "Enabled"

    transition {
      days          = 90
      storage_class = "STANDARD_IA"
    }

    transition {
      days          = 365
      storage_class = "GLACIER"
    }

    expiration {
      days = 2555  # 7 years
    }

    filter {
      prefix = "raw/"
    }
  }
}

resource "aws_s3_bucket_public_access_block" "data_lake" {
  bucket = aws_s3_bucket.data_lake.id

  block_public_acls       = true
  block_public_policy     = true
  ignore_public_acls      = true
  restrict_public_buckets = true
}

# =============================================================================
# RDS POSTGRESQL
# =============================================================================

resource "aws_db_subnet_group" "main" {
  name       = "${local.name_prefix}-db-subnet"
  subnet_ids = data.aws_subnets.private.ids

  tags = {
    Name = "${local.name_prefix}-db-subnet"
  }
}

resource "aws_security_group" "database" {
  name        = "${local.name_prefix}-db-sg"
  description = "Security group for marketing database"
  vpc_id      = data.aws_vpc.main.id

  ingress {
    from_port       = 5432
    to_port         = 5432
    protocol        = "tcp"
    security_groups = [aws_security_group.api.id, aws_security_group.etl.id]
  }

  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }

  tags = {
    Name = "${local.name_prefix}-db-sg"
  }
}

resource "random_password" "db_password" {
  length  = 32
  special = false
}

resource "aws_secretsmanager_secret" "db_credentials" {
  name        = "${local.name_prefix}/db-credentials"
  description = "Database credentials for marketing data platform"
  kms_key_id  = aws_kms_key.data_pipeline.arn
}

resource "aws_secretsmanager_secret_version" "db_credentials" {
  secret_id = aws_secretsmanager_secret.db_credentials.id
  secret_string = jsonencode({
    username = "marketing_admin"
    password = random_password.db_password.result
    host     = aws_db_instance.main.address
    port     = 5432
    database = "marketing"
  })
}

resource "aws_db_instance" "main" {
  identifier = "${local.name_prefix}-db"

  engine               = "postgres"
  engine_version       = "15.4"
  instance_class       = local.config[var.environment].db_instance_class
  allocated_storage    = 100
  max_allocated_storage = 1000
  storage_type         = "gp3"
  storage_encrypted    = true
  kms_key_id           = aws_kms_key.data_pipeline.arn

  db_name  = "marketing"
  username = "marketing_admin"
  password = random_password.db_password.result

  db_subnet_group_name   = aws_db_subnet_group.main.name
  vpc_security_group_ids = [aws_security_group.database.id]

  multi_az               = var.environment == "production"
  publicly_accessible    = false
  deletion_protection    = var.environment == "production"
  skip_final_snapshot    = var.environment != "production"
  final_snapshot_identifier = var.environment == "production" ? "${local.name_prefix}-final-snapshot" : null

  backup_retention_period = var.environment == "production" ? 30 : 7
  backup_window          = "03:00-04:00"
  maintenance_window     = "Mon:04:00-Mon:05:00"

  performance_insights_enabled          = true
  performance_insights_retention_period = 7
  performance_insights_kms_key_id       = aws_kms_key.data_pipeline.arn

  enabled_cloudwatch_logs_exports = ["postgresql", "upgrade"]

  tags = {
    Name = "${local.name_prefix}-db"
  }
}

# =============================================================================
# ELASTICACHE REDIS
# =============================================================================

resource "aws_security_group" "redis" {
  name        = "${local.name_prefix}-redis-sg"
  description = "Security group for marketing Redis"
  vpc_id      = data.aws_vpc.main.id

  ingress {
    from_port       = 6379
    to_port         = 6379
    protocol        = "tcp"
    security_groups = [aws_security_group.api.id, aws_security_group.etl.id]
  }

  tags = {
    Name = "${local.name_prefix}-redis-sg"
  }
}

resource "aws_elasticache_subnet_group" "main" {
  name       = "${local.name_prefix}-redis-subnet"
  subnet_ids = data.aws_subnets.private.ids
}

resource "aws_elasticache_replication_group" "main" {
  replication_group_id = "${local.name_prefix}-redis"
  description          = "Marketing data platform Redis cluster"

  node_type            = local.config[var.environment].redis_node_type
  num_cache_clusters   = var.environment == "production" ? 3 : 2
  port                 = 6379

  subnet_group_name  = aws_elasticache_subnet_group.main.name
  security_group_ids = [aws_security_group.redis.id]

  at_rest_encryption_enabled = true
  transit_encryption_enabled = true
  kms_key_id                 = aws_kms_key.data_pipeline.arn

  automatic_failover_enabled = var.environment == "production"
  multi_az_enabled           = var.environment == "production"

  snapshot_retention_limit = 7
  snapshot_window          = "05:00-06:00"

  tags = {
    Name = "${local.name_prefix}-redis"
  }
}

# =============================================================================
# MSK KAFKA
# =============================================================================

resource "aws_security_group" "kafka" {
  name        = "${local.name_prefix}-kafka-sg"
  description = "Security group for marketing Kafka"
  vpc_id      = data.aws_vpc.main.id

  ingress {
    from_port       = 9092
    to_port         = 9096
    protocol        = "tcp"
    security_groups = [aws_security_group.api.id, aws_security_group.etl.id]
  }

  ingress {
    from_port = 9092
    to_port   = 9096
    protocol  = "tcp"
    self      = true
  }

  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }

  tags = {
    Name = "${local.name_prefix}-kafka-sg"
  }
}

resource "aws_msk_cluster" "main" {
  cluster_name           = "${local.name_prefix}-kafka"
  kafka_version          = "3.5.1"
  number_of_broker_nodes = local.config[var.environment].kafka_brokers

  broker_node_group_info {
    instance_type   = var.environment == "production" ? "kafka.m5.2xlarge" : "kafka.m5.large"
    client_subnets  = data.aws_subnets.private.ids
    security_groups = [aws_security_group.kafka.id]

    storage_info {
      ebs_storage_info {
        volume_size = 1000
      }
    }
  }

  encryption_info {
    encryption_at_rest_kms_key_arn = aws_kms_key.data_pipeline.arn
    encryption_in_transit {
      client_broker = "TLS"
      in_cluster    = true
    }
  }

  configuration_info {
    arn      = aws_msk_configuration.main.arn
    revision = aws_msk_configuration.main.latest_revision
  }

  logging_info {
    broker_logs {
      cloudwatch_logs {
        enabled   = true
        log_group = aws_cloudwatch_log_group.kafka.name
      }
    }
  }

  tags = {
    Name = "${local.name_prefix}-kafka"
  }
}

resource "aws_msk_configuration" "main" {
  name              = "${local.name_prefix}-kafka-config"
  kafka_versions    = ["3.5.1"]

  server_properties = <<PROPERTIES
auto.create.topics.enable=false
default.replication.factor=3
min.insync.replicas=2
num.partitions=12
log.retention.hours=168
log.retention.bytes=1073741824000
compression.type=lz4
PROPERTIES
}

resource "aws_cloudwatch_log_group" "kafka" {
  name              = "/aws/msk/${local.name_prefix}"
  retention_in_days = 30
  kms_key_id        = aws_kms_key.data_pipeline.arn
}

# =============================================================================
# SECURITY GROUPS FOR SERVICES
# =============================================================================

resource "aws_security_group" "api" {
  name        = "${local.name_prefix}-api-sg"
  description = "Security group for marketing API"
  vpc_id      = data.aws_vpc.main.id

  ingress {
    from_port   = 8080
    to_port     = 8080
    protocol    = "tcp"
    cidr_blocks = ["10.0.0.0/8"]
  }

  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }

  tags = {
    Name = "${local.name_prefix}-api-sg"
  }
}

resource "aws_security_group" "etl" {
  name        = "${local.name_prefix}-etl-sg"
  description = "Security group for marketing ETL workers"
  vpc_id      = data.aws_vpc.main.id

  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }

  tags = {
    Name = "${local.name_prefix}-etl-sg"
  }
}

# =============================================================================
# KUBERNETES DEPLOYMENTS
# =============================================================================

resource "kubernetes_namespace" "marketing" {
  metadata {
    name = "marketing-${var.environment}"

    labels = {
      name        = "marketing"
      environment = var.environment
    }
  }
}

resource "kubernetes_deployment" "api" {
  metadata {
    name      = "marketing-api"
    namespace = kubernetes_namespace.marketing.metadata[0].name

    labels = {
      app = "marketing-api"
    }
  }

  spec {
    replicas = local.config[var.environment].api_replicas

    selector {
      match_labels = {
        app = "marketing-api"
      }
    }

    template {
      metadata {
        labels = {
          app = "marketing-api"
        }
      }

      spec {
        container {
          name  = "api"
          image = "ghcr.io/roanoke/data-pipeline/api:${var.image_tag}"

          port {
            container_port = 8080
          }

          resources {
            requests = {
              cpu    = "500m"
              memory = "1Gi"
            }
            limits = {
              cpu    = "2"
              memory = "4Gi"
            }
          }

          env_from {
            secret_ref {
              name = kubernetes_secret.api_config.metadata[0].name
            }
          }

          liveness_probe {
            http_get {
              path = "/health/live"
              port = 8080
            }
            initial_delay_seconds = 30
            period_seconds        = 10
          }

          readiness_probe {
            http_get {
              path = "/health/ready"
              port = 8080
            }
            initial_delay_seconds = 5
            period_seconds        = 5
          }
        }
      }
    }
  }
}

resource "kubernetes_secret" "api_config" {
  metadata {
    name      = "marketing-api-config"
    namespace = kubernetes_namespace.marketing.metadata[0].name
  }

  data = {
    DATABASE_URL = "postgresql://${aws_db_instance.main.username}:${random_password.db_password.result}@${aws_db_instance.main.address}:5432/marketing"
    REDIS_URL    = "rediss://${aws_elasticache_replication_group.main.primary_endpoint_address}:6379"
    KAFKA_BROKERS = aws_msk_cluster.main.bootstrap_brokers_tls
  }
}

# =============================================================================
# OUTPUTS
# =============================================================================

output "database_endpoint" {
  description = "RDS PostgreSQL endpoint"
  value       = aws_db_instance.main.address
  sensitive   = true
}

output "redis_endpoint" {
  description = "ElastiCache Redis endpoint"
  value       = aws_elasticache_replication_group.main.primary_endpoint_address
}

output "kafka_brokers" {
  description = "MSK Kafka bootstrap brokers"
  value       = aws_msk_cluster.main.bootstrap_brokers_tls
}

output "data_lake_bucket" {
  description = "S3 data lake bucket name"
  value       = aws_s3_bucket.data_lake.id
}
