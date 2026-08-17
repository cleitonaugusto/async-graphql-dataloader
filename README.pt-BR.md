# async-graphql-dataloader

[![Crates.io](https://img.shields.io/crates/v/async-graphql-dataloader)](https://crates.io/crates/async-graphql-dataloader)
[![Documentação](https://docs.rs/async-graphql-dataloader/badge.svg)](https://docs.rs/async-graphql-dataloader)
[![Licença: MIT/Apache-2.0](https://img.shields.io/badge/License-MIT%2FApache--2.0-blue.svg)](LICENSE-MIT)

*[English version](README.md)*

Uma implementação de DataLoader para Rust com agrupamento automático de
requisições (batching) e cache, construída sobre o Tokio.

Ela resolve o problema N+1: em vez de uma chamada ao backend por resolução de
campo, chamadas concorrentes a `load` feitas dentro de uma pequena janela de
tempo são combinadas em uma única chamada em lote à sua fonte de dados.

> **Qual DataLoader usar?**
> O [`async-graphql`](https://docs.rs/async-graphql) já traz o próprio
> DataLoader por trás da feature `dataloader`, e para a maioria dos projetos
> que usam `async-graphql` esse é o padrão correto — ele é mantido junto com o
> framework e integra direto com o contexto dele.
> Este crate é uma alternativa independente, que também funciona fora de
> GraphQL: qualquer busca chave/valor agrupável pode usá-lo.

## Instalação

```toml
[dependencies]
async-graphql-dataloader = "0.2"
```

Com integração ao `async-graphql`:

```toml
[dependencies]
async-graphql-dataloader = { version = "0.2", features = ["graphql"] }
```

## Começo rápido

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
        // Uma única chamada ao banco para o lote inteiro.
        keys.iter().map(|&k| (k, Ok(format!("Usuário {}", k)))).collect()
    }
}

#[tokio::main]
async fn main() {
    let loader = DataLoader::new(UserLoader);

    // Estas duas são combinadas em uma única chamada a `load`.
    let (a, b) = tokio::join!(loader.load(1), loader.load(2));

    println!("{:?} {:?}", a, b);
}
```

## Configuração

```rust
use std::time::Duration;

let loader = DataLoader::new(UserLoader)
    .with_max_batch_size(50)                // dispara ao acumular 50 chaves
    .with_delay(Duration::from_millis(10)); // ou após 10ms, o que vier primeiro
```

## O que está incluído

- **Batching automático** — chamadas concorrentes a `load` viram uma só chamada
- **Cache** — cache chave/valor em memória com TTL opcional (`Cache::with_ttl`)
- **Rate limiting** — limitador de janela fixa (`RateLimiter`), opcional por loader
- **Análise de custo de query** — estimativa por regras (`QueryCostAnalyzer`), opcional
- **Telemetria** — contadores de lote e cache (`TelemetryCollector`)
- **Integração `sqlx`** — helper em `integrations::sqlx`

Todos os recursos rodam em processo. Este crate **não** inclui cache
distribuído, isolamento multi-tenant nem ferramentas de conformidade.

## Exemplos

Veja o diretório [`examples/`](examples/):

- [`basic_usage.rs`](examples/basic_usage.rs) — batching e cache básicos
- [`axum_graphql.rs`](examples/axum_graphql.rs) — integração Axum + async-graphql
- [`debug_batch.rs`](examples/debug_batch.rs) — inspecionando o comportamento dos lotes

Para rodar:

```bash
cargo run --example basic_usage
```

## Status

Mantido conforme disponibilidade. Relatos de bug e pull requests são bem-vindos;
por favor abra uma issue antes de começar um trabalho grande.

## Licença

Licenciado sob [MIT](LICENSE-MIT) ou [Apache-2.0](LICENSE-APACHE), à sua escolha.
