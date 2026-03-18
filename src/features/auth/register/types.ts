export type RegistrationFeedbackState =
  | { kind: "idle" }
  | { kind: "emailSent"; email: string }
  | { kind: "passwordSetup"; token: string; email: string; name: string };
