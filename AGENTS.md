# Rust AI Agents Protocol: Enterprise Clean Architecture

This document defines the mandatory standards for any AI agent interacting with this codebase. Compliance is required for every code generation, refactor, or documentation task.

---

## 1. Architectural Integrity (Clean Architecture)

The project follows a strict **Layered Clean Architecture**. Agents must respect these boundaries:

### Layer Definitions
| Layer | Responsibility | Contents |
| :--- | :--- | :--- |
| **Domain** | Pure business logic and entities. | Structs, Enums, Traits (Interfaces). **Strictly no external IO or framework dependencies.** |
| **Application** | Use cases and orchestration. | Application services and ports. Coordinates domain entities to fulfill business requirements. |
| **Infrastructure** | External implementations. | DB adapters (SQLx/Diesel), Web servers (Axum), API clients, File System. |



### The Dependency Rule
* **Inward Only:** Dependencies must only point towards the Domain. Infrastructure depends on Application; Application depends on Domain.
* **Abstraction:** Use **Traits** to decouple logic. The Domain/Application layers must never know about specific database or framework implementations.

---

## 2. Idiomatic Rust & Clean Code Standards

### Memory & Ownership
* **Borrowing First:** Prefer `&T` or `&mut T` over `.clone()` or moving ownership unless strictly required by lifetimes or threads.
* **Explicit Lifetimes:** Only use explicit lifetimes when the compiler cannot elide them. Prefer readable, descriptive lifetime names if necessary.
* **Smart Pointers:** Use `Arc<T>` or `Box<T>` only when shared ownership or heap allocation is architecturally justified.

### Error Handling Protocol
* **Zero Panics:** The use of `.unwrap()`, `.expect()`, or `panic!` is strictly prohibited in production-ready code.
* **Result-Oriented:** Always return `Result<T, E>` or `Option<T>`.
* **Custom Errors:** Implement custom error types for each layer using the `thiserror` crate or standard `enum` patterns.

### Naming Conventions (RFC 430)
* **Variables/Modules/Functions:** `snake_case`.
* **Structs/Enums/Traits:** `PascalCase`.
* **Constants:** `SCREAMING_SNAKE_CASE`.

---

## 3. Mandatory Documentation & Style

### Rustdoc Standards
* **Public Items:** Every `pub` function, struct, trait, or enum MUST have a `///` doc comment.
* **Doc Examples:** Utility functions and Public APIs must include a `/// # Examples` section with valid, runnable code.
* **Safety Blocks:** Every `unsafe` block must be preceded by a `// SAFETY: <reason>` comment explaining why the invariants are upheld.

### Code Style
* **Internal Logic:** Use `//` comments for complex algorithm steps. 
* **ASCII Only:** Do not use icons, emojis, or decorative symbols in comments or code.
* **Formatting:** All output must be compatible with `cargo fmt`.

---

## 4. Componentization & Modularity

* **Single Responsibility:** Each module/file must handle exactly one concern.
* **File Size Limit:** If a file exceeds 300 lines of code, it MUST be broken down into sub-modules.
* **Visibility Control:** Use `pub(crate)` by default. Only use `pub` for items that are truly part of the public interface.
* **Feature Flags:** Large components should be guarded by feature flags in `Cargo.toml` if they are optional.

---

## 5. Agent Operational Workflow

When an agent receives a task, it must:

1.  **Analyze Layer:** Determine if the change belongs to Domain, Application, or Infrastructure.
2.  **Define Interface:** Create or update Traits before implementing logic.
3.  **Draft Implementation:** Write clean, idiomatic Rust.
4.  **Self-Document:** Generate all required `rustdoc` and internal comments.
5.  **Verify Standards:** Ensure no icons are used and naming follows RFC 430.

---

## 6. Prohibited Practices
- No Magic Numbers (use constants or config structs).
- No deep nesting (use guard clauses and early returns).
- No ignoring compiler or Clippy warnings.
- No global mutable state.