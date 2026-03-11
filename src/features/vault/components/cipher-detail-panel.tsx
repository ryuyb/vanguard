import { ChevronDown, Eye, EyeOff } from "lucide-react";
import { type ReactNode, useEffect, useState } from "react";
import { commands, type VaultCipherDetailDto } from "@/bindings";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader } from "@/components/ui/card";
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "@/components/ui/collapsible";
import {
  CipherIcon,
  toCipherTypeIcon,
} from "@/features/vault/components/cipher-icon";
import {
  firstNonEmptyText,
  getCipherIconUrl,
  toCipherIconAlt,
} from "@/features/vault/utils";
import { useCipherFieldCopy } from "@/features/vault/hooks";
import { cn } from "@/lib/utils";

const CUSTOM_FIELD_TYPE_TEXT = 0;
const CUSTOM_FIELD_TYPE_HIDDEN = 1;
const CUSTOM_FIELD_TYPE_BOOLEAN = 2;

// TOTP 倒计时圆环组件
function TotpCountdownRing({
  remaining,
  total = 30,
}: {
  remaining: number;
  total?: number;
}) {
  const percentage = (remaining / total) * 100;
  const circumference = 2 * Math.PI * 6; // 半径为 6
  const strokeDashoffset = circumference - (percentage / 100) * circumference;

  // 根据剩余时间计算颜色：绿色 -> 黄色 -> 红色
  const getColor = () => {
    if (percentage > 50) return "#10b981"; // green-500
    if (percentage > 20) return "#f59e0b"; // amber-500
    return "#ef4444"; // red-500
  };

  return (
    <div className="relative inline-flex items-center gap-1.5">
      <svg width="16" height="16" className="transform -rotate-90">
        {/* 背景圆环 */}
        <circle
          cx="8"
          cy="8"
          r="6"
          fill="none"
          stroke="#e5e7eb"
          strokeWidth="3"
        />
        {/* 进度圆环 */}
        <circle
          cx="8"
          cy="8"
          r="6"
          fill="none"
          stroke={getColor()}
          strokeWidth="3"
          strokeDasharray={circumference}
          strokeDashoffset={strokeDashoffset}
          strokeLinecap="round"
          className="transition-all duration-1000 ease-linear"
        />
      </svg>
      <span
        className="text-xs font-semibold tabular-nums"
        style={{ color: getColor() }}
      >
        {remaining}
      </span>
    </div>
  );
}

function toDate(value: string | null | undefined): Date | null {
  if (!value) {
    return null;
  }
  const timestamp = Date.parse(value);
  if (Number.isNaN(timestamp)) {
    return null;
  }
  return new Date(timestamp);
}

function toDisplayDate(value: string | null | undefined): string | null {
  const date = toDate(value);
  if (!date) {
    return null;
  }
  return date.toLocaleString();
}

function toDateFromTimestamp(value: number | null | undefined): Date | null {
  if (value == null || !Number.isFinite(value)) {
    return null;
  }
  return new Date(value);
}

type DetailFieldProps = {
  label: string;
  value?: string | null | undefined;
  children?: ReactNode;
  contentClassName?: string;
  onCopy?: () => void;
};

function DetailField({
  label,
  value,
  children,
  contentClassName,
  onCopy,
}: DetailFieldProps) {
  if (!value && !children) {
    return null;
  }
  return (
    <div
      className="group relative min-w-0 w-full rounded-lg border border-slate-200 bg-white px-3.5 py-3 shadow-sm hover:shadow-md transition-all"
      onClick={onCopy}
      role={onCopy ? "button" : undefined}
      tabIndex={onCopy ? 0 : undefined}
      style={{ cursor: onCopy ? "pointer" : "default" }}
    >
      <div className="flex items-center justify-between">
        <div className="text-[10px] font-semibold tracking-wider text-slate-500 uppercase">
          {label}
        </div>
      </div>
      <div
        className={cn(
          "mt-2 break-words text-sm leading-relaxed text-slate-900 font-medium",
          contentClassName
        )}
      >
        {children ?? value}
      </div>
    </div>
  );
}

type CipherDetailPanelProps = {
  cipher: VaultCipherDetailDto;
  iconUrl?: string | null;
  iconServer?: string | null;
};

export function CipherDetailPanel({
  cipher,
  iconUrl: iconUrlProp,
  iconServer,
}: CipherDetailPanelProps) {
  const { copyField } = useCipherFieldCopy(cipher.id);
  const iconUrl = iconUrlProp ?? getCipherIconUrl(cipher, iconServer);
  const username = firstNonEmptyText(
    cipher.login?.username,
    cipher.data?.username,
  );
  const password = firstNonEmptyText(
    cipher.login?.password,
    cipher.data?.password,
  );
  const hasOneTimePassword = cipher.hasTotp;
  const [oneTimePasswordCode, setOneTimePasswordCode] = useState<string | null>(
    null,
  );
  const [oneTimePasswordRemaining, setOneTimePasswordRemaining] = useState<
    number | null
  >(null);
  const [oneTimePasswordFailed, setOneTimePasswordFailed] = useState(false);
  const [isPasswordVisible, setIsPasswordVisible] = useState(false);
  const [isTimelineOpen, setIsTimelineOpen] = useState(false);
  const [revealedCustomFieldKeys, setRevealedCustomFieldKeys] = useState<
    Set<string>
  >(new Set());
  const notes = firstNonEmptyText(cipher.notes, cipher.data?.notes);
  const organizationId = cipher.organizationId;
  const lastEditedAt = toDisplayDate(cipher.revisionDate) ?? "未知";
  const passwordRevisionDate = firstNonEmptyText(
    cipher.login?.passwordRevisionDate,
    cipher.data?.passwordRevisionDate,
  );
  const customFields = (() => {
    const rawFields = [...cipher.fields, ...(cipher.data?.fields ?? [])];
    const seen = new Set<string>();
    const normalizedFields: Array<{
      key: string;
      name: string;
      value: string;
      fieldType: number;
      linkedId: number | null;
    }> = [];

    for (const [index, field] of rawFields.entries()) {
      const name = (field.name ?? "").trim();
      const value = (field.value ?? "").trim();
      if (!name && !value) {
        continue;
      }

      const normalizedName = name || "Unnamed field";
      const normalizedValue = value || "—";
      const fieldType = field.type ?? CUSTOM_FIELD_TYPE_TEXT;
      const linkedId = field.linkedId ?? null;
      const dedupeKey = `${normalizedName}\u0000${normalizedValue}\u0000${fieldType}\u0000${linkedId ?? ""}`;

      if (seen.has(dedupeKey)) {
        continue;
      }
      seen.add(dedupeKey);
      normalizedFields.push({
        key: `${dedupeKey}\u0000${index}`,
        name: normalizedName,
        value: normalizedValue,
        fieldType,
        linkedId,
      });
    }

    return normalizedFields;
  })();
  const uniqueUris = (() => {
    const values = [
      cipher.login?.uri,
      cipher.data?.uri,
      ...(cipher.login?.uris ?? []).map((item) => item.uri),
      ...(cipher.data?.uris ?? []).map((item) => item.uri),
    ];
    const normalized = values
      .filter((value): value is string => Boolean(value))
      .map((value) => value.trim())
      .filter((value) => value.length > 0);

    return [...new Set(normalized)];
  })();
  const loginPasskeyCredentials = cipher.login?.fido2Credentials ?? [];
  const dataPasskeyCredentials = cipher.data?.fido2Credentials ?? [];
  const passkeyCredentials =
    loginPasskeyCredentials.length > 0
      ? loginPasskeyCredentials
      : dataPasskeyCredentials;
  const hasPasskey = passkeyCredentials.length > 0;

  const passkeyCreationTimestamp = (() => {
    if (!hasPasskey) {
      return null;
    }

    const validTimestamps = passkeyCredentials
      .map((credential) => credential.creationDate)
      .filter((value): value is string => Boolean(value))
      .map((value) => Date.parse(value))
      .filter((timestamp) => Number.isFinite(timestamp));

    if (validTimestamps.length === 0) {
      return null;
    }
    return Math.min(...validTimestamps);
  })();
  const passkeyCreationDate = passkeyCreationTimestamp
    ? new Date(passkeyCreationTimestamp).toLocaleString()
    : "未知";
  const passkeyDetailValue = passkeyCreationDate
    ? `创建于 ${passkeyCreationDate}`
    : null;
  const timelineEvents = [
    { label: "最后编辑", date: toDate(cipher.revisionDate) },
    { label: "创建", date: toDate(cipher.creationDate) },
    { label: "密码更新", date: toDate(passwordRevisionDate) },
    {
      label: "Passkey 创建",
      date: toDateFromTimestamp(passkeyCreationTimestamp),
    },
    { label: "归档", date: toDate(cipher.archivedDate) },
    { label: "删除", date: toDate(cipher.deletedDate) },
  ]
    .filter((event): event is { label: string; date: Date } =>
      Boolean(event.date),
    )
    .sort((left, right) => right.date.getTime() - left.date.getTime());

  useEffect(() => {
    let disposed = false;
    let expiresAtMs: number | null = null;
    let loading = false;

    setIsPasswordVisible(false);

    const loadTotpSnapshot = async () => {
      if (loading || disposed) {
        return;
      }
      loading = true;
      try {
        const result = await commands.vaultGetCipherTotpCode({
          cipherId: cipher.id,
        });
        if (disposed) {
          return;
        }
        if (result.status === "error") {
          expiresAtMs = null;
          setOneTimePasswordCode(null);
          setOneTimePasswordRemaining(null);
          setOneTimePasswordFailed(true);
          return;
        }

        expiresAtMs = result.data.expiresAtMs;
        setOneTimePasswordCode(result.data.code);
        setOneTimePasswordRemaining(result.data.remainingSeconds);
        setOneTimePasswordFailed(false);
      } catch {
        if (disposed) {
          return;
        }
        expiresAtMs = null;
        setOneTimePasswordCode(null);
        setOneTimePasswordRemaining(null);
        setOneTimePasswordFailed(true);
      } finally {
        loading = false;
      }
    };
    const intervalId = window.setInterval(() => {
      if (!expiresAtMs) {
        return;
      }
      const remaining = Math.max(
        0,
        Math.ceil((expiresAtMs - Date.now()) / 1000),
      );
      setOneTimePasswordRemaining(remaining);
      if (remaining <= 0) {
        void loadTotpSnapshot();
      }
    }, 1000);

    setOneTimePasswordCode(null);
    setOneTimePasswordRemaining(null);
    setOneTimePasswordFailed(false);
    if (!hasOneTimePassword) {
      return () => {
        disposed = true;
        window.clearInterval(intervalId);
      };
    }

    void loadTotpSnapshot();

    return () => {
      disposed = true;
      window.clearInterval(intervalId);
    };
  }, [cipher.id, hasOneTimePassword]);

  useEffect(() => {
    setIsTimelineOpen(false);
    setRevealedCustomFieldKeys(new Set());
  }, []);

  return (
    <Card className="h-full min-h-0 min-w-0 w-full gap-0 overflow-x-hidden overflow-y-auto border-none bg-white py-0 shadow-none">
      <CardHeader className="gap-2 border-b border-slate-200 bg-gradient-to-br from-slate-50 via-white to-blue-50/30 px-6 py-4">
        <div className="flex min-h-9 items-center gap-3.5">
          <CipherIcon
            alt={toCipherIconAlt(cipher.name)}
            className="size-11 bg-white border border-slate-200 text-slate-500 shadow-sm"
            iconUrl={iconUrl}
            isVisible={Boolean(iconUrl)}
            loadState={iconUrl ? "loading" : "fallback"}
          >
            {toCipherTypeIcon(cipher.type)}
          </CipherIcon>
          <div className="min-w-0 flex-1">
            <h2 className="m-0 truncate leading-tight text-xl font-bold text-slate-900">
              {cipher.name ?? "Untitled cipher"}
            </h2>
          </div>
        </div>
      </CardHeader>

      <CardContent className="min-w-0 space-y-3 pt-5">
        {/* 主要凭证区块 - 类似 1Password 的统一卡片 */}
        {(username || password || hasOneTimePassword || hasPasskey) && (
          <div className="rounded-lg border border-slate-200 bg-white shadow-sm divide-y divide-slate-200">
            {username && (
              <div
                className="group px-3.5 py-3 cursor-pointer hover:bg-slate-50 transition-colors first:rounded-t-lg"
                onClick={() => copyField("username")}
                role="button"
                tabIndex={0}
              >
                <div className="text-[10px] font-semibold tracking-wider text-slate-500 uppercase">
                  Username
                </div>
                <div className="mt-2 text-sm font-medium text-slate-900">
                  {username}
                </div>
              </div>
            )}

            {password && (
              <div
                className="group px-3.5 py-3 cursor-pointer hover:bg-slate-50 transition-colors"
                onClick={() => copyField("password")}
                role="button"
                tabIndex={0}
              >
                <div className="text-[10px] font-semibold tracking-wider text-slate-500 uppercase">
                  Password
                </div>
                <div className="mt-2 flex min-w-0 items-center justify-between gap-2">
                  <div className="min-w-0 break-all font-mono text-sm text-slate-900 font-semibold">
                    {isPasswordVisible ? password : "••••••••••••"}
                  </div>
                  <Button
                    type="button"
                    variant="ghost"
                    className="size-8 shrink-0 rounded-full p-0 hover:bg-slate-200"
                    onClick={(e) => {
                      e.stopPropagation();
                      setIsPasswordVisible((visible) => !visible);
                    }}
                    aria-label={isPasswordVisible ? "隐藏密码" : "显示密码"}
                    title={isPasswordVisible ? "隐藏密码" : "显示密码"}
                  >
                    {isPasswordVisible ? (
                      <EyeOff className="size-4 text-slate-600" />
                    ) : (
                      <Eye className="size-4 text-slate-600" />
                    )}
                  </Button>
                </div>
              </div>
            )}

            {hasOneTimePassword && (
              <div
                className="group px-3.5 py-3 cursor-pointer hover:bg-slate-50 transition-colors"
                onClick={() => copyField("totp")}
                role="button"
                tabIndex={0}
              >
                <div className="text-[10px] font-semibold tracking-wider text-slate-500 uppercase">
                  One-time password
                </div>
                <div className="mt-2 flex items-center justify-between gap-3">
                  <div className="font-mono text-sm font-bold text-slate-900 tracking-wider">
                    {oneTimePasswordCode ? (
                      <>
                        {oneTimePasswordCode.slice(0, 3)}
                        <span className="mx-0.5">·</span>
                        {oneTimePasswordCode.slice(3)}
                      </>
                    ) : (
                      oneTimePasswordFailed ? "Unavailable" : "Loading..."
                    )}
                  </div>
                  {oneTimePasswordRemaining != null && oneTimePasswordCode && (
                    <TotpCountdownRing remaining={oneTimePasswordRemaining} total={30} />
                  )}
                </div>
              </div>
            )}

            {hasPasskey && (
              <div className="group px-3.5 py-3 hover:bg-slate-50 transition-colors last:rounded-b-lg">
                <div className="text-[10px] font-semibold tracking-wider text-slate-500 uppercase">
                  Passkey
                </div>
                <div className="mt-2 text-sm font-medium text-slate-900">
                  {passkeyDetailValue}
                </div>
              </div>
            )}
          </div>
        )}

        <DetailField label="Org" value={organizationId} />

        {uniqueUris.length > 0 && (
          <DetailField label="URIs" contentClassName="overflow-hidden">
            <div className="min-w-0 w-full space-y-1 overflow-hidden">
              {uniqueUris.map((uri, index) => (
                <div
                  key={uri}
                  title={uri}
                  className="block min-w-0 w-full truncate text-slate-900 cursor-pointer hover:text-slate-700"
                  onClick={(e) => {
                    e.stopPropagation();
                    copyField({ uri: index });
                  }}
                >
                  {uri}
                </div>
              ))}
            </div>
          </DetailField>
        )}
      </CardContent>

      {(notes || customFields.length > 0) && (
        <CardContent className="min-w-0 pt-4">
          <div className="space-y-3">
            {notes && (
              <div className="space-y-2">
                <div className="text-[10px] font-semibold tracking-wider text-slate-500 uppercase">
                  Notes
                </div>
                <div
                  className="group relative rounded-lg border border-slate-200 bg-slate-50 p-3.5 cursor-pointer hover:shadow-md transition-all"
                  onClick={() => copyField("notes")}
                  role="button"
                  tabIndex={0}
                >
                  <pre className="whitespace-pre-wrap text-sm leading-relaxed text-slate-800">
                    {notes}
                  </pre>
                </div>
              </div>
            )}

            {customFields.length > 0 && (
              <div className="space-y-2">
                <div className="text-[10px] font-semibold tracking-wider text-slate-500 uppercase">
                  Custom fields
                </div>
                <div className="space-y-2.5">
                  {customFields.map((field, index) => {
                    const isHiddenType =
                      field.fieldType === CUSTOM_FIELD_TYPE_HIDDEN;
                    const isRevealed = revealedCustomFieldKeys.has(field.key);
                    const hiddenValue = isRevealed ? field.value : "••••••••";
                    const isBooleanType =
                      field.fieldType === CUSTOM_FIELD_TYPE_BOOLEAN;
                    const normalizedValue = field.value.toLowerCase();
                    const booleanValue =
                      normalizedValue === "true"
                        ? "是"
                        : normalizedValue === "false"
                          ? "否"
                          : field.value;
                    const displayValue = isHiddenType
                      ? hiddenValue
                      : isBooleanType
                        ? booleanValue
                        : field.value;
                    const canCopy =
                      field.fieldType === CUSTOM_FIELD_TYPE_TEXT ||
                      field.fieldType === CUSTOM_FIELD_TYPE_HIDDEN;

                    return (
                      <div
                        key={field.key}
                        className={`group relative rounded-lg border border-slate-200 bg-white px-3.5 py-3 shadow-sm hover:shadow-md transition-all ${canCopy ? "cursor-pointer" : ""}`}
                        onClick={
                          canCopy
                            ? () => copyField({ customField: index })
                            : undefined
                        }
                        role={canCopy ? "button" : undefined}
                        tabIndex={canCopy ? 0 : undefined}
                      >
                        <div className="flex items-center justify-between">
                          <div className="text-[10px] font-semibold tracking-wider text-slate-500 uppercase">
                            {field.name}
                          </div>
                        </div>
                        <div className="mt-2 flex min-w-0 items-center justify-between gap-2">
                          <div className="min-w-0 break-all text-sm font-medium text-slate-900">
                            {displayValue}
                          </div>
                          {isHiddenType && (
                            <Button
                              type="button"
                              variant="outline"
                              className="size-8 shrink-0 rounded-full p-0 border-slate-300 hover:bg-slate-100"
                              onClick={(e) => {
                                e.stopPropagation();
                                setRevealedCustomFieldKeys((previous) => {
                                  const next = new Set(previous);
                                  if (next.has(field.key)) {
                                    next.delete(field.key);
                                  } else {
                                    next.add(field.key);
                                  }
                                  return next;
                                });
                              }}
                              aria-label={
                                isRevealed ? "隐藏字段值" : "显示字段值"
                              }
                              title={isRevealed ? "隐藏字段值" : "显示字段值"}
                            >
                              {isRevealed ? (
                                <EyeOff className="size-4 text-slate-600" />
                              ) : (
                                <Eye className="size-4 text-slate-600" />
                              )}
                            </Button>
                          )}
                        </div>
                      </div>
                    );
                  })}
                </div>
              </div>
            )}
          </div>
        </CardContent>
      )}

      <CardContent className="min-w-0 pt-4">
        <Collapsible open={isTimelineOpen} onOpenChange={setIsTimelineOpen}>
          <CollapsibleTrigger asChild>
            <button
              type="button"
              className="flex w-full items-center justify-between rounded-lg border border-slate-200 bg-slate-50 px-3.5 py-2.5 text-left transition-all hover:bg-slate-100 hover:shadow-md"
            >
              <span className="text-sm font-semibold text-slate-700">
                最后编辑 {lastEditedAt}
              </span>
              <ChevronDown
                className={[
                  "size-4 text-slate-400 transition-transform",
                  isTimelineOpen ? "rotate-180" : "",
                ].join(" ")}
              />
            </button>
          </CollapsibleTrigger>

          <CollapsibleContent className="pt-3">
            {timelineEvents.length > 0 ? (
              <ol className="relative ml-1 space-y-3 border-l-2 border-blue-200 pl-5">
                {timelineEvents.map((event) => (
                  <li
                    key={`${event.label}-${event.date.toISOString()}`}
                    className="relative"
                  >
                    <span className="absolute -left-6.5 top-1.5 size-2.5 rounded-full bg-blue-500 ring-4 ring-white" />
                    <div className="text-xs font-medium text-slate-500">{event.label}</div>
                    <div className="text-sm font-semibold text-slate-900">
                      {event.date.toLocaleString()}
                    </div>
                  </li>
                ))}
              </ol>
            ) : (
              <div className="text-sm text-slate-500">暂无可显示的时间线。</div>
            )}
          </CollapsibleContent>
        </Collapsible>
      </CardContent>
    </Card>
  );
}
