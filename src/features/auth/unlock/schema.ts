import { z } from "zod";

export const unlockSchema = z.object({
  masterPassword: z.string(),
  pin: z.string(),
});

export type UnlockFormValues = z.input<typeof unlockSchema>;

export const unlockFormDefaults: UnlockFormValues = {
  masterPassword: "",
  pin: "",
};
