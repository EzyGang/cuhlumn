import type { JSX } from "preact";
import { Button } from "../../shared/ui/Button.view";
import { Field } from "../../shared/ui/Field.view";
import { Input } from "../../shared/ui/Input.view";
import type { LoginModel } from "../../../types/auth";
import { CAT_ART } from "../catArt";

export const LoginView = ({ data, status, actions }: LoginModel): JSX.Element => (
  <main class="flex min-h-dvh flex-col items-center justify-center gap-6 px-5 py-16">
    <pre
      aria-hidden="true"
      class="select-none font-mono text-[7px] leading-[1.15] text-secondary sm:text-[8px]"
    >
      {CAT_ART}
    </pre>
    <form
      class="flex w-full max-w-sm flex-col gap-6 border border-border bg-surface p-6"
      onSubmit={actions.submit}
    >
      <header class="flex flex-col gap-2">
        <p class="eyebrow">Cuhlumn</p>
        <h1 class="text-2xl font-semibold tracking-tight">Management console</h1>
        <p class="text-sm text-secondary">Enter the admin token of this server.</p>
      </header>
      <Field label="Admin token" error={data.error ?? undefined}>
        <Input
          type="password"
          value={data.token}
          onValueChange={actions.setToken}
          autoComplete="current-password"
          required
        />
      </Field>
      <Button type="submit" variant="primary" disabled={status.submitting || data.token === ""}>
        {status.submitting ? "Signing in" : "Log in"}
      </Button>
    </form>
  </main>
);
