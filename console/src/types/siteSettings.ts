import type { BadgeTone } from "../components/shared/ui/Badge.view";
import type { AnnouncementStatus } from "./api/AnnouncementStatus";
import type { CardStyle } from "./api/CardStyle";
import type { CodeTheme } from "./api/CodeTheme";
import type { Font } from "./api/Font";
import type { HomeLayout } from "./api/HomeLayout";
import type { Link } from "./api/Link";
import type { Network } from "./api/Network";
import type { Palette } from "./api/Palette";
import type { SocialLink } from "./api/SocialLink";
import type { Theme } from "./api/Theme";
import type { Tone } from "./api/Tone";
import type { Width } from "./api/Width";
import type { RadioCardItem } from "./radioCards";
import type { SelectItem } from "./select";

export type SiteTextKey =
  | "host"
  | "title"
  | "description"
  | "author"
  | "author_url"
  | "project_name"
  | "project_url"
  | "cta"
  | "lang"
  | "home"
  | "indexnow_key"
  | "accent"
  | "logo"
  | "favicon"
  | "og_image"
  | "head"
  | "footer";

export type ImageKey = "logo" | "favicon" | "og_image";

/** The on/off settings of a site. */
export type SiteFlagKey =
  | "credit"
  | "theme_toggle"
  | "social_in_header"
  | "card_meta"
  | "indexnow"
  | "show_cta";

/** The appearance settings that the console shows as a set of choices. */
export interface AppearanceChoices {
  palette: Palette;
  theme: Theme;
  width: Width;
  home_layout: HomeLayout;
  font: Font;
  code_theme: CodeTheme;
  card_style: CardStyle;
}

export type AppearanceOptions = {
  [K in keyof AppearanceChoices]: readonly RadioCardItem<AppearanceChoices[K]>[];
};

export type AppearanceSetters = {
  [K in keyof AppearanceChoices]: (value: AppearanceChoices[K]) => void;
};

/** The announcement as the form edits it. Times use the `datetime-local` format. */
export interface AnnouncementForm {
  enabled: boolean;
  text: string;
  link: string;
  link_label: string;
  tone: Tone;
  starts_at: string;
  ends_at: string;
  dismissible: boolean;
}

export type AnnouncementTextKey = "text" | "link" | "link_label" | "starts_at" | "ends_at";

/** The state of the announcement bar: off, or the status of an enabled announcement. */
export type AnnouncementState = "off" | AnnouncementStatus;

export interface SiteForm extends Record<SiteTextKey, string>, AppearanceChoices {
  aliases: string[];
  links: Link[];
  social: SocialLink[];
  credit: boolean;
  theme_toggle: boolean;
  social_in_header: boolean;
  card_meta: boolean;
  indexnow: boolean;
  show_cta: boolean;
  announcement: AnnouncementForm;
}

/** Merges a change into the form, and clears the errors of the field at `path`. */
export type SiteFormUpdate = (patch: Partial<SiteForm>, path: string) => void;

export interface LinkRowModel extends Link {
  id: number;
  labelText: string;
  urlText: string;
  removeText: string;
  labelError: string | undefined;
  urlError: string | undefined;
  setLabel: (value: string) => void;
  setUrl: (value: string) => void;
  remove: () => void;
}

export interface SocialRowModel extends SocialLink {
  id: number;
  networkText: string;
  urlText: string;
  removeText: string;
  placeholder: string;
  urlError: string | undefined;
  setNetwork: (network: Network) => void;
  setUrl: (value: string) => void;
  remove: () => void;
}

export interface SocialModel {
  rows: SocialRowModel[];
  networks: readonly SelectItem<Network>[];
  title: string;
  canAdd: boolean;
}

export interface AnnouncementModel {
  state: AnnouncementState;
  stateText: string;
  stateTone: BadgeTone;
  textCount: string;
  timeZone: string;
  tones: readonly RadioCardItem<Tone>[];
}

export interface AnnouncementActions {
  setEnabled: (enabled: boolean) => void;
  text: Record<AnnouncementTextKey, (value: string) => void>;
  setTone: (tone: Tone) => void;
  setDismissible: (dismissible: boolean) => void;
}

export interface MediaChoice {
  name: string;
  url: string;
  path: string;
  pick: () => void;
}

export interface MediaPickerModel {
  target: ImageKey | null;
  items: MediaChoice[];
  loading: boolean;
  error: string | null;
  /** True when the dialog lists the media library. Only a saved site has a library. */
  library: boolean;
  uploading: boolean;
  accept: string;
  setOpen: Record<ImageKey, (open: boolean) => void>;
  upload: (event: Event) => void;
}

/** An image file that uploads when the admin creates the site. */
export interface PendingImage {
  file: File;
  /** A `data:` address that shows the file before the upload, or null until the browser reads it. */
  preview: string | null;
}

export type PendingImages = Partial<Record<ImageKey, PendingImage>>;

export interface ImageFieldModel {
  name: ImageKey;
  label: string;
  value: string;
  thumbnail: string | null;
  error: string | undefined;
  /** True when the field holds a file that uploads when the admin creates the site. */
  pending: boolean;
  removeText: string;
  setValue: (value: string) => void;
  removePending: () => void;
}
