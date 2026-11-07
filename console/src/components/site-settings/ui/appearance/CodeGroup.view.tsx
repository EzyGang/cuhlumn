import type { JSX } from "preact";
import { RadioCards } from "../../../shared/ui/RadioCards.view";
import { useSiteSettingsContext } from "../../context/SiteSettingsContext";
import { AppearanceGroup } from "./AppearanceGroup.view";

export const CodeGroup = (): JSX.Element => {
  const { data, actions } = useSiteSettingsContext();
  return (
    <AppearanceGroup title="Code">
      <RadioCards
        label="Code colours"
        value={data.form.code_theme}
        items={data.options.code_theme}
        onValueChange={actions.choose.code_theme}
      />
    </AppearanceGroup>
  );
};
