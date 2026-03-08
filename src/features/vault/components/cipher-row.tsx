import type { VaultCipherItemDto } from "@/bindings";
import {
  CipherIcon,
  toCipherTypeIcon,
} from "@/features/vault/components/cipher-icon";
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
        "w-full rounded-lg px-3 py-2 text-left transition-colors",
        selected
          ? "bg-sky-100/85 text-sky-900"
          : "bg-slate-50/80 text-slate-800 hover:bg-slate-100",
      ].join(" ")}
    >
      <div className="flex items-center gap-3 min-w-0">
        <CipherIcon
          alt={toCipherIconAlt(cipher.name)}
          className="bg-white/90 text-slate-500"
          iconUrl={cipher.iconUrl ?? null}
          isVisible={shouldLoadIcon}
          loadState={shouldLoadIcon ? iconLoadState : "idle"}
          onError={onIconError}
          onLoad={onIconLoad}
        >
          {toCipherTypeIcon(cipher.type)}
        </CipherIcon>
        <div className="min-w-0 flex-1">
          <div className="truncate text-sm font-medium">
            {cipher.name ?? "Untitled cipher"}
          </div>
          <div className="mt-1 truncate text-xs text-slate-600">
            {cipher.username ?? ""}
          </div>
        </div>
      </div>
    </button>
  );
}
