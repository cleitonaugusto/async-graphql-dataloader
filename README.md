# async-graphql-dataloader

[![Crates.io](https://img.shields.io/crates/v/async-graphql-dataloader)](https://crates.io/crates/async-graphql-dataloader)
[![Documentation](https://docs.rs/async-graphql-dataloader/badge.svg)](https://docs.rs/async-graphql-dataloader)
[![License: MIT/Apache-2.0](https://img.shields.io/badge/License-MIT%2FApache--2.0-blue.svg)](LICENSE-MIT)

A DataLoader implementation for Rust with automatic request batching and
per-request caching, built on Tokio.

It addresses the N+1 query problem: instead of one backend call per field
resolution, concurrent `load` calls made within a short time window are
coalesced into a single batched call to your data source.

> **Which DataLoader should you use?**
> [`async-graphql`](https://docs.rs/async-graphql) ships its own DataLoader
> behind its `dataloader` feature, and for most `async-graphql` projects that
> is the right default — it is maintained alongside the framework and
> integrates directly with its context.
> This crate is a standalone alternative that also works outside of GraphQL:
> any batchable key/value lookup can use it.

## Installation

```toml
[dependencies]
async-graphql-dataloader = "0.2"
```

With `async-graphql` integration:

```toml
[dependencies]
async-graphql-dataloader = { version = "0.2", features = ["graphql"] }
```

## Quick start

```rust
use async_graphql_dataloader::{BatchLoad, DataLoader};
use std::collections::HashMap;

struct UserLoader;

#[async_trait::async_trait]
impl BatchLoad for UserLoader {
    type Key = i32;
    type Value = String;
    type Error = String;

    async fn load(&self, keys: &[i32]) -> HashMap<i32, Result<String, String>> {
        // One call to your database for the whole batch.
        keys.iter().map(|&k| (k, Ok(format!("User {}", k)))).collect()
    }
}

#[tokio::main]
async fn main() {
    let loader = DataLoader::new(UserLoader);

    // These two are coalesced into a single `load` call.
    let (a, b) = tokio::join!(loader.load(1), loader.load(2));

    println!("{:?} {:?}", a, b);
}
```

## Configuration

```rust
use std::time::Duration;

let loader = DataLoader::new(UserLoader)
    .with_max_batch_size(50)               // flush once 50 keys are queued
    .with_delay(Duration::from_millis(10)); // or after 10ms, whichever comes first
```

## What's included

- **Automatic batching** — concurrent `load` calls are coalesced into one call
- **Caching** — in-memory key/value cache with optional TTL (`Cache::with_ttl`)
- **Rate limiting** — fixed-window limiter (`RateLimiter`), opt-in per loader
- **Query cost analysis** — rule-based cost estimation (`QueryCostAnalyzer`), opt-in
- **Telemetry** — batch/cache counters (`TelemetryCollector`)
- **`sqlx` integration** — helper in `integrations::sqlx`

All features run in-process. There is no distributed cache, no multi-tenant
isolation layer, and no compliance tooling in this crate.

## Examples

See [`examples/`](examples/):

- [`basic_usage.rs`](examples/basic_usage.rs) — core batching and caching
- [`axum_graphql.rs`](examples/axum_graphql.rs) — Axum + async-graphql wiring
- [`debug_batch.rs`](examples/debug_batch.rs) — inspecting batch behaviour

Run one with:

```bash
cargo run --example basic_usage
```

## Status

Actively maintained. Bug reports and pull requests are welcome; please open an
issue before starting significant work, and read
[CONTRIBUTING.md](CONTRIBUTING.md) first — contributions require a sign-off.

> **Heads-up on licensing:** 0.2.x is MIT OR Apache-2.0 and stays that way
> permanently. Starting at 0.3.0 the project is dual-licensed under AGPL-3.0 or
> a paid commercial license. If a permissive license is a hard requirement for
> you, pin `async-graphql-dataloader = "0.2"`, or use the DataLoader built into
> `async-graphql` — it is MIT OR Apache-2.0 and, for most `async-graphql`
> projects, the better default anyway.

## License

**This version (0.2.x) is licensed under [MIT](LICENSE-MIT) OR
[Apache-2.0](LICENSE-APACHE), at your option.** That grant is perpetual and
will not be withdrawn.

**From 0.3.0 the project becomes dual-licensed:**
[AGPL-3.0-or-later](LICENSE-AGPL-3.0) free of charge, or a paid
[commercial license](COMMERCIAL-LICENSE.md) that lifts the AGPL's
source-disclosure obligation. Commercial tiers start at $500/year per
organization.

The AGPL is enough for evaluation, personal and academic work, AGPL-licensed
projects, and anything you don't expose over a network outside your
organization. You need the commercial license to ship 0.3.0+ in a
network-facing product without releasing that product's source.

See [LICENSING.md](LICENSING.md) for the full picture, and
[CONTRIBUTING.md](CONTRIBUTING.md) for what this means for pull requests.
