import {
  ChevronDown,
  Edit2,
  Eye,
  EyeOff,
  MoreVertical,
  RotateCcw,
  Trash2,
} from "lucide-react";
import { type ReactNode, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { commands, type VaultCipherDetailDto } from "@/bindings";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader } from "@/components/ui/card";
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "@/components/ui/collapsible";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import {
  CipherIcon,
  toCipherTypeIcon,
} from "@/features/vault/components/cipher-icon";
import { TruncatableText } from "@/features/vault/components/truncatable-text";
import { useCipherFieldCopy } from "@/features/vault/hooks";
import { useIcon } from "@/features/vault/hooks/use-icon";
import { firstNonEmptyText, toCipherIconHost } from "@/features/vault/utils";
import { cn } from "@/lib/utils";

const CUSTOM_FIELD_TYPE_TEXT = 0;
const CUSTOM_FIELD_TYPE_HIDDEN = 1;
const CUSTOM_FIELD_TYPE_BOOLEAN = 2;

// TOTP 倒计时圆环组件
function TotpCountdownRing({
  remaining,
  total = 30,
  ariaLabel,
  title,
}: {
  remaining: number;
  total?: number;
  ariaLabel: string;
  title: string;
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
      <svg
        width="16"
        height="16"
        className="transform -rotate-90"
        aria-label={ariaLabel}
      >
        <title>{title}</title>
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
  return onCopy ? (
    <button
      type="button"
      className="group relative min-w-0 w-full rounded-lg border border-slate-200 bg-white px-3.5 py-3 shadow-sm hover:shadow-md transition-all text-left"
      onClick={onCopy}
    >
      <div className="flex items-center justify-between">
        <div className="text-[10px] font-semibold tracking-wider text-slate-500 uppercase">
          {label}
        </div>
      </div>
      <div
        className={cn(
          "mt-2 break-words text-sm leading-relaxed text-slate-900 font-medium",
          contentClassName,
        )}
      >
        {children ?? value}
      </div>
    </button>
  ) : (
    <div className="group relative min-w-0 w-full rounded-lg border border-slate-200 bg-white px-3.5 py-3 shadow-sm">
      <div className="flex items-center justify-between">
        <div className="text-[10px] font-semibold tracking-wider text-slate-500 uppercase">
          {label}
        </div>
      </div>
      <div
        className={cn(
          "mt-2 break-words text-sm leading-relaxed text-slate-900 font-medium",
          contentClassName,
        )}
      >
        {children ?? value}
      </div>
    </div>
  );
}

type CipherDetailPanelProps = {
  cipher: VaultCipherDetailDto;
  mode?: "normal" | "trash";
  onEdit?: () => void;
  onDelete?: () => void;
  onRestore?: () => void;
  onPermanentDelete?: () => void;
  isActionLoading?: boolean;
};

export function CipherDetailPanel({
  cipher,
  mode = "normal",
  onEdit,
  onDelete,
  onRestore,
  onPermanentDelete,
  isActionLoading = false,
}: CipherDetailPanelProps) {
  const { t } = useTranslation();
  const { copyField } = useCipherFieldCopy(cipher.id);

  // Extract hostname from cipher URIs for icon lookup
  const firstUri =
    cipher.login?.uris?.[0]?.uri ?? cipher.data?.uris?.[0]?.uri ?? null;
  const hostname = firstUri ? toCipherIconHost(firstUri) : null;
  const { data: iconData } = useIcon(hostname);
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
  const lastEditedAt =
    toDisplayDate(cipher.revisionDate) ?? t("vault.detail.unknown");
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

      const normalizedName =
        name || t("vault.detail.customFields.unnamedField");
      const normalizedValue =
        value || t("vault.detail.customFields.emptyValue");
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
    : t("vault.detail.unknown");
  const passkeyDetailValue = passkeyCreationDate
    ? t("vault.detail.passkey.createdAt", { date: passkeyCreationDate })
    : null;
  const timelineEvents = [
    {
      label: t("vault.detail.timeline.lastEdited"),
      date: toDate(cipher.revisionDate),
    },
    {
      label: t("vault.detail.timeline.created"),
      date: toDate(cipher.creationDate),
    },
    {
      label: t("vault.detail.timeline.passwordUpdated"),
      date: toDate(passwordRevisionDate),
    },
    {
      label: t("vault.detail.timeline.passkeyCreated"),
      date: toDateFromTimestamp(passkeyCreationTimestamp),
    },
    {
      label: t("vault.detail.timeline.archived"),
      date: toDate(cipher.archivedDate),
    },
    {
      label: t("vault.detail.timeline.deleted"),
      date: toDate(cipher.deletedDate),
    },
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
    <Card className="h-full min-h-0 min-w-0 w-full gap-0 border-none bg-white py-0 shadow-none flex flex-col">
      <CardHeader className="gap-2 border-b border-slate-200 bg-gradient-to-br from-slate-50 via-white to-blue-50/30 px-6 py-4 shrink-0">
        <div className="flex min-h-9 items-center gap-3.5 min-w-0">
          <CipherIcon
            alt={cipher.name ?? "Cipher"}
            className="size-11 bg-white border border-slate-200 text-slate-500 shadow-sm shrink-0"
            iconData={iconData}
          >
            {toCipherTypeIcon(cipher.type)}
          </CipherIcon>
          <div className="min-w-0 flex-1 shrink overflow-hidden">
            <TruncatableText
              text={cipher.name ?? t("vault.page.cipher.untitled")}
              as="h2"
              className="m-0 leading-tight text-xl font-bold text-slate-900 cursor-text"
            />
          </div>
          {mode === "normal" && (
            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <Button
                  type="button"
                  variant="ghost"
                  size="sm"
                  className="h-8 w-8 p-0 shrink-0"
                >
                  <MoreVertical className="size-4" />
                </Button>
              </DropdownMenuTrigger>
              <DropdownMenuContent align="end" className="w-44">
                <DropdownMenuItem onSelect={onEdit}>
                  <Edit2 className="size-4" />
                  {t("vault.page.actions.edit")}
                </DropdownMenuItem>
                <DropdownMenuSeparator />
                <DropdownMenuItem variant="destructive" onSelect={onDelete}>
                  <Trash2 className="size-4" />
                  {t("vault.page.actions.delete")}
                </DropdownMenuItem>
              </DropdownMenuContent>
            </DropdownMenu>
          )}
          {mode === "trash" && (
            <div className="flex items-center gap-2 shrink-0">
              <Button
                type="button"
                variant="outline"
                size="sm"
                className="h-8 px-3 text-xs"
                disabled={isActionLoading}
                onClick={onRestore}
              >
                <RotateCcw className="size-3.5" />
                {t("vault.page.actions.restore")}
              </Button>
              <Button
                type="button"
                variant="destructive"
                size="sm"
                className="h-8 px-3 text-xs"
                disabled={isActionLoading}
                onClick={onPermanentDelete}
              >
                <Trash2 className="size-3.5" />
                {t("vault.page.actions.permanentDelete")}
              </Button>
            </div>
          )}
        </div>
      </CardHeader>

      <div className="flex-1 min-h-0 overflow-y-auto overflow-x-hidden">
        <CardContent className="min-w-0 space-y-3 pt-5">
          {/* 主要凭证区块 - 类似 1Password 的统一卡片 */}
          {(username || password || hasOneTimePassword || hasPasskey) && (
            <div className="rounded-lg border border-slate-200 bg-white shadow-sm divide-y divide-slate-200">
              {username && (
                <button
                  type="button"
                  className="group w-full px-3.5 py-3 text-left hover:bg-slate-50 transition-colors first:rounded-t-lg"
                  onClick={() => copyField("username")}
                >
                  <div className="text-[10px] font-semibold tracking-wider text-slate-500 uppercase">
                    {t("vault.detail.fields.username")}
                  </div>
                  <div className="mt-2 text-sm font-medium text-slate-900">
                    {username}
                  </div>
                </button>
              )}

              {password && (
                <div className="group w-full px-3.5 py-3 hover:bg-slate-50 transition-colors">
                  <div className="text-[10px] font-semibold tracking-wider text-slate-500 uppercase">
                    {t("vault.detail.fields.password")}
                  </div>
                  <div className="mt-2 flex min-w-0 items-center justify-between gap-2">
                    <button
                      type="button"
                      className="min-w-0 flex-1 text-left"
                      onClick={() => copyField("password")}
                    >
                      <div className="min-w-0 break-all font-mono text-sm text-slate-900 font-semibold">
                        {isPasswordVisible ? password : "••••••••••••"}
                      </div>
                    </button>
                    <Button
                      type="button"
                      variant="ghost"
                      className="size-8 shrink-0 rounded-full p-0 hover:bg-slate-200"
                      onClick={() => {
                        setIsPasswordVisible((visible) => !visible);
                      }}
                      aria-label={
                        isPasswordVisible
                          ? t("vault.detail.actions.hidePassword")
                          : t("vault.detail.actions.showPassword")
                      }
                      title={
                        isPasswordVisible
                          ? t("vault.detail.actions.hidePassword")
                          : t("vault.detail.actions.showPassword")
                      }
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
                <button
                  type="button"
                  className="group w-full px-3.5 py-3 text-left hover:bg-slate-50 transition-colors"
                  onClick={() => copyField("totp")}
                >
                  <div className="text-[10px] font-semibold tracking-wider text-slate-500 uppercase">
                    {t("vault.detail.fields.oneTimePassword")}
                  </div>
                  <div className="mt-2 flex items-center justify-between gap-3">
                    <div className="font-mono text-sm font-bold text-slate-900 tracking-wider">
                      {oneTimePasswordCode ? (
                        <>
                          {oneTimePasswordCode.slice(0, 3)}
                          <span className="mx-0.5">·</span>
                          {oneTimePasswordCode.slice(3)}
                        </>
                      ) : oneTimePasswordFailed ? (
                        t("common.states.unavailable")
                      ) : (
                        t("common.states.loading")
                      )}
                    </div>
                    {oneTimePasswordRemaining != null &&
                      oneTimePasswordCode && (
                        <TotpCountdownRing
                          remaining={oneTimePasswordRemaining}
                          total={30}
                          ariaLabel={t("vault.detail.totp.countdownAria")}
                          title={t("vault.detail.totp.countdownTitle")}
                        />
                      )}
                  </div>
                </button>
              )}

              {hasPasskey && (
                <div className="group px-3.5 py-3 hover:bg-slate-50 transition-colors last:rounded-b-lg">
                  <div className="text-[10px] font-semibold tracking-wider text-slate-500 uppercase">
                    {t("vault.detail.fields.passkey")}
                  </div>
                  <div className="mt-2 text-sm font-medium text-slate-900">
                    {passkeyDetailValue}
                  </div>
                </div>
              )}
            </div>
          )}

          <DetailField
            label={t("vault.detail.fields.organization")}
            value={organizationId}
          />

          {uniqueUris.length > 0 && (
            <DetailField
              label={t("vault.detail.fields.uris")}
              contentClassName="overflow-hidden"
            >
              <div className="min-w-0 w-full space-y-1 overflow-hidden">
                {uniqueUris.map((uri, index) => (
                  <button
                    type="button"
                    key={uri}
                    title={uri}
                    className="block min-w-0 w-full truncate text-left text-slate-900 hover:text-slate-700"
                    onClick={(e) => {
                      e.stopPropagation();
                      copyField({ uri: index });
                    }}
                  >
                    {uri}
                  </button>
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
                    {t("vault.detail.fields.notes")}
                  </div>
                  <button
                    type="button"
                    className="group relative w-full rounded-lg border border-slate-200 bg-slate-50 p-3.5 text-left hover:shadow-md transition-all"
                    onClick={() => copyField("notes")}
                  >
                    <pre className="whitespace-pre-wrap text-sm leading-relaxed text-slate-800">
                      {notes}
                    </pre>
                  </button>
                </div>
              )}

              {customFields.length > 0 && (
                <div className="space-y-2">
                  <div className="text-[10px] font-semibold tracking-wider text-slate-500 uppercase">
                    {t("vault.detail.fields.customFields")}
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
                          ? t("vault.detail.boolean.true")
                          : normalizedValue === "false"
                            ? t("vault.detail.boolean.false")
                            : field.value;
                      const displayValue = isHiddenType
                        ? hiddenValue
                        : isBooleanType
                          ? booleanValue
                          : field.value;
                      const canCopy =
                        field.fieldType === CUSTOM_FIELD_TYPE_TEXT ||
                        field.fieldType === CUSTOM_FIELD_TYPE_HIDDEN;

                      return canCopy ? (
                        <div
                          key={field.key}
                          className="group relative w-full rounded-lg border border-slate-200 bg-white px-3.5 py-3 shadow-sm hover:shadow-md transition-all text-left"
                        >
                          <div className="flex items-center justify-between">
                            <div className="text-[10px] font-semibold tracking-wider text-slate-500 uppercase">
                              {field.name}
                            </div>
                          </div>
                          <div className="mt-2 flex min-w-0 items-center justify-between gap-2">
                            <button
                              type="button"
                              className="min-w-0 flex-1 text-left"
                              onClick={() => copyField({ customField: index })}
                            >
                              <div className="min-w-0 break-all text-sm font-medium text-slate-900">
                                {displayValue}
                              </div>
                            </button>
                            {isHiddenType && (
                              <Button
                                type="button"
                                variant="outline"
                                className="size-8 shrink-0 rounded-full p-0 border-slate-300 hover:bg-slate-100"
                                onClick={() => {
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
                                  isRevealed
                                    ? t("vault.detail.actions.hideFieldValue")
                                    : t("vault.detail.actions.showFieldValue")
                                }
                                title={
                                  isRevealed
                                    ? t("vault.detail.actions.hideFieldValue")
                                    : t("vault.detail.actions.showFieldValue")
                                }
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
                      ) : (
                        <div
                          key={field.key}
                          className="group relative rounded-lg border border-slate-200 bg-white px-3.5 py-3 shadow-sm"
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
                  {t("vault.detail.timeline.lastEditedWithValue", {
                    date: lastEditedAt,
                  })}
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
                      <div className="text-xs font-medium text-slate-500">
                        {event.label}
                      </div>
                      <div className="text-sm font-semibold text-slate-900">
                        {event.date.toLocaleString()}
                      </div>
                    </li>
                  ))}
                </ol>
              ) : (
                <div className="text-sm text-slate-500">
                  {t("vault.detail.timeline.empty")}
                </div>
              )}
            </CollapsibleContent>
          </Collapsible>
        </CardContent>
      </div>
    </Card>
  );
}
