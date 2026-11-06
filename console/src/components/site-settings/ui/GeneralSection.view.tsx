import type { JSX } from "preact";
import { Section } from "../../shared/ui/Section.view";
import { TextField } from "../../shared/ui/TextField.view";
import { useSiteSettingsContext } from "../context/SiteSettingsContext";
import { AuthorFields } from "./AuthorFields.view";
import { LocaleFields } from "./LocaleFields.view";
import { TextFields } from "./TextFields.view";

export const GeneralSection = (): JSX.Element => {
  const { data, actions } = useSiteSettingsContext();
  return (
    <Section title="General" description="The name and text that the blog shows to readers.">
      <TextField
        label="Title"
        value={data.form.title}
        onValueChange={actions.text.title}
        error={data.errors["title"]}
      />
      <TextFields />
      <AuthorFields />
      <LocaleFields />
    </Section>
  );
};
