import { useForm } from "@tanstack/react-form";
import { useState } from "react";
import { commands } from "@/bindings";
import { CUSTOM_SERVER_URL_OPTION } from "@/features/auth/login/constants";
import { normalizeBaseUrl } from "@/features/auth/login/login-flow-helpers";
import {
  registerFormDefaults,
  registerSchema,
} from "@/features/auth/register/schema";
import type { RegistrationFeedbackState } from "@/features/auth/register/types";
import { appI18n } from "@/i18n";
import { errorHandler } from "@/lib/error-handler";

export type RegistrationForm = ReturnType<typeof useRegistrationFlow>["form"];

export function useRegistrationFlow() {
  const [feedback, setFeedback] = useState<RegistrationFeedbackState>({ kind: "idle" });
  const [submitProgressText, setSubmitProgressText] = useState("");

  const form = useForm({
    defaultValues: registerFormDefaults,
    validators: { onSubmit: registerSchema },
    onSubmit: async ({ value }) => {
      const effectiveBaseUrl =
        value.serverUrlOption === CUSTOM_SERVER_URL_OPTION
          ? value.customBaseUrl
          : value.serverUrlOption;
      const baseUrl = normalizeBaseUrl(effectiveBaseUrl);
      const email = value.email.trim();
      const name = value.name.trim();

      setFeedback({ kind: "idle" });
      setSubmitProgressText(appI18n.t("auth.register.progress.sendingVerification"));

      try {
        const result = await commands.authSendVerificationEmail({
          baseUrl,
          email,
          name: name || null,
        });

        if (result.status === "error") {
          errorHandler.handle(result.error);
          return;
        }

        const data = result.data;

        if (data.outcome === "disabled") {
          setFeedback({
            kind: "disabled",
            text: data.message || appI18n.t("auth.register.messages.registrationDisabled.description"),
          });
        } else if (data.outcome === "emailVerificationRequired") {
          setFeedback({
            kind: "emailSent",
            text: appI18n.t("auth.register.messages.emailVerificationRequired.description"),
          });
        } else if (data.outcome === "directRegistration") {
          // Phase 2 placeholder: direct registration with token
          setFeedback({
            kind: "directRegistration",
            text: appI18n.t("errors.internal.notImplemented.description"),
          });
        }
      } catch (error) {
        errorHandler.handle(error);
      } finally {
        setSubmitProgressText("");
      }
    },
  });

  return { form, feedback, submitProgressText };
}
