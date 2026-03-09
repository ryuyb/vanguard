import {
  CreditCard,
  FileText,
  Globe,
  IdCard,
  KeyRound,
  LoaderCircle,
  Lock,
} from "lucide-react";
import type { ReactNode } from "react";
import { useState } from "react";
import type { VaultCipherItemDto } from "@/bindings";
import type { CipherIconLoadState } from "@/features/vault/types";
import { cn } from "@/lib/utils";

type CipherIconProps = {
  alt: string;
  children?: ReactNode;
  className?: string;
  iconUrl: string | null;
  isVisible?: boolean;
  loadState?: CipherIconLoadState;
  onError?: () => void;
  onLoad?: () => void;
};

function toFallbackIcon(
  iconUrl: string | null,
  alt: string,
  children?: ReactNode,
): ReactNode {
  if (children) {
    return children;
  }

  const title = alt.trim().charAt(0).toUpperCase();

  if (title) {
    return <span className="text-xs font-semibold">{title}</span>;
  }

  return iconUrl ? (
    <Globe className="size-4" />
  ) : (
    <FileText className="size-4" />
  );
}

function toLoadingVisual(loadState: CipherIconLoadState) {
  if (loadState !== "loading") {
    return null;
  }

  return <LoaderCircle className="size-3.5 animate-spin text-slate-400" />;
}

export function CipherIcon({
  alt,
  children,
  className,
  iconUrl,
  isVisible = true,
  loadState = "fallback",
  onError,
  onLoad,
}: CipherIconProps) {
  const [didFail, setDidFail] = useState(false);

  // Once loaded successfully, keep showing the image even if not visible
  // This prevents flickering when scrolling
  const shouldShowImage =
    Boolean(iconUrl) &&
    loadState !== "fallback" &&
    !didFail &&
    (loadState === "loaded" || isVisible);

  return (
    <div
      aria-hidden="true"
      className={cn(
        "flex size-9 shrink-0 items-center justify-center overflow-hidden rounded-xl border border-slate-200/80 bg-slate-50 text-slate-500 shadow-xs",
        className,
      )}
    >
      {shouldShowImage ? (
        <img
          alt={alt}
          className="size-full object-cover"
          loading="lazy"
          src={iconUrl ?? undefined}
          onError={() => {
            setDidFail(true);
            onError?.();
          }}
          onLoad={() => {
            setDidFail(false);
            onLoad?.();
          }}
        />
      ) : (
        (toLoadingVisual(loadState) ?? toFallbackIcon(iconUrl, alt, children))
      )}
    </div>
  );
}

export function toCipherTypeIcon(cipherType: VaultCipherItemDto["type"]) {
  if (cipherType === 1) {
    return <CreditCard className="size-4" />;
  }
  if (cipherType === 2) {
    return <IdCard className="size-4" />;
  }
  if (cipherType === 3) {
    return <FileText className="size-4" />;
  }
  if (cipherType === 5) {
    return <KeyRound className="size-4" />;
  }
  return <Lock className="size-4" />;
}
