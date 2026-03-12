import type { Resource } from "i18next";
import type { AppLocale } from "@/i18n";
import { enTranslationCatalog } from "./en";
import {
  type AppTranslationCatalog,
  TRANSLATION_NAMESPACES,
  type TranslationNamespace,
} from "./types";
import { zhTranslationCatalog } from "./zh";

export type TranslationResourceTree = Record<AppLocale, AppTranslationCatalog>;

export const DEFAULT_TRANSLATION_NAMESPACE: TranslationNamespace = "common";

export const translationResources = {
  zh: zhTranslationCatalog,
  en: enTranslationCatalog,
} satisfies TranslationResourceTree & Resource;

export { TRANSLATION_NAMESPACES };
export type { AppTranslationCatalog, TranslationNamespace };
