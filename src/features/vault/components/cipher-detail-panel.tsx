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
import { firstNonEmptyText } from "@/features/vault/utils";

const CUSTOM_FIELD_TYPE_TEXT = 0;
const CUSTOM_FIELD_TYPE_HIDDEN = 1;
const CUSTOM_FIELD_TYPE_BOOLEAN = 2;

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

function DetailField({
  label,
  value,
  children,
}: {
  label: string;
  value?: string | null | undefined;
  children?: ReactNode;
}) {
  if (!value && !children) {
    return null;
  }
  return (
    <div className="w-full rounded-xl border border-slate-200/80 bg-white/90 px-3 py-2.5 shadow-xs">
      <div className="text-[11px] font-medium tracking-wide text-slate-500 uppercase">
        {label}
      </div>
      <div className="mt-1 min-w-0 break-all text-sm font-medium text-slate-900">
        {children ?? value}
      </div>
    </div>
  );
}

type CipherDetailPanelProps = {
  cipher: VaultCipherDetailDto;
};

export function CipherDetailPanel({ cipher }: CipherDetailPanelProps) {
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

  const oneTimePasswordDisplay =
    oneTimePasswordCode && oneTimePasswordRemaining != null
      ? `${oneTimePasswordCode} (${oneTimePasswordRemaining}s)`
      : oneTimePasswordFailed
        ? "Unavailable"
        : hasOneTimePassword
          ? "Loading..."
          : null;

  return (
    <Card className="h-full min-h-0 gap-0 overflow-y-auto border-slate-200/80 bg-white/90 py-0 shadow-sm">
      <CardHeader className="gap-2 border-b border-slate-200/80 bg-gradient-to-br from-slate-50 via-white to-sky-50/50 px-6 py-3">
        <div className="flex h-9 items-center">
          <h2 className="m-0 leading-none text-lg font-semibold text-slate-900">
            {cipher.name ?? "Untitled cipher"}
          </h2>
        </div>
      </CardHeader>

      <CardContent className="space-y-3 pt-4">
        <div className="flex flex-col gap-2">
          <DetailField label="Username" value={username} />
          <DetailField label="Org" value={organizationId} />
          {hasOneTimePassword && (
            <DetailField label="One-time password">
              <span className="font-mono tracking-wide">
                {oneTimePasswordDisplay}
              </span>
            </DetailField>
          )}
          {hasPasskey && (
            <DetailField label="Passkey" value={passkeyDetailValue} />
          )}
        </div>

        {password && (
          <DetailField label="Password">
            <div className="flex min-w-0 items-center justify-between gap-2">
              <div className="min-w-0 break-all font-mono text-[13px] text-slate-900">
                {isPasswordVisible ? password : "••••••••••••"}
              </div>
              <Button
                type="button"
                variant="outline"
                className="size-7 shrink-0 rounded-full p-0"
                onClick={() => setIsPasswordVisible((visible) => !visible)}
                aria-label={isPasswordVisible ? "隐藏密码" : "显示密码"}
                title={isPasswordVisible ? "隐藏密码" : "显示密码"}
              >
                {isPasswordVisible ? (
                  <EyeOff className="size-4" />
                ) : (
                  <Eye className="size-4" />
                )}
              </Button>
            </div>
          </DetailField>
        )}

        {uniqueUris.length > 0 && (
          <DetailField label="URIs">
            <div className="space-y-1">
              {uniqueUris.map((uri) => (
                <div key={uri} className="break-all text-slate-900">
                  {uri}
                </div>
              ))}
            </div>
          </DetailField>
        )}
      </CardContent>

      {(notes || customFields.length > 0) && (
        <CardContent className="pt-4">
          <div className="space-y-3">
            {notes && (
              <div className="space-y-2">
                <div className="text-[11px] font-medium tracking-wide text-slate-500 uppercase">
                  Notes
                </div>
                <div className="rounded-xl border border-slate-200/80 bg-slate-50/80 p-3">
                  <pre className="whitespace-pre-wrap text-sm leading-relaxed text-slate-800">
                    {notes}
                  </pre>
                </div>
              </div>
            )}

            {customFields.length > 0 && (
              <div className="space-y-2">
                <div className="text-[11px] font-medium tracking-wide text-slate-500 uppercase">
                  Custom fields
                </div>
                <div className="space-y-2">
                  {customFields.map((field) => {
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

                    return (
                      <div
                        key={field.key}
                        className="rounded-xl border border-slate-200/80 bg-white/90 px-3 py-2.5"
                      >
                        <div className="text-[11px] font-medium tracking-wide text-slate-500">
                          {field.name}
                        </div>
                        <div className="mt-1 flex min-w-0 items-center justify-between gap-2">
                          <div className="min-w-0 break-all text-sm font-medium text-slate-900">
                            {displayValue}
                          </div>
                          {isHiddenType && (
                            <Button
                              type="button"
                              variant="outline"
                              className="size-7 shrink-0 rounded-full p-0"
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
                                isRevealed ? "隐藏字段值" : "显示字段值"
                              }
                              title={isRevealed ? "隐藏字段值" : "显示字段值"}
                            >
                              {isRevealed ? (
                                <EyeOff className="size-4" />
                              ) : (
                                <Eye className="size-4" />
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

      <CardContent className="pt-3">
        <Collapsible open={isTimelineOpen} onOpenChange={setIsTimelineOpen}>
          <CollapsibleTrigger asChild>
            <button
              type="button"
              className="flex w-full items-center justify-between rounded-xl border border-slate-200/80 bg-slate-50/70 px-3 py-2 text-left transition-colors hover:bg-slate-100/80"
            >
              <span className="text-sm font-medium text-slate-700">
                最后编辑 {lastEditedAt}
              </span>
              <ChevronDown
                className={[
                  "size-4 text-slate-500 transition-transform",
                  isTimelineOpen ? "rotate-180" : "",
                ].join(" ")}
              />
            </button>
          </CollapsibleTrigger>

          <CollapsibleContent className="pt-3">
            {timelineEvents.length > 0 ? (
              <ol className="relative ml-1 space-y-3 border-l border-slate-200 pl-4">
                {timelineEvents.map((event) => (
                  <li
                    key={`${event.label}-${event.date.toISOString()}`}
                    className="relative"
                  >
                    <span className="absolute -left-5.25 top-1.5 size-2 rounded-full bg-slate-400" />
                    <div className="text-xs text-slate-500">{event.label}</div>
                    <div className="text-sm font-medium text-slate-900">
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
