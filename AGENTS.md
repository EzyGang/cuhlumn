## Rust

Target stable Rust, edition 2024. `rustfmt` and Clippy define mechanical style.

### Correctness

- No `unsafe`
- No production `unwrap()`, `expect()`, `panic!()`, `todo!()`, or `unimplemented!()`
- Return narrow structured errors for expected domain failures
- Use `thiserror` for reusable domain errors
- Use `anyhow` only at the CLI boundary
- Return errors or add actionable boundary context; never swallow them
- Use full-path `tracing` macros for runtime diagnostics
- Do not suppress Clippy warnings without a targeted, documented external limitation

### Types and ownership

- Make invalid states unrepresentable with enums, newtypes, and validated constructors
- Return typed structs and enums instead of untyped collections
- Accept `&str` and `&[T]`, not `&String` and `&Vec<T>`
- Avoid `clone()` unless a distinct owner is required
- Convert `&str` with `.to_owned()`
- Put behavior on its owning type
- Put trait bounds in `where` clauses
- Use conventional generic names such as `S`, `Fut`, and `F`
- Add generic conversions only when callers need multiple input types
- Derive standard value traits when semantically valid
- Derive `Default` only for a genuine default
- Mark side-effect-free result functions `#[must_use]` when ignoring them is usually a bug
- Prefer `Self` inside implementations
- Do not add one-consumer abstractions unless they name an invariant or isolate an external boundary

### Control flow and imports

- Prefer guard clauses and early returns
- Use `match` when multiple variants have behavior
- Use `=> (),` for intentionally empty match arms
- Do not use glob imports or add `pub use` re-exports
- Declare modules first
- Group imports as `std`, external crates, `crate`, then local modules
- Use one `use` declaration per crate and blank lines between groups
- Prefer `crate::...` when it identifies the defining path clearly

### Files and modules

- Order files as public types, public functions, private types, implementations, private helpers
- Keep production code beside its owning domain
- Keep domain roots focused on declarations, primary public types, and the smallest cohesive boundary
- Group three or more files with one responsibility into a semantic submodule
- Remove redundant prefixes inside named submodules: `cache/entry.rs`, not `cache/cache_entry.rs`
- Do not create one-file directories, empty modules, or speculative nesting
- Keep production Rust files at or below 300 lines
- Generated files and declarative Tree-sitter queries are exempt
- Make module moves as clean cutovers; update all declarations and imports

# Output

Use ASD-STE100 Issue 9 as the writing guide.

- Use one term for one meaning.
- Use active voice unless the actor is unknown.
- Limit descriptive sentences to 25 words.
- Limit procedural sentences to 20 words and one instruction.
- Write each complete sentence on one Markdown line. Do not wrap a sentence manually.
- Use a vertical list when one sentence contains many items or actions.
- Give information in a general-to-specific order.
- Do not use contractions or semicolons.
- Use no more than six sentences in one paragraph.
- Write for readers who do not know this conversation.
- Do not include intermediate steps or breadcrumbs.
- Present final code, comments, and output without development history.
