import type { Signal } from "@preact/signals";
import type { Network } from "../../../types/api/Network";
import type { SocialLink } from "../../../types/api/SocialLink";
import type { SelectItem } from "../../../types/select";
import type {
  SiteForm,
  SiteFormUpdate,
  SocialModel,
  SocialRowModel,
} from "../../../types/siteSettings";
import type { FieldErrors } from "../../../utils/fieldErrors";
import { MAX_SOCIAL } from "../../../utils/siteForm";

const NETWORKS: readonly SelectItem<Network>[] = [
  { value: "github", label: "GitHub" },
  { value: "gitlab", label: "GitLab" },
  { value: "codeberg", label: "Codeberg" },
  { value: "mastodon", label: "Mastodon" },
  { value: "bluesky", label: "Bluesky" },
  { value: "x", label: "X" },
  { value: "threads", label: "Threads" },
  { value: "linkedin", label: "LinkedIn" },
  { value: "youtube", label: "YouTube" },
  { value: "instagram", label: "Instagram" },
  { value: "discord", label: "Discord" },
  { value: "telegram", label: "Telegram" },
  { value: "email", label: "Email" },
];

type SetSocial = (links: SocialLink[]) => void;

const NEW_LINK: SocialLink = { network: "github", url: "" };

const socialRow = (
  links: SocialLink[],
  link: SocialLink,
  index: number,
  errors: FieldErrors,
  setSocial: SetSocial,
): SocialRowModel => {
  const replace = (patch: Partial<SocialLink>): void =>
    setSocial(links.map((row, position) => (position === index ? { ...row, ...patch } : row)));
  return {
    ...link,
    id: index,
    networkText: `Profile ${index + 1} network`,
    urlText: `Profile ${index + 1} address`,
    removeText: `Remove profile ${index + 1}`,
    placeholder: link.network === "email" ? "mailto:" : "https://",
    urlError: errors[`brand.social.${index}.url`] ?? errors[`brand.social.${index}`],
    setNetwork: (network: Network): void => replace({ network }),
    setUrl: (url: string): void => replace({ url }),
    remove: (): void => setSocial(links.filter((_, position) => position !== index)),
  };
};

/** Returns the rows of the profile links editor, and adds a row on request. */
export const useSocialLinks = (
  form: Signal<SiteForm>,
  errors: FieldErrors,
  update: SiteFormUpdate,
): { social: SocialModel; addSocial: () => void } => {
  const links = form.value.social;
  const setSocial: SetSocial = (next) => update({ social: next }, "brand.social");
  return {
    social: {
      rows: links.map((link, index) => socialRow(links, link, index, errors, setSocial)),
      networks: NETWORKS,
      title: `Profiles (${links.length}/${MAX_SOCIAL})`,
      canAdd: links.length < MAX_SOCIAL,
    },
    addSocial: (): void => setSocial([...form.value.social, NEW_LINK].slice(0, MAX_SOCIAL)),
  };
};
