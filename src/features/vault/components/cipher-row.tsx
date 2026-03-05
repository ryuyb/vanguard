import type { VaultCipherItemDto } from "@/bindings";

type CipherRowProps = {
  cipher: VaultCipherItemDto;
  selected: boolean;
  onClick: () => void;
};

export function CipherRow({ cipher, selected, onClick }: CipherRowProps) {
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
      <div className="truncate text-sm font-medium">
        {cipher.name ?? "Untitled cipher"}
      </div>
      <div className="mt-1 truncate text-xs text-slate-600">
        {cipher.username ?? ""}
      </div>
    </button>
  );
}
