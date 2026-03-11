/**
 * Toast 通知封装
 *
 * 封装 Sonner 的 toast 方法,提供统一的通知接口
 */

import { toast as sonnerToast } from "sonner";

/**
 * Toast 选项
 */
export interface ToastOptions {
  /** Toast 描述 */
  description?: string;

  /** 持续时间 (毫秒),默认 4000ms */
  duration?: number;

  /** 操作按钮 */
  action?: {
    label: string;
    onClick: () => void;
  };
}

/**
 * 显示信息 Toast (蓝色)
 */
function info(title: string, options?: ToastOptions): void {
  sonnerToast.info(title, {
    description: options?.description,
    duration: options?.duration || 4000,
    action: options?.action,
  });
}

/**
 * 显示警告 Toast (黄色)
 */
function warning(title: string, options?: ToastOptions): void {
  sonnerToast.warning(title, {
    description: options?.description,
    duration: options?.duration || 4000,
    action: options?.action,
  });
}

/**
 * 显示错误 Toast (红色)
 */
function error(title: string, options?: ToastOptions): void {
  sonnerToast.error(title, {
    description: options?.description,
    duration: options?.duration || 5000, // 错误持续时间稍长
    action: options?.action,
  });
}

/**
 * 显示成功 Toast (绿色)
 */
function success(title: string, options?: ToastOptions): void {
  sonnerToast.success(title, {
    description: options?.description,
    duration: options?.duration || 3000,
    action: options?.action,
  });
}

/**
 * 导出 toast 对象
 */
export const toast = {
  info,
  warning,
  error,
  success,
};
