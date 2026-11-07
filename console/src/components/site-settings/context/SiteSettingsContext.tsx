import { createContext } from "preact";
import type { ComponentChildren, JSX } from "preact";
import { useContext } from "preact/hooks";
import type { ChipInputModel } from "../../../types/chips";
import type { LoadState } from "../../../types/load";
import type {
  AnnouncementActions,
  AnnouncementModel,
  AppearanceOptions,
  AppearanceSetters,
  ImageFieldModel,
  ImageKey,
  LinkRowModel,
  MediaPickerModel,
  SiteFlagKey,
  SiteForm,
  SiteTextKey,
  SocialModel,
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
    options: AppearanceOptions;
    images: Record<ImageKey, ImageFieldModel>;
    linksTitle: string;
    canAddLink: boolean;
    social: SocialModel;
    announcement: AnnouncementModel;
  };
  status: { load: LoadState; saving: boolean };
  actions: {
    text: Record<SiteTextKey, (value: string) => void>;
    toggle: Record<SiteFlagKey, (value: boolean) => void>;
    choose: AppearanceSetters;
    addLink: () => void;
    addSocial: () => void;
    announcement: AnnouncementActions;
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
