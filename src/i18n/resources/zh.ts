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
    common: {
      unknown: {
        title: "未知错误",
        description: "发生未知错误,请重试",
      },
      action: "操作",
    },
    auth: {
      invalidCredentials: {
        title: "登录失败",
        description: "用户名或密码错误,请检查后重试",
      },
      tokenExpired: {
        title: "会话已过期",
        description: "请重新登录",
        action: "重新登录",
      },
      tokenInvalid: {
        title: "认证失败",
        description: "认证信息无效,请重新登录",
        action: "重新登录",
      },
      permissionDenied: {
        title: "权限不足",
        description: "您没有权限执行此操作",
      },
      accountLocked: {
        title: "账户已锁定",
        description: "您的账户已被锁定,请联系管理员",
      },
      twoFactorRequired: {
        title: "需要两步验证",
        description: "请输入两步验证码",
      },
      invalidPin: {
        title: "PIN 码错误",
        description: "PIN 码不正确,请重新输入",
      },
    },
    vault: {
      cipherNotFound: {
        title: "密码项不存在",
        description: "未找到该密码项,可能已被删除",
      },
      decryptionFailed: {
        title: "解密失败",
        description: "无法解密数据,请检查主密码是否正确",
      },
      syncConflict: {
        title: "同步冲突",
        description: "检测到数据冲突,请手动解决",
      },
      locked: {
        title: "保险库已锁定",
        description: "请先解锁保险库",
        action: "解锁",
      },
      corrupted: {
        title: "数据损坏",
        description: "保险库数据已损坏,请联系技术支持",
      },
    },
    validation: {
      fieldError: {
        title: "输入有误",
        description: "请检查输入的数据是否正确",
      },
      formatError: {
        title: "格式错误",
        description: "数据格式不正确,请重新输入",
      },
      required: {
        title: "缺少必填项",
        description: "请填写所有必填字段",
      },
    },
    network: {
      connectionFailed: {
        title: "网络连接失败",
        description: "无法连接到服务器,请检查网络连接",
      },
      timeout: {
        title: "请求超时",
        description: "服务器响应超时,请稍后重试",
      },
      remoteError: {
        title: "服务器错误",
        description: "服务器返回错误,请稍后重试",
      },
      dnsResolutionFailed: {
        title: "无法连接",
        description: "无法解析服务器地址,请检查网络设置",
      },
    },
    storage: {
      databaseError: {
        title: "数据保存失败",
        description: "无法保存数据,请重试",
      },
      fileNotFound: {
        title: "文件未找到",
        description: "请求的文件不存在",
      },
      permissionDenied: {
        title: "权限不足",
        description: "无权限访问该文件",
      },
    },
    crypto: {
      keyDerivationFailed: {
        title: "密钥生成失败",
        description: "无法生成加密密钥",
      },
      encryptionFailed: {
        title: "加密失败",
        description: "数据加密失败,请重试",
      },
      decryptionFailed: {
        title: "解密失败",
        description: "无法解密数据,请检查密码",
      },
      invalidKey: {
        title: "密钥无效",
        description: "加密密钥无效,无法继续操作",
      },
    },
    internal: {
      unexpected: {
        title: "意外错误",
        description: "发生意外错误,请联系技术支持",
      },
      notImplemented: {
        title: "功能未实现",
        description: "该功能暂未实现",
      },
    },
  },
};
