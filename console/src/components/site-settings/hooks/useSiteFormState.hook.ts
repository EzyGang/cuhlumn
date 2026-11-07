import type { Signal } from "@preact/signals";
import { useChipInput } from "../../../hooks/useChipInput.hook";
import type { Link } from "../../../types/api/Link";
import type {
  SiteFlagKey,
  SiteForm,
  SiteFormUpdate,
  SiteTextKey,
} from "../../../types/siteSettings";
import { ALIAS_RULES } from "../../../utils/chips";
import { bindKeys } from "../../../utils/bindKeys";
import { firstItemError, withoutFieldError } from "../../../utils/fieldErrors";
import type { FieldErrors } from "../../../utils/fieldErrors";
import { MAX_LINKS } from "../../../utils/siteForm";
import type { SiteSettingsContextValue } from "../context/SiteSettingsContext";

type FormState = Pick<SiteSettingsContextValue["data"], "aliases" | "aliasError" | "links">;
type FormActions = Pick<SiteSettingsContextValue["actions"], "text" | "toggle" | "addLink">;

const BRAND_KEYS: Partial<Record<SiteTextKey | SiteFlagKey, true>> = {
  accent: true,
  logo: true,
  favicon: true,
  og_image: true,
  head: true,
  footer: true,
  credit: true,
  theme_toggle: true,
  social_in_header: true,
  card_meta: true,
};

const TEXT_KEYS: readonly SiteTextKey[] = [
  "host",
  "title",
  "description",
  "author",
  "author_url",
  "project_name",
  "project_url",
  "cta",
  "lang",
  "home",
  "indexnow_key",
  "accent",
  "logo",
  "favicon",
  "og_image",
  "head",
  "footer",
];

const FLAG_KEYS: readonly SiteFlagKey[] = [
  "credit",
  "theme_toggle",
  "social_in_header",
  "card_meta",
  "indexnow",
];

const linkRows = (
  links: Link[],
  errors: FieldErrors,
  setLinks: (links: Link[]) => void,
): FormState["links"] =>
  links.map((link, index) => {
    const replace = (patch: Partial<Link>): void =>
      setLinks(links.map((row, position) => (position === index ? { ...row, ...patch } : row)));
    return {
      ...link,
      id: index,
      labelText: `Link ${index + 1} label`,
      urlText: `Link ${index + 1} URL`,
      removeText: `Remove link ${index + 1}`,
      labelError: errors[`brand.links.${index}.label`] ?? errors[`brand.links.${index}`],
      urlError: errors[`brand.links.${index}.url`],
      setLabel: (label: string): void => replace({ label }),
      setUrl: (url: string): void => replace({ url }),
      remove: (): void => setLinks(links.filter((_, position) => position !== index)),
    };
  });

const fieldPath = (key: SiteTextKey | SiteFlagKey): string =>
  BRAND_KEYS[key] === true ? `brand.${key}` : key;

const useAliasInput = (
  form: Signal<SiteForm>,
  errors: Signal<FieldErrors>,
  update: SiteFormUpdate,
): Pick<FormState, "aliases" | "aliasError"> => {
  const [aliasIndex, aliasError] = firstItemError(errors.value, "aliases");
  const setAliases = (values: string[]): void => update({ aliases: values }, "aliases");
  return {
    aliases: useChipInput(form.value.aliases, setAliases, ALIAS_RULES, aliasIndex),
    aliasError,
  };
};

export const useSiteFormState = (
  form: Signal<SiteForm>,
  errors: Signal<FieldErrors>,
): FormState & { update: SiteFormUpdate; actions: FormActions } => {
  const update: SiteFormUpdate = (patch, path) => {
    form.value = { ...form.value, ...patch };
    errors.value = withoutFieldError(errors.value, path);
  };
  const setLinks = (links: Link[]): void => update({ links }, "brand.links");
  return {
    ...useAliasInput(form, errors, update),
    links: linkRows(form.value.links, errors.value, setLinks),
    update,
    actions: {
      text: bindKeys(TEXT_KEYS, (key, value: string): void =>
        update({ [key]: value }, fieldPath(key)),
      ),
      toggle: bindKeys(FLAG_KEYS, (key, value: boolean): void =>
        update({ [key]: value }, fieldPath(key)),
      ),
      addLink: (): void =>
        setLinks([...form.value.links, { label: "", url: "" }].slice(0, MAX_LINKS)),
    },
  };
};
