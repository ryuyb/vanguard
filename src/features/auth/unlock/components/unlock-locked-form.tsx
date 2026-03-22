import {
  Eye,
  EyeOff,
  Fingerprint,
  KeyRound,
  LoaderCircle,
  LogOut,
} from "lucide-react";
import type { SubmitEventHandler } from "react";
import { useTranslation } from "react-i18next";
import type { AccountContextDto } from "@/bindings";
import { TextInput } from "@/components/text-input";
import { Button } from "@/components/ui/button";
import { InputGroup, InputGroupAddon } from "@/components/ui/input-group";
import { Label } from "@/components/ui/label";
import { UnlockFeedbackAlert } from "@/features/auth/unlock/components/unlock-feedback-alert";
import type { UnlockForm } from "@/features/auth/unlock/hooks/use-unlock-flow";
import type { UnlockFeedback } from "@/features/auth/unlock/types";

type UnlockMethod = "pin" | "masterPassword";

type UnlockLockedFormProps = {
  form: UnlockForm;
  account: AccountContextDto | null;
  biometricSupported: boolean;
  biometricEnabled: boolean;
  canBiometricUnlock: boolean;
  feedback: UnlockFeedback;
  isActionBlocked: boolean;
  isBiometricUnlocking: boolean;
  isLoggingOut: boolean;
  isPinUnlocking: boolean;
  onBiometricUnlock: () => void;
  onLogout: () => void;
  onPinUnlock: SubmitEventHandler<HTMLFormElement>;
  onShowMasterPasswordUnlock: () => void;
  onShowPinUnlock: () => void;
  onToggleShowPassword: () => void;
  pinEnabled: boolean;
  showPassword: boolean;
  unlockMethod: UnlockMethod;
};

export function UnlockLockedForm({
  form,
  account,
  biometricSupported,
  biometricEnabled,
  canBiometricUnlock,
  feedback,
  isActionBlocked,
  isBiometricUnlocking,
  isLoggingOut,
  isPinUnlocking,
  onBiometricUnlock,
  onLogout,
  onPinUnlock,
  onShowMasterPasswordUnlock,
  onShowPinUnlock,
  onToggleShowPassword,
  pinEnabled,
  showPassword,
  unlockMethod,
}: UnlockLockedFormProps) {
  const { t } = useTranslation();
  const isPinMode = unlockMethod === "pin";

  return (
    <div className="space-y-6">
      <div className="space-y-2 rounded-xl border border-slate-200/60 bg-slate-50/50 px-4 py-3.5">
        <div className="flex items-center justify-between text-xs">
          <span className="font-medium text-slate-500">
            {t("auth.unlock.form.account.label")}
          </span>
          <span className="text-slate-700">
            {account?.email ?? t("auth.unlock.form.account.unknown")}
          </span>
        </div>
        <div className="flex items-center justify-between text-xs">
          <span className="font-medium text-slate-500">
            {t("auth.unlock.form.server.label")}
          </span>
          <span className="text-slate-700">
            {account?.baseUrl ?? t("auth.unlock.form.server.unknown")}
          </span>
        </div>
      </div>

      <form
        className="space-y-5"
        onSubmit={
          isPinMode
            ? onPinUnlock
            : (e) => {
                e.preventDefault();
                e.stopPropagation();
                form.handleSubmit();
              }
        }
      >
        {isPinMode ? (
          <div className="space-y-2.5">
            <Label
              htmlFor="unlock-pin"
              className="text-sm font-medium text-slate-700"
            >
              {t("auth.unlock.form.pin.label")}
            </Label>
            <form.Field name="pin">
              {(field) => (
                <InputGroup>
                  <InputGroupAddon>
                    <KeyRound className="h-5 w-5 text-slate-400" />
                  </InputGroupAddon>
                  <TextInput
                    inputGroup
                    id="unlock-pin"
                    type="password"
                    inputMode="numeric"
                    autoComplete="off"
                    placeholder={t("auth.unlock.form.pin.placeholder")}
                    value={field.state.value}
                    onChange={(e) => field.handleChange(e.target.value)}
                    onBlur={field.handleBlur}
                    disabled={isActionBlocked}
                    className="h-12 text-base"
                  />
                </InputGroup>
              )}
            </form.Field>
          </div>
        ) : (
          <div className="space-y-2.5">
            <Label
              htmlFor="unlock-master-password"
              className="text-sm font-medium text-slate-700"
            >
              {t("auth.unlock.form.masterPassword.label")}
            </Label>
            <form.Field name="masterPassword">
              {(field) => (
                <InputGroup>
                  <InputGroupAddon>
                    <KeyRound className="h-5 w-5 text-slate-400" />
                  </InputGroupAddon>
                  <TextInput
                    inputGroup
                    id="unlock-master-password"
                    type={showPassword ? "text" : "password"}
                    autoComplete="current-password"
                    placeholder={t(
                      "auth.unlock.form.masterPassword.placeholder",
                    )}
                    value={field.state.value}
                    onChange={(e) => field.handleChange(e.target.value)}
                    onBlur={field.handleBlur}
                    disabled={isActionBlocked}
                    className="h-12 text-base"
                  />
                  <InputGroupAddon align="inline-end" className="px-1.5">
                    <Button
                      type="button"
                      variant="ghost"
                      size="icon-sm"
                      className="text-slate-400 hover:text-slate-700 transition-colors"
                      onClick={onToggleShowPassword}
                      disabled={isActionBlocked}
                      aria-label={
                        showPassword
                          ? t("auth.unlock.form.masterPassword.hidePassword")
                          : t("auth.unlock.form.masterPassword.showPassword")
                      }
                    >
                      {showPassword ? (
                        <EyeOff className="h-5 w-5" />
                      ) : (
                        <Eye className="h-5 w-5" />
                      )}
                    </Button>
                  </InputGroupAddon>
                </InputGroup>
              )}
            </form.Field>
          </div>
        )}

        <UnlockFeedbackAlert feedback={feedback} />

        <form.Subscribe
          selector={(s) => [s.canSubmit, s.isSubmitting] as const}
        >
          {([canSubmit, isSubmitting]) => (
            <Button
              type="submit"
              size="lg"
              className="h-12 w-full bg-blue-600 text-base font-medium hover:bg-blue-700 transition-colors"
              disabled={
                isPinMode
                  ? isActionBlocked || !form.getFieldValue("pin").trim()
                  : !canSubmit || !form.getFieldValue("masterPassword").trim()
              }
            >
              {(isPinMode ? isPinUnlocking : isSubmitting) && (
                <LoaderCircle className="h-5 w-5 animate-spin" />
              )}
              {isPinMode
                ? isPinUnlocking
                  ? t("auth.unlock.actions.unlockingWithPin")
                  : t("auth.unlock.actions.unlockWithPin")
                : isSubmitting
                  ? t("auth.unlock.actions.unlocking")
                  : t("auth.unlock.actions.unlock")}
            </Button>
          )}
        </form.Subscribe>
      </form>

      {pinEnabled && (
        <Button
          type="button"
          variant="ghost"
          className="w-full text-sm text-slate-600 hover:text-slate-900 transition-colors"
          disabled={isActionBlocked}
          onClick={isPinMode ? onShowMasterPasswordUnlock : onShowPinUnlock}
        >
          {isPinMode
            ? t("auth.unlock.actions.switchToMasterPassword")
            : t("auth.unlock.actions.switchToPin")}
        </Button>
      )}

      {canBiometricUnlock && (
        <Button
          type="button"
          variant="outline"
          size="lg"
          className="h-12 w-full border-slate-300 text-base font-medium hover:bg-slate-50 transition-colors"
          onClick={onBiometricUnlock}
          disabled={isActionBlocked}
        >
          {isBiometricUnlocking ? (
            <LoaderCircle className="h-5 w-5 animate-spin" />
          ) : (
            <Fingerprint className="h-5 w-5" />
          )}
          {isBiometricUnlocking
            ? t("auth.unlock.actions.biometricVerifying")
            : t("auth.unlock.actions.biometric")}
        </Button>
      )}

      {biometricSupported && biometricEnabled && !canBiometricUnlock && (
        <div className="rounded-xl border border-amber-200/60 bg-amber-50/50 px-4 py-3 text-sm text-amber-900">
          <p className="font-medium">
            {t("auth.unlock.states.biometricUnavailable.title")}
          </p>
          <p className="mt-1 text-amber-800">
            {t("auth.unlock.states.biometricUnavailable.description")}
          </p>
        </div>
      )}

      <div className="pt-2 border-t border-slate-200">
        <Button
          type="button"
          variant="ghost"
          className="w-full text-sm text-slate-600 hover:text-red-600 transition-colors"
          disabled={isActionBlocked}
          onClick={onLogout}
        >
          {isLoggingOut ? (
            <LoaderCircle className="h-4 w-4 animate-spin" />
          ) : (
            <LogOut className="h-4 w-4" />
          )}
          {isLoggingOut
            ? t("auth.unlock.actions.loggingOut")
            : t("auth.unlock.actions.logout")}
        </Button>
      </div>
    </div>
  );
}
