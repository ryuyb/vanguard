import type { KeyboardEvent as ReactKeyboardEvent } from "react";
import type { CopyField } from "@/features/spotlight/types";

// 支持 React 和原生两种 KeyboardEvent 类型
type KeyboardEventLike = Pick<
  KeyboardEvent,
  "key" | "metaKey" | "ctrlKey" | "altKey" | "shiftKey"
>;

export function toCopyFieldFromShortcut(
  event: ReactKeyboardEvent<HTMLInputElement> | KeyboardEventLike,
): CopyField | null {
  const normalizedKey = event.key.toLowerCase();
  const isCopyShortcut =
    (event.metaKey || event.ctrlKey) && normalizedKey === "c";
  if (!isCopyShortcut) {
    return null;
  }

  if (event.altKey && !event.shiftKey) {
    return "totp";
  }
  if (event.shiftKey && !event.altKey) {
    return "password";
  }
  if (!event.shiftKey && !event.altKey) {
    return "username";
  }
  return null;
}

export function isInputCaretAtEnd(inputElement: HTMLInputElement): boolean {
  const inputValueLength = inputElement.value.length;
  return (
    inputElement.selectionStart === inputValueLength &&
    inputElement.selectionEnd === inputValueLength
  );
}
