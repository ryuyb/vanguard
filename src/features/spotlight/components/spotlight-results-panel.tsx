import { Command as CommandPrimitive } from "cmdk";
import { CommandItem } from "@/components/ui/command";
import { Kbd, KbdGroup } from "@/components/ui/kbd";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Skeleton } from "@/components/ui/skeleton";
import type {
  CopyField,
  DetailAction,
  SpotlightItem,
} from "@/features/spotlight/types";
import { cn } from "@/lib/utils";

type SpotlightResultsPanelProps = {
  shouldShowResults: boolean;
  isLoadingVault: boolean;
  hasQuery: boolean;
  detailItem: SpotlightItem | null;
  detailActions: readonly DetailAction[];
  detailActionIndex: number;
  copiedDetailField: CopyField | null;
  visibleItems: SpotlightItem[];
  copiedItemId: string | null;
};

export function SpotlightResultsPanel({
  shouldShowResults,
  isLoadingVault,
  hasQuery,
  detailItem,
  detailActions,
  detailActionIndex,
  copiedDetailField,
  visibleItems,
  copiedItemId,
}: SpotlightResultsPanelProps) {
  if (!shouldShowResults) {
    return null;
  }

  return (
    <ScrollArea className="mt-2.5 h-68">
      {isLoadingVault && hasQuery ? (
        <div className="grid gap-2 py-0.5" aria-hidden>
          {Array.from({ length: 6 }).map((_, index) => (
            <div key={`skeleton-${index}`} className="rounded-xl px-3 py-2.5">
              <Skeleton className="h-4 w-3/5" />
              <Skeleton className="mt-2 h-3 w-2/5" />
            </div>
          ))}
        </div>
      ) : detailItem ? (
        <section className="px-2.5 pt-3 pb-2.5" aria-label="Cipher detail">
          <p className="text-base leading-tight font-semibold text-slate-900">
            {detailItem.title}
          </p>
          <p className="mt-1 text-xs text-slate-600">{detailItem.subtitle}</p>
          <div className="mt-3 grid gap-2" role="listbox">
            {detailActions.map((action, index) => (
              <div
                key={action.label}
                role="option"
                tabIndex={-1}
                aria-selected={detailActionIndex === index}
                className={cn(
                  "flex items-center justify-between gap-3 rounded-xl border border-slate-400/25 bg-white/50 px-3 py-2.5 text-[13px] leading-[1.3] text-slate-900 transition-colors",
                  detailActionIndex === index &&
                    "border-blue-800/30 bg-blue-800/12",
                  copiedDetailField === action.field &&
                    "animate-in fade-in-0 duration-150 bg-blue-800/25",
                )}
              >
                <span>{action.label}</span>
                <KbdGroup className="shrink-0 text-slate-500">
                  {action.shortcut.map((shortcutKey) => (
                    <Kbd key={`${action.label}-${shortcutKey}`}>
                      {shortcutKey}
                    </Kbd>
                  ))}
                </KbdGroup>
              </div>
            ))}
          </div>
        </section>
      ) : (
        <CommandPrimitive.List className="px-0.5">
          {visibleItems.map((item) => (
            <CommandItem
              key={item.id}
              value={item.id}
              data-spotlight-item="true"
              className={cn(
                "w-full justify-between gap-3 rounded-xl border border-transparent bg-transparent px-3 py-2.5 text-left data-[selected=true]:bg-blue-800/12",
                copiedItemId === item.id &&
                  "animate-in fade-in-0 duration-150 bg-blue-800/20 data-[selected=true]:bg-blue-800/20",
              )}
            >
              <div>
                <p className="text-sm text-slate-900">{item.title}</p>
                <p className="mt-0.5 text-xs text-slate-600">{item.subtitle}</p>
              </div>
            </CommandItem>
          ))}
        </CommandPrimitive.List>
      )}
    </ScrollArea>
  );
}
