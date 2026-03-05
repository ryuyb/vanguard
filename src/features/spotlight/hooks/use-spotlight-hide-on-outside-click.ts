import { useEffect } from "react";

type UseHideOnOutsideClickParams = {
  cardId: string;
  onOutsideClick: () => void;
};

export function useSpotlightHideOnOutsideClick({
  cardId,
  onOutsideClick,
}: UseHideOnOutsideClickParams): void {
  useEffect(() => {
    const onDocumentMouseDown = (event: globalThis.MouseEvent) => {
      const target = event.target;
      if (!(target instanceof Node)) {
        return;
      }
      const cardElement = document.getElementById(cardId);
      if (cardElement?.contains(target)) {
        return;
      }
      onOutsideClick();
    };

    document.addEventListener("mousedown", onDocumentMouseDown, true);
    return () => {
      document.removeEventListener("mousedown", onDocumentMouseDown, true);
    };
  }, [cardId, onOutsideClick]);
}
