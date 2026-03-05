import { Eye, EyeOff } from "lucide-react";
import { useEffect, useState } from "react";
import { commands, type VaultCipherDetailDto } from "@/bindings";
import { Button } from "@/components/ui/button";
import { firstNonEmptyText } from "@/features/vault/utils";

function DetailRow({
  label,
  value,
}: {
  label: string;
  value: string | null | undefined;
}) {
  if (!value) {
    return null;
  }
  return (
    <div className="grid grid-cols-[88px_minmax(0,1fr)] gap-2 text-sm">
      <div className="text-slate-500">{label}</div>
      <div className="break-all text-slate-800">{value}</div>
    </div>
  );
}

type CipherDetailPanelProps = {
  cipher: VaultCipherDetailDto;
};

export function CipherDetailPanel({ cipher }: CipherDetailPanelProps) {
  const username = firstNonEmptyText(
    cipher.login?.username,
    cipher.data?.username,
  );
  const password = firstNonEmptyText(
    cipher.login?.password,
    cipher.data?.password,
  );
  const hasOneTimePassword = cipher.hasTotp;
  const [oneTimePasswordCode, setOneTimePasswordCode] = useState<string | null>(
    null,
  );
  const [oneTimePasswordRemaining, setOneTimePasswordRemaining] = useState<
    number | null
  >(null);
  const [oneTimePasswordFailed, setOneTimePasswordFailed] = useState(false);
  const [isPasswordVisible, setIsPasswordVisible] = useState(false);
  const notes = firstNonEmptyText(cipher.notes, cipher.data?.notes);
  const organizationId = cipher.organizationId;

  useEffect(() => {
    let disposed = false;
    let expiresAtMs: number | null = null;
    let loading = false;

    setIsPasswordVisible(false);

    const loadTotpSnapshot = async () => {
      if (loading || disposed) {
        return;
      }
      loading = true;
      try {
        const result = await commands.vaultGetCipherTotpCode({
          cipherId: cipher.id,
        });
        if (disposed) {
          return;
        }
        if (result.status === "error") {
          expiresAtMs = null;
          setOneTimePasswordCode(null);
          setOneTimePasswordRemaining(null);
          setOneTimePasswordFailed(true);
          return;
        }

        expiresAtMs = result.data.expiresAtMs;
        setOneTimePasswordCode(result.data.code);
        setOneTimePasswordRemaining(result.data.remainingSeconds);
        setOneTimePasswordFailed(false);
      } catch {
        if (disposed) {
          return;
        }
        expiresAtMs = null;
        setOneTimePasswordCode(null);
        setOneTimePasswordRemaining(null);
        setOneTimePasswordFailed(true);
      } finally {
        loading = false;
      }
    };
    const intervalId = window.setInterval(() => {
      if (!expiresAtMs) {
        return;
      }
      const remaining = Math.max(
        0,
        Math.ceil((expiresAtMs - Date.now()) / 1000),
      );
      setOneTimePasswordRemaining(remaining);
      if (remaining <= 0) {
        void loadTotpSnapshot();
      }
    }, 1000);

    setOneTimePasswordCode(null);
    setOneTimePasswordRemaining(null);
    setOneTimePasswordFailed(false);
    if (!hasOneTimePassword) {
      return () => {
        disposed = true;
        window.clearInterval(intervalId);
      };
    }

    void loadTotpSnapshot();

    return () => {
      disposed = true;
      window.clearInterval(intervalId);
    };
  }, [cipher.id, hasOneTimePassword]);

  const oneTimePasswordDisplay =
    oneTimePasswordCode && oneTimePasswordRemaining != null
      ? `${oneTimePasswordCode} (${oneTimePasswordRemaining}s)`
      : oneTimePasswordFailed
        ? "Unavailable"
        : hasOneTimePassword
          ? "Loading..."
          : null;

  return (
    <div className="space-y-3">
      <div className="space-y-1">
        <div className="text-lg font-semibold text-slate-900">
          {cipher.name ?? "Untitled cipher"}
        </div>
      </div>

      <div className="space-y-2">
        <DetailRow label="Org" value={organizationId} />
        <DetailRow label="Username" value={username} />
        {password && (
          <div className="grid grid-cols-[88px_minmax(0,1fr)] gap-2 text-sm">
            <div className="text-slate-500">Password</div>
            <div className="flex min-w-0 items-center gap-1">
              <div className="break-all text-slate-800">
                {isPasswordVisible ? password : "••••••••"}
              </div>
              <Button
                type="button"
                variant="ghost"
                className="size-7 shrink-0 px-0"
                onClick={() => setIsPasswordVisible((visible) => !visible)}
                aria-label={isPasswordVisible ? "隐藏密码" : "显示密码"}
                title={isPasswordVisible ? "隐藏密码" : "显示密码"}
              >
                {isPasswordVisible ? (
                  <EyeOff className="size-4" />
                ) : (
                  <Eye className="size-4" />
                )}
              </Button>
            </div>
          </div>
        )}
        {hasOneTimePassword && (
          <DetailRow label="One-time password" value={oneTimePasswordDisplay} />
        )}
      </div>

      {notes && (
        <div className="rounded-lg bg-slate-50/90 p-2">
          <div className="mb-1 text-xs text-slate-500">Notes</div>
          <pre className="whitespace-pre-wrap text-sm text-slate-800">
            {notes}
          </pre>
        </div>
      )}
    </div>
  );
}
