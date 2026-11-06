import type { JSX } from "preact";
import { ChipInput } from "../../shared/ui/ChipInput.view";
import { Section } from "../../shared/ui/Section.view";
import { TextField } from "../../shared/ui/TextField.view";
import { useSiteSettingsContext } from "../context/SiteSettingsContext";
import { IndexNowField } from "./IndexNowField.view";

export const DomainSection = (): JSX.Element => {
  const { data, actions } = useSiteSettingsContext();
  return (
    <Section
      title="Domains"
      description="The canonical domain and the domains that redirect to it."
    >
      <TextField
        label="Host"
        description={
          data.mode === "create" ? "For example blog.example.com." : "The host cannot change."
        }
        value={data.form.host}
        onValueChange={actions.text.host}
        error={data.errors["host"]}
        disabled={data.mode === "edit"}
        placeholder="blog.example.com"
      />
      <ChipInput
        label="Aliases"
        model={data.aliases}
        error={data.aliasError}
        placeholder="Type a domain and press Enter"
      />
      <IndexNowField />
    </Section>
  );
};
