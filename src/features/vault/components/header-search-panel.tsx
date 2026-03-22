import { motion } from "motion/react";
import { useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import {
  CipherIcon,
  toCipherTypeIcon,
} from "@/features/vault/components/cipher-icon";
import { useIcon } from "@/features/vault/hooks/use-icon";
import type { CipherWithIcon } from "@/features/vault/types";

type HeaderSearchPanelProps = {
  results: CipherWithIcon[];
  onSelect: (cipherId: string) => void;
  onClose: () => void;
};

const SearchResultItem = ({
  cipher,
  isSelected,
  onClick,
  itemRef,
}: {
  cipher: CipherWithIcon;
  isSelected: boolean;
  onClick: () => void;
  itemRef: (el: HTMLButtonElement | null) => void;
}) => {
  const { data: iconData } = useIcon(cipher.iconHostname ?? null);

  return (
    <button
      ref={itemRef}
      type="button"
      onClick={onClick}
      className={[
        "w-full min-w-0 rounded-lg px-3 py-2 text-left transition-all border",
        isSelected
          ? "bg-blue-50 border-blue-200 text-blue-900"
          : "bg-white border-transparent hover:bg-slate-50 hover:border-slate-200",
      ].join(" ")}
    >
      <div className="flex items-center gap-3 min-w-0">
        <CipherIcon
          alt={cipher.name ?? "Cipher"}
          className={[
            "bg-white text-slate-500 border shrink-0",
            isSelected ? "border-blue-200" : "border-slate-200",
          ].join(" ")}
          iconData={iconData}
        >
          {toCipherTypeIcon(cipher.type)}
        </CipherIcon>
        <div className="min-w-0 flex-1 shrink overflow-hidden">
          <div className="truncate text-sm font-semibold">
            {cipher.name ?? "Untitled cipher"}
          </div>
          <div className="truncate text-xs text-slate-500">
            {cipher.username ?? ""}
          </div>
        </div>
      </div>
    </button>
  );
};

export function HeaderSearchPanel({
  results,
  onSelect,
  onClose,
}: HeaderSearchPanelProps) {
  const { t } = useTranslation();
  const panelRef = useRef<HTMLDivElement>(null);
  const itemRefs = useRef<(HTMLButtonElement | null)[]>([]);
  const [selectedIndex, setSelectedIndex] = useState(0);

  // Reset selection when results change and cleanup refs
  useEffect(() => {
    setSelectedIndex(0);
    // Cleanup refs that are no longer needed
    itemRefs.current = itemRefs.current.slice(0, results.length);
  }, [results]);

  // Auto-scroll to selected item
  useEffect(() => {
    const selectedItem = itemRefs.current[selectedIndex];
    if (selectedItem) {
      selectedItem.scrollIntoView({ block: "nearest" });
    }
  }, [selectedIndex]);

  // Close on click outside
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (
        panelRef.current &&
        !panelRef.current.contains(event.target as Node)
      ) {
        onClose();
      }
    };

    document.addEventListener("mousedown", handleClickOutside);
    return () => {
      document.removeEventListener("mousedown", handleClickOutside);
    };
  }, [onClose]);

  // Keyboard navigation - only when panel is visible
  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      if (results.length === 0) return;

      // Only handle keys when the panel is active (has visible results)
      const activeElement = document.activeElement;
      const isInputFocused = activeElement?.tagName === "INPUT";

      switch (event.key) {
        case "ArrowDown":
          event.preventDefault();
          setSelectedIndex((prev) => (prev + 1) % results.length);
          break;
        case "ArrowUp":
          event.preventDefault();
          setSelectedIndex((prev) =>
            prev === 0 ? results.length - 1 : prev - 1,
          );
          break;
        case "Enter":
          // Only handle Enter if input is focused or panel has focus
          if (
            !isInputFocused &&
            !panelRef.current?.contains(activeElement as Node)
          ) {
            return;
          }
          event.preventDefault();
          if (results[selectedIndex]) {
            onSelect(results[selectedIndex].id);
            onClose();
          }
          break;
        case "Escape":
          event.preventDefault();
          onClose();
          break;
        case "Home":
          event.preventDefault();
          setSelectedIndex(0);
          break;
        case "End":
          event.preventDefault();
          setSelectedIndex(results.length - 1);
          break;
      }
    };

    document.addEventListener("keydown", handleKeyDown);
    return () => {
      document.removeEventListener("keydown", handleKeyDown);
    };
  }, [results, selectedIndex, onSelect, onClose]);

  if (results.length === 0) {
    return null;
  }

  return (
    <motion.div
      ref={panelRef}
      initial={{ opacity: 0, y: -8, scale: 0.96 }}
      animate={{ opacity: 1, y: 0, scale: 1 }}
      exit={{ opacity: 0, y: -8, scale: 0.96 }}
      transition={{ duration: 0.15, ease: "easeOut" }}
      className="absolute top-full left-0 right-0 z-50 mt-1 rounded-xl border border-slate-200 bg-white shadow-lg overflow-hidden"
    >
      <div className="max-h-80 overflow-y-auto p-2">
        <div className="space-y-1">
          {results.map((cipher, index) => (
            <SearchResultItem
              key={cipher.id}
              cipher={cipher}
              isSelected={index === selectedIndex}
              itemRef={(el) => {
                itemRefs.current[index] = el;
              }}
              onClick={() => {
                onSelect(cipher.id);
                onClose();
              }}
            />
          ))}
        </div>
      </div>
      {results.length > 0 && (
        <div className="border-t border-slate-100 bg-slate-50/50 px-3 py-1.5 text-xs text-slate-400">
          {t("vault.page.search.resultCount", { count: results.length })}
        </div>
      )}
    </motion.div>
  );
}
