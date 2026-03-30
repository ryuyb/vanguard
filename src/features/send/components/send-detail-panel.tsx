import {
  Copy,
  Edit2,
  Eye,
  EyeOff,
  FileText,
  Paperclip,
  Trash2,
} from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";
import type { SendItemDto } from "@/bindings";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { toast } from "@/lib/toast";
import { cn } from "@/lib/utils";
import { generateSendLink, isSendExpired } from "../utils";

type SendDetailPanelProps = {
  sendId: string | null;
  sends: SendItemDto[];
  baseUrl: string;
  onEdit: (send: SendItemDto) => void;
  onDelete: (sendId: string, sendName: string) => void;
};

function DetailRow({ label, value }: { label: string; value?: string | null }) {
  if (!value) return null;
  return (
    <div className="flex items-start justify-between gap-4 py-1.5 border-b border-slate-100 last:border-0">
      <span className="text-xs text-slate-500 shrink-0 w-32">{label}</span>
      <span className="text-xs text-slate-800 font-medium text-right break-all">
        {value}
      </span>
    </div>
  );
}

export function SendDetailPanel({
  sendId,
  sends,
  baseUrl,
  onEdit,
  onDelete,
}: SendDetailPanelProps) {
  const { t } = useTranslation();
  const [textVisible, setTextVisible] = useState(false);

  const send = sends.find((s) => s.id === sendId) ?? null;

  if (!send) {
    return (
      <div className="flex h-full items-center justify-center text-sm text-slate-400">
        {t("send.detail.selectPrompt")}
      </div>
    );
  }

  const isFile = send.type === 1;
  const expired = isSendExpired(send);
  const Icon = isFile ? Paperclip : FileText;
  const sendLink = generateSendLink(
    baseUrl,
    (send as SendItemDto & { accessId?: string }).accessId,
    (send as SendItemDto & { key?: string }).key,
  );
  const handleCopyLink = async () => {
    if (!sendLink) return;
    await navigator.clipboard.writeText(sendLink);
    toast.success(t("send.detail.linkCopied"));
  };

  return (
    <div className="flex flex-col gap-4 p-4">
      {/* Header */}
      <div className="flex items-start justify-between gap-2">
        <div className="flex items-center gap-2 min-w-0">
          <span className="inline-flex size-9 shrink-0 items-center justify-center rounded-lg border border-slate-200 bg-white">
            <Icon className="size-4 text-slate-500" />
          </span>
          <div className="min-w-0">
            <p className="text-base font-bold text-slate-900 truncate">
              {send.name ?? t("send.list.untitled")}
            </p>
            <div className="flex items-center gap-1.5 mt-0.5">
              <Badge variant="secondary" className="text-xs">
                {isFile ? t("send.types.file") : t("send.types.text")}
              </Badge>
              {send.disabled && (
                <Badge variant="secondary" className="text-xs">
                  {t("send.list.disabled")}
                </Badge>
              )}
              {expired && (
                <Badge variant="destructive" className="text-xs">
                  {t("send.list.expired")}
                </Badge>
              )}
            </div>
          </div>
        </div>
        <div className="flex items-center gap-1 shrink-0">
          <Button
            variant="ghost"
            size="sm"
            className="h-8 w-8 p-0"
            onClick={() => onEdit(send)}
          >
            <Edit2 className="size-4" />
          </Button>
          <Button
            variant="ghost"
            size="sm"
            className="h-8 w-8 p-0 text-red-500 hover:text-red-600 hover:bg-red-50"
            onClick={() =>
              onDelete(send.id, send.name ?? t("send.list.untitled"))
            }
          >
            <Trash2 className="size-4" />
          </Button>
        </div>
      </div>

      {/* Send Link */}
      {sendLink && (
        <div className="rounded-lg border border-slate-200 bg-white px-3.5 py-3 shadow-sm">
          <div className="text-[10px] font-semibold tracking-wider text-slate-500 uppercase mb-2">
            {t("send.detail.sendLink")}
          </div>
          <div className="flex items-center gap-2">
            <span className="flex-1 truncate text-xs text-slate-700 font-mono">
              {sendLink}
            </span>
            <Button
              variant="ghost"
              size="sm"
              className="h-7 w-7 p-0 shrink-0"
              onClick={() => void handleCopyLink()}
            >
              <Copy className="size-3.5" />
            </Button>
          </div>
        </div>
      )}

      {/* Text content */}
      {!isFile && (
        <div className="rounded-lg border border-slate-200 bg-white px-3.5 py-3 shadow-sm">
          <div className="flex items-center justify-between mb-2">
            <div className="text-[10px] font-semibold tracking-wider text-slate-500 uppercase">
              {t("send.detail.textContent")}
            </div>
            <Button
              variant="ghost"
              size="sm"
              className="h-6 w-6 p-0"
              onClick={() => setTextVisible((v) => !v)}
            >
              {textVisible ? (
                <EyeOff className="size-3.5" />
              ) : (
                <Eye className="size-3.5" />
              )}
            </Button>
          </div>
          <div
            className={cn(
              "text-sm font-medium text-slate-900 break-words",
              !textVisible && "blur-sm select-none",
            )}
          >
            {t("send.detail.textHidden")}
          </div>
        </div>
      )}

      {/* Details */}
      <div className="rounded-lg border border-slate-200 bg-white px-3.5 py-3 shadow-sm">
        <div className="text-[10px] font-semibold tracking-wider text-slate-500 uppercase mb-2">
          {t("send.detail.details")}
        </div>
        <DetailRow
          label={t("send.detail.password")}
          value={
            send.hasPassword
              ? t("send.detail.passwordProtected")
              : t("send.detail.passwordNone")
          }
        />
        <DetailRow
          label={t("send.detail.maxViews")}
          value={
            send.maxAccessCount != null ? String(send.maxAccessCount) : null
          }
        />
        <DetailRow
          label={t("send.detail.currentViews")}
          value={send.accessCount != null ? String(send.accessCount) : "0"}
        />
        <DetailRow
          label={t("send.detail.disabled")}
          value={send.disabled ? t("common.yes") : t("common.no")}
        />
      </div>

      {/* Dates */}
      <div className="rounded-lg border border-slate-200 bg-white px-3.5 py-3 shadow-sm">
        <div className="text-[10px] font-semibold tracking-wider text-slate-500 uppercase mb-2">
          {t("send.detail.dates")}
        </div>
        <DetailRow
          label={t("send.detail.expiration")}
          value={
            send.expirationDate
              ? new Date(send.expirationDate).toLocaleString()
              : t("send.detail.noExpiration")
          }
        />
        <DetailRow
          label={t("send.detail.deletion")}
          value={
            send.deletionDate
              ? new Date(send.deletionDate).toLocaleString()
              : undefined
          }
        />
        <DetailRow
          label={t("send.detail.lastUpdated")}
          value={
            send.revisionDate
              ? new Date(send.revisionDate).toLocaleString()
              : undefined
          }
        />
      </div>
    </div>
  );
}
