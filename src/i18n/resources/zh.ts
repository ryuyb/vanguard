import type { AppTranslationCatalog } from "./types";

export const zhTranslationCatalog: AppTranslationCatalog = {
  common: {
    app: {
      name: "Vanguard",
    },
    locale: {
      label: "语言",
      options: {
        zh: "中文",
        en: "English",
      },
    },
    actions: {
      cancel: "取消",
      confirm: "确认",
      close: "关闭",
      save: "保存",
    },
    states: {
      loading: "加载中...",
      unavailable: "暂不可用",
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
