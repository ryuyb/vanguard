import { useEffect, useState } from "react";
import { z } from "zod";

import {
  AlertDialog,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";

type SelfHostedModalProps = {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  value?: SelfHostedConfig;
  onSave?: (value: SelfHostedConfig) => void | Promise<void>;
};

const serverUrlSchema = z.url({ message: "Enter a valid URL" });

type SelfHostedConfig = {
  serverUrl: string;
  webVaultUrl?: string;
  apiUrl?: string;
  identityUrl?: string;
  notificationsUrl?: string;
  iconsUrl?: string;
};

type FieldErrors = Partial<Record<keyof SelfHostedConfig, string>>;

export function SelfHostedModal({
  open,
  onOpenChange,
  value,
  onSave,
}: SelfHostedModalProps) {
  const [formValues, setFormValues] = useState<SelfHostedConfig>({
    serverUrl: value?.serverUrl ?? "",
    webVaultUrl: value?.webVaultUrl ?? "",
    apiUrl: value?.apiUrl ?? "",
    identityUrl: value?.identityUrl ?? "",
    notificationsUrl: value?.notificationsUrl ?? "",
    iconsUrl: value?.iconsUrl ?? "",
  });
  const [errors, setErrors] = useState<FieldErrors>({});

  useEffect(() => {
    if (!open) {
      return;
    }
    setFormValues({
      serverUrl: value?.serverUrl ?? "",
      webVaultUrl: value?.webVaultUrl ?? "",
      apiUrl: value?.apiUrl ?? "",
      identityUrl: value?.identityUrl ?? "",
      notificationsUrl: value?.notificationsUrl ?? "",
      iconsUrl: value?.iconsUrl ?? "",
    });
    setErrors({});
  }, [open, value?.apiUrl, value?.iconsUrl, value?.identityUrl, value?.notificationsUrl, value?.serverUrl, value?.webVaultUrl]);

  const validateServerUrl = (value: string) => {
    if (!value.trim()) {
      return "Server URL is required";
    }
    const result = serverUrlSchema.safeParse(value.trim());
    if (result.success) {
      return null;
    }
    return result.error.issues[0]?.message ?? "Enter a valid URL";
  };

  const validateOptionalUrl = (value: string) => {
    if (!value.trim()) {
      return null;
    }
    const result = serverUrlSchema.safeParse(value.trim());
    if (result.success) {
      return null;
    }
    return result.error.issues[0]?.message ?? "Enter a valid URL";
  };

  return (
    <AlertDialog open={open} onOpenChange={onOpenChange}>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>Self-hosted environment</AlertDialogTitle>
        </AlertDialogHeader>
        <form
          className="space-y-4"
          noValidate
          onSubmit={async (event) => {
            event.preventDefault();
            event.stopPropagation();
            const nextErrors: FieldErrors = {
              serverUrl: validateServerUrl(formValues.serverUrl) ?? undefined,
              webVaultUrl:
                validateOptionalUrl(formValues.webVaultUrl ?? "") ?? undefined,
              apiUrl: validateOptionalUrl(formValues.apiUrl ?? "") ?? undefined,
              identityUrl:
                validateOptionalUrl(formValues.identityUrl ?? "") ?? undefined,
              notificationsUrl:
                validateOptionalUrl(formValues.notificationsUrl ?? "") ??
                undefined,
              iconsUrl:
                validateOptionalUrl(formValues.iconsUrl ?? "") ?? undefined,
            };

            const hasError = Object.values(nextErrors).some(Boolean);
            if (hasError) {
              setErrors(nextErrors);
              return;
            }
            setErrors({});
            try {
              await onSave?.({
                ...formValues,
                serverUrl: formValues.serverUrl.trim(),
              });
            } catch {
              return;
            }
            onOpenChange(false);
          }}
        >
          <div className="space-y-2">
            <Label htmlFor="server-url">Server URL</Label>
            <Input
              id="server-url"
              name="server-url"
              type="text"
              placeholder="https://vault.company.com"
              value={formValues.serverUrl}
              onChange={(event) => {
                const nextValue = event.target.value;
                setFormValues((current) => ({
                  ...current,
                  serverUrl: nextValue,
                }));
                setErrors((current) => ({
                  ...current,
                  serverUrl: validateServerUrl(nextValue) ?? undefined,
                }));
              }}
              aria-invalid={Boolean(errors.serverUrl)}
              aria-describedby={
                errors.serverUrl ? "server-url-error" : "server-url-description"
              }
            />
            {errors.serverUrl ? (
              <p id="server-url-error" className="text-destructive text-xs">
                {errors.serverUrl}
              </p>
            ) : null}
            <p
              id="server-url-description"
              className="text-muted-foreground text-xs"
            >
              Specify the base URL of your on-premises hosted Bitwarden
              installation. Example: https://vault.company.com
            </p>
          </div>
          <details className="border-border/60 bg-muted/20 rounded-lg border p-3">
            <summary className="text-foreground cursor-pointer text-sm font-medium">
              Custom environment
            </summary>
            <p className="text-muted-foreground mt-2 text-xs">
              For advanced configuration, you can specify the base URL of each
              service independently.
            </p>
            <div className="mt-3 space-y-3">
              <div className="space-y-2">
                <Label htmlFor="web-vault-url">Web vault server URL</Label>
              <Input
                id="web-vault-url"
                name="web-vault-url"
                type="text"
                placeholder="https://vault.company.com"
                value={formValues.webVaultUrl}
                onChange={(event) => {
                  const nextValue = event.target.value;
                  setFormValues((current) => ({
                    ...current,
                    webVaultUrl: nextValue,
                  }));
                  setErrors((current) => ({
                    ...current,
                    webVaultUrl:
                      validateOptionalUrl(nextValue) ?? undefined,
                  }));
                }}
                aria-invalid={Boolean(errors.webVaultUrl)}
                aria-describedby={
                  errors.webVaultUrl ? "web-vault-url-error" : undefined
                }
              />
                {errors.webVaultUrl ? (
                  <p
                    id="web-vault-url-error"
                    className="text-destructive text-xs"
                  >
                    {errors.webVaultUrl}
                  </p>
                ) : null}
              </div>
              <div className="space-y-2">
                <Label htmlFor="api-url">API server URL</Label>
              <Input
                id="api-url"
                name="api-url"
                type="text"
                placeholder="https://api.company.com"
                value={formValues.apiUrl}
                onChange={(event) => {
                  const nextValue = event.target.value;
                  setFormValues((current) => ({
                    ...current,
                    apiUrl: nextValue,
                  }));
                  setErrors((current) => ({
                    ...current,
                    apiUrl: validateOptionalUrl(nextValue) ?? undefined,
                  }));
                }}
                aria-invalid={Boolean(errors.apiUrl)}
                aria-describedby={errors.apiUrl ? "api-url-error" : undefined}
              />
                {errors.apiUrl ? (
                  <p id="api-url-error" className="text-destructive text-xs">
                    {errors.apiUrl}
                  </p>
                ) : null}
              </div>
              <div className="space-y-2">
                <Label htmlFor="identity-url">Identity Server URL</Label>
              <Input
                id="identity-url"
                name="identity-url"
                type="text"
                placeholder="https://identity.company.com"
                value={formValues.identityUrl}
                onChange={(event) => {
                  const nextValue = event.target.value;
                  setFormValues((current) => ({
                    ...current,
                    identityUrl: nextValue,
                  }));
                  setErrors((current) => ({
                    ...current,
                    identityUrl:
                      validateOptionalUrl(nextValue) ?? undefined,
                  }));
                }}
                aria-invalid={Boolean(errors.identityUrl)}
                aria-describedby={
                  errors.identityUrl ? "identity-url-error" : undefined
                }
              />
                {errors.identityUrl ? (
                  <p
                    id="identity-url-error"
                    className="text-destructive text-xs"
                  >
                    {errors.identityUrl}
                  </p>
                ) : null}
              </div>
              <div className="space-y-2">
                <Label htmlFor="notifications-url">
                  Notifications server URL
                </Label>
              <Input
                id="notifications-url"
                name="notifications-url"
                type="text"
                placeholder="https://notifications.company.com"
                value={formValues.notificationsUrl}
                onChange={(event) => {
                  const nextValue = event.target.value;
                  setFormValues((current) => ({
                    ...current,
                    notificationsUrl: nextValue,
                  }));
                  setErrors((current) => ({
                    ...current,
                    notificationsUrl:
                      validateOptionalUrl(nextValue) ?? undefined,
                  }));
                }}
                aria-invalid={Boolean(errors.notificationsUrl)}
                aria-describedby={
                  errors.notificationsUrl
                    ? "notifications-url-error"
                    : undefined
                }
              />
                {errors.notificationsUrl ? (
                  <p
                    id="notifications-url-error"
                    className="text-destructive text-xs"
                  >
                    {errors.notificationsUrl}
                  </p>
                ) : null}
              </div>
              <div className="space-y-2">
                <Label htmlFor="icons-url">Icons server URL</Label>
              <Input
                id="icons-url"
                name="icons-url"
                type="text"
                placeholder="https://icons.company.com"
                value={formValues.iconsUrl}
                onChange={(event) => {
                  const nextValue = event.target.value;
                  setFormValues((current) => ({
                    ...current,
                    iconsUrl: nextValue,
                  }));
                  setErrors((current) => ({
                    ...current,
                    iconsUrl: validateOptionalUrl(nextValue) ?? undefined,
                  }));
                }}
                aria-invalid={Boolean(errors.iconsUrl)}
                aria-describedby={
                  errors.iconsUrl ? "icons-url-error" : undefined
                }
              />
                {errors.iconsUrl ? (
                  <p id="icons-url-error" className="text-destructive text-xs">
                    {errors.iconsUrl}
                  </p>
                ) : null}
              </div>
            </div>
          </details>
          <AlertDialogFooter>
            <AlertDialogCancel type="button">Cancel</AlertDialogCancel>
            <Button type="submit">Save</Button>
          </AlertDialogFooter>
        </form>
      </AlertDialogContent>
    </AlertDialog>
  );
}

export type { SelfHostedConfig };
