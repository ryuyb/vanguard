import { z } from "zod";
import { CUSTOM_SERVER_URL_OPTION } from "@/features/auth/login/constants";
import {
  isValidServerUrl,
  normalizeBaseUrl,
} from "@/features/auth/login/login-flow-helpers";

export const registerSchema = z
  .object({
    serverUrlOption: z.string(),
    customBaseUrl: z.string(),
    email: z.string(),
    name: z.string(),
  })
  .refine(
    (data) => {
      const baseUrl =
        data.serverUrlOption === CUSTOM_SERVER_URL_OPTION
          ? data.customBaseUrl
          : data.serverUrlOption;
      return normalizeBaseUrl(baseUrl).length > 0;
    },
    {
      message: "auth.register.validation.missingServerUrl",
      path: ["serverUrlOption"],
    },
  )
  .refine(
    (data) => {
      const baseUrl =
        data.serverUrlOption === CUSTOM_SERVER_URL_OPTION
          ? data.customBaseUrl
          : data.serverUrlOption;
      const normalized = normalizeBaseUrl(baseUrl);
      return normalized.length === 0 || isValidServerUrl(normalized);
    },
    {
      message: "auth.register.validation.invalidServerUrl",
      path: ["customBaseUrl"],
    },
  )
  .refine((data) => data.email.trim().length > 0, {
    message: "auth.register.validation.missingEmail",
    path: ["email"],
  })
  .refine(
    (data) => {
      const trimmed = data.email.trim();
      return trimmed.length === 0 || trimmed.includes("@");
    },
    {
      message: "auth.register.validation.invalidEmail",
      path: ["email"],
    },
  )
  .refine((data) => data.name.trim().length > 0, {
    message: "auth.register.validation.missingName",
    path: ["name"],
  });

export type RegisterFormValues = z.input<typeof registerSchema>;

export const registerFormDefaults: RegisterFormValues = {
  serverUrlOption: CUSTOM_SERVER_URL_OPTION,
  customBaseUrl: "",
  email: "",
  name: "",
};
