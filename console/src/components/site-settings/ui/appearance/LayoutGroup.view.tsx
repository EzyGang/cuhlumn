import type { JSX } from "preact";
import { RadioCards } from "../../../shared/ui/RadioCards.view";
import { useSiteSettingsContext } from "../../context/SiteSettingsContext";
import { AppearanceGroup } from "./AppearanceGroup.view";
import { FontField } from "./FontField.view";
import { ImageFields } from "./ImageFields.view";
import { LinksEditor } from "./LinksEditor.view";

export const LayoutGroup = (): JSX.Element => {
  const { data, actions } = useSiteSettingsContext();
  return (
    <AppearanceGroup title="Layout">
      <RadioCards
        label="Width"
        value={data.form.width}
        items={data.options.width}
        onValueChange={actions.choose.width}
        columns={3}
      />
      <RadioCards
        label="Post list"
        value={data.form.home_layout}
        items={data.options.home_layout}
        onValueChange={actions.choose.home_layout}
        columns={3}
      />
      <FontField />
      <ImageFields />
      <LinksEditor />
    </AppearanceGroup>
  );
};
