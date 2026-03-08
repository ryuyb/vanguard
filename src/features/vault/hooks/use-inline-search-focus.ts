import { useEffect } from "react";

export function useInlineSearchFocus({
  inlineSearchInputRef,
  isInlineSearchOpen,
}: {
  inlineSearchInputRef: React.RefObject<HTMLInputElement | null>;
  isInlineSearchOpen: boolean;
}) {
  useEffect(() => {
    if (!isInlineSearchOpen) {
      return;
    }
    const frameId = requestAnimationFrame(() => {
      inlineSearchInputRef.current?.focus();
    });
    return () => {
      cancelAnimationFrame(frameId);
    };
  }, [inlineSearchInputRef, isInlineSearchOpen]);
}
