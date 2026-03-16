import { z } from "zod";
import { CUSTOM_SERVER_URL_OPTION } from "@/features/auth/login/constants";
import {
  isValidServerUrl,
  normalizeBaseUrl,
} from "@/features/auth/login/login-flow-helpers";

export const loginSchema = z
  .object({
    serverUrlOption: z.string(),
    customBaseUrl: z.string(),
    email: z.string(),
    masterPassword: z.string(),
    twoFactorToken: z.string().optional().default(""),
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
      message: "auth.login.validation.missingCredentials",
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
      message: "auth.login.validation.invalidServerUrl",
      path: ["customBaseUrl"],
    },
  )
  .refine((data) => data.email.trim().length > 0, {
    message: "auth.login.validation.missingCredentials",
    path: ["email"],
  })
  .refine(
    (data) => {
      const trimmed = data.email.trim();
      return trimmed.length === 0 || trimmed.includes("@");
    },
    {
      message: "auth.login.validation.invalidEmail",
      path: ["email"],
    },
  )
  .refine((data) => data.masterPassword.length > 0, {
    message: "auth.login.validation.missingCredentials",
    path: ["masterPassword"],
  });

export type LoginFormValues = z.input<typeof loginSchema>;

export const loginFormDefaults: LoginFormValues = {
  serverUrlOption: CUSTOM_SERVER_URL_OPTION,
  customBaseUrl: "",
  email: "",
  masterPassword: "",
  twoFactorToken: "",
};
