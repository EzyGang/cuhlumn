import type { JSX } from "preact";
import { Copy, Eye, EyeOff, Sparkles } from "lucide-preact";
import { Button } from "../../shared/ui/Button.view";
import { Field } from "../../shared/ui/Field.view";
import { Input } from "../../shared/ui/Input.view";
import type { TokenDraftModel } from "../../../types/auth";
import { MIN_TOKEN_LENGTH } from "../../../utils/token";

interface TokenFieldProps {
  draft: TokenDraftModel;
  label: string;
  error?: string | undefined;
  disabled?: boolean;
}

export const TokenField = ({
  draft,
  label,
  error,
  disabled = false,
}: TokenFieldProps): JSX.Element => (
  <div class="flex flex-col gap-3">
    <Field label={label} error={error} description={`At least ${MIN_TOKEN_LENGTH} characters.`}>
      <Input
        type={draft.data.visible ? "text" : "password"}
        value={draft.data.token}
        onValueChange={draft.actions.setToken}
        inputRef={draft.data.input}
        autoComplete="new-password"
        spellCheck={false}
        disabled={disabled}
        class="font-mono"
      />
    </Field>
    <div class="flex flex-wrap gap-2">
      <Button variant="ghost" onClick={draft.actions.toggleVisible} disabled={disabled}>
        {draft.data.visible ? (
          <EyeOff size={14} aria-hidden="true" />
        ) : (
          <Eye size={14} aria-hidden="true" />
        )}
        {draft.data.visible ? "Hide" : "Show"}
      </Button>
      <Button variant="ghost" onClick={draft.actions.generate} disabled={disabled}>
        <Sparkles size={14} aria-hidden="true" />
        Generate
      </Button>
      <Button
        variant="ghost"
        onClick={draft.actions.copy}
        disabled={disabled || draft.data.token === ""}
      >
        <Copy size={14} aria-hidden="true" />
        Copy
      </Button>
    </div>
  </div>
);
