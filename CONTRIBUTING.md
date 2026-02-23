# Contributing to sharp-cli

Welcome to the party! We're glad you're here. Whether you stumbled in from a late-night ClickHouse debugging session or you just genuinely enjoy writing partition heuristics for fun — we don't judge.

`sharp` is an opinionated infrastructure tool. We prioritize clarity, determinism, and performance-oriented design. If that sounds boring, you clearly haven't spent enough time staring at a 4TB table with the wrong `ORDER BY`.

## Development setup

You know the drill:

```bash
git clone https://github.com/lucasarieta/sharp-cli
cd sharp-cli
cargo build
cargo test
```

Requires Rust 2024 edition (see `Cargo.toml`). If you're still on 2021, we respect the vintage energy but please upgrade.

## Project structure

The codebase is small on purpose. If you need a map, here it is — but honestly you could read the whole thing during a coffee break:

```
src/
  cli.rs              # CLI argument parsing (clap)
  main.rs             # Command dispatch
  errors.rs           # Error types
  config/
    schema.rs          # YAML schema definition (EventSchema, EventTable)
    workload.rs        # WorkloadProfile derived from schema
  engine/
    partitioning.rs    # Partition strategy heuristics (the brain)
    ordering.rs        # ORDER BY heuristics (the other brain)
    ttl.rs             # TTL generation (the grim reaper)
    projections.rs     # Projection recommendations
    heuristics.rs      # General recommendations engine
  sql/
    ast.rs             # SQL AST types (CreateTable, ColumnExpr)
    builder.rs         # AST construction from schema + heuristics
  output/
    formatter.rs       # SQL output formatting
```

## What we accept

Good contributions make `sharp` smarter without making it bigger. Think "sharper," not "wider":

- **Improved heuristics** — if you've got battle scars from a real workload that taught you something, encode it
- **Bug fixes** — yes, even heuristic engines have bugs. Shocking.
- **Tests** — especially for edge cases. The boundary between "daily" and "monthly" partitioning is not the place for "it probably works"
- **Clearer reasoning output** — if `sharp explain` made you squint, make it better
- **Performance improvements** — though if the CLI itself is your bottleneck, something else has gone very wrong
- **Documentation improvements** — typos happen. We won't tell anyone.

## What we avoid

We say no to things so `sharp` stays sharp. Nothing personal:

- **Generic ORM-like features** — this is not Diesel, and it never will be
- **Full SQL parsing engines** — we generate SQL, we don't interrogate it
- **Database connectivity layers** — `sharp` doesn't talk to your database. It doesn't even know your database exists. That's a feature.
- **Over-configurability** — if everything is configurable, nothing is opinionated. We picked a side.
- **Large abstraction layers** — if your PR adds a `trait SchemaStrategyFactoryProvider`, we need to talk

If you're unsure whether a contribution fits, open an issue first. We'd rather have a five-minute conversation than review a 2,000-line PR that doesn't land.

## Code style

- Follow Rust 2024 idioms
- Keep modules small and cohesive — if a file needs a table of contents, it's too big
- Avoid unnecessary abstractions. Three similar lines of code are better than a premature `impl`.
- Keep heuristics explicit and readable — they are the heart of this project. Clever code is the enemy of maintainable heuristics.
- Add comments explaining **why** decisions are made, not what the code does. We can read Rust. We can't read your mind.

## Testing

All heuristic logic must have unit tests. No exceptions. "It works on my YAML file" is not a test strategy.

Partitioning, ordering, TTL, and recommendation logic should be testable in isolation — no file I/O, no CLI invocation, no "well you have to spin up ClickHouse first."

Run the full suite:

```bash
cargo test
```

If it passes locally, you're probably fine. If it doesn't, you're definitely not.

## Pull request guidelines

- **Keep PRs small and focused** — one concern per PR. We're not reviewing your life's work in a single diff.
- **Include reasoning in the PR description** — explain why, not just what. "Improved partitioning" tells us nothing. "Changed the 5M threshold to 3M because X workload showed Y behavior" tells us everything.
- **Add or update tests** when modifying heuristics. If you change a threshold and don't add a test, the PR isn't done.
- **Don't break CLI contracts** (command names, flags, output format) without discussion first. People have scripts that depend on this. Those people will find you.
- Run `cargo test` and `cargo clippy` before submitting. Let the robots catch the easy stuff.

## Issue guidelines

When opening an issue, give us something to work with:

- Workload details (events/day, retention, multi-tenant or not)
- Example queries you're optimizing for
- Current schema (if applicable)
- Observed performance issues

"It's slow" is not an issue. "50M events/day, daily partitions, queries by event_name take 30s" is an issue. The more concrete, the faster we can help.

## License

By contributing, you agree that your contributions will be licensed under the MIT License. No surprises — just like `sharp` itself.
