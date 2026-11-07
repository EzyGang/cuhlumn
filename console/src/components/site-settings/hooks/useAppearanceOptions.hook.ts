import type {
  AppearanceChoices,
  AppearanceOptions,
  AppearanceSetters,
  SiteFormUpdate,
} from "../../../types/siteSettings";

const OPTIONS: AppearanceOptions = {
  palette: [
    { value: "paper", label: "Paper", description: "Warm white pages with dark ink." },
    { value: "slate", label: "Slate", description: "Cool blue-grey tones." },
    { value: "sepia", label: "Sepia", description: "Soft brown tones, like an old book." },
    { value: "mono", label: "Mono", description: "Pure black, white, and grey." },
    { value: "contrast", label: "Contrast", description: "Maximum contrast for easy reading." },
  ],
  theme: [
    { value: "system", label: "System", description: "Follows the setting of the reader." },
    { value: "light", label: "Light", description: "Always starts light." },
    { value: "dark", label: "Dark", description: "Always starts dark." },
  ],
  width: [
    { value: "narrow", label: "Narrow", description: "A short line for focused reading." },
    { value: "normal", label: "Normal", description: "The standard text column." },
    { value: "wide", label: "Wide", description: "More room for code and tables." },
  ],
  home_layout: [
    { value: "list", label: "List", description: "Title, date, and summary of each post." },
    { value: "compact", label: "Compact", description: "One line per post with the date." },
    { value: "cards", label: "Cards", description: "A grid with the cover images." },
  ],
  font: [
    { value: "serif", label: "Serif", description: "A classic book face for post text." },
    { value: "sans", label: "Sans serif", description: "A clean modern face for post text." },
  ],
  code_theme: [
    { value: "github", label: "GitHub", description: "Familiar colours on a neutral ground." },
    { value: "solarized", label: "Solarized", description: "Calm colours on a tinted ground." },
    { value: "ocean", label: "Ocean", description: "Cool blue and green colours." },
    { value: "none", label: "None", description: "Plain code without colours." },
  ],
  card_style: [
    { value: "dark", label: "Dark", description: "Light text on a dark ground." },
    { value: "light", label: "Light", description: "Dark text on a light ground." },
    { value: "accent", label: "Accent", description: "The accent colour as the ground." },
  ],
};

/** Returns the appearance choices with a setter for each choice. */
export const useAppearanceOptions = (
  update: SiteFormUpdate,
): { options: AppearanceOptions; choose: AppearanceSetters } => {
  const choose =
    <K extends keyof AppearanceChoices>(key: K) =>
    (value: AppearanceChoices[K]): void =>
      update({ [key]: value }, `brand.${key}`);
  return {
    options: OPTIONS,
    choose: {
      palette: choose("palette"),
      theme: choose("theme"),
      width: choose("width"),
      home_layout: choose("home_layout"),
      font: choose("font"),
      code_theme: choose("code_theme"),
      card_style: choose("card_style"),
    },
  };
};
