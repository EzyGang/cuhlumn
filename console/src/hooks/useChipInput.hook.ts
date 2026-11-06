import type { Signal } from "@preact/signals";
import { useSignal } from "@preact/signals";
import type { ChipInputModel } from "../types/chips";
import { addChips, removeChip } from "../utils/chips";
import type { ChipRules } from "../utils/chips";

interface ChipState {
  values: readonly string[];
  setValues: (values: string[]) => void;
  rules: ChipRules;
  draft: Signal<string>;
  error: Signal<string | null>;
}

const commit = ({ values, setValues, rules, draft, error }: ChipState): void => {
  if (draft.value.trim() === "") {
    draft.value = "";
    return;
  }
  const result = addChips(values, draft.value, rules);
  setValues(result.values);
  error.value = result.error;
  if (result.error === null) {
    draft.value = "";
  }
};

const keyDown = (state: ChipState, event: KeyboardEvent): void => {
  if (event.key === "Enter" || event.key === ",") {
    event.preventDefault();
    commit(state);
  } else if (event.key === "Backspace" && state.draft.value === "" && state.values.length > 0) {
    state.setValues(removeChip(state.values, state.values.length - 1));
  }
};

/** Binds a chip list to a draft text: Enter or a comma adds chips, Backspace removes the last chip. */
export const useChipInput = (
  values: readonly string[],
  setValues: (values: string[]) => void,
  rules: ChipRules,
  invalidIndex: number | null,
): ChipInputModel => {
  const draft = useSignal("");
  const error = useSignal<string | null>(null);
  const state: ChipState = { values, setValues, rules, draft, error };
  return {
    chips: values.map((label, index) => ({
      label,
      invalid: index === invalidIndex,
      remove: (): void => setValues(removeChip(values, index)),
    })),
    draft: draft.value,
    error: error.value,
    full: values.length >= rules.max,
    setDraft: (value: string): void => {
      draft.value = value;
      error.value = null;
    },
    keyDown: (event: KeyboardEvent): void => keyDown(state, event),
    commit: (): void => commit(state),
  };
};
