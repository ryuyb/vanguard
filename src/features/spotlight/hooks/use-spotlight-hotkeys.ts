import type { Dispatch, KeyboardEvent, SetStateAction } from "react";
import { useCallback } from "react";
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
        setDetailActionIndex((current) => {
          if (event.key === "ArrowDown") {
            return (current + 1) % detailActions.length;
          }
          return (current - 1 + detailActions.length) % detailActions.length;
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

  return {
    onCommandInputKeyDown,
  };
}
