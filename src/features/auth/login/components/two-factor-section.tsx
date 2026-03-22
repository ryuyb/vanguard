import { LoaderCircle, Send, ShieldCheck } from "lucide-react";
import { useTranslation } from "react-i18next";
import { TextInput } from "@/components/text-input";
import { Button } from "@/components/ui/button";
import { InputGroup, InputGroupAddon } from "@/components/ui/input-group";
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { toProviderLabel } from "@/features/auth/login/login-flow-helpers";
import type { TwoFactorState } from "@/features/auth/login/types";

type TwoFactorSectionProps = {
  state: TwoFactorState;
  isSubmitting: boolean;
  onProviderChange: (value: string) => void;
  onTokenChange: (value: string) => void;
  onSendEmailCode: () => void;
};

export function TwoFactorSection({
  state,
  isSubmitting,
  onProviderChange,
  onTokenChange,
  onSendEmailCode,
}: TwoFactorSectionProps) {
  const { t } = useTranslation();
  const isDisabled = isSubmitting || state.isSendingEmailCode;

  return (
    <div className="space-y-4 rounded-xl border border-amber-200 bg-amber-50/80 p-4">
      <div className="flex items-center gap-2 text-sm font-medium text-amber-800">
        <ShieldCheck className="size-4" />
        {t("auth.login.form.twoFactor.title")}
      </div>

      <div className="space-y-2">
        <Label htmlFor="two-factor-provider">
          {t("auth.login.form.twoFactor.provider.label")}
        </Label>
        <Select
          value={state.selectedProvider}
          onValueChange={onProviderChange}
          disabled={isDisabled}
        >
          <SelectTrigger id="two-factor-provider" className="w-full bg-white">
            <SelectValue
              placeholder={t("auth.login.form.twoFactor.provider.placeholder")}
            />
          </SelectTrigger>
          <SelectContent>
            {state.providers.map((provider) => (
              <SelectItem key={provider} value={provider}>
                {toProviderLabel(provider)}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </div>

      <div className="space-y-2">
        <Label htmlFor="two-factor-token">
          {t("auth.login.form.twoFactor.token.label")}
        </Label>
        <InputGroup>
          <InputGroupAddon>
            <ShieldCheck className="text-slate-500" />
          </InputGroupAddon>
          <TextInput
            inputGroup
            id="two-factor-token"
            type="text"
            inputMode="numeric"
            autoComplete="one-time-code"
            placeholder={t("auth.login.form.twoFactor.token.placeholder")}
            value={state.token}
            onChange={(event) => onTokenChange(event.target.value)}
            disabled={isDisabled}
          />
        </InputGroup>
      </div>

      {state.selectedProvider === "1" && (
        <Button
          type="button"
          variant="outline"
          className="w-full"
          disabled={isDisabled}
          onClick={onSendEmailCode}
        >
          {state.isSendingEmailCode && (
            <LoaderCircle className="animate-spin" />
          )}
          {!state.isSendingEmailCode && <Send />}
          {state.isSendingEmailCode
            ? t("auth.login.form.twoFactor.sendingEmail")
            : t("auth.login.form.twoFactor.sendEmail")}
        </Button>
      )}
    </div>
  );
}
