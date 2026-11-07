import type { JSX } from "preact";
import { Button } from "../../shared/ui/Button.view";
import { ExternalLink } from "../../shared/ui/ExternalLink.view";
import { Field } from "../../shared/ui/Field.view";
import { Section } from "../../shared/ui/Section.view";
import { Select } from "../../shared/ui/Select.view";
import { TextField } from "../../shared/ui/TextField.view";
import type { AddressModel } from "../../../types/serverSettings";

const HOST_HINT = "Leave empty to serve the console on every host that is not a site.";

export const AddressSection = ({ data, status, actions }: AddressModel): JSX.Element => (
  <Section title="Addresses" description="The console host and the scheme of public addresses.">
    <form class="flex flex-col gap-5" onSubmit={actions.save}>
      <TextField
        label="Console host"
        value={data.host}
        onValueChange={actions.setHost}
        placeholder="console.harbor.example.com"
        description={data.hostLock ?? HOST_HINT}
        error={data.hostError}
        disabled={data.hostLock !== null}
        mono
      />
      {data.schemeLock === null ? (
        <Select
          label="Scheme"
          value={data.scheme}
          items={data.schemes}
          onValueChange={actions.setScheme}
          class="w-48"
        />
      ) : (
        <Field label="Scheme" description={data.schemeLock} class="w-48">
          <p class="border border-border px-3 py-2.5 text-sm text-muted uppercase">{data.scheme}</p>
        </Field>
      )}
      {data.movedTo === null ? null : (
        <div class="flex flex-col gap-1 border border-border bg-subtle p-3" role="status">
          <p class="text-sm text-secondary">
            The console moved. The session stays on this host, so log in again at the new address.
          </p>
          <ExternalLink href={data.movedTo}>{data.movedTo}</ExternalLink>
        </div>
      )}
      <div>
        <Button type="submit" variant="primary" disabled={status.saving || !status.dirty}>
          {status.saving ? "Saving" : "Save"}
        </Button>
      </div>
    </form>
  </Section>
);
