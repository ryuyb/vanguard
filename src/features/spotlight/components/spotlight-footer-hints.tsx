import { useTranslation } from "react-i18next";
import { Kbd, KbdGroup } from "@/components/ui/kbd";
import type { SpotlightItem } from "@/features/spotlight/types";

type SpotlightFooterHintsProps = {
  hasVisibleResults: boolean;
  detailItem: SpotlightItem | null;
};

export function SpotlightFooterHints({
  hasVisibleResults,
  detailItem,
}: SpotlightFooterHintsProps) {
  const { t } = useTranslation();

  if (hasVisibleResults) {
    return (
      <>
        {detailItem ? null : (
          <>
            <span className="inline-flex items-center gap-1 text-[11px] leading-none text-slate-600">
              <KbdGroup>
                <Kbd className="bg-slate-200 text-slate-700">⌘</Kbd>
                <Kbd className="bg-slate-200 text-slate-700">C</Kbd>
              </KbdGroup>
              {t("spotlight.hints.copyUsername")}
            </span>
            <span className="inline-flex items-center gap-1 text-[11px] leading-none text-slate-600">
              <KbdGroup>
                <Kbd className="bg-slate-200 text-slate-700">⌘</Kbd>
                <Kbd className="bg-slate-200 text-slate-700">⇧</Kbd>
                <Kbd className="bg-slate-200 text-slate-700">C</Kbd>
              </KbdGroup>
              {t("spotlight.hints.copyPassword")}
            </span>
          </>
        )}
        <span className="inline-flex items-center gap-1 text-[11px] leading-none text-slate-600">
          <Kbd className="bg-slate-200 text-slate-700">
            {detailItem ? "←" : "→"}
          </Kbd>
          {detailItem
            ? t("spotlight.hints.backToResults")
            : t("spotlight.hints.moreActions")}
        </span>
        {detailItem ? (
          <span className="inline-flex items-center gap-1 text-[11px] leading-none text-slate-600">
            <KbdGroup>
              <Kbd className="bg-slate-200 text-slate-700">↑</Kbd>
              <Kbd className="bg-slate-200 text-slate-700">↓</Kbd>
            </KbdGroup>
            {t("spotlight.hints.select")}
          </span>
        ) : null}
      </>
    );
  }

  return (
    <>
      <span className="inline-flex items-center gap-1 text-[11px] leading-none text-slate-600">
        <KbdGroup>
          <Kbd className="bg-slate-200 text-slate-700">⇧</Kbd>
          <Kbd className="bg-slate-200 text-slate-700">⌃</Kbd>
          <Kbd className="bg-slate-200 text-slate-700">Space</Kbd>
        </KbdGroup>
        {t("spotlight.hints.openShortcut")}
      </span>
      <span className="inline-flex items-center gap-1 text-[11px] leading-none text-slate-600">
        <Kbd className="bg-slate-200 text-slate-700">Esc</Kbd>
        {t("spotlight.hints.close")}
      </span>
    </>
  );
}
