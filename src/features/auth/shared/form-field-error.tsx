import { useTranslation } from "react-i18next";

export function FormFieldError({ errors }: { errors: unknown[] }) {
  const { t } = useTranslation();
  if (!errors.length) return null;

  const raw = errors[0];
  const message =
    typeof raw === "string"
      ? raw
      : raw && typeof raw === "object" && "message" in raw
        ? String((raw as { message: unknown }).message)
        : null;
  if (!message) return null;

  return (
    <p role="alert" className="text-destructive text-sm">
      {t(message, { defaultValue: message })}
    </p>
  );
}
