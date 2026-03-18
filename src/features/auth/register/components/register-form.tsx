import { Globe, Mail, User } from "lucide-react";
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
import type { RegistrationForm } from "@/features/auth/register/hooks/use-registration-flow";
import { FormFieldError } from "@/features/auth/shared/form-field-error";

type RegisterFormProps = {
  form: RegistrationForm;
};

export function RegisterForm({ form }: RegisterFormProps) {
  const { t } = useTranslation();

  return (
    <>
      {/* Server URL */}
      <div className="space-y-2.5">
        <Label
          htmlFor="reg-base-url"
          className="text-sm font-medium text-slate-700"
        >
          {t("auth.register.form.serverUrl.label")}
        </Label>
        <form.Field name="serverUrlOption">
          {(field) => (
            <>
              <Select
                value={field.state.value}
                onValueChange={field.handleChange}
              >
                <SelectTrigger
                  id="reg-base-url"
                  className="h-12 w-full bg-white"
                >
                  <SelectValue
                    placeholder={t("auth.register.form.serverUrl.placeholder")}
                  />
                </SelectTrigger>
                <SelectContent>
                  {SERVER_URL_OPTIONS.map((option) => (
                    <SelectItem key={option.value} value={option.value}>
                      {option.label}
                    </SelectItem>
                  ))}
                  <SelectItem value={CUSTOM_SERVER_URL_OPTION}>
                    {t("auth.register.form.serverUrl.customOption")}
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
                      <InputGroupInput
                        id="reg-base-url-custom"
                        type="url"
                        autoComplete="url"
                        placeholder={t(
                          "auth.register.form.serverUrl.customPlaceholder",
                        )}
                        value={field.state.value}
                        onChange={(e) => field.handleChange(e.target.value)}
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

      {/* Email */}
      <div className="space-y-2.5">
        <Label
          htmlFor="reg-email"
          className="text-sm font-medium text-slate-700"
        >
          {t("auth.register.form.email.label")}
        </Label>
        <form.Field name="email">
          {(field) => (
            <>
              <InputGroup>
                <InputGroupAddon>
                  <Mail className="h-5 w-5 text-slate-400" />
                </InputGroupAddon>
                <InputGroupInput
                  id="reg-email"
                  type="email"
                  autoComplete="email"
                  placeholder={t("auth.register.form.email.placeholder")}
                  value={field.state.value}
                  onChange={(e) => field.handleChange(e.target.value)}
                  onBlur={field.handleBlur}
                  className="h-12 text-base"
                />
              </InputGroup>
              <FormFieldError errors={field.state.meta.errors} />
            </>
          )}
        </form.Field>
      </div>

      {/* Name */}
      <div className="space-y-2.5">
        <Label
          htmlFor="reg-name"
          className="text-sm font-medium text-slate-700"
        >
          {t("auth.register.form.name.label")}
        </Label>
        <form.Field name="name">
          {(field) => (
            <>
              <InputGroup>
                <InputGroupAddon>
                  <User className="h-5 w-5 text-slate-400" />
                </InputGroupAddon>
                <InputGroupInput
                  id="reg-name"
                  type="text"
                  autoComplete="name"
                  placeholder={t("auth.register.form.name.placeholder")}
                  value={field.state.value}
                  onChange={(e) => field.handleChange(e.target.value)}
                  onBlur={field.handleBlur}
                  className="h-12 text-base"
                />
              </InputGroup>
              <FormFieldError errors={field.state.meta.errors} />
            </>
          )}
        </form.Field>
      </div>
    </>
  );
}
