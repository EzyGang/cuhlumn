## Project

Cuhlumn is a blogging platform that serves many blogs, each on its own domain, from one server.
It renders every page as server-side HTML with structured data, so search engines and AI assistants can find, read, and cite it.
Writers, scripts, and AI agents publish Markdown posts through an authenticated HTTP API, the `cuhlumn` CLI, or the web management console.
The server is a single Rust binary that keeps its data in one SQLite file.

The repository has these parts:

- `crates/api`: the `cuhlumn-api` library with the shared wire types that the server, the CLI, and the console types use
- `crates/server`: the `cuhlumn-server` binary that serves the blogs and the HTTP API
- `crates/cli`: the `cuhlumn` binary that agents use to call the HTTP API
- `console/`: the management console, a single-page app that the server embeds
- `skills/cuhlumn/SKILL.md`: the agent skill that explains how to use the `cuhlumn` CLI

Update `skills/cuhlumn/SKILL.md` when you change the behavior of the CLI.

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

## Frontend

The management console is in `console/`.
Paths in this section are relative to `console/`.

### Stack

- Language: TypeScript
- Build tool: Vite
- UI framework: Preact
- React compatibility: `preact/compat`
- State: `@preact/signals`
- Components: `@base-ui/react`
- Routing: `wouter-preact`
- Icons: `lucide-preact`
- Styling: Tailwind CSS v4
- Linter: Oxlint
- Formatter: Oxfmt
- Tests: Vitest
- Package manager: pnpm
- Workspace definition: root `pnpm-workspace.yaml`
- Dependency lockfile: root `pnpm-lock.yaml`

Do not add other package managers or lockfiles.

### Commands

Run these commands from the repository root:

```sh
pnpm install
pnpm dev                 # console at http://localhost:5173, /api goes to http://127.0.0.1:8080
pnpm build               # type-check, then production build in console/dist
pnpm preview
pnpm lint                # Oxlint
pnpm type-check          # TypeScript
pnpm validate            # lint and type-check
pnpm format              # Oxfmt
pnpm fix                 # Oxlint fixes, Oxfmt, and type-check
pnpm test                # Vitest
```

The development server keeps the `Host` header, so the server must run with `CUHLUMN_CONSOLE_HOST=localhost`.
Set `API_PROXY_TARGET` to send `/api` to a different server address.

### Architecture and style

- Use strict TypeScript without implicit `any`.
- Do not use `any` unless no precise type is possible.
- Use type-only imports where appropriate.
- Use arrow definitions, such as `export const Foo = (...) => ...`.
- Do not use function declarations.
- Export at the declaration.
- Do not put export lists at the end of files.
- Do not add barrel exports or `index.ts` re-export files.
- Give all function parameters and return values a type.
- Do not use `ReturnType<>`. Use the type name.
- Do not use inline imports such as `await import()` or `import("pkg").Type`.
- Use the Preact `class` attribute.
- Use `className` only when a third-party component requires that prop.
- Use `for...of`, not `forEach`.
- Import native components from `preact` and native hooks from `preact/hooks`.
- Use Tailwind v4 utilities for styling.
- Do not use CSS-in-JS.
- Keep functions at or below 30 lines.
- Keep containers at or below 20 lines.
- Keep source files at or below 200 lines and presentational views at or below 300 lines.
- Keep hook-order lint checks enabled.

Oxlint disables `react/immutability` because Preact signals require mutation.

### React aliases

`react`, `react-dom`, and their JSX and DOM entry points must resolve to Preact.
Without the aliases, `@base-ui/react` renders through a separate React runtime, and signal updates do not reach its components.
Keep the aliases equal in these three places:

- `vite.config.ts` `resolve.alias`
- `tsconfig.json` `paths`
- root `pnpm-workspace.yaml` `overrides`

After a dependency change, run `pnpm why react` and make sure that only `@preact/compat` appears.

### Base UI

- Import `@base-ui/react` only in `src/components/shared/ui/`.
- Wrap each Base UI component in a styled view in that folder before a feature uses it.
- Read the `base-ui` skill in `.agents/skills/base-ui/SKILL.md` before you add or change a wrapper.

### Feature structure

Each feature has three layers:

- `src/components/<feature>/hooks/useFeature.hook.ts`: signals, data loading, side effects, and bound callbacks
- `src/components/<feature>/ui/Feature.view.tsx`: presentational styles and layout
- `src/components/<feature>/containers/Feature.container.tsx`: hook-to-view prop forwarding

- Keep views free of side effects.
- Define handlers in hooks, not inline in views.
- Keep business logic, helper functions, and local variables out of views.
- Use containers only to forward props.
- Render one container from each page in `src/pages/`.
- Add routing or error fallbacks to pages only when necessary.
- Group hook output and view props as `{ data, status, actions }`.
- Keep the container-to-view prop count at or below 8.
- Use a context when the prop count is more than that limit.
- Name context files `context/FooContext.tsx`.
- Export `FooProvider`, `useFooContext()`, and `FooContextValue` from the context file.
- Return the context value directly from the feature hook.
- Give views that use a context zero props, and get their data from the context hook.
- Group a growing feature into semantic subfolders in each layer, for example `ui/form/` and `hooks/form/`.
- Do not create unused directories.

Shared directories:

- `src/hooks/`: shared hooks
- `src/stores/`: global signal stores
- `src/services/`: API request helpers
- `src/utils/`: shared utilities
- `src/types/`: TypeScript types
- `src/routes/`: route definitions

### State and API boundaries

- Use `signal()` for global state.
- Use `useSignal()` for component state.
- Do not use `useState`.
- Write stores as plain objects that contain typed signals.
- Do not write stores with classes, getters, or setters.
- Update signals with `store.field.value = value`.
- Keep service calls, effects, and callbacks in hooks.
- Export pure typed async request helpers from service modules.
- Keep one owner for each piece of server data.
- Read the server routes in `crates/server/src/api/` before you integrate an endpoint.

The API contracts come from the generated files in `src/types/api/`.
The `cuhlumn-api` crate generates these files with `ts-rs`.

- Import the generated types with `import type`.
- Do not edit the generated files.
- Do not write a second type for a wire type that the generated files contain.
- Run `cargo test -p cuhlumn-api --features ts` to generate the files again after a change to the wire types.

### Visual conventions

The console uses a dark design with black backgrounds, charcoal surfaces, white text, and faint borders.

- Do not use heavy shadows or gradients.
- Define theme tokens in the `@theme` block of `src/style.css`.
- Use OKLCH for theme tokens.
- Do not use hex or RGBA theme tokens.

Surface tokens:

- Page: `oklch(0% 0 0)`
- Surface: `oklch(14% 0 0)`
- Card: `oklch(20% 0 0)`
- Subtle: `oklch(22% 0 0)`
- Raised: `oklch(23% 0 0)`

Text tokens:

- Primary: `oklch(100% 0 0)`
- Secondary: `oklch(100% 0 0 / 0.8)`
- Muted: `oklch(65% 0 0)`

Border and hover tokens:

- Border: `oklch(100% 0 0 / 0.1)`
- Hover background: `oklch(100% 0 0 / 0.12)`
- Hover border: `oklch(100% 0 0 / 0.25)`

Accent tokens:

- Success: `oklch(82% 0.2 145)`
- Emerald: `oklch(78% 0.18 165)`
- Error: `oklch(55% 0.2 25)`
- Orange: `oklch(65% 0.2 45)`

Typography and layout:

- Use Inter for UI text and JetBrains Mono for code, with system font fallbacks.
- Do not load fonts from other hosts, because the console Content Security Policy blocks them.
- Use 16px body text with a line height of 1.5.
- Use uppercase text only for small labels, navigation, and calls to action.
- Use 0.05em letter spacing for uppercase text.
- Use flexbox with `gap` for sibling spacing, and set the spacing on the parent.
- Do not use `space-y-*` or child margins for sibling spacing.
- Use approximately `p-5` card padding.
- Keep navigation compact.
- Keep the keyboard focus visible.

Components:

- Buttons: square, approximately `px-4 py-3`, small uppercase text
- Primary buttons: white background, black text, opacity change on hover
- Secondary buttons: transparent background, faint border, white text, light fill on hover
- Cards: borders, no shadows, a radius of 0.25rem or less
- Inputs: surface background, faint border, lighter border on focus
- Links: white text, underline and opacity change on hover
- Badges: small uppercase text, dark background, faint border

### Verification

- Run Oxlint for console changes.
- Run the TypeScript check for console changes.
- Build the console for production.
- Examine changed UI in a browser.
- Use Vitest for permanent console tests.
- Put console tests in `src/tests/`.
- Test observable behavior, edge cases, and failure paths.
- Do not write tautological tests or tests that depend on the implementation.
- Keep browser artifacts in `.playwright-cli/` at the repository root.
- Do not commit build output, local databases, or secrets.

## CI

The `CI` workflow in `.github/workflows/ci.yml` runs these commands from the repository root.
Run them before you push.

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p cuhlumn-api --features ts --test ts_export --locked   # then commit the changes in console/src/types/api
cargo test --workspace --all-features --locked
pnpm install --frozen-lockfile
pnpm validate
pnpm format:check
pnpm test
(cd docs && uv sync --locked --group dev && uv run zensical build)
```

The `Release` workflow in `.github/workflows/release.yml` publishes a version.
Start it manually on `main` with a semver version without the `v` prefix.

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
