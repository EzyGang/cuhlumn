import type { JSX } from "preact";
import { KeyRound } from "lucide-preact";
import { ConfirmDialog } from "../../shared/ui/ConfirmDialog.view";
import { Section } from "../../shared/ui/Section.view";
import type { RotationModel } from "../../../types/serverSettings";
import { TokenField } from "../../setup/ui/TokenField.view";

const DESCRIPTION =
  "A new token ends all sessions. This browser gets a new session. The CLI and agents need the new token.";

export const TokenSection = ({ data, status, actions }: RotationModel): JSX.Element => (
  <Section title="Admin token" description="The server keeps only a hash of the token.">
    {data.lock === null ? null : <p class="text-sm text-muted">{data.lock}</p>}
    <TokenField draft={data.token} label="New admin token" disabled={data.lock !== null} />
    <div>
      <ConfirmDialog
        trigger={
          <>
            <KeyRound size={14} aria-hidden="true" />
            {status.rotating ? "Changing the token" : "Change the token"}
          </>
        }
        title="Change the admin token?"
        description={DESCRIPTION}
        confirmLabel="End all sessions"
        onConfirm={actions.rotate}
        disabled={data.lock !== null || status.rotating || !data.token.data.longEnough}
      />
    </div>
  </Section>
);
