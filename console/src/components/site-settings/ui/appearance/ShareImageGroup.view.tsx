import type { JSX } from "preact";
import { Field } from "../../../shared/ui/Field.view";
import { RadioCards } from "../../../shared/ui/RadioCards.view";
import { Switch } from "../../../shared/ui/Switch.view";
import { useSiteSettingsContext } from "../../context/SiteSettingsContext";
import { AppearanceGroup } from "./AppearanceGroup.view";
import { ImageField } from "./ImageField.view";

export const ShareImageGroup = (): JSX.Element => {
  const { data, actions } = useSiteSettingsContext();
  return (
    <AppearanceGroup title="Share image">
      <ImageField field={data.images.og_image} picker={data.picker} />
      <RadioCards
        label="Generated card"
        value={data.form.card_style}
        items={data.options.card_style}
        onValueChange={actions.choose.card_style}
        columns={3}
      />
      <Field label="Date and reading time" description="Shows the post details on the card.">
        <Switch checked={data.form.card_meta} onCheckedChange={actions.toggle.card_meta} />
      </Field>
    </AppearanceGroup>
  );
};
