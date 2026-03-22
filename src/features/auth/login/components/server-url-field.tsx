import { Globe } from "lucide-react";
import { useTranslation } from "react-i18next";
import { TextInput } from "@/components/text-input";
import { InputGroup, InputGroupAddon } from "@/components/ui/input-group";
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
import { FormFieldError } from "@/features/auth/shared/form-field-error";

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
          <>
            <Select
              value={field.state.value}
              onValueChange={(value) => {
                clearTwoFactorChallenge();
                field.handleChange(value);
              }}
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
            <FormFieldError errors={field.state.meta.errors} />
          </>
        )}
      </form.Field>

      <form.Subscribe selector={(s) => s.values.serverUrlOption}>
        {(serverUrlOption) =>
          serverUrlOption === CUSTOM_SERVER_URL_OPTION ? (
            <form.Field name="customBaseUrl">
              {(field) => (
                <>
                  <InputGroup>
                    <InputGroupAddon>
                      <Globe className="h-5 w-5 text-slate-400" />
                    </InputGroupAddon>
                    <TextInput
                      inputGroup
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
                      className="h-12 text-base"
                    />
                  </InputGroup>
                  <FormFieldError errors={field.state.meta.errors} />
                </>
              )}
            </form.Field>
          ) : null
        }
      </form.Subscribe>
    </div>
  );
}
