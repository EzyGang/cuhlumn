import type { Font } from "./api/Font";
import type { Link } from "./api/Link";

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

export interface SiteForm extends Record<SiteTextKey, string> {
  aliases: string[];
  font: Font;
  links: Link[];
  credit: boolean;
}

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
  available: boolean;
  setOpen: Record<ImageKey, (open: boolean) => void>;
}

export interface ImageFieldModel {
  name: ImageKey;
  label: string;
  value: string;
  thumbnail: string | null;
  error: string | undefined;
  setValue: (value: string) => void;
}
