export interface ChipValue {
  label: string;
  invalid: boolean;
  remove: () => void;
}

export interface ChipInputModel {
  chips: ChipValue[];
  draft: string;
  error: string | null;
  full: boolean;
  setDraft: (value: string) => void;
  keyDown: (event: KeyboardEvent) => void;
  commit: () => void;
}
