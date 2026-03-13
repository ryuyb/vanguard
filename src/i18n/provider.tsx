import type { PropsWithChildren } from "react";
import { I18nextProvider } from "react-i18next";
import { appI18n } from "./app-i18n";

export function AppLocaleProvider({ children }: PropsWithChildren) {
  return <I18nextProvider i18n={appI18n}>{children}</I18nextProvider>;
}
