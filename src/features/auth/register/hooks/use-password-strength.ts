import { sha1 } from "@noble/hashes/legacy.js";
import { bytesToHex } from "@noble/hashes/utils.js";
import { useMemo } from "react";

export type PasswordStrength = "weak" | "fair" | "good" | "strong";

export interface PasswordStrengthResult {
  strength: PasswordStrength;
  score: number; // 0-4
  feedback: string[];
}

export interface PwnedCheckResult {
  isPwned: boolean;
  count: number;
  error?: string;
}

/**
 * 计算密码强度
 * 基于长度、字符类型多样性、常见模式等因素
 */
function calculatePasswordStrength(password: string): PasswordStrengthResult {
  const feedback: string[] = [];
  let score = 0;

  // 长度评分
  if (password.length >= 8) score += 1;
  if (password.length >= 12) score += 1;
  if (password.length >= 16) score += 1;

  // 字符类型多样性
  const hasLowercase = /[a-z]/.test(password);
  const hasUppercase = /[A-Z]/.test(password);
  const hasNumbers = /\d/.test(password);
  const hasSpecialChars = /[^a-zA-Z0-9]/.test(password);

  const varietyCount = [
    hasLowercase,
    hasUppercase,
    hasNumbers,
    hasSpecialChars,
  ].filter(Boolean).length;

  if (varietyCount >= 3) score += 1;

  // 反馈建议
  if (password.length < 8) {
    feedback.push("密码至少需要 8 个字符");
  } else if (password.length < 12) {
    feedback.push("建议使用 12 个或更多字符");
  }

  if (varietyCount < 3) {
    feedback.push("建议混合使用大小写字母、数字和特殊字符");
  }

  // 检测常见弱密码模式
  const commonPatterns = [
    /^123+/,
    /^abc+/i,
    /password/i,
    /qwerty/i,
    /^(.)\1+$/, // 重复字符
  ];

  for (const pattern of commonPatterns) {
    if (pattern.test(password)) {
      feedback.push("避免使用常见的密码模式");
      score = Math.max(0, score - 1);
      break;
    }
  }

  // 确定强度等级
  let strength: PasswordStrength;
  if (score <= 1) strength = "weak";
  else if (score === 2) strength = "fair";
  else if (score === 3) strength = "good";
  else strength = "strong";

  return { strength, score, feedback };
}

/**
 * 使用 Have I Been Pwned API 检查密码是否泄露
 * 使用 k-anonymity 模型保护隐私
 */
async function checkPasswordPwned(password: string): Promise<PwnedCheckResult> {
  try {
    // 将字符串转换为 Uint8Array
    const encoder = new TextEncoder();
    const passwordBytes = encoder.encode(password);

    // 计算 SHA-1 哈希
    const hash = bytesToHex(sha1(passwordBytes)).toUpperCase();
    const prefix = hash.slice(0, 5);
    const suffix = hash.slice(5);

    // 调用 HIBP API
    const response = await fetch(
      `https://api.pwnedpasswords.com/range/${prefix}`,
      {
        headers: {
          "Add-Padding": "true", // 增强隐私保护
        },
      },
    );

    if (!response.ok) {
      return {
        isPwned: false,
        count: 0,
        error: `API 请求失败: ${response.status}`,
      };
    }

    const text = await response.text();
    const lines = text.split("\n");

    // 查找匹配的哈希后缀
    for (const line of lines) {
      const [hashSuffix, countStr] = line.split(":");
      if (hashSuffix === suffix) {
        return {
          isPwned: true,
          count: Number.parseInt(countStr, 10),
        };
      }
    }

    return { isPwned: false, count: 0 };
  } catch (error) {
    return {
      isPwned: false,
      count: 0,
      error: error instanceof Error ? error.message : "未知错误",
    };
  }
}

export function usePasswordStrength(password: string) {
  const strengthResult = useMemo(
    () => calculatePasswordStrength(password),
    [password],
  );

  return {
    ...strengthResult,
    checkPwned: () => checkPasswordPwned(password),
  };
}
