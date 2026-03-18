export type RegistrationFeedbackState =
  | { kind: "idle" }
  | { kind: "error"; text: string }
  | { kind: "disabled"; text: string }
  | { kind: "emailSent"; text: string }
  | { kind: "directRegistration"; text: string };
