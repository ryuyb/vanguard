export type UnlockFeedback =
  | { kind: "idle" }
  | { kind: "success"; text: string }
  | { kind: "error"; text: string };
