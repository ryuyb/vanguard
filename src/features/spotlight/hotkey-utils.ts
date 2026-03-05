import type { KeyboardEvent } from "react";
import type { CopyField } from "@/features/spotlight/types";

export function toCopyFieldFromShortcut(
  event: KeyboardEvent<HTMLInputElement>,
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
