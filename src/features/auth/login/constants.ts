export const CUSTOM_SERVER_URL_OPTION = "__custom__";

export const SERVER_URL_OPTIONS = [
  {
    value: "https://bitwarden.com",
    label: "Bitwarden.com",
  },
  {
    value: "https://bitwarden.eu",
    label: "Bitwarden.eu",
  },
] as const;

export const TWO_FACTOR_PROVIDER_LABELS: Record<string, string> = {
  "0": "Authenticator",
  "1": "Email",
  "2": "Duo",
  "3": "YubiKey",
  "5": "Remember",
  "7": "WebAuthn",
  "8": "Recovery Code",
};
