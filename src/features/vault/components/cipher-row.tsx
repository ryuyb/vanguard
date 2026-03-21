import type { VaultCipherItemDto } from "@/bindings";
import {
  CipherIcon,
  toCipherTypeIcon,
} from "@/features/vault/components/cipher-icon";
import { TruncatableText } from "@/features/vault/components/truncatable-text";
import { useIcon } from "@/features/vault/hooks/use-icon";

type CipherRowProps = {
  cipher: VaultCipherItemDto & {
    iconHostname?: string | null;
  };
  selected: boolean;
  onClick: () => void;
};

export function CipherRow({ cipher, selected, onClick }: CipherRowProps) {
  const { data: iconData } = useIcon(cipher.iconHostname ?? null);

  return (
    <button
      type="button"
      onClick={onClick}
      className={[
        "w-full min-w-0 rounded-lg px-3 py-2.5 text-left transition-all border overflow-hidden",
        selected
          ? "bg-blue-50 border-blue-200 text-blue-900 shadow-sm"
          : "bg-white border-slate-200 text-slate-800 hover:bg-slate-50 hover:border-slate-300",
      ].join(" ")}
    >
      <div className="flex items-center gap-3 min-w-0">
        <CipherIcon
          alt={cipher.name ?? "Cipher"}
          className={[
            "bg-white text-slate-500 border shrink-0",
            selected ? "border-blue-200" : "border-slate-200",
          ].join(" ")}
          iconData={iconData}
        >
          {toCipherTypeIcon(cipher.type)}
        </CipherIcon>
        <div className="min-w-0 flex-1 shrink overflow-hidden">
          <TruncatableText
            text={cipher.name ?? "Untitled cipher"}
            className="min-w-0 w-full text-sm font-semibold select-none"
          />
          <div className="mt-1 truncate text-xs text-slate-500">
            {cipher.username ?? ""}
          </div>
        </div>
      </div>
    </button>
  );
}
