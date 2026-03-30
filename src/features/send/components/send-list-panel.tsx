import { ChevronDown, Copy, Edit2, Plus, Send, Trash2 } from "lucide-react";
import { useTranslation } from "react-i18next";
import type { SendItemDto } from "@/bindings";
import { Button } from "@/components/ui/button";
import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuTrigger,
} from "@/components/ui/context-menu";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuRadioGroup,
  DropdownMenuRadioItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { ScrollArea } from "@/components/ui/scroll-area";
import type { SendTypeFilter } from "../types";
import { generateSendLink } from "../utils";
import { SendRow } from "./send-row";

type SendListPanelProps = {
  sends: SendItemDto[];
  isLoading: boolean;
  selectedSendId: string | null;
  sendTypeFilter: SendTypeFilter;
  baseUrl: string;
  onSelectSend: (id: string) => void;
  onCreateSend: () => void;
  onFilterChange: (filter: SendTypeFilter) => void;
  onEdit: (send: SendItemDto) => void;
  onDelete: (sendId: string, sendName: string) => void;
};

export function SendListPanel({
  sends,
  isLoading,
  selectedSendId,
  sendTypeFilter,
  baseUrl,
  onSelectSend,
  onCreateSend,
  onFilterChange,
  onEdit,
  onDelete,
}: SendListPanelProps) {
  const { t } = useTranslation();

  const filterLabel: Record<SendTypeFilter, string> = {
    all: t("send.types.all"),
    text: t("send.types.text"),
    file: t("send.types.file"),
  };

  return (
    <section className="flex h-full min-h-0 flex-col bg-slate-50/50 border-r border-slate-200">
      {/* Toolbar */}
      <div className="flex items-center justify-between gap-2 px-3 pt-3 pb-2 bg-white border-b border-slate-200">
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button
              type="button"
              variant="ghost"
              className="h-8 justify-start gap-1 rounded-md px-2.5 text-xs font-medium text-slate-700 hover:bg-slate-100 data-[state=open]:bg-slate-100"
            >
              <span>{filterLabel[sendTypeFilter]}</span>
              <ChevronDown className="size-3.5 text-slate-400" />
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align="start" className="w-36">
            <DropdownMenuRadioGroup
              value={sendTypeFilter}
              onValueChange={(v) => onFilterChange(v as SendTypeFilter)}
            >
              <DropdownMenuRadioItem value="all">
                {t("send.types.all")}
              </DropdownMenuRadioItem>
              <DropdownMenuRadioItem value="text">
                {t("send.types.text")}
              </DropdownMenuRadioItem>
              <DropdownMenuRadioItem value="file">
                {t("send.types.file")}
              </DropdownMenuRadioItem>
            </DropdownMenuRadioGroup>
          </DropdownMenuContent>
        </DropdownMenu>

        <Button
          type="button"
          variant="ghost"
          size="sm"
          onClick={onCreateSend}
          className="h-8 w-8 p-0"
          title={t("send.list.create")}
        >
          <Plus className="size-4" />
        </Button>
      </div>

      {/* List */}
      <ScrollArea className="min-h-0 flex-1 overflow-hidden [&>div>div]:!block">
        <div className="space-y-1.5 px-3 py-2">
          {isLoading ? null : sends.length === 0 ? (
            <div className="flex flex-col items-center justify-center py-16 text-center">
              <Send className="size-10 text-slate-300 mb-3" />
              <p className="text-sm font-medium text-slate-600">
                {t("send.list.empty.title")}
              </p>
              <p className="mt-1 text-xs text-slate-400">
                {t("send.list.empty.description")}
              </p>
              <Button
                type="button"
                variant="outline"
                size="sm"
                onClick={onCreateSend}
                className="mt-4"
              >
                <Plus className="size-3.5 mr-1" />
                {t("send.list.empty.action")}
              </Button>
            </div>
          ) : (
            sends.map((send) => (
              <ContextMenu key={send.id}>
                <ContextMenuTrigger asChild>
                  <SendRow
                    send={send}
                    selected={send.id === selectedSendId}
                    onClick={() => onSelectSend(send.id)}
                  />
                </ContextMenuTrigger>
                <ContextMenuContent>
                  <ContextMenuItem onClick={() => onEdit(send)}>
                    <Edit2 className="size-4 mr-2" />
                    {t("send.contextMenu.edit")}
                  </ContextMenuItem>
                  <ContextMenuItem
                    onClick={() => {
                      const link = generateSendLink(
                        baseUrl,
                        send.accessId,
                        send.urlKey,
                      );
                      if (link) void navigator.clipboard.writeText(link);
                    }}
                  >
                    <Copy className="size-4 mr-2" />
                    {t("send.contextMenu.copyLink")}
                  </ContextMenuItem>
                  <ContextMenuItem
                    className="text-red-600 focus:text-red-600"
                    onClick={() =>
                      onDelete(send.id, send.name ?? t("send.list.untitled"))
                    }
                  >
                    <Trash2 className="size-4 mr-2" />
                    {t("send.contextMenu.delete")}
                  </ContextMenuItem>
                </ContextMenuContent>
              </ContextMenu>
            ))
          )}
        </div>
      </ScrollArea>
    </section>
  );
}
