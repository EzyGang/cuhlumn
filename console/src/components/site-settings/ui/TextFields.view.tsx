import type { JSX } from "preact";
import { TextField } from "../../shared/ui/TextField.view";
import { useSiteSettingsContext } from "../context/SiteSettingsContext";

export const TextFields = (): JSX.Element => {
  const { data, actions } = useSiteSettingsContext();
  return (
    <>
      <TextField
        label="Description"
        value={data.form.description}
        onValueChange={actions.text.description}
        error={data.errors["description"]}
        multiline={3}
      />
      <TextField
        label="Call to action"
        description="Text after each post."
        value={data.form.cta}
        onValueChange={actions.text.cta}
        error={data.errors["cta"]}
        multiline={2}
      />
    </>
  );
};
