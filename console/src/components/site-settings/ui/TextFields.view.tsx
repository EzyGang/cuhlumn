import type { JSX } from "preact";
import { Field } from "../../shared/ui/Field.view";
import { Switch } from "../../shared/ui/Switch.view";
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
        description="Text after posts, above the project button."
        value={data.form.cta}
        onValueChange={actions.text.cta}
        error={data.errors["cta"]}
        multiline={2}
      />
      <Field
        label="Show the call to action on posts by default"
        description="Each post can choose for itself in the editor."
      >
        <Switch checked={data.form.show_cta} onCheckedChange={actions.toggle.show_cta} />
      </Field>
    </>
  );
};
