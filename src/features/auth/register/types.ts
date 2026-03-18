export type RegistrationFeedbackState =
  | { kind: "idle" }
  | { kind: "emailSent"; email: string }
  | { kind: "directRegistration"; text: string };
