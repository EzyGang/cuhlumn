import type { JSX } from "preact";
import { Link } from "wouter-preact";
import { RadioCards } from "../../../shared/ui/RadioCards.view";
import type { FontModel } from "../../../../types/fonts";
import { useSiteSettingsContext } from "../../context/SiteSettingsContext";

interface FontNotesProps {
  font: FontModel;
}

const FontNotes = ({ font }: FontNotesProps): JSX.Element => (
  <>
    {font.note === null ? null : (
      <p class="text-xs text-muted" role="status">
        {font.note}
      </p>
    )}
    {font.error === undefined ? null : (
      <p class="text-xs text-error" role="alert">
        {font.error}
      </p>
    )}
    <Link href={font.manageHref} class="text-xs text-secondary underline hover:text-white">
      Add fonts in the font library
    </Link>
  </>
);

export const FontField = (): JSX.Element => {
  const { data, actions } = useSiteSettingsContext();
  return (
    <div class="flex flex-col gap-3">
      <RadioCards
        label="Font"
        value={data.font.value}
        items={data.font.items}
        onValueChange={actions.chooseFont}
      />
      <FontNotes font={data.font} />
      {data.font.usesFamily ? (
        <RadioCards
          label="Fallback"
          value={data.form.font}
          items={data.options.font}
          onValueChange={actions.choose.font}
        />
      ) : null}
    </div>
  );
};
