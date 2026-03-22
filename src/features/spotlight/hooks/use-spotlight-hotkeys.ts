import type { Dispatch, KeyboardEvent, SetStateAction } from "react";
import { useCallback, useEffect } from "react";
import {
  isInputCaretAtEnd,
  toCopyFieldFromShortcut,
} from "@/features/spotlight/hotkey-utils";
import type {
  CopyField,
  DetailAction,
  SpotlightItem,
} from "@/features/spotlight/types";

type UseSpotlightHotkeysParams = {
  detailActionIndex: number;
  detailActions: readonly DetailAction[];
  detailHasTotp: boolean;
  detailItem: SpotlightItem | null;
  hasVisibleResults: boolean;
  hideSpotlight: () => Promise<void>;
  query: string;
  runCopyAction: (item: SpotlightItem, field: CopyField) => Promise<void>;
  setDetailActionIndex: Dispatch<SetStateAction<number>>;
  setDetailItemId: Dispatch<SetStateAction<string | null>>;
  setQuery: Dispatch<SetStateAction<string>>;
  visibleItems: SpotlightItem[];
};

type UseSpotlightHotkeysResult = {
  onCommandInputKeyDown: (event: KeyboardEvent<HTMLInputElement>) => void;
};

export function useSpotlightHotkeys({
  detailActionIndex,
  detailActions,
  detailHasTotp,
  detailItem,
  hasVisibleResults,
  hideSpotlight,
  query,
  runCopyAction,
  setDetailActionIndex,
  setDetailItemId,
  setQuery,
  visibleItems,
}: UseSpotlightHotkeysParams): UseSpotlightHotkeysResult {
  const resolveSelectedItemId = useCallback(() => {
    const selectedElement = document.querySelector<HTMLElement>(
      "#spotlight-card [data-spotlight-item='true'][data-selected='true']",
    );
    const selectedValue = selectedElement?.getAttribute("data-value");
    if (
      selectedValue &&
      visibleItems.some((item) => item.id === selectedValue)
    ) {
      return selectedValue;
    }
    return visibleItems[0]?.id ?? null;
  }, [visibleItems]);

  const onCommandInputKeyDown = useCallback(
    (event: KeyboardEvent<HTMLInputElement>) => {
      const field = toCopyFieldFromShortcut(event);
      if (field) {
        if (field === "totp" && (!detailItem || !detailHasTotp)) {
          return;
        }

        event.preventDefault();
        if (detailItem) {
          void runCopyAction(detailItem, field);
          return;
        }
        const selectedItemId = resolveSelectedItemId();
        if (!selectedItemId) {
          return;
        }
        const selectedItem =
          visibleItems.find((item) => item.id === selectedItemId) ?? null;
        if (!selectedItem) {
          return;
        }
        void runCopyAction(selectedItem, field);
        return;
      }

      if (event.key === "Enter" && detailItem) {
        event.preventDefault();
        const field = detailActions[detailActionIndex]?.field ?? "username";
        void runCopyAction(detailItem, field);
        return;
      }

      if (
        detailItem &&
        (event.key === "ArrowDown" || event.key === "ArrowUp")
      ) {
        event.preventDefault();
        const key = event.key;
        setDetailActionIndex((current) => {
          return key === "ArrowDown"
            ? (current + 1) % detailActions.length
            : (current - 1 + detailActions.length) % detailActions.length;
        });
        return;
      }

      if (event.key === "ArrowRight" && hasVisibleResults && !detailItem) {
        if (!isInputCaretAtEnd(event.currentTarget)) {
          return;
        }
        event.preventDefault();
        const selectedItemId = resolveSelectedItemId();
        if (selectedItemId) {
          setDetailItemId(selectedItemId);
        }
        return;
      }

      if (event.key === "ArrowLeft" && detailItem) {
        event.preventDefault();
        setDetailItemId(null);
        return;
      }

      if (event.key === "Escape") {
        event.preventDefault();
        if (detailItem) {
          setDetailItemId(null);
          return;
        }
        if (query.trim().length > 0) {
          setQuery("");
          return;
        }
        void hideSpotlight();
      }
    },
    [
      detailActions,
      detailActionIndex,
      detailHasTotp,
      detailItem,
      hasVisibleResults,
      hideSpotlight,
      query,
      resolveSelectedItemId,
      runCopyAction,
      setDetailActionIndex,
      setDetailItemId,
      setQuery,
      visibleItems,
    ],
  );

  // 全局键盘监听：处理焦点不在输入框时的情况
  useEffect(() => {
    const handleGlobalKeyDown = (event: globalThis.KeyboardEvent) => {
      // 忽略输入框有焦点的情况（避免与 onCommandInputKeyDown 重复触发）
      const activeElement = document.activeElement;
      if (activeElement?.tagName === "INPUT" || activeElement?.getAttribute("role") === "combobox") return;

      // ========== 列表页模式 ==========
      if (!detailItem && hasVisibleResults) {
        // 处理 ESC：关闭 spotlight
        if (event.key === "Escape") {
          event.preventDefault();
          if (query.trim().length > 0) {
            setQuery("");
          } else {
            void hideSpotlight();
          }
          return;
        }

        // 处理 Enter：进入详情页
        if (event.key === "Enter") {
          event.preventDefault();
          const selectedItemId = resolveSelectedItemId();
          if (selectedItemId) {
            setDetailItemId(selectedItemId);
          }
          return;
        }

        // 处理上下方向键：手动导航列表项
        if (event.key === "ArrowDown" || event.key === "ArrowUp") {
          event.preventDefault();
          const items = document.querySelectorAll<HTMLElement>(
            "#spotlight-card [data-spotlight-item='true']",
          );
          if (items.length === 0) return;

          let currentIndex = Array.from(items).findIndex(
            (item) => item.getAttribute("data-selected") === "true",
          );
          if (currentIndex === -1) currentIndex = 0;

          const newIndex = event.key === "ArrowDown"
            ? (currentIndex + 1) % items.length
            : (currentIndex - 1 + items.length) % items.length;

          // 更新 cmdk 的选中状态
          items.forEach((item, i) => {
            item.setAttribute("data-selected", i === newIndex ? "true" : "false");
          });

          // 滚动到可见
          items[newIndex]?.scrollIntoView({ block: "nearest" });
          return;
        }

        // 处理右键：进入详情页
        if (event.key === "ArrowRight") {
          event.preventDefault();
          const selectedItemId = resolveSelectedItemId();
          if (selectedItemId) {
            setDetailItemId(selectedItemId);
          }
          return;
        }

        // 处理复制快捷键
        const field = toCopyFieldFromShortcut(event);
        if (field) {
          event.preventDefault();
          const selectedItemId = resolveSelectedItemId();
          if (!selectedItemId) return;
          const selectedItem =
            visibleItems.find((item) => item.id === selectedItemId) ?? null;
          if (!selectedItem) return;
          void runCopyAction(selectedItem, field);
          return;
        }

        return;
      }

      // ========== 详情页模式 ==========
      if (!detailItem) return;

      // 处理详情页的上下导航
      if (event.key === "ArrowDown" || event.key === "ArrowUp") {
        event.preventDefault();
        const key = event.key;
        setDetailActionIndex((current) => {
          return key === "ArrowDown"
            ? (current + 1) % detailActions.length
            : (current - 1 + detailActions.length) % detailActions.length;
        });
        return;
      }

      // 处理左键：返回列表
      if (event.key === "ArrowLeft") {
        event.preventDefault();
        setDetailItemId(null);
        return;
      }

      // 处理 ESC：返回列表
      if (event.key === "Escape") {
        event.preventDefault();
        setDetailItemId(null);
        return;
      }

      // 处理 Enter：执行当前选中的操作
      if (event.key === "Enter") {
        event.preventDefault();
        const field = detailActions[detailActionIndex]?.field ?? "username";
        void runCopyAction(detailItem, field);
        return;
      }

      // 处理复制快捷键（U/P/T 等）
      const field = toCopyFieldFromShortcut(event);
      if (field) {
        if (field === "totp" && !detailHasTotp) {
          return;
        }
        event.preventDefault();
        void runCopyAction(detailItem, field);
      }
    };

    window.addEventListener("keydown", handleGlobalKeyDown);
    return () => window.removeEventListener("keydown", handleGlobalKeyDown);
  }, [
    detailItem,
    detailActions,
    detailActionIndex,
    detailHasTotp,
    hasVisibleResults,
    query,
    visibleItems,
    setDetailActionIndex,
    setDetailItemId,
    setQuery,
    hideSpotlight,
    resolveSelectedItemId,
    runCopyAction,
  ]);

  return {
    onCommandInputKeyDown,
  };
}
