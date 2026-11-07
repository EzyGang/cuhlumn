import { createContext } from "preact";
import type { ComponentChildren, JSX } from "preact";
import { useContext } from "preact/hooks";
import type { BadgeTone } from "../../shared/ui/Badge.view";
import type { Kind } from "../../../types/api/Kind";
import type { Status } from "../../../types/api/Status";
import type { ChipInputModel } from "../../../types/chips";
import type { LoadState } from "../../../types/load";
import type { CtaChoice, PostForm, PostTextKey, PreviewModel } from "../../../types/postEditor";
import type { SelectItem } from "../../../types/select";
import type { FieldErrors } from "../../../utils/fieldErrors";

export interface PostEditorContextValue {
  data: {
    mode: "create" | "edit";
    form: PostForm;
    errors: FieldErrors;
    tags: ChipInputModel;
    tagError: string | undefined;
    kinds: readonly SelectItem<Kind>[];
    ctaChoices: readonly SelectItem<CtaChoice>[];
    savedStatus: Status | null;
    statusTone: BadgeTone;
    publicUrl: string | null;
    primaryLabel: string;
    secondaryLabel: string;
    preview: PreviewModel;
  };
  status: { load: LoadState; saving: boolean };
  actions: {
    text: Record<PostTextKey, (value: string) => void>;
    setKind: (kind: Kind) => void;
    setShowCta: (choice: CtaChoice) => void;
    primary: () => void;
    secondary: () => void;
    remove: () => void;
  };
}

const PostEditorContext = createContext<PostEditorContextValue | null>(null);

interface PostEditorProviderProps {
  value: PostEditorContextValue;
  children: ComponentChildren;
}

export const PostEditorProvider = ({ value, children }: PostEditorProviderProps): JSX.Element => (
  <PostEditorContext.Provider value={value}>{children}</PostEditorContext.Provider>
);

export const usePostEditorContext = (): PostEditorContextValue => {
  const value = useContext(PostEditorContext);
  if (value === null) {
    throw new Error("usePostEditorContext needs a PostEditorProvider.");
  }
  return value;
};
