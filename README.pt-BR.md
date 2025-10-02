# async-graphql-dataloader

🚀 **Implementação de DataLoader de alta performance para async-graphql em Rust**

https://img.shields.io/crates/v/async-graphql-dataloader
https://img.shields.io/badge/License-MIT-blue.svg
https://img.shields.io/badge/rust-1.60%252B-orange.svg
https://docs.rs/async-graphql-dataloader/badge.svg

## 🎯 **Sobre o Projeto**

**Criado e desenvolvido por: [Cleiton Augusto Correa Bezerra](https://github.com/cleitonaugusto)**

Este projeto resolve um dos problemas mais comuns em aplicações GraphQL: o **problema N+1**.

### ⚡ **Por que usar este DataLoader?**

- **🚀 Performance**: Batch loading inteligente e cache
- **🦀 Segurança**: Garantias de segurança de memória do Rust
- **⚡ Concorrência**: Async/await nativo com Tokio
- **🔧 Flexível**: Fácil integração com qualquer fonte de dados

## 📦 **Instalação**

Adicione ao seu Cargo.toml:
[dependencies]
async-graphql-dataloader = "0.1.0"

Para integração com async-graphql:
[dependencies]
async-graphql-dataloader = { version = "0.1.0", features = ["graphql"] }

🚀 Começo Rápido
use async_graphql_dataloader::{DataLoader, Loader};
use std::collections::HashMap;

struct UserLoader;

#[async_trait::async_trait]
impl Loader<i32> for UserLoader {
    type Value = String;
    type Error = std::convert::Infallible;

    async fn load(&self, keys: &[i32]) -> Result<HashMap<i32, Self::Value>, Self::Error> {
        let mut users = HashMap::new();
        for &key in keys {
            users.insert(key, format!("Usuário {}", key));
        }
        Ok(users)
    }
}

#[tokio::main]
async fn main() {
    let loader = DataLoader::new(UserLoader);
    
    // Carregamento em lote automático - serão combinados em uma chamada
    let user1 = loader.load(1).await;
    let user2 = loader.load(2).await;
    
    println!("Usuário 1: {:?}", user1);
    println!("Usuário 2: {:?}", user2);
}

📚 Funcionalidades
✅ Carregamento em Lote Automático: Múltiplas requisições combinadas em lotes únicos

✅ Cache Inteligente: Cache por requisição com DashMap

✅ Pronto para Async: Construído no runtime async Tokio

✅ Type Safe: Segurança de tipos completa do Rust

✅ Tratamento de Erros: Tratamento de erros configurável

✅ Integração async-graphql: Integração perfeita com async-graphql

🔧 Uso Avançado
Veja o diretório examples para padrões de uso mais avançados:

Uso Básico

Integração Axum + GraphQL

📖 Documentação
Documentação completa da API disponível em docs.rs

🤝 Contribuindo
Contribuições são bem-vindas! Sinta-se à vontade para enviar pull requests ou abrir issues.

📄 Licença
Este projeto está licenciado sob a licença MIT - veja o arquivo LICENSE para detalhes.

Feito com ❤️ e Rust

English Version Available: This README is also available in English.