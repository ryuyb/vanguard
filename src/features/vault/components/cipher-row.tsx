import type { VaultCipherItemDto } from "@/bindings";
import {
  CipherIcon,
  toCipherTypeIcon,
} from "@/features/vault/components/cipher-icon";
import { TruncatableText } from "@/features/vault/components/truncatable-text";
import type { CipherIconLoadState } from "@/features/vault/types";
import { toCipherIconAlt } from "@/features/vault/utils";

type CipherRowProps = {
  cipher: VaultCipherItemDto & {
    iconUrl?: string | null;
  };
  selected: boolean;
  onClick: () => void;
  iconLoadState?: CipherIconLoadState;
  onIconError?: () => void;
  onIconLoad?: () => void;
  shouldLoadIcon?: boolean;
};

export function CipherRow({
  cipher,
  selected,
  onClick,
  iconLoadState = "fallback",
  onIconError,
  onIconLoad,
  shouldLoadIcon = false,
}: CipherRowProps) {
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
          alt={toCipherIconAlt(cipher.name)}
          className={[
            "bg-white text-slate-500 border shrink-0",
            selected ? "border-blue-200" : "border-slate-200",
          ].join(" ")}
          iconUrl={cipher.iconUrl ?? null}
          isVisible={shouldLoadIcon}
          loadState={shouldLoadIcon ? iconLoadState : "idle"}
          onError={onIconError}
          onLoad={onIconLoad}
        >
          {toCipherTypeIcon(cipher.type)}
        </CipherIcon>
        <div className="min-w-0 flex-1 shrink overflow-hidden">
          <TruncatableText
            text={cipher.name ?? "Untitled cipher"}
            className="min-w-0 w-full text-sm font-semibold cursor-text"
          />
          <div className="mt-1 truncate text-xs text-slate-500">
            {cipher.username ?? ""}
          </div>
        </div>
      </div>
    </button>
  );
}
