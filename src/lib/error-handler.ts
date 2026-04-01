/**
 * 统一错误处理器
 *
 * 负责拦截所有 API 错误响应,解析错误代码,并触发相应的处理逻辑
 */

import { debug as logDebug, error as logError } from "@tauri-apps/plugin-log";
import { commands } from "@/bindings";
import { getErrorMessage } from "./error-messages";
import { toast } from "./toast";

/**
 * 标准化错误响应接口
 */
export interface ErrorResponse {
  /** 唯一错误代码 */
  code: string;

  /** 错误消息 (用于日志) */
  message: string;

  /** 可选的详细信息 */
  details?: Record<string, unknown>;

  /** Unix 时间戳 (秒) */
  timestamp: number;

  /** 严重程度 */
  severity: "info" | "warning" | "error" | "fatal";
}

/**
 * 统一错误处理器类
 */
export class ErrorHandler {
  private recentErrors: Map<string, number>;
  private readonly DEDUPE_WINDOW = 3000; // 3秒去重窗口

  constructor() {
    this.recentErrors = new Map();
  }

  /**
   * 处理错误并显示 Toast 通知
   */
  handle(error: unknown): void {
    // 1. 解析错误
    const response = this.parseError(error);

    // 2. 去重检查
    if (this.isDuplicate(response.code)) {
      void logDebug(`[ErrorHandler] Duplicate error ignored: ${response.code}`);
      return;
    }

    // 3. 记录错误
    this.recordError(response.code);

    // 4. 特殊错误处理
    if (this.handleSpecialError(response)) {
      void logDebug(`[ErrorHandler] Special error handled: ${response.code}`);
      return;
    }

    // 5. 显示 Toast
    this.showToast(response);
  }

  /**
   * 解析错误对象为标准格式
   */
  private parseError(error: unknown): ErrorResponse {
    // 情况 1: 已经是 ErrorResponse 对象
    if (this.isErrorResponse(error)) {
      return error;
    }

    // 情况 2: Tauri 返回的 JSON 字符串
    if (typeof error === "string") {
      try {
        const parsed = JSON.parse(error);
        if (this.isErrorResponse(parsed)) {
          return parsed;
        }
      } catch {
        // 解析失败,继续降级处理
      }
    }

    // 情况 3: 包含 ErrorResponse 的对象
    if (error && typeof error === "object") {
      const obj = error as Record<string, unknown>;
      if (obj.code && typeof obj.code === "string") {
        return {
          code: obj.code,
          message: (obj.message as string) || "Unknown error",
          details: obj.details as Record<string, unknown> | undefined,
          timestamp: (obj.timestamp as number) || Date.now() / 1000,
          severity: (obj.severity as ErrorResponse["severity"]) || "error",
        };
      }
    }

    // 情况 4: 降级处理 - 返回 UNKNOWN_ERROR
    return {
      code: "UNKNOWN_ERROR",
      message: error instanceof Error ? error.message : String(error),
      timestamp: Date.now() / 1000,
      severity: "error",
    };
  }

  /**
   * 检查对象是否为有效的 ErrorResponse
   */
  private isErrorResponse(obj: unknown): obj is ErrorResponse {
    if (!obj || typeof obj !== "object") return false;
    const err = obj as Record<string, unknown>;
    return (
      typeof err.code === "string" &&
      typeof err.message === "string" &&
      typeof err.timestamp === "number" &&
      (err.severity === "info" ||
        err.severity === "warning" ||
        err.severity === "error" ||
        err.severity === "fatal")
    );
  }

  /**
   * 检查是否为重复错误
   */
  private isDuplicate(code: string): boolean {
    const lastTime = this.recentErrors.get(code);
    if (!lastTime) return false;

    const now = Date.now();
    return now - lastTime < this.DEDUPE_WINDOW;
  }

  /**
   * 记录错误用于去重
   */
  private recordError(code: string): void {
    this.recentErrors.set(code, Date.now());

    // 清理过期的错误记录
    const now = Date.now();
    for (const [errorCode, timestamp] of this.recentErrors.entries()) {
      if (now - timestamp > this.DEDUPE_WINDOW) {
        this.recentErrors.delete(errorCode);
      }
    }
  }

  /**
   * 处理特殊错误 (如 token 过期)
   * @returns true 表示已处理,不需要显示 Toast
   */
  private handleSpecialError(response: ErrorResponse): boolean {
    // 处理 token 过期 - 自动跳转到登录页
    if (
      response.code === "AUTH_TOKEN_EXPIRED" ||
      response.code === "AUTH_TOKEN_INVALID"
    ) {
      // 清理后端会话状态
      void commands.authLogout({});

      // 跳转到登录页 (/ 是登录页面的正确路由)
      window.location.href = "/";

      return true; // 已处理,不显示 Toast
    }

    return false; // 未处理,继续显示 Toast
  }

  /**
   * 显示 Toast 通知
   */
  private showToast(response: ErrorResponse): void {
    // 获取用户友好的错误消息
    const errorMessage = getErrorMessage(response.code);

    // 记录错误日志
    void logError(
      `[ErrorHandler] ${response.severity.toUpperCase()} - ${errorMessage.title}: ${errorMessage.description || response.message} (code: ${response.code})`,
    );

    // 根据严重程度显示不同类型的 Toast
    switch (response.severity) {
      case "info":
        toast.info(errorMessage.title, {
          description: errorMessage.description,
          action: errorMessage.action
            ? {
                label: errorMessage.actionLabel || "操作",
                onClick: errorMessage.action,
              }
            : undefined,
        });
        break;

      case "warning":
        toast.warning(errorMessage.title, {
          description: errorMessage.description,
          action: errorMessage.action
            ? {
                label: errorMessage.actionLabel || "操作",
                onClick: errorMessage.action,
              }
            : undefined,
        });
        break;

      case "error":
      case "fatal":
        toast.error(errorMessage.title, {
          description: errorMessage.description,
          duration: response.severity === "fatal" ? 10000 : 5000, // 致命错误显示更久
          action: errorMessage.action
            ? {
                label: errorMessage.actionLabel || "操作",
                onClick: errorMessage.action,
              }
            : undefined,
        });
        break;
    }
  }
}

/**
 * 全局错误处理器单例
 */
export const errorHandler = new ErrorHandler();
