import type { JSX } from "preact";
import { Field } from "../../../shared/ui/Field.view";
import { RadioCards } from "../../../shared/ui/RadioCards.view";
import { Switch } from "../../../shared/ui/Switch.view";
import { useSiteSettingsContext } from "../../context/SiteSettingsContext";
import { AccentInputs } from "./AccentInputs.view";
import { AppearanceGroup } from "./AppearanceGroup.view";

export const ColourGroup = (): JSX.Element => {
  const { data, actions } = useSiteSettingsContext();
  return (
    <AppearanceGroup title="Colours">
      <AccentInputs />
      <RadioCards
        label="Palette"
        value={data.form.palette}
        items={data.options.palette}
        onValueChange={actions.choose.palette}
      />
      <RadioCards
        label="Theme"
        value={data.form.theme}
        items={data.options.theme}
        onValueChange={actions.choose.theme}
        columns={3}
      />
      <Field label="Theme button" description="Lets readers switch between light and dark.">
        <Switch checked={data.form.theme_toggle} onCheckedChange={actions.toggle.theme_toggle} />
      </Field>
    </AppearanceGroup>
  );
};
