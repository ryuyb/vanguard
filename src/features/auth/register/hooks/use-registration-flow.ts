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
import { toast } from "@/lib/toast";

export type RegistrationForm = ReturnType<typeof useRegistrationFlow>["form"];

type UseRegistrationFlowOptions = {
  onRegistrationComplete?: () => Promise<void>;
};

export function useRegistrationFlow(options?: UseRegistrationFlowOptions) {
  const [feedback, setFeedback] = useState<RegistrationFeedbackState>({
    kind: "idle",
  });
  const [submitProgressText, setSubmitProgressText] = useState("");
  const [isFinishingRegistration, setIsFinishingRegistration] = useState(false);

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
      setSubmitProgressText(
        appI18n.t("auth.register.progress.sendingVerification"),
      );

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
          toast.warning(
            appI18n.t("auth.register.messages.registrationDisabled.title"),
            {
              description:
                data.message ||
                appI18n.t(
                  "auth.register.messages.registrationDisabled.description",
                ),
            },
          );
        } else if (data.outcome === "emailVerificationRequired") {
          setFeedback({
            kind: "emailSent",
            email,
          });
        } else if (data.outcome === "directRegistration") {
          // Store token and show password setup form
          setFeedback({
            kind: "passwordSetup",
            token: data.token,
            email,
            name,
          });
        }
      } catch (error) {
        errorHandler.handle(error);
      } finally {
        setSubmitProgressText("");
      }
    },
  });

  const resetFeedback = () => setFeedback({ kind: "idle" });

  const finishRegistration = async (
    password: string,
    passwordHint?: string,
  ) => {
    if (feedback.kind !== "passwordSetup") {
      return;
    }

    const { token, email, name } = feedback;

    setIsFinishingRegistration(true);
    setSubmitProgressText(appI18n.t("auth.register.progress.creatingAccount"));

    try {
      // Get base URL from form
      const effectiveBaseUrl =
        form.state.values.serverUrlOption === CUSTOM_SERVER_URL_OPTION
          ? form.state.values.customBaseUrl
          : form.state.values.serverUrlOption;
      const baseUrl = normalizeBaseUrl(effectiveBaseUrl);

      // Call register_finish command
      const result = await commands.authRegisterFinish({
        baseUrl,
        email,
        name,
        masterPassword: password,
        masterPasswordHint: passwordHint || null,
        token,
        kdf: 0, // PBKDF2
        kdfIterations: 600000,
        kdfMemory: null,
        kdfParallelism: null,
      });

      if (result.status === "error") {
        errorHandler.handle(result.error);
        return;
      }

      // Registration and auto-login successful
      setSubmitProgressText(appI18n.t("auth.register.progress.loginSuccess"));

      // Navigate to vault
      if (options?.onRegistrationComplete) {
        await options.onRegistrationComplete();
      }
    } catch (error) {
      errorHandler.handle(error);
    } finally {
      setIsFinishingRegistration(false);
      setSubmitProgressText("");
    }
  };

  return {
    form,
    feedback,
    resetFeedback,
    submitProgressText,
    finishRegistration,
    isFinishingRegistration,
  };
}
