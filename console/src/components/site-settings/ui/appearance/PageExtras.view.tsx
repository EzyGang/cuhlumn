import type { JSX } from "preact";
import { Field } from "../../../shared/ui/Field.view";
import { Switch } from "../../../shared/ui/Switch.view";
import { TextField } from "../../../shared/ui/TextField.view";
import { useSiteSettingsContext } from "../../context/SiteSettingsContext";

export const PageExtras = (): JSX.Element => {
  const { data, actions } = useSiteSettingsContext();
  return (
    <>
      <TextField
        label="Head HTML"
        description="Raw HTML for the page head, such as analytics tags."
        value={data.form.head}
        onValueChange={actions.text.head}
        error={data.errors["brand.head"]}
        multiline={4}
        mono
      />
      <TextField
        label="Footer"
        value={data.form.footer}
        onValueChange={actions.text.footer}
        error={data.errors["brand.footer"]}
        multiline={2}
      />
      <Field label="Credit" description="Show the “Published with Cuhlumn” line in the footer.">
        <Switch checked={data.form.credit} onCheckedChange={actions.setCredit} />
      </Field>
    </>
  );
};
