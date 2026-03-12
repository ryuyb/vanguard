import type { AppLocale } from "@/i18n";

export const TRANSLATION_NAMESPACES = [
  "common",
  "auth",
  "vault",
  "spotlight",
  "errors",
] as const;

export type TranslationNamespace = (typeof TRANSLATION_NAMESPACES)[number];

export type TranslationDictionary = {
  [key: string]: string | TranslationDictionary;
};

export interface AppTranslationCatalog {
  [namespace: string]: TranslationDictionary;
  common: {
    app: {
      name: string;
    };
    locale: {
      label: string;
      options: Record<AppLocale, string>;
    };
    actions: {
      cancel: string;
      confirm: string;
      close: string;
      save: string;
    };
    states: {
      loading: string;
      unavailable: string;
    };
  };
  auth: {
    login: TranslationDictionary;
    unlock: TranslationDictionary;
    feedback: TranslationDictionary;
  };
  vault: {
    settings: TranslationDictionary;
    page: TranslationDictionary;
    dialogs: TranslationDictionary;
    detail: TranslationDictionary;
    feedback: TranslationDictionary;
  };
  spotlight: {
    search: TranslationDictionary;
    hints: TranslationDictionary;
    actions: TranslationDictionary;
  };
  errors: {
    common: TranslationDictionary;
    auth: TranslationDictionary;
    vault: TranslationDictionary;
    validation: TranslationDictionary;
    network: TranslationDictionary;
    storage: TranslationDictionary;
    crypto: TranslationDictionary;
    internal: TranslationDictionary;
  };
}
