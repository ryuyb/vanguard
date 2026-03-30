import { FileText, Paperclip } from "lucide-react";
import { forwardRef } from "react";
import { useTranslation } from "react-i18next";
import type { SendItemDto } from "@/bindings";
import { Badge } from "@/components/ui/badge";
import { isSendExpired } from "../utils";

type SendRowProps = React.ComponentPropsWithoutRef<"button"> & {
  send: SendItemDto;
  selected: boolean;
};

export const SendRow = forwardRef<HTMLButtonElement, SendRowProps>(
  function SendRow({ send, selected, onClick, ...props }, ref) {
    const { t } = useTranslation();
    const expired = isSendExpired(send);
    const Icon = send.type === 1 ? Paperclip : FileText;

    const subtitle = send.expirationDate
      ? `${t("send.list.expires")} ${new Date(send.expirationDate).toLocaleDateString()}`
      : send.maxAccessCount != null
        ? `${send.accessCount ?? 0}/${send.maxAccessCount} ${t("send.list.views")}`
        : t("send.list.noExpiration");

    return (
      <button
        ref={ref}
        type="button"
        onClick={onClick}
        {...props}
        className={[
          "w-full min-w-0 rounded-lg px-3 py-2.5 text-left transition-all border overflow-hidden",
          selected
            ? "bg-blue-50 border-blue-200 text-blue-900 shadow-sm"
            : "bg-white border-slate-200 text-slate-800 hover:bg-slate-50 hover:border-slate-300",
        ].join(" ")}
      >
        <div className="flex items-center gap-3 min-w-0">
          <span
            className={[
              "inline-flex size-9 shrink-0 items-center justify-center rounded-lg border",
              selected
                ? "bg-white border-blue-200"
                : "bg-white border-slate-200",
            ].join(" ")}
          >
            <Icon className="size-4 text-slate-500" />
          </span>
          <div className="min-w-0 flex-1 overflow-hidden">
            <div className="flex items-center gap-2 min-w-0">
              <span className="truncate text-sm font-semibold">
                {send.name ?? t("send.list.untitled")}
              </span>
              {send.disabled && (
                <Badge variant="secondary" className="shrink-0 text-xs">
                  {t("send.list.disabled")}
                </Badge>
              )}
              {expired && !send.disabled && (
                <Badge variant="destructive" className="shrink-0 text-xs">
                  {t("send.list.expired")}
                </Badge>
              )}
            </div>
            <div className="mt-0.5 truncate text-xs text-slate-500">
              {subtitle}
            </div>
          </div>
        </div>
      </button>
    );
  },
);
