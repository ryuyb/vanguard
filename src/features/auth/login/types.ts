export type LoginFeedback =
  | { kind: "idle" }
  | { kind: "success"; text: string }
  | { kind: "twoFactor"; text: string }
  | { kind: "error"; text: string };

export type TwoFactorState = {
  providers: string[];
  selectedProvider: string;
  token: string;
  isSendingEmailCode: boolean;
};
