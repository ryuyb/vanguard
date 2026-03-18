export type RegistrationFeedbackState =
  | { kind: "idle" }
  | { kind: "emailSent"; text: string }
  | { kind: "directRegistration"; text: string };
