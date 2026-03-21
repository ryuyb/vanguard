import { Command as CommandPrimitive } from "cmdk";
import { useTranslation } from "react-i18next";
import { CommandItem } from "@/components/ui/command";
import { Kbd, KbdGroup } from "@/components/ui/kbd";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Skeleton } from "@/components/ui/skeleton";
import type {
  CopyField,
  DetailAction,
  SpotlightItem,
} from "@/features/spotlight/types";
import {
  CipherIcon,
  toCipherTypeIcon,
} from "@/features/vault/components/cipher-icon";
import { useIcon } from "@/features/vault/hooks/use-icon";
import { cn } from "@/lib/utils";

// Component to load and display icon for spotlight items
function SpotlightItemIcon({ item }: { item: SpotlightItem }) {
  const { data: iconData } = useIcon(item.iconHostname);
  return (
    <CipherIcon
      alt={item.title}
      iconData={iconData}
      className="size-8 border-slate-200"
    >
      {toCipherTypeIcon(item.type)}
    </CipherIcon>
  );
}

// Component to load and display icon for detail panel
function SpotlightDetailIcon({ item }: { item: SpotlightItem }) {
  const { data: iconData } = useIcon(item.iconHostname);
  return (
    <CipherIcon
      alt={item.title}
      iconData={iconData}
      className="size-10 border-slate-200"
    >
      {toCipherTypeIcon(item.type)}
    </CipherIcon>
  );
}

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
  const { t } = useTranslation();

  if (!shouldShowResults) {
    return null;
  }

  return (
    <ScrollArea className="mt-3 h-68">
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
        <section
          className="px-2 pt-3 pb-2.5"
          aria-label={t("spotlight.search.detailRegionLabel")}
        >
          <div className="flex items-start gap-3 rounded-xl border border-slate-200 bg-slate-50/50 p-3">
            <SpotlightDetailIcon item={detailItem} />
            <div className="flex-1 min-w-0">
              <p className="text-base leading-tight font-semibold text-slate-900">
                {detailItem.title}
              </p>
              <p className="mt-1.5 text-sm text-slate-600">
                {detailItem.subtitle}
              </p>
            </div>
          </div>
          <div className="mt-3 grid gap-2" role="listbox">
            {detailActions.map((action, index) => (
              <div
                key={action.label}
                role="option"
                tabIndex={-1}
                aria-selected={detailActionIndex === index}
                className={cn(
                  "flex items-center justify-between gap-3 rounded-xl border px-3.5 py-3 text-sm leading-[1.3] transition-all",
                  detailActionIndex === index
                    ? "border-blue-200 bg-blue-50 text-blue-900 shadow-sm"
                    : "border-slate-200 bg-white text-slate-900 hover:bg-slate-50",
                  copiedDetailField === action.field &&
                    "animate-in fade-in-0 duration-150 border-emerald-200 bg-emerald-50",
                )}
              >
                <span className="font-medium">{action.label}</span>
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
      ) : hasQuery && visibleItems.length === 0 ? (
        <section className="px-2 pt-3 pb-2.5">
          <div className="rounded-xl border border-slate-200 bg-slate-50/50 px-4 py-6 text-center">
            <p className="text-sm font-medium text-slate-900">
              {t("spotlight.search.states.noResults.title")}
            </p>
            <p className="mt-1 text-xs text-slate-600">
              {t("spotlight.search.states.noResults.description")}
            </p>
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
                "w-full justify-between gap-3 rounded-xl border px-3 py-2.5 text-left transition-all",
                "border-transparent bg-transparent hover:border-slate-200 hover:bg-slate-50",
                "data-[selected=true]:border-blue-200 data-[selected=true]:bg-blue-50 data-[selected=true]:shadow-sm",
                copiedItemId === item.id &&
                  "animate-in fade-in-0 duration-150 border-emerald-200 bg-emerald-50 data-[selected=true]:border-emerald-200 data-[selected=true]:bg-emerald-50",
              )}
            >
              <div className="flex items-center gap-3">
                <SpotlightItemIcon item={item} />
                <div className="flex-1 min-w-0">
                  <p className="text-sm font-medium text-slate-900">
                    {item.title}
                  </p>
                  <p className="mt-0.5 text-xs text-slate-600">
                    {item.subtitle}
                  </p>
                </div>
              </div>
            </CommandItem>
          ))}
        </CommandPrimitive.List>
      )}
    </ScrollArea>
  );
}
