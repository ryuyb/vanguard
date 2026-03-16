import { Globe } from "lucide-react";
import { useTranslation } from "react-i18next";
import {
  InputGroup,
  InputGroupAddon,
  InputGroupInput,
} from "@/components/ui/input-group";
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import {
  CUSTOM_SERVER_URL_OPTION,
  SERVER_URL_OPTIONS,
} from "@/features/auth/login/constants";
import type { LoginForm } from "@/features/auth/login/hooks/use-login-flow";

type ServerUrlFieldProps = {
  form: LoginForm;
  clearTwoFactorChallenge: () => void;
};

export function ServerUrlField({
  form,
  clearTwoFactorChallenge,
}: ServerUrlFieldProps) {
  const { t } = useTranslation();

  return (
    <div className="space-y-2.5">
      <Label htmlFor="base-url" className="text-sm font-medium text-slate-700">
        {t("auth.login.form.serverUrl.label")}
      </Label>
      <form.Field name="serverUrlOption">
        {(field) => (
          <Select
            value={field.state.value}
            onValueChange={(value) => {
              clearTwoFactorChallenge();
              field.handleChange(value);
            }}
            disabled={form.state.isSubmitting}
          >
            <SelectTrigger id="base-url" className="h-12 w-full bg-white">
              <SelectValue
                placeholder={t("auth.login.form.serverUrl.placeholder")}
              />
            </SelectTrigger>
            <SelectContent>
              {SERVER_URL_OPTIONS.map((option) => (
                <SelectItem key={option.value} value={option.value}>
                  {option.label}
                </SelectItem>
              ))}
              <SelectItem value={CUSTOM_SERVER_URL_OPTION}>
                {t("auth.login.form.serverUrl.customOption")}
              </SelectItem>
            </SelectContent>
          </Select>
        )}
      </form.Field>

      <form.Subscribe selector={(s) => s.values.serverUrlOption}>
        {(serverUrlOption) =>
          serverUrlOption === CUSTOM_SERVER_URL_OPTION ? (
            <form.Field name="customBaseUrl">
              {(field) => (
                <InputGroup>
                  <InputGroupAddon>
                    <Globe className="h-5 w-5 text-slate-400" />
                  </InputGroupAddon>
                  <InputGroupInput
                    id="base-url-custom"
                    type="url"
                    autoComplete="url"
                    placeholder={t(
                      "auth.login.form.serverUrl.customPlaceholder",
                    )}
                    value={field.state.value}
                    onChange={(e) => {
                      clearTwoFactorChallenge();
                      field.handleChange(e.target.value);
                    }}
                    onBlur={field.handleBlur}
                    disabled={form.state.isSubmitting}
                    className="h-12 text-base"
                  />
                </InputGroup>
              )}
            </form.Field>
          ) : null
        }
      </form.Subscribe>
    </div>
  );
}
