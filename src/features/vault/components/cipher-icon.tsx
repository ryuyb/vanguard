import {
  CreditCard,
  FileText,
  Globe,
  IdCard,
  KeyRound,
  Lock,
} from "lucide-react";
import type { ReactNode } from "react";
import { useEffect, useState } from "react";
import type { VaultCipherItemDto } from "@/bindings";
import { cn } from "@/lib/utils";

type CipherIconProps = {
  alt: string;
  children?: ReactNode;
  className?: string;
  iconData: string | null;
};

function toFallbackIcon(
  hasIconData: boolean,
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

  return hasIconData ? (
    <Globe className="size-4" />
  ) : (
    <FileText className="size-4" />
  );
}

/**
 * Cipher icon component.
 *
 * Shows fallback icon by default, then seamlessly switches to
 * the actual icon when data is available. No loading state shown.
 */
export function CipherIcon({
  alt,
  children,
  className,
  iconData,
}: CipherIconProps) {
  const [didFail, setDidFail] = useState(false);

  // Reset didFail when iconData changes
  // biome-ignore lint/correctness/useExhaustiveDependencies: iconData is a prop that can change
  useEffect(() => {
    setDidFail(false);
  }, [iconData]);

  // Show icon image only when data is available and hasn't failed
  const shouldShowImage = iconData !== null && !didFail;

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
          src={`data:image/png;base64,${iconData}`}
          onError={() => {
            setDidFail(true);
          }}
        />
      ) : (
        toFallbackIcon(iconData !== null, alt, children)
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
