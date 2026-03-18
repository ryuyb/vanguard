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
    login: {
      title: "登录 Vanguard",
      subtitle: "输入凭据以访问你的密码库",
      hero: {
        badge: "Vanguard 密码库",
        title: "欢迎回来,继续管理你的密码库",
        description:
          "输入服务地址、邮箱和主密码后,即可完成登录并自动准备好你的密码库。",
        illustrationAlt: "密码库登录插图",
      },
      form: {
        serverUrl: {
          label: "服务器地址",
          placeholder: "选择服务地址",
          customOption: "自定义地址",
          customPlaceholder: "https://vault.example.com",
        },
        email: {
          label: "邮箱地址",
          placeholder: "you@example.com",
        },
        masterPassword: {
          label: "主密码",
          placeholder: "输入主密码",
          showPassword: "显示密码",
          hidePassword: "隐藏密码",
        },
        twoFactor: {
          title: "二步验证",
          provider: {
            label: "验证方式",
            placeholder: "选择验证方式",
            providers: {
              "0": "验证器",
              "1": "邮箱",
              "2": "Duo",
              "3": "YubiKey",
              "5": "记住我",
              "7": "WebAuthn",
              "8": "恢复代码",
              unknown: "方式 {{provider}}",
            },
          },
          token: {
            label: "二步验证码",
            placeholder: "输入验证码",
          },
          sendEmail: "发送邮箱验证码",
          sendingEmail: "正在发送邮箱验证码...",
        },
      },
      actions: {
        submit: "登录",
        submitting: "正在登录...",
        verifyAndContinue: "验证后继续",
      },
      states: {
        checkingSession: "正在检查会话...",
      },
      validation: {
        missingCredentials: "请先填写服务地址、登录邮箱和主密码。",
        missingServerUrl: "请输入服务器地址。",
        missingEmail: "请输入邮箱地址。",
        missingPassword: "请输入主密码。",
        invalidServerUrl: "服务地址格式不正确,请以 http:// 或 https:// 开头。",
        invalidEmail: "邮箱格式看起来不正确,请检查后重试。",
        incompleteTwoFactor: "请输入完整的二步验证码后再继续。",
        nonEmailProvider: "当前不是邮箱验证方式,无法发送邮件验证码。",
        missingEmailCodeRequirements:
          "发送验证码前,请先填写服务地址、登录邮箱和主密码。",
      },
      progress: {
        verifyingAccount: "正在验证账号信息...",
        preparingVault: "正在准备你的密码库...",
        unlockingLocalVault: "正在解锁本地密码库...",
        syncingLatestData: "正在同步最新数据...",
        firstSync: "正在首次同步密码库...",
        finishingUnlock: "正在完成解锁...",
      },
      messages: {
        loginFailed: "登录失败,请稍后重试。",
        twoFactorPrompt:
          "需要二步验证,请输入验证码继续（可用方式：{{providers}}）。",
        unknownTwoFactorProvider: "未知方式",
        emailCodeSent: "验证码已发送到邮箱,请查收后输入并继续登录。",
      },
    },
    unlock: {
      title: "解锁 Vanguard",
      subtitle: "输入凭据以访问你的密码库",
      hero: {
        badge: "密码库解锁",
        title: "会话已锁定,请输入主密码解锁",
        description: "输入主密码后即可解锁,继续安全访问你的密码库。",
        illustrationAlt: "密码库锁定插图",
      },
      form: {
        account: {
          label: "账户",
          unknown: "未知",
        },
        server: {
          label: "服务器",
          unknown: "未知",
        },
        pin: {
          label: "PIN 码",
          placeholder: "输入 PIN 解锁",
        },
        masterPassword: {
          label: "主密码",
          placeholder: "输入主密码解锁",
          showPassword: "显示密码",
          hidePassword: "隐藏密码",
        },
      },
      actions: {
        unlock: "解锁",
        unlocking: "正在解锁...",
        unlockWithPin: "使用 PIN 解锁",
        unlockingWithPin: "正在解锁...",
        switchToMasterPassword: "改用主密码解锁",
        switchToPin: "改用 PIN 解锁",
        biometric: "使用生物识别",
        biometricVerifying: "正在验证...",
        logout: "登出账户",
        loggingOut: "正在登出...",
        goToLogin: "前往登录",
      },
      states: {
        checkingSession: "正在检查会话状态...",
        unlocked: "密码库已解锁,正在跳转...",
        needsLogin: {
          title: "需要登录",
          description: "当前没有可解锁的会话,请先登录。",
        },
        biometricUnavailable: {
          title: "生物识别不可用",
          description:
            "当前设备还没有可用于解锁的本地数据,请先用密码解锁并完成同步。",
        },
      },
      validation: {
        sessionNotLocked: "当前会话不是锁定状态,无法执行解锁。",
        missingMasterPassword: "请输入主密码。",
        pinNotEnabled: "当前账号未启用 PIN 解锁,请使用主密码解锁。",
        missingPin: "请输入 PIN。",
        sessionNotLockedBiometric:
          "当前会话不是锁定状态,无法执行生物识别解锁。",
      },
      messages: {
        unlockFailed: "解锁失败,请稍后重试。",
      },
    },
    register: {
      title: "创建 Vanguard 账户",
      subtitle: "注册新账户以开始使用密码库",
      hero: {
        badge: "账户注册",
        title: "创建新账户,开始管理你的密码",
        description: "输入服务器地址和邮箱信息,即可创建新的密码库账户。",
        illustrationAlt: "账户注册插图",
      },
      form: {
        serverUrl: {
          label: "服务器地址",
          placeholder: "选择服务地址",
          customOption: "自定义地址",
          customPlaceholder: "https://vault.example.com",
        },
        email: {
          label: "邮箱地址",
          placeholder: "you@example.com",
        },
        name: {
          label: "姓名",
          placeholder: "输入你的姓名",
        },
        masterPassword: {
          label: "主密码",
          placeholder: "创建主密码",
          showPassword: "显示密码",
          hidePassword: "隐藏密码",
        },
        confirmPassword: {
          label: "确认密码",
          placeholder: "再次输入主密码",
        },
        passwordHint: {
          label: "密码提示（可选）",
          placeholder: "帮助你记住密码的提示",
        },
      },
      actions: {
        submit: "创建账户",
        submitting: "正在创建账户...",
        backToLogin: "返回登录",
        finishRegistration: "完成注册",
        finishing: "正在完成注册...",
      },
      passwordSetup: {
        title: "设置主密码",
        subtitle: "为 {{email}} 创建主密码",
      },
      emailVerification: {
        title: "检查您的电子邮箱",
        description:
          "点击发送到 {{email}} 的电子邮件中的链接，然后继续创建您的账户。",
        notReceived: "没有收到电子邮件吗？",
        editEmail: "返回编辑您的电子邮箱地址。",
        backToLogin: "返回登录页面",
      },
      validation: {
        missingServerUrl: "请输入服务器地址。",
        invalidServerUrl:
          "服务器地址格式不正确,请以 http:// 或 https:// 开头。",
        missingEmail: "请输入邮箱地址。",
        invalidEmail: "邮箱格式看起来不正确,请检查后重试。",
        missingName: "请输入姓名。",
        missingPassword: "请输入主密码。",
        passwordTooShort: "主密码至少需要 8 个字符。",
        passwordMismatch: "两次输入的密码不一致。",
      },
      progress: {
        creatingAccount: "正在创建账户...",
        sendingVerification: "正在发送验证邮件...",
        generatingKeys: "正在生成加密密钥...",
        finishingRegistration: "正在完成注册...",
        loggingIn: "正在登录...",
        loginSuccess: "登录成功",
      },
      messages: {
        registrationDisabled: {
          title: "注册已关闭",
          description: "该服务器不允许新用户注册,请联系管理员。",
        },
        emailVerificationRequired: {
          title: "需要邮箱验证",
          description:
            "验证邮件已发送到你的邮箱,请查收并点击邮件中的链接完成注册。完成后请返回登录页面登录。",
        },
        registrationFailed: "注册失败,请稍后重试。",
        passwordStrength: {
          weak: "弱",
          fair: "一般",
          good: "良好",
          strong: "强",
        },
        passwordCompromised: "该密码已在数据泄露中出现,建议使用其他密码。",
        checkingPassword: "正在检查密码安全性...",
      },
    },
    feedback: {
      login: {
        error: "登录失败",
        success: "登录成功",
        twoFactorRequired: "需要二步验证",
      },
      unlock: {
        error: "验证失败",
      },
      register: {
        error: "注册失败",
        success: "注册成功",
      },
    },
  },
  vault: {
    settings: {},
    page: {
      user: {
        notSignedIn: "未登录",
        unknownService: "未知服务",
      },
      actions: {
        create: "创建",
        rename: "重命名",
        delete: "删除",
        edit: "编辑",
        settings: "设置",
        lock: "锁定",
        locking: "正在锁定...",
        logout: "登出",
        loggingOut: "正在登出...",
        restore: "恢复",
        restoring: "恢复中...",
        permanentDelete: "彻底删除",
      },
      cipher: {
        create: "创建条目",
        untitled: "未命名条目",
        contextMenu: {
          view: "查看",
          edit: "编辑",
          clone: "克隆",
        },
        cloneSuffix: "副本",
      },
      filters: {
        ariaLabel: "筛选条目",
        types: {
          all: "全部类型",
          login: "登录",
          card: "卡片",
          identity: "身份",
          note: "安全笔记",
          sshKey: "SSH 密钥",
        },
      },
      folders: {
        title: "文件夹",
        untitledFolder: "未命名文件夹",
        expandFolder: "展开文件夹",
        collapseFolder: "折叠文件夹",
      },
      menus: {
        allItems: "全部条目",
        favorites: "收藏",
        trash: "回收站",
        noFolder: "无文件夹",
        unknownFolder: "未知文件夹",
      },
      search: {
        placeholder: "搜索密码库",
        inlinePlaceholder: "在 {{menu}} 中搜索",
        close: "关闭搜索",
      },
      sort: {
        ariaLabel: "排序条目",
        byLabel: "排序字段",
        by: {
          title: "标题",
          created: "创建时间",
          modified: "修改时间",
        },
        directionLabel: "排序方向",
        direction: {
          alphaAsc: "A 到 Z",
          alphaDesc: "Z 到 A",
          newestFirst: "最新优先",
          oldestFirst: "最早优先",
        },
      },
      states: {
        loading: "正在加载密码库...",
        loadError: "加载密码库数据失败。",
        emptyFiltered: "当前筛选条件下没有条目。",
        loadingCipherDetail: "正在加载条目详情...",
      },
    },
    dialogs: {
      folder: {
        createTitle: "创建文件夹",
        createDescription: "创建一个新文件夹来整理条目。",
        createSubFolderTitle: "创建子文件夹",
        createSubFolderDescription:
          "将在 {{parentFolderName}} 下创建子文件夹。",
        renameTitle: "重命名文件夹",
        renameDescription: "更新文件夹名称。",
        fullPathLabel: "完整路径：",
        folderNameLabel: "文件夹名称",
        subFolderNameLabel: "子文件夹名称",
        folderNamePlaceholder: "输入文件夹名称",
        subFolderNamePlaceholder: "输入子文件夹名称",
        processing: "处理中...",
      },
      deleteFolder: {
        title: "删除文件夹",
        descriptionPrefix: "确定要删除",
        descriptionSuffix: "吗？",
        descriptionHint: "该文件夹内的条目不会被删除，但将不再属于这个文件夹。",
        deleting: "删除中...",
      },
      deleteCipher: {
        title: "删除条目",
        descriptionPrefix: "确定要删除",
        descriptionSuffix: "吗？",
        deleting: "删除中...",
      },
      restoreCipher: {
        title: "恢复条目",
        descriptionPrefix: "确定要恢复",
        descriptionSuffix: "吗？",
        confirming: "恢复中...",
      },
      permanentDeleteCipher: {
        title: "彻底删除条目",
        descriptionPrefix: "确定要彻底删除",
        descriptionSuffix: "吗？此操作不可撤销。",
        confirming: "删除中...",
      },
      cipherForm: {
        createTitle: "创建条目",
        createDescription: "向密码库添加新条目。",
        editTitle: "编辑条目",
        editDescription: "更新条目详情。",
        fields: {
          type: "类型",
          name: "名称",
          folder: "文件夹",
          username: "用户名",
          password: "密码",
          totp: "一次性密码（TOTP）",
          uris: "网站地址",
          cardholderName: "持卡人姓名",
          cardNumber: "卡号",
          cardBrand: "卡品牌",
          expMonth: "到期月",
          expYear: "到期年",
          securityCode: "安全码",
          sshPrivateKey: "SSH 私钥",
          sshPublicKey: "SSH 公钥",
          sshFingerprint: "SSH 指纹",
          notes: "备注",
          customFields: "自定义字段",
        },
        placeholders: {
          name: "输入条目名称",
          username: "输入用户名",
          password: "输入密码",
          totp: "输入 TOTP 密钥",
          uri: "https://example.com",
          notes: "添加备注",
          cardholderName: "卡片上的姓名",
          cardBrand: "选择卡品牌",
          cardNumber: "1234 5678 9012 3456",
          month: "月份",
          year: "年份",
          securityCode: "CVV/CVC",
          sshPrivateKey: "-----BEGIN OPENSSH PRIVATE KEY-----",
          sshPublicKey: "ssh-rsa AAAA...",
          sshFingerprint: "SHA256:...",
          customFieldName: "字段名",
          customFieldValue: "字段值",
          hiddenValue: "隐藏值",
          linkValue: "关联值",
          booleanValue: "选择 true 或 false",
        },
        types: {
          login: "登录",
          note: "安全笔记",
          card: "卡片",
          sshKey: "SSH 密钥",
        },
        brands: {
          visa: "Visa",
          mastercard: "Mastercard",
          americanExpress: "American Express",
          discover: "Discover",
          jcb: "JCB",
          unionPay: "银联",
          other: "其他",
        },
        customFieldTypes: {
          text: "文本",
          hidden: "隐藏",
          boolean: "布尔",
          linked: "关联",
        },
        actions: {
          addUri: "添加地址",
          addCustomField: "添加自定义字段",
          creating: "创建中...",
          saving: "保存中...",
        },
        validation: {
          nameRequired: "名称不能为空。",
        },
        noFolder: "无文件夹",
      },
      settings: {
        title: "密码库设置",
        description: "管理密码库的通用偏好和安全行为。",
        sections: {
          general: "通用",
          security: "安全",
        },
        general: {
          title: "通用",
          description: "设置外观和基础行为偏好。",
          launchOnLogin: "开机自动启动",
          showWebsiteIcon: "显示网站图标",
          shortcuts: {
            title: "快捷键",
            quickAccess: "快速访问快捷键",
            lock: "锁定密码库快捷键",
            unset: "未设置",
            inputHint: "按下按键以记录快捷键",
            clear: "清除",
            clearQuickAccess: "清除快速访问快捷键",
            clearLock: "清除锁定快捷键",
            keys: {
              space: "空格",
              esc: "Esc",
              up: "上",
              down: "下",
              left: "左",
              right: "右",
            },
          },
        },
        security: {
          unlock: {
            title: "解锁",
            description: "配置解锁方式。",
          },
          biometric: {
            label: "生物识别解锁",
            enabledHint: "使用设备生物识别进行解锁。",
            checkingHint: "正在检查生物识别可用性...",
          },
          pin: {
            label: "PIN 解锁",
            enabledHint: "使用 PIN 快速解锁。",
            unsupportedHint: "当前设备不支持 PIN 解锁。",
          },
          requireMasterPassword: "重新要求主密码的时间",
          autoLock: {
            title: "自动锁定",
            description: "在空闲后自动锁定密码库。",
          },
          lockOnSleep: "设备休眠时锁定",
          idleLockDelay: "空闲后锁定时间",
          clipboard: {
            title: "剪贴板",
            description: "控制复制敏感信息后的清除行为。",
            clearAfter: "复制后清除时间",
          },
        },
        placeholders: {
          language: "选择语言",
          requireMasterPassword: "选择重新要求主密码的时间",
          autoLockIdle: "选择空闲锁定时间",
          clipboardClear: "选择剪贴板清除时间",
        },
        options: {
          requireMasterPassword: {
            "1d": "1 天",
            "7d": "7 天",
            "14d": "14 天",
            "30d": "30 天",
            never: "永不",
          },
          autoLockIdle: {
            "1m": "1 分钟",
            "2m": "2 分钟",
            "5m": "5 分钟",
            "10m": "10 分钟",
            "15m": "15 分钟",
            "30m": "30 分钟",
            "1h": "1 小时",
            "4h": "4 小时",
            "8h": "8 小时",
            never: "永不",
          },
          clipboardClear: {
            "10s": "10 秒",
            "20s": "20 秒",
            "30s": "30 秒",
            "1m": "1 分钟",
            "2m": "2 分钟",
            "5m": "5 分钟",
            never: "永不",
          },
        },
        pinDialog: {
          title: "启用 PIN 解锁",
          description: "设置一个便于记忆且安全的 PIN 用于快速解锁。",
          pinPlaceholder: "输入 PIN",
          enabling: "启用中...",
        },
        errors: {
          loadBiometricStatus: "加载生物识别状态失败。",
          loadPinStatus: "加载 PIN 状态失败。",
          loadSecuritySettings: "加载安全设置失败。",
          enableBiometric: "启用生物识别解锁失败。",
          disableBiometric: "禁用生物识别解锁失败。",
          enablePin: "启用 PIN 解锁失败。",
          disablePin: "禁用 PIN 解锁失败。",
          pinRequired: "请输入 PIN。",
          saveFailed: "保存设置失败。",
        },
      },
    },
    detail: {
      unknown: "未知",
      fields: {
        username: "用户名",
        password: "密码",
        oneTimePassword: "一次性密码",
        passkey: "通行密钥",
        organization: "组织",
        uris: "地址",
        notes: "备注",
        customFields: "自定义字段",
      },
      actions: {
        showPassword: "显示密码",
        hidePassword: "隐藏密码",
        showFieldValue: "显示字段值",
        hideFieldValue: "隐藏字段值",
      },
      boolean: {
        true: "是",
        false: "否",
      },
      customFields: {
        unnamedField: "未命名字段",
        emptyValue: "（空）",
      },
      passkey: {
        createdAt: "创建于 {{date}}",
      },
      timeline: {
        lastEdited: "最后编辑",
        created: "创建时间",
        passwordUpdated: "密码更新时间",
        passkeyCreated: "通行密钥创建时间",
        archived: "归档时间",
        deleted: "删除时间",
        lastEditedWithValue: "最后编辑：{{date}}",
        empty: "暂无时间线数据。",
      },
      totp: {
        countdownAria: "TOTP 倒计时",
        countdownTitle: "当前 TOTP 代码剩余有效时间",
      },
    },
    feedback: {
      loadError: "加载密码库数据失败，请重试。",
      copiedToClipboard: "已复制到剪贴板",
      iconAlt: "{{name}} 图标",
      iconAltFallback: "密码库条目图标",
      folder: {
        createSuccess: {
          title: "文件夹已创建",
          description: "已创建“{{name}}”。",
        },
        createError: {
          title: "创建文件夹失败",
          description: "无法创建文件夹，请重试。",
        },
        renameSuccess: {
          title: "文件夹已重命名",
          description: "文件夹已重命名为“{{name}}”。",
        },
        renameError: {
          title: "重命名文件夹失败",
          description: "无法重命名文件夹，请重试。",
        },
        deleteSuccess: {
          title: "文件夹已删除",
          description: "已删除“{{name}}”。",
        },
        deleteError: {
          title: "删除文件夹失败",
          description: "无法删除文件夹，请重试。",
        },
      },
      cipher: {
        createSuccess: {
          title: "条目已创建",
          description: "已创建“{{name}}”。",
        },
        createError: {
          title: "创建条目失败",
          description: "无法创建条目，请重试。",
        },
        saveSuccess: {
          title: "条目已保存",
          description: "已更新“{{name}}”。",
        },
        saveError: {
          title: "保存条目失败",
          description: "无法保存条目，请重试。",
        },
        deleteSuccess: {
          title: "条目已删除",
          description: "已删除“{{name}}”。",
        },
        deleteError: {
          title: "删除条目失败",
          description: "无法删除条目，请重试。",
        },
        restoreSuccess: {
          title: "条目已恢复",
          description: "已恢复\u201c{{name}}\u201d。",
        },
        restoreError: {
          title: "恢复条目失败",
          description: "无法恢复条目，请重试。",
        },
        permanentDeleteSuccess: {
          title: "条目已彻底删除",
          description: "已彻底删除\u201c{{name}}\u201d。",
        },
        permanentDeleteError: {
          title: "彻底删除条目失败",
          description: "无法彻底删除条目，请重试。",
        },
      },
    },
  },
  spotlight: {
    search: {
      ariaLabel: "搜索",
      placeholder: "搜索密码库...",
      detailRegionLabel: "密码项详情",
      states: {
        noResults: {
          title: "未找到匹配结果",
          description: "请尝试其他关键词。",
        },
      },
    },
    hints: {
      copyUsername: "复制用户名",
      copyPassword: "复制密码",
      moreActions: "更多操作",
      backToResults: "返回结果",
      select: "选择",
      openShortcut: "打开快速访问",
      close: "关闭",
    },
    actions: {
      copyUsername: "复制用户名",
      copyPassword: "复制密码",
      copyTotp: "复制一次性密码",
    },
    items: {
      untitledCipher: "未命名密码项",
      defaultSubtitle: "密码库项目",
    },
    states: {
      unknownError: "未知错误",
    },
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
