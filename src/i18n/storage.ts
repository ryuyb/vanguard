import { commands } from "@/bindings";
import { type AppLocale, resolveAppLocale } from "./locales";

export async function loadSavedLocale(): Promise<AppLocale | null> {
  try {
    const result = await commands.configGetAppConfig();
    if (result.status === "error") {
      return null;
    }
    return resolveAppLocale(result.data.locale);
  } catch {
    return null;
  }
}

export async function saveLocale(locale: AppLocale): Promise<void> {
  try {
    await commands.configUpdateAppConfig({ locale });
  } catch {
    // Ignore storage errors
  }
}
