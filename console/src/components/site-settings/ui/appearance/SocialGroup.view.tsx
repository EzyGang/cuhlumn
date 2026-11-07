import type { JSX } from "preact";
import { Field } from "../../../shared/ui/Field.view";
import { Switch } from "../../../shared/ui/Switch.view";
import { useSiteSettingsContext } from "../../context/SiteSettingsContext";
import { AppearanceGroup } from "./AppearanceGroup.view";
import { SocialEditor } from "./SocialEditor.view";

export const SocialGroup = (): JSX.Element => {
  const { data, actions } = useSiteSettingsContext();
  return (
    <AppearanceGroup title="Social">
      <SocialEditor />
      <Field label="Show in header" description="The footer always shows the profile icons.">
        <Switch
          checked={data.form.social_in_header}
          onCheckedChange={actions.toggle.social_in_header}
        />
      </Field>
    </AppearanceGroup>
  );
};
