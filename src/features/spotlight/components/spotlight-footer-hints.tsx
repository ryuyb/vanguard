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
              复制 用户名
            </span>
            <span className="inline-flex items-center gap-1 text-[11px] leading-none text-slate-600">
              <KbdGroup>
                <Kbd className="bg-slate-200 text-slate-700">⌘</Kbd>
                <Kbd className="bg-slate-200 text-slate-700">⇧</Kbd>
                <Kbd className="bg-slate-200 text-slate-700">C</Kbd>
              </KbdGroup>
              复制 密码
            </span>
          </>
        )}
        <span className="inline-flex items-center gap-1 text-[11px] leading-none text-slate-600">
          <Kbd className="bg-slate-200 text-slate-700">{detailItem ? "←" : "→"}</Kbd>
          {detailItem ? "返回结果" : "更多操作"}
        </span>
        {detailItem ? (
          <span className="inline-flex items-center gap-1 text-[11px] leading-none text-slate-600">
            <KbdGroup>
              <Kbd className="bg-slate-200 text-slate-700">↑</Kbd>
              <Kbd className="bg-slate-200 text-slate-700">↓</Kbd>
            </KbdGroup>
            选择
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
        Open quick access
      </span>
      <span className="inline-flex items-center gap-1 text-[11px] leading-none text-slate-600">
        <Kbd className="bg-slate-200 text-slate-700">Esc</Kbd>
        Close
      </span>
    </>
  );
}
