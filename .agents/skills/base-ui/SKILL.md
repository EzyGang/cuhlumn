---
name: base-ui
description: Reference for using @base-ui/react unstyled components within the frontend. Covers import conventions, Preact compat, Tailwind v4 + OKLCH design token styling, compound component wrappers, and state management rules. Use it when in need of designing or creating a new component for the app or refactoring existing ones
---

# Base UI (Component Library Skill)

The management console in `console/` uses **@base-ui/react@1.9.0** as its unstyled component foundation. The console is **Preact 10** (via `@preact/preset-vite` + `preact/compat` aliases for React compatibility) and **Tailwind CSS v4** with **OKLCH** design tokens.

This skill is the single reference for creating or modifying shared UI components.
Paths below are relative to `console/` unless they start with `console/`.

## 1. Folder Organization & Import Rules

The codebase is split into **two distinct zones**:

| Zone                         | Path                                | Responsibility                                                                                                                                                        | Import rule                                                                               |
| ---------------------------- | ----------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------- |
| **Shared UI primitives**     | `console/src/components/shared/ui/` | The **only** place that imports from `@base-ui/react`. Wraps every unstyled primitive with this app's design tokens, focus rings, sizing, and accessibility defaults. | ✅ `@base-ui/react/*` allowed here ONLY.                                                  |
| **Feature / app components** | `console/src/components/<feature>/` | Composes **shared UI primitives** into domain-specific views.                                                                                                         | ❌ Never import `@base-ui/react/*` directly. Always consume from `components/shared/ui/`. |

### Enforcement

- If a feature needs a new primitive that does not yet have a shared wrapper, **create the wrapper in `shared/ui/` first**, then use it in the feature. Do not import the raw Base UI component inline inside a feature file.
- If an existing feature file imports `@base-ui/react/...`, treat it as a bug and move the import into a new or existing `shared/ui/` wrapper.
- Shared wrappers may be simple (e.g. `Switch.view.tsx`) or compound (e.g. `Dialog.view.tsx`). Either way, the wrapper is the single owner of the `@base-ui/react` dependency.

### Why

This guarantees that design tokens, focus styles, accessibility patterns, and sizing are consistent across every screen. If a designer changes the button hover color, only `shared/ui/Button.view.tsx` needs updating — not every feature file.

## 2. Tech Context

- **Renderer**: Preact 10 (`jsxImportSource: "preact"` in `tsconfig.json` and the Preact Vite preset).
- **React compat**: `react` and `react-dom` resolve to `preact/compat`. See section 2a.
- **State**:
  - Server state → typed request helpers in `src/services/`, called from feature hooks.
  - Global/shared client state → `@preact/signals` (`signal()` in `src/stores/`).
  - Component-local / ephemeral UI state → `useSignal` or Base UI’s built-in state (`open` / `onOpenChange`, etc.).
- **Styling**: Tailwind CSS v4 only; no CSS-in-JS.
- **Class prop**: In Preact JSX, write `class="..."` on native elements. Base UI parts read `className`, so pass `className` to Base UI parts only.

## 2a. Preact compat aliases

`@base-ui/react` imports `react`, `react-dom`, and `react/jsx-runtime`.
These imports must resolve to Preact, or Base UI renders through a separate React runtime and signal updates do not reach its components.
The aliases must match in three places:

- `console/vite.config.ts` `resolve.alias`: `react/jsx-runtime`, `react/jsx-dev-runtime`, `react-dom/test-utils`, `react-dom/client`, `react-dom`, and `react` point to the Preact equivalents.
- `console/tsconfig.json` `paths`: the same module names point to `./node_modules/preact/...`, so TypeScript reads the Preact types.
- `pnpm-workspace.yaml` `overrides`: `react` and `react-dom` map to `npm:@preact/compat`, so pnpm does not install a second React.

Change all three places together.
After a dependency change, run `pnpm why react` from the repository root and confirm that only `@preact/compat` appears.

## 3. Import Conventions

**Only files inside `console/src/components/shared/ui/` are permitted to import from `@base-ui/react`.**

All other code — feature views, containers, pages, hooks — must import the styled wrapper from the shared UI folder.

```tsx
// ✅ SHARED UI (allowed to import Base UI directly)
// In components/shared/ui/Button.view.tsx
import { Button as BaseButton } from "@base-ui/react/button";

// ❌ FEATURE (never do this)
// In components/posts/ui/PostCard.view.tsx
import { Button } from "@base-ui/react/button"; // BANNED

// ✅ FEATURE (correct)
// In components/posts/ui/PostCard.view.tsx
import { Button } from "../../shared/ui/Button.view";
```

Base UI components are imported from `@base-ui/react/<component>` exactly as documented in the Base UI docs. Because of the `preact/compat` alias, React-specific hooks (`useState`, `useEffect`, `useId`) and JSX runtime are automatically satisfied by Preact. No extra shims are needed.

```tsx
// Allowed inside shared/ui/ only: named imports
import { Button } from "@base-ui/react/button";
import { Dialog } from "@base-ui/react/dialog";
import { Input } from "@base-ui/react/input";
```

**Preact-specific rules**:

- Do **not** import from `"react"` directly; if you need hooks, import from `"preact/hooks"` or `"preact/compat"`.
- In JSX, write `class="..."` on native elements and `className="..."` on Base UI parts.
- Base UI compound components (`Dialog.Root`, `Dialog.Trigger`, etc.) work normally through the compat layer.

## 4. Design Tokens → Tailwind Utilities

Tokens are defined in `console/src/style.css` in the `@theme` block. Use the Tailwind utility names below; never hard-code raw OKLCH values in components.

| Semantic usage          | Tailwind utility                        | Maps to token (dark)     |
| ----------------------- | --------------------------------------- | ------------------------ |
| Page background         | `bg-background`                         | `oklch(0% 0 0)`          |
| Surface background      | `bg-surface`                            | `oklch(14% 0 0)`         |
| Card background         | `bg-card`                               | `oklch(20% 0 0)`         |
| Subtle background       | `bg-subtle`                             | `oklch(22% 0 0)`         |
| Raised background       | `bg-raised`                             | `oklch(23% 0 0)`         |
| Primary text            | `text-white`                            | `oklch(100% 0 0)`        |
| Secondary text          | `text-secondary`                        | `oklch(100% 0 0 / 0.8)`  |
| Muted text              | `text-muted`                            | `oklch(65% 0 0)`         |
| Default border          | `border-border`                         | `oklch(100% 0 0 / 0.1)`  |
| Hover background        | `bg-hover`                              | `oklch(100% 0 0 / 0.12)` |
| Hover and focus border  | `border-hover-border`                   | `oklch(100% 0 0 / 0.25)` |
| Success                 | `text-success`, `border-success/20`     | `oklch(82% 0.2 145)`     |
| Emerald                 | `text-emerald`                          | `oklch(78% 0.18 165)`    |
| Error                   | `text-error`, `border-error/50`         | `oklch(55% 0.2 25)`      |
| Warning                 | `text-orange`, `border-orange/30`       | `oklch(65% 0.2 45)`      |

The console is dark only. Add light tokens only when a task requests a theme switcher.

## 5. Wrapper Component Pattern

Every unstyled Base UI component used in the app **must** be wrapped in a styled view inside `console/src/components/shared/ui/ComponentName.view.tsx`. This guarantees a single import for feature code, a single place for design tokens, and consistent overrides.

### File naming

- `src/components/shared/ui/Button.view.tsx`
- `src/components/shared/ui/Input.view.tsx`
- `src/components/shared/ui/Dialog.view.tsx`

### Simple wrapper (single-part component)

Use `forwardRef` from `preact/compat` so the wrapper remains transparent to `ref` consumers:

```tsx
import { forwardRef } from "preact/compat";
import { Button as BaseButton } from "@base-ui/react/button";
import type { ComponentPropsWithoutRef } from "preact/compat";
import type { JSX } from "preact";

interface ButtonProps extends ComponentPropsWithoutRef<"button"> {
  variant?: "primary" | "secondary" | "ghost";
}

const VARIANT_MAP: Record<NonNullable<ButtonProps["variant"]>, string> = {
  primary: "border border-white bg-white text-background px-4 py-3 text-sm uppercase tracking-wide hover:opacity-90",
  secondary:
    "border border-border text-white px-4 py-3 text-sm uppercase tracking-wide hover:border-hover-border hover:bg-hover",
  ghost: "text-secondary hover:text-white hover:bg-hover px-3 py-2 text-sm",
};

export const Button = forwardRef<HTMLButtonElement, ButtonProps>(
  ({ variant = "secondary", class: classProp, ...rest }, ref): JSX.Element => (
    <BaseButton
      ref={ref}
      className={`inline-flex items-center justify-center transition-colors duration-150 ${VARIANT_MAP[variant]} ${classProp ?? ""}`}
      {...rest}
    />
  ),
);
```

**Rules for simple wrappers**:

- Always pass through the caller's `class` prop so callers can override / extend.
- Combine the design token classes (the static look) with the caller-supplied class.
- Keep wrappers thin: only styling, layout, and prop mapping. No logic.

### Compound wrapper (multi-part component)

For compound APIs (Dialog, Toast, Select, etc.), export a single object with sub-components so features consume one import:

```tsx
import { Dialog as BaseDialog } from "@base-ui/react/dialog";
import type { ComponentChildren, JSX } from "preact";

// --- Dialog Root ---

interface DialogProps {
  children: ComponentChildren;
  defaultOpen?: boolean;
  open?: boolean;
  onOpenChange?: (open: boolean) => void;
}

export const Dialog = ({ children, ...props }: DialogProps): JSX.Element => (
  <BaseDialog.Root {...props}>{children}</BaseDialog.Root>
);

// --- Dialog Trigger ---

interface DialogTriggerProps {
  children: ComponentChildren;
  class?: string;
}

Dialog.Trigger = ({ children, class: classProp }: DialogTriggerProps): JSX.Element => (
  <BaseDialog.Trigger className={`button-secondary ${classProp ?? ""}`}>{children}</BaseDialog.Trigger>
);

// --- Dialog Backdrop ---

Dialog.Backdrop = (): JSX.Element => (
  <BaseDialog.Backdrop className="fixed inset-0 bg-background/80 transition-opacity" />
);

// --- Dialog Popup ---

interface DialogPopupProps {
  children: ComponentChildren;
}

Dialog.Popup = ({ children }: DialogPopupProps): JSX.Element => (
  <BaseDialog.Portal>
    <BaseDialog.Popup className="fixed top-1/2 left-1/2 flex w-full max-w-lg -translate-x-1/2 -translate-y-1/2 flex-col gap-4 border border-border bg-card p-5">
      {children}
    </BaseDialog.Popup>
  </BaseDialog.Portal>
);

// --- Dialog Title ---

interface DialogTextProps {
  children: string;
}

Dialog.Title = ({ children }: DialogTextProps): JSX.Element => (
  <BaseDialog.Title className="text-lg font-semibold text-white">{children}</BaseDialog.Title>
);

// --- Dialog Description ---

Dialog.Description = ({ children }: DialogTextProps): JSX.Element => (
  <BaseDialog.Description className="text-sm text-secondary">{children}</BaseDialog.Description>
);

// --- Dialog Close ---

interface DialogCloseProps {
  children: ComponentChildren;
}

Dialog.Close = ({ children }: DialogCloseProps): JSX.Element => (
  <BaseDialog.Close className="button-secondary">{children}</BaseDialog.Close>
);
```

Usage in a feature:

```tsx
import { Dialog } from "../../shared/ui/Dialog.view";
import { Button } from "../../shared/ui/Button.view";

export const DeleteConfirmView = ({ actions }: DeleteConfirmModel): JSX.Element => (
  <Dialog>
    <Dialog.Trigger>Delete</Dialog.Trigger>
    <Dialog.Backdrop />
    <Dialog.Popup>
      <Dialog.Title>Confirm deletion</Dialog.Title>
      <Dialog.Description>This cannot be undone.</Dialog.Description>
      <div class="flex justify-end gap-3">
        <Dialog.Close>Cancel</Dialog.Close>
        <Button variant="primary" onClick={actions.remove}>
          Delete
        </Button>
      </div>
    </Dialog.Popup>
  </Dialog>
);
```

## 7. Component Availability Table

All Base UI components below may be wrapped in `components/shared/ui/` when a feature needs them. Do not import Base UI directly in feature code. Always consume through a wrapper.

| Base UI Component | Wrapper Path                                | Notes                                                                  |
| ----------------- | ------------------------------------------- | ---------------------------------------------------------------------- |
| `Button`          | `components/shared/ui/Button.view.tsx`      | Primary CTA, secondary, Ghost                                          |
| `Input`           | `components/shared/ui/Input.view.tsx`       | Text inputs with project focus/error states                            |
| `Dialog`          | `components/shared/ui/Dialog.view.tsx`      | Compound wrapper (Trigger, Backdrop, Popup, Title, Description, Close) |
| `Toast`           | `components/shared/ui/Toast.view.tsx`       | Compound wrapper (Provider, Toast)                                     |
| `Select`          | `components/shared/ui/Select.view.tsx`      | Compound wrapper (Trigger, Value, Popup, Option)                       |
| `Switch`          | `components/shared/ui/Switch.view.tsx`      | Toggle with project track/thumb tokens                                 |
| `Checkbox`        | `components/shared/ui/Checkbox.view.tsx`    | Check indicator with project tokens                                    |
| `Tooltip`         | `components/shared/ui/Tooltip.view.tsx`     | Compound wrapper (Provider, Trigger, Positioner)                       |
| `Popover`         | `components/shared/ui/Popover.view.tsx`     | Compound wrapper (Trigger, Positioner, Popup, Arrow)                   |
| `Menu`            | `components/shared/ui/Menu.view.tsx`        | Compound wrapper (Trigger, Positioner, Popup, Item)                    |
| `Collapsible`     | `components/shared/ui/Collapsible.view.tsx` | Expand/collapse sections                                               |
| `Tabs`            | `components/shared/ui/Tabs.view.tsx`        | Compound wrapper (List, Tab, Panel)                                    |
| `Field`           | `components/shared/ui/Field.view.tsx`       | Wrapper for label + error text pairing                                 |
| `Form`            | `components/shared/ui/Form.view.tsx`        | Validation integration wrapper                                         |
| `Separator`       | `components/shared/ui/Separator.view.tsx`   | Minimal divider line                                                   |

If a needed component is not yet wrapped, create it in `components/shared/ui/` following the patterns above. Keep wrapper files under 150 lines; split sub-parts into separate files if the compound API is large.

## 8. State Management Rules

| State type                                      | Tool                                            | When to use                                     |
| ----------------------------------------------- | ----------------------------------------------- | ----------------------------------------------- |
| **Open / closed** (Dialog, Popover Toast, Menu) | Base UI `open` / `defaultOpen` + `onOpenChange` | Local ephemeral UI state                        |
| **Form field values**                           | Base UI `Field` / `Form` or `useSignal`         | Local, unless shared across components          |
| **Shared selection / filter**                   | `@preact/signals` store                         | Cross-component or persisted                    |
| **Server data**                                 | Feature hook + `src/services/` helper           | Load in the hook that owns the screen           |
| **Session / toasts**                            | `@preact/signals` global store                  | Singletons                                      |

Guidelines:

- Use Base UI’s built-in state APIs (`Dialog.Root open={...} onOpenChange={...}`) for open/closed, focus, and selection state that lives inside the component tree.
- Use `@preact/signals` only when state is shared between distant components or must survive unmounting.
- Keep one owner for each piece of server data. Do not copy it into a second store.

## 9. Styling Details

- **No border-radius**: unless the Base UI primitive explicitly requires it for focus rings, use `rounded-none` (0px) by default per the design system. Cards may use at most 0.25rem.
- **Borders**: use `border border-border` for cards/panels.
- **Hover**: `hover:bg-hover` or `hover:border-hover-border`; do not invent new colors.
- **Focus rings**: the global `:focus-visible` outline in `src/style.css` is the standard focus indicator. Do not remove it.
- **Transitions**: prefer `transition-colors duration-150` for interactive elements.
- **Flexbox gap**: always use `flex` with `gap-*`; never use `space-y-*` or margins on children for spacing.
- **State attributes**: style Base UI states with data attributes, for example `data-[checked]:bg-white`.

## 10. Examples

### Switch

See `console/src/components/shared/ui/Switch.view.tsx`.

### Input

```tsx
import { forwardRef } from "preact/compat";
import { Input as BaseInput } from "@base-ui/react/input";
import type { ComponentPropsWithoutRef } from "preact/compat";
import type { JSX } from "preact";

interface InputProps extends ComponentPropsWithoutRef<"input"> {
  invalid?: boolean;
}

export const Input = forwardRef<HTMLInputElement, InputProps>(
  ({ invalid, class: classProp, ...rest }, ref): JSX.Element => (
    <BaseInput
      ref={ref}
      className={`${invalid ? "border-error" : "border-border"} ${classProp ?? ""}`}
      {...rest}
    />
  ),
);
```

The global input rules in `src/style.css` give the surface background, padding, and focus border.

### Dialog

See the compound wrapper example in section 5.

## 11. Migration Policy

- **Do not** migrate existing feature components to Base UI wrappers unless explicitly tasked.
- New shared UI components **must** be built as Base UI wrappers.
- Existing one-off components can remain as-is until a refactor story is created.

## 12. Further Reading

- Base UI docs: <https://base-ui.com/react>
- Preact signals: <https://preactjs.com/guide/v10/signals/>
- Tailwind CSS v4: <https://tailwindcss.com/docs/styling-with-utility-classes>
