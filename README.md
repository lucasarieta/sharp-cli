# sharp-cli

Opinionated schema intelligence for ClickHouse event analytics.

Most ClickHouse performance problems are not query problems — they are schema problems. You know the ones. Someone picked `toYYYYMM` for a table doing 200M events/day, and now your merges take longer than your stand-ups.

`sharp` generates production-ready event storage schemas so you don't have to learn the hard way that your `ORDER BY` was backwards. You describe your workload, it gives you an optimized schema — and explains every decision, because "trust me bro" is not a partitioning strategy.

This is not a database. This is the friend who actually read the ClickHouse docs.

## Why?

ClickHouse is incredibly powerful. It will also let you shoot yourself in the foot at 1 million rows per second.

- Partitioning is easy to get wrong, and you won't notice until it's *very* wrong
- ORDER BY design is subtle — get it backwards and your queries scan the entire table while looking perfectly reasonable
- Multi-tenant schemas are often "single-tenant schemas with a `WHERE` clause and a prayer"
- Projections and rollups exist, but most teams discover them six months after they needed them
- TTL and retention are either forgotten entirely or set by whoever was on-call that one Friday

`sharp` encodes real-world analytics heuristics into deterministic SQL generation. Same input, same output. No surprises. Your on-call rotation will thank you.

## Installation

```bash
cargo install sharp-cli
```

Or build from source, if you're the kind of person who reads `Cargo.toml` for fun:

```bash
git clone https://github.com/lucasarieta/sharp-cli
cd sharp-cli
cargo build --release
```

## Usage

Three commands. That's it. We don't believe in CLIs that need their own wiki.

### 1. Initialize a schema file

```bash
sharp init
```

Generates a starter YAML workload definition. Fill in the blanks — we're not going to guess your event volume for you:

```yaml
event_table:
  name: user_events
  multi_tenant: true
  expected_events_per_day: 50000000
  retention_days: 90
```

### 2. Generate optimized SQL

```bash
sharp generate schema.yaml
```

Outputs a complete `CREATE TABLE` statement with partition strategy, ORDER BY clause, TTL configuration, and recommended projections. Copy-paste ready. No tweaking required (but we won't stop you).

```sql
CREATE TABLE user_events (
    project_id UInt32,
    timestamp DateTime,
    event_name LowCardinality(String),
    distinct_id String,
    properties JSON
) ENGINE = MergeTree
PARTITION BY toYYYYMMDD(timestamp)
ORDER BY (project_id, event_name, timestamp, distinct_id)
TTL timestamp + INTERVAL 90 DAY;
```

### 3. Explain decisions

```bash
sharp explain schema.yaml
```

Shows the reasoning behind every schema choice. Because "I found it on a ClickHouse blog post from 2019" is not documentation:

```
Partitioning:
  Strategy: Daily
  SQL:      PARTITION BY toYYYYMMDD(timestamp)
  Reason:   Daily partitioning selected: 50000000 events/day falls in the
            5M–200M range, balancing query pruning against part management overhead.

Ordering:
  SQL:      ORDER BY (project_id, event_name, timestamp, distinct_id)
  Reason:   Multi-tenant workloads filter heavily by project_id, then event_name.

Recommendations:
  - Enable TTL to auto-expire old data (`TTL timestamp + INTERVAL 90 DAY`)
  - Add a projection on (project_id, toDate(timestamp)) for per-tenant dashboards
  - Ensure project_id has LowCardinality(String) type for efficient filtering
  - Use LZ4 compression (ClickHouse default) — switch to ZSTD if storage-constrained
```

Every recommendation comes with a reason. If `sharp` tells you to do something, it can tell you *why*. Your future self debugging at 2am deserves that much.

## Heuristics

`sharp` makes opinionated decisions based on your workload profile. These aren't magic numbers — they're battle-tested thresholds from teams that learned the hard way so you don't have to:

| Workload | Partition Strategy | Why |
|---|---|---|
| < 5M events/day | Monthly | You don't need daily partitions. Calm down. |
| 5M–200M events/day | Daily | The sweet spot — good pruning without drowning in parts |
| > 200M events/day + multi-tenant | Daily + tenant | At this scale, tenant isolation isn't optional |

Additional recommendations kick in at volume thresholds:

- **100M+ events/day** — enables `wide_parts_only` because your merges are about to get expensive
- **500M+ events/day** — suggests sharding, because one node can only do so much before it starts sending you passive-aggressive log messages
- **50M+ events/day + multi-tenant** — adds per-tenant projections so your dashboards don't crawl

All three thresholds are checked automatically. You get the right advice for your scale without having to know the right questions to ask.

## Schema file reference

Four fields. That's all `sharp` needs to make every decision:

```yaml
event_table:
  name: string           # Table name
  multi_tenant: bool     # Multi-tenant workload? (default: false)
  expected_events_per_day: int  # Daily event volume — be honest
  retention_days: int    # How long to keep data (0 = forever, good luck)
```

## Design philosophy

`sharp` is:

- **Opinionated** — good defaults over endless configuration. You don't need 47 knobs when 4 will do.
- **Deterministic** — same input, same output. Every time. No "it worked on my machine."
- **Workload-driven** — decisions are based on real analytics patterns, not theoretical best practices from a textbook nobody finished.
- **Narrowly focused** — event analytics schemas, nothing else. We do one thing and we do it well.

`sharp` is NOT a database, query engine, migration framework, or ORM. If you're looking for any of those, you're in the wrong repo. No hard feelings.

## License

MIT
