---
name: senior-data-engineer
description: Data engineering heuristics. Pipelines, data modeling, lakehouses, streaming, dbt, Polars, DuckDB, Spark, Kafka, Airflow/Dagster, data quality, and data contracts. Load when designing or implementing data infrastructure, pipelines, or storage systems.
metadata:
  audience: senior-data-engineer, architect, fullstack-lt, developer
---

# Senior Data Engineer

## When to Use

- Designing a data pipeline (batch or streaming)
- Selecting a storage format, engine, or architecture
- Designing a data model (dimensional, medallion, data vault)
- Implementing data quality tests or contracts
- Reviewing pipeline code for correctness, idempotency, or performance
- Designing lakehouse or warehouse architecture
- Building ETL/ELT with dbt, Polars, Spark, or DuckDB
- Defining data governance, lineage, or PII handling

## Triggers

data pipeline, etl, elt, dbt, polars, spark, kafka, flink, airflow, dagster, prefect, parquet, delta, iceberg, hudi, duckdb, lakehouse, warehouse, data model, schema, data quality, data contract, streaming, batch, medallion, bronze silver gold, dimensional, kimball, data vault, feature store, embedding pipeline

## Core Workflow

1. **Understand the data contract** — source, format, frequency, volume, consumer, freshness SLA.
2. **Model before building** — define schema and layer structure before writing any transformation.
3. **Test first** — write the data quality contract (schema test, row count reconciliation, business rule) before writing the pipeline.
4. **Build idempotently** — every pipeline re-run on the same input must produce the same output.
5. **Instrument** — emit row counts, null rates, latency, and freshness metrics at every stage.
6. **Review** — request `@code-review` for pipeline code; `@sec-ops-expert` for PII and access control.

## Storage Format Decision Matrix

| Format | When | Avoid when |
|---|---|---|
| **Parquet** | Analytics, columnar reads, long-term storage | Frequent small updates, streaming sinks |
| **Delta Lake** | ACID on object storage, upserts, time travel | No Spark/Databricks dependency desired |
| **Apache Iceberg** | Multi-engine ACID (Spark + Trino + Athena), table evolution | Small projects, no need for table versioning |
| **Apache Hudi** | Streaming upserts, near-real-time analytics | Batch-only workflows |
| **Avro** | Row-streaming (Kafka), schema registry, Java ecosystem | Analytical queries (columnar reads) |
| **ORC** | Hive-native analytics | Modern Spark/Trino workloads (Parquet preferred) |
| **CSV/JSON** | Source systems, quick scripts | Production pipelines (no schema enforcement) |

## Data Modeling Patterns

### Medallion Architecture
```
Bronze (Raw)     → exact copy of source, immutable, append-only
Silver (Cleaned) → deduplicated, typed, validated, joined where needed
Gold (Serving)   → business aggregates, optimised for query patterns
```

### Kimball Dimensional
```
Fact tables:  grain = atomic event, surrogate keys, foreign keys to dims
Dim tables:   descriptive attributes, slowly changing (SCD1/2/4)
SCD Type 2:   current_flag + effective_from/to for full history
```

### Data Vault 2.0
```
Hubs:    unique business keys (one per entity)
Links:   relationships between hubs
Sats:    descriptive attributes (history preserved, insert-only)
```

### When to choose
| Pattern | When |
|---|---|
| Medallion | Event-driven, streaming ingestion, broad reuse across consumers |
| Kimball | BI reporting, star-schema-driven dashboards, Kimball-savvy team |
| Data Vault | Enterprise, enterprise-level audit requirements, frequent source system changes |
| OBT (One Big Table) | Single consumer, simple queries, small–medium scale |

## Incremental Load Strategies

| Strategy | Trigger | Idempotency | Use case |
|---|---|---|---|
| **Full load** | Schedule | Replace partition | Small tables, no deletes |
| **Append incremental** | `updated_at > last_run` | Deduplicate on read | Immutable events |
| **Upsert (merge)** | CDC / unique key | MERGE INTO | Mutable records |
| **SCD Type 2** | Change detection | Insert with versioning | Historical dimension |
| **Snapshot** | Schedule | Timestamped insert | Point-in-time analysis |

## Data Quality Contract Template

```yaml
# Example: dbt schema.yml style
models:
  - name: orders
    description: Clean orders — one row per order
    columns:
      - name: order_id
        tests:
          - not_null
          - unique
      - name: customer_id
        tests:
          - not_null
          - relationships:
              to: ref('customers')
              field: customer_id
      - name: order_amount
        tests:
          - not_null
          - dbt_utils.accepted_range:
              min_value: 0
              inclusive: true
    tests:
      - dbt_utils.recency:
          datepart: hour
          field: created_at
          interval: 25   # freshness SLA: data must be < 25h old
      - dbt_utils.equal_rowcount:
          compare_model: ref('raw_orders')   # row count reconciliation
```

## Pipeline Code Standards

### Idempotency Pattern (Polars / Python)
```python
def process_batch(execution_date: date) -> pl.DataFrame:
    """
    Idempotent: calling this with the same execution_date
    always produces the same output. Never reads 'today'.
    """
    raw = read_source(date=execution_date)
    return (
        raw
        .filter(pl.col("event_date") == execution_date)
        .with_columns([
            pl.col("amount").cast(pl.Float64),
            pl.lit(execution_date).alias("partition_date"),
        ])
        .unique(subset=["event_id"])   # deduplication
    )
```

### Partition Write Pattern (Parquet on object storage)
```python
# Write with atomic rename pattern — never partial partitions
# Hive partition layout: provider=X/year=YYYY/month=MM/day=DD/
def write_partition(df: pl.DataFrame, base_path: str, partition_date: date) -> None:
    path = (
        f"{base_path}/year={partition_date.year}"
        f"/month={partition_date.month:02d}"
        f"/day={partition_date.day:02d}"
    )
    # Write to temp, then atomic rename
    tmp = f"{path}/_tmp_{uuid4()}.parquet"
    final = f"{path}/data-{uuid4()}.parquet"
    df.write_parquet(tmp, compression="zstd")
    os.rename(tmp, final)   # atomic on same filesystem
```

### DuckDB Query Pattern (analytics on local Parquet)
```sql
-- Hive partition pruning: DuckDB reads only matching partitions
SELECT
    subscription_id,
    resource_type,
    COUNT(*) AS resource_count,
    SUM(cost) AS total_cost
FROM read_parquet(
    'data/provider=azure/**/*.parquet',
    hive_partitioning = true    -- enables partition pruning
)
WHERE year = 2026 AND month = 5
GROUP BY 1, 2
ORDER BY total_cost DESC;
```

## Observability Checklist

```
[ ] Row count emitted at source read (input)
[ ] Row count emitted at sink write (output)
[ ] Row count reconciliation: input == output (or delta documented)
[ ] Null rate per critical column tracked
[ ] Schema validation at ingestion boundary
[ ] Freshness check: last_event_time vs. expected interval
[ ] Pipeline execution time tracked
[ ] Cost tracked (cloud query cost if applicable)
[ ] Alert on: zero rows, >5% null rate increase, freshness breach, failure
```

## Streaming Patterns

| Pattern | When | Tool |
|---|---|---|
| **At-least-once + idempotent sink** | Default; simplest to implement | Kafka + upsert sink |
| **Exactly-once** | Transactions required; financial data | Kafka transactions + ACID sink |
| **Micro-batch** | Near-real-time (minutes), not true streaming | Spark Structured Streaming |
| **Event sourcing** | Full audit trail required; rebuild state from events | Kafka + Flink |
| **Watermarking** | Late data tolerance required | Flink / Spark with watermark |

## PII Handling Standards

```
1. Classify at ingestion: mark PII fields in schema metadata
2. Bronze layer: hash or tokenise before writing
   - Direct identifier (name, email, phone): SHA-256 hash or tokenise to opaque ID
   - Indirect identifier (IP, device ID, location): pseudonymise
   - Sensitive (health, financial, passwords): never store in analytics layer
3. Silver/Gold layers: no raw PII — only hashed/tokenised forms
4. Access control: PII fields partitioned or access-restricted at storage level
5. Audit log: access to PII-containing tables is logged
6. Retention: enforce delete-by date for GDPR/CCPA compliance via partition drops
```

## Output Template

```
## Data Contract
- Source: <system, format, frequency>
- Consumer: <who, freshness SLA>
- Volume: <rows/day, GB/day>
- Schema: <key fields with types>

## Model Design
<medallion layers / dimensional schema / ERD>

## Pipeline Design
- Strategy: <full / incremental / streaming>
- Partitioning: <key(s)>
- Idempotency: <how guaranteed>
- Backfill approach: <parameterised date range>

## Quality Contract
- Schema tests: <list>
- Row count check: <source vs sink>
- Business rule tests: <list>
- Freshness SLA: <expected interval, alert threshold>

## Storage
- Format: <Parquet / Delta / etc>
- Location: <path pattern>
- Retention: <days, compaction strategy>

## PII
- Fields: <list>
- Handling: <hash / tokenise / drop>

## Performance
- Estimated rows/run: <N>
- Estimated runtime: <Ns / Nm>
- Query p95 (if DuckDB/warehouse): <Ns>
```

## References

- dbt documentation: https://docs.getdbt.com/
- Apache Iceberg spec: https://iceberg.apache.org/spec/
- Polars documentation: https://docs.pola.rs/
- DuckDB documentation: https://duckdb.org/docs/
- Structured Streaming (Spark): https://spark.apache.org/docs/latest/structured-streaming-programming-guide.html
- The Data Warehouse Toolkit (Kimball)
- Fundamentals of Data Engineering (Reis & Housley)
- Data Management at Scale (Schmarzo)
