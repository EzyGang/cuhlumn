import type { JSX } from "preact";
import { TriangleAlert } from "lucide-preact";
import { Button } from "../../shared/ui/Button.view";
import type { SetupModel } from "../../../types/auth";
import { CAT_ART } from "../../auth/catArt";
import { TokenField } from "./TokenField.view";

export const SetupView = ({ data, status, actions }: SetupModel): JSX.Element => (
  <main class="flex min-h-dvh flex-col items-center justify-center gap-6 px-5 py-16">
    <pre
      aria-hidden="true"
      class="select-none font-mono text-[7px] leading-[1.15] text-secondary sm:text-[8px]"
    >
      {CAT_ART}
    </pre>
    <form
      class="flex w-full max-w-md flex-col gap-6 border border-border bg-surface p-6"
      onSubmit={actions.submit}
    >
      <header class="flex flex-col gap-2">
        <p class="eyebrow">Cuhlumn</p>
        <h1 class="text-2xl font-semibold tracking-tight">Set up the server</h1>
        <p class="text-sm text-secondary">
          Choose the admin token of this server. The console, the CLI, and agents use it to sign in.
        </p>
      </header>
      <TokenField draft={data.token} label="Admin token" error={data.error ?? undefined} />
      <p
        class="flex gap-3 border border-orange/40 bg-orange/10 p-3 text-sm text-secondary"
        role="note"
      >
        <TriangleAlert size={16} aria-hidden="true" class="mt-0.5 shrink-0 text-orange" />
        Store the token in a safe place now. The server keeps only a hash of the token and cannot
        show it again.
      </p>
      <Button
        type="submit"
        variant="primary"
        disabled={status.submitting || !data.token.data.longEnough}
      >
        {status.submitting ? "Saving the token" : "Save the token and log in"}
      </Button>
    </form>
  </main>
);
