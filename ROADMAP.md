# 🗺️ rust-socialite Roadmap

Bem-vindo ao roadmap oficial do `rust-socialite`! A biblioteca já suporta 33 provedores, suporte a tokens dinâmicos, extração via `get_user_from_token`, proteção CSRF (via `state`) e customização de `scopes`. 

Para o nosso caminho rumo à versão **`v1.0.0`** (Nível Enterprise), planejamos as seguintes adições maiores que já estão ativamente sendo trabalhadas:

## ⏳ Em Progresso (Fase 3 & 4)

- [x] **Macros de Redução de Boilerplate:** Usar `define_provider!` para cortar drasticamente o código repetitivo nas structs de provedores, facilitando a contribuição de novos provedores pela comunidade.
- [ ] **Integração Nativa com Frameworks:** Criar as features opcionais `axum` e `actix`, provendo Extractors (como `AuthCallback`) para que a extração de códigos, estados e erros da URL seja 100% mágica.
- [ ] **Revogação de Tokens (Logout):** Adicionar método `revoke_token` à trait para permitir logout direto nos servidores de provedores suportados.
- [ ] **Ferramentas de Mocking (TDD):** Um `MockProvider` para facilitar a escrita de testes unitários pelos usuários finais da biblioteca.
- [ ] **Suporte OIDC:** Validação automática e ultra-rápida de `id_token` do Google e da Apple usando JWT (sem calls HTTP extras).

## 🔮 Futuro Distante

- **Agnóstico de Cliente HTTP:** Suporte genérico a clientes (via trait `HttpClient`) permitindo o uso de `surf`, `reqwest-middleware` ou outros, ao invés de forçar o `reqwest`.
- **Suporte Universal a PKCE:** Trazer o padrão PKCE (Proof Key for Code Exchange) nativamente para todos os provedores via método `.with_pkce()`, focando em máxima segurança para SPA/Mobile.
- **Integração com Bancos de Dados (SQLx/Diesel):** Traits auxiliares (ex: `IntoDatabaseUser`) para facilitar o salvamento direto do usuário no banco de dados.
- **Suporte a Proxy HTTP:** Permitir a configuração de proxies corporativos para ambientes bloqueados.
- **Módulo de Refrescos:** Suporte a `refresh_token` automatizado caso o token principal tenha expirado (lidando automaticamente com os endpoints `/token` em background).
- **Padronização Universal de Avatar:** Fazer parsing avançado para garantir resoluções ótimas nas fotos de perfis retornadas.
- **Integração com Leptos e Dioxus:** Extractors para frameworks Fullstack / WebAssembly Rust.

---

Quer ajudar a implementar alguma dessas features? Fique à vontade para abrir uma PR!
