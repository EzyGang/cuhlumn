import { createContext } from "preact";
import type { ComponentChildren, JSX } from "preact";
import { useContext } from "preact/hooks";
import type { Font } from "../../../types/api/Font";
import type { ChipInputModel } from "../../../types/chips";
import type { SelectItem } from "../../../types/select";
import type { LoadState } from "../../../types/load";
import type {
  ImageFieldModel,
  LinkRowModel,
  MediaPickerModel,
  SiteForm,
  SiteTextKey,
} from "../../../types/siteSettings";
import type { FieldErrors } from "../../../utils/fieldErrors";

export interface SiteSettingsContextValue {
  data: {
    mode: "create" | "edit";
    form: SiteForm;
    errors: FieldErrors;
    aliases: ChipInputModel;
    aliasError: string | undefined;
    links: LinkRowModel[];
    picker: MediaPickerModel;
    colorValue: string;
    fonts: readonly SelectItem<Font>[];
    imageFields: ImageFieldModel[];
    linksTitle: string;
    canAddLink: boolean;
  };
  status: { load: LoadState; saving: boolean };
  actions: {
    text: Record<SiteTextKey, (value: string) => void>;
    setFont: (font: Font) => void;
    setCredit: (credit: boolean) => void;
    addLink: () => void;
    save: (event: Event) => void;
    remove: () => void;
  };
}

const SiteSettingsContext = createContext<SiteSettingsContextValue | null>(null);

interface SiteSettingsProviderProps {
  value: SiteSettingsContextValue;
  children: ComponentChildren;
}

export const SiteSettingsProvider = ({
  value,
  children,
}: SiteSettingsProviderProps): JSX.Element => (
  <SiteSettingsContext.Provider value={value}>{children}</SiteSettingsContext.Provider>
);

export const useSiteSettingsContext = (): SiteSettingsContextValue => {
  const value = useContext(SiteSettingsContext);
  if (value === null) {
    throw new Error("useSiteSettingsContext needs a SiteSettingsProvider.");
  }
  return value;
};
