import type { AppTranslationCatalog } from "./types";

export const enTranslationCatalog: AppTranslationCatalog = {
  common: {
    app: {
      name: "Vanguard",
    },
    locale: {
      label: "Language",
      options: {
        zh: "中文",
        en: "English",
      },
    },
    actions: {
      cancel: "Cancel",
      confirm: "Confirm",
      close: "Close",
      save: "Save",
    },
    states: {
      loading: "Loading...",
      unavailable: "Unavailable",
    },
  },
  auth: {
    login: {},
    unlock: {},
    feedback: {},
  },
  vault: {
    settings: {},
    page: {},
    dialogs: {},
    detail: {},
    feedback: {},
  },
  spotlight: {
    search: {},
    hints: {},
    actions: {},
  },
  errors: {
    common: {},
    auth: {},
    vault: {},
    validation: {},
    network: {},
    storage: {},
    crypto: {},
    internal: {},
  },
};
