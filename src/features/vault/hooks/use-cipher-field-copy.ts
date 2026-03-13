import { useState } from "react";
import { toast } from "sonner";
import { commands } from "@/bindings";
import { appI18n } from "@/i18n";
import { errorHandler } from "@/lib/error-handler";

type CopyableField =
  | "username"
  | "password"
  | "totp"
  | "notes"
  | { customField: number }
  | { uri: number }
  | "cardNumber"
  | "cardCode"
  | "email"
  | "phone"
  | "sshPrivateKey"
  | "sshPublicKey";

export function useCipherFieldCopy(cipherId: string) {
  const [copiedField, setCopiedField] = useState<string | null>(null);

  const copyField = async (field: CopyableField) => {
    // 转换为后端 API 格式
    const apiField =
      typeof field === "string"
        ? field
        : "customField" in field
          ? { customField: { index: field.customField } }
          : { uri: { index: field.uri } };

    const result = await commands.vaultCopyCipherField({
      cipherId,
      field: apiField,
      clearAfterMs: null,
    });

    if (result.status === "ok") {
      const fieldKey = JSON.stringify(field);
      setCopiedField(fieldKey);
      toast.success(appI18n.t("vault.feedback.copiedToClipboard"));
      setTimeout(() => setCopiedField(null), 1500);
    } else {
      errorHandler.handle(result.error);
    }
  };

  return { copyField, copiedField };
}
