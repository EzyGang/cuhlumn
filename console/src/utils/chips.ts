export interface ChipRules {
  max: number;
  maxLength: number;
  lowercase: boolean;
  noun: string;
  plural: string;
}

export interface ChipResult {
  values: string[];
  error: string | null;
}

export const TAG_RULES: ChipRules = {
  max: 20,
  maxLength: 50,
  lowercase: false,
  noun: "tag",
  plural: "tags",
};
export const ALIAS_RULES: ChipRules = {
  max: 100,
  maxLength: 253,
  lowercase: true,
  noun: "alias",
  plural: "aliases",
};

const splitDraft = (draft: string, rules: ChipRules): string[] =>
  draft
    .split(",")
    .map((part) => (rules.lowercase ? part.trim().toLowerCase() : part.trim()))
    .filter((part) => part.length > 0);

export const addChips = (
  current: readonly string[],
  draft: string,
  rules: ChipRules,
): ChipResult => {
  const values = [...current];
  const seen = new Set(current.map((value) => value.toLowerCase()));
  for (const candidate of splitDraft(draft, rules)) {
    if ([...candidate].length > rules.maxLength) {
      return { values, error: `A ${rules.noun} has at most ${rules.maxLength} characters.` };
    }
    if (seen.has(candidate.toLowerCase())) {
      continue;
    }
    if (values.length >= rules.max) {
      return { values, error: `The limit is ${rules.max} ${rules.plural}.` };
    }
    seen.add(candidate.toLowerCase());
    values.push(candidate);
  }
  return { values, error: null };
};

export const removeChip = (current: readonly string[], index: number): string[] =>
  current.filter((_, position) => position !== index);
