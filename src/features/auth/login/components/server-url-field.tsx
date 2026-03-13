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

type ServerUrlFieldProps = {
  customBaseUrl: string;
  serverUrlOption: string;
  isSubmitting: boolean;
  onServerUrlOptionChange: (value: string) => void;
  onCustomBaseUrlChange: (value: string) => void;
};

export function ServerUrlField({
  customBaseUrl,
  serverUrlOption,
  isSubmitting,
  onServerUrlOptionChange,
  onCustomBaseUrlChange,
}: ServerUrlFieldProps) {
  const { t } = useTranslation();

  return (
    <div className="space-y-2.5">
      <Label htmlFor="base-url" className="text-sm font-medium text-slate-700">
        {t("auth.login.form.serverUrl.label")}
      </Label>
      <Select
        value={serverUrlOption}
        onValueChange={onServerUrlOptionChange}
        disabled={isSubmitting}
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

      {serverUrlOption === CUSTOM_SERVER_URL_OPTION && (
        <InputGroup>
          <InputGroupAddon>
            <Globe className="h-5 w-5 text-slate-400" />
          </InputGroupAddon>
          <InputGroupInput
            id="base-url-custom"
            type="url"
            autoComplete="url"
            placeholder={t("auth.login.form.serverUrl.customPlaceholder")}
            value={customBaseUrl}
            onChange={(event) => onCustomBaseUrlChange(event.target.value)}
            disabled={isSubmitting}
            className="h-12 text-base"
          />
        </InputGroup>
      )}
    </div>
  );
}
