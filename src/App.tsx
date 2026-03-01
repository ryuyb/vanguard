import { type ReactNode, useEffect, useState } from "react";
import { Badge } from "./components/ui/badge";
import { Button } from "./components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "./components/ui/card";
import { Input } from "./components/ui/input";
import { Label } from "./components/ui/label";
import { Separator } from "./components/ui/separator";
import { Textarea } from "./components/ui/textarea";
import {
  commands,
  events,
  type PasswordLoginResponseDto,
  type SessionResponseDto,
  type SyncStatusResponseDto,
  type TwoFactorChallengeDto,
  type VaultCipherDetailResponseDto,
  type VaultViewDataResponseDto,
} from "./bindings";

type LogLevel = "info" | "error";

type LogEntry = {
  at: string;
  level: LogLevel;
  message: string;
};

function App() {
  const [authBootstrapStatus, setAuthBootstrapStatus] = useState<
    "unknown" | "needsLogin" | "locked" | "authenticated"
  >("unknown");
  const [baseUrl, setBaseUrl] = useState("http://127.0.0.1:8080");
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [twoFactorProvider, setTwoFactorProvider] = useState("");
  const [twoFactorToken, setTwoFactorToken] = useState("");
  const [twoFactorRemember, setTwoFactorRemember] = useState(true);

  const [email2faPasswordOverride, setEmail2faPasswordOverride] = useState("");
  const [verifyUserId, setVerifyUserId] = useState("");
  const [verifyToken, setVerifyToken] = useState("");
  const [syncExcludeDomains, setSyncExcludeDomains] = useState(false);
  const [unlockMasterPassword, setUnlockMasterPassword] = useState("");
  const [vaultPageInput, setVaultPageInput] = useState("1");
  const [vaultPageSizeInput, setVaultPageSizeInput] = useState("50");
  const [vaultCipherIdInput, setVaultCipherIdInput] = useState("");

  const [session, setSession] = useState<SessionResponseDto | null>(null);
  const [challenge, setChallenge] = useState<TwoFactorChallengeDto | null>(null);
  const [syncStatus, setSyncStatus] = useState<SyncStatusResponseDto | null>(null);
  const [vaultViewData, setVaultViewData] = useState<VaultViewDataResponseDto | null>(null);
  const [vaultCipherDetail, setVaultCipherDetail] =
    useState<VaultCipherDetailResponseDto | null>(null);
  const [busyAction, setBusyAction] = useState<string | null>(null);
  const [rawResult, setRawResult] = useState("");
  const [logs, setLogs] = useState<LogEntry[]>([]);

  const addLog = (level: LogLevel, message: string) => {
    setLogs((prev) => [
      { at: new Date().toLocaleTimeString(), level, message },
      ...prev.slice(0, 19),
    ]);
  };

  const setRaw = (payload: unknown) => {
    setRawResult(JSON.stringify(payload, null, 2));
  };

  const run = async (name: string, fn: () => Promise<void>) => {
    setBusyAction(name);
    try {
      await fn();
      addLog("info", `${name} success`);
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      addLog("error", `${name} failed: ${message}`);
    } finally {
      setBusyAction(null);
    }
  };

  const runForegroundRevisionCheck = async () => {
    if (!session) {
      return;
    }

    const result = await commands.vaultSyncCheckRevision({});

    if (result.status === "error") {
      addLog(
        "error",
        `foreground-revision-check failed: ${result.error || "backend returned an empty error message"}`,
      );
      return;
    }

    setSyncStatus(result.data);
    addLog("info", "foreground-revision-check success");
  };

  useEffect(() => {
    const restore = async () => {
      const result = await commands.authRestoreState({});
      if (result.status === "error") {
        addLog(
          "error",
          `auth-restore-state failed: ${result.error || "backend returned an empty error message"}`,
        );
        return;
      }

      setAuthBootstrapStatus(result.data.status);
      if (result.data.baseUrl) {
        setBaseUrl(result.data.baseUrl);
      }
      if (result.data.email) {
        setEmail(result.data.email);
      }
      if (result.data.status === "locked") {
        addLog("info", "restored persisted login context, master password unlock is ready");
      } else if (result.data.status === "authenticated") {
        addLog("info", "backend session is active");
      } else {
        addLog("info", "no persisted login context found");
      }
      setRaw(result.data);
    };

    void restore();
  }, []);

  useEffect(() => {
    let lastTriggeredAt = 0;

    const maybeTrigger = () => {
      if (document.visibilityState !== "visible") {
        return;
      }

      const now = Date.now();
      if (now - lastTriggeredAt < 1500) {
        return;
      }
      lastTriggeredAt = now;
      void runForegroundRevisionCheck();
    };

    const onVisibilityChange = () => {
      maybeTrigger();
    };
    const onFocus = () => {
      maybeTrigger();
    };

    document.addEventListener("visibilitychange", onVisibilityChange);
    window.addEventListener("focus", onFocus);
    return () => {
      document.removeEventListener("visibilitychange", onVisibilityChange);
      window.removeEventListener("focus", onFocus);
    };
  }, [session]);

  useEffect(() => {
    const unlistenPromise = events.vaultSyncAuthRequired.listen((event) => {
      setSession(null);
      setChallenge(null);
      setVaultViewData(null);
      setVaultCipherDetail(null);
      setAuthBootstrapStatus("needsLogin");
      addLog(
        "error",
        `session invalidated (${event.payload.status}): ${event.payload.message}`,
      );
    });

    return () => {
      void unlistenPromise.then((unlisten) => unlisten());
    };
  }, []);

  const handleLogin = async () => {
    await run("password-login", async () => {
      const result = await commands.authLoginWithPassword({
        baseUrl,
        email,
        masterPassword: password,
        twoFactorProvider: emptyToNullNumber(twoFactorProvider),
        twoFactorToken: emptyToNull(twoFactorToken),
        twoFactorRemember: twoFactorToken.trim() ? twoFactorRemember : null,
        authrequest: null,
      });

      if (result.status === "error") {
        setRaw(result);
        throw new Error(result.error || "backend returned an empty error message");
      }

      handleLoginPayload(result.data);
      setRaw(result.data);
    });
  };

  const handleSendEmailLogin = async () => {
    await run("send-email-login", async () => {
      const result = await commands.authSendEmailLogin({
        baseUrl,
        email: emptyToNull(email),
        masterPassword: emptyToNull(
          email2faPasswordOverride.trim() ? email2faPasswordOverride : password,
        ),
        authRequestId: null,
        authRequestAccessCode: null,
      });

      if (result.status === "error") {
        setRaw(result);
        throw new Error(result.error || "backend returned an empty error message");
      }

      setRaw({ ok: true });
    });
  };

  const handleVerifyEmailToken = async () => {
    await run("verify-email-token", async () => {
      const result = await commands.authVerifyEmailToken({
        baseUrl,
        userId: verifyUserId,
        token: verifyToken,
      });

      if (result.status === "error") {
        setRaw(result);
        throw new Error(result.error || "backend returned an empty error message");
      }

      setRaw({ ok: true });
    });
  };

  const handleAuthLogout = async () => {
    await run("auth-logout", async () => {
      const result = await commands.authLogout({});

      if (result.status === "error") {
        setRaw(result);
        throw new Error(result.error || "backend returned an empty error message");
      }

      setSession(null);
      setChallenge(null);
      setSyncStatus(null);
      setVaultViewData(null);
      setVaultCipherDetail(null);
      setAuthBootstrapStatus("needsLogin");
      setRaw({ ok: true });
    });
  };

  const handleSyncNow = async () => {
    await run("vault-sync-now", async () => {
      const result = await commands.vaultSyncNow({
        excludeDomains: syncExcludeDomains,
      });

      if (result.status === "error") {
        setRaw(result);
        throw new Error(result.error || "backend returned an empty error message");
      }

      setSyncStatus(result.data);
      setRaw(result.data);
    });
  };

  const handleSyncStatus = async () => {
    await run("vault-sync-status", async () => {
      const result = await commands.vaultSyncStatus({});

      if (result.status === "error") {
        setRaw(result);
        throw new Error(result.error || "backend returned an empty error message");
      }

      setSyncStatus(result.data);
      setRaw(result.data);
    });
  };

  const handleVaultUnlock = async () => {
    await run("vault-unlock-with-password", async () => {
      if (!unlockMasterPassword.trim()) {
        throw new Error("unlock master password is empty");
      }

      const result = await commands.vaultUnlockWithPassword({
        masterPassword: unlockMasterPassword,
      });

      if (result.status === "error") {
        setRaw(result);
        throw new Error(result.error || "backend returned an empty error message");
      }

      setAuthBootstrapStatus("authenticated");
      setRaw({ ok: true });
    });
  };

  const handleVaultLock = async () => {
    await run("vault-lock", async () => {
      const result = await commands.vaultLock({});

      if (result.status === "error") {
        setRaw(result);
        throw new Error(result.error || "backend returned an empty error message");
      }

      setAuthBootstrapStatus("locked");
      setVaultCipherDetail(null);
      setRaw({ ok: true });
    });
  };

  const handleVaultGetViewData = async () => {
    await run("vault-get-view-data", async () => {
      const result = await commands.vaultGetViewData({
        page: toPositiveNumberOrNull(vaultPageInput),
        pageSize: toPositiveNumberOrNull(vaultPageSizeInput),
      });

      if (result.status === "error") {
        setRaw(result);
        throw new Error(result.error || "backend returned an empty error message");
      }

      setVaultViewData(result.data);
      if (!vaultCipherIdInput.trim() && result.data.ciphers.length > 0) {
        setVaultCipherIdInput(result.data.ciphers[0].id);
      }
      setRaw(result.data);
    });
  };

  const fetchVaultCipherDetail = async (cipherId: string) => {
    const result = await commands.vaultGetCipherDetail({
      cipherId,
    });

    if (result.status === "error") {
      setRaw(result);
      throw new Error(result.error || "backend returned an empty error message");
    }

    setVaultCipherDetail(result.data);
    setRaw(result.data);
  };

  const handleVaultGetCipherDetail = async () => {
    await run("vault-get-cipher-detail", async () => {
      const cipherId = vaultCipherIdInput.trim();
      if (!cipherId) {
        throw new Error("cipher id is empty");
      }
      await fetchVaultCipherDetail(cipherId);
    });
  };

  const handleSelectCipherAndGetDetail = async (cipherId: string) => {
    setVaultCipherIdInput(cipherId);
    await run("vault-get-cipher-detail", async () => {
      await fetchVaultCipherDetail(cipherId);
    });
  };

  const handleLoginPayload = (payload: PasswordLoginResponseDto) => {
    if (payload.status === "authenticated") {
      const { status, ...sessionResult } = payload;
      void status;
      setSession(sessionResult);
      setChallenge(null);
      setAuthBootstrapStatus("authenticated");
      return;
    }

    const { status, ...challengeResult } = payload;
    void status;
    setChallenge(challengeResult);
    setSession(null);
    setAuthBootstrapStatus("needsLogin");

    if (challengeResult.providers.length > 0) {
      const firstProvider = Number(challengeResult.providers[0]);
      if (Number.isFinite(firstProvider)) {
        setTwoFactorProvider(String(firstProvider));
      }
    }
  };

  return (
    <main className="mx-auto w-full max-w-6xl space-y-4 p-4 md:p-6">
      <header className="space-y-1">
        <h1 className="text-2xl font-semibold tracking-tight">Vaultwarden 登录流程验证</h1>
        <p className="text-muted-foreground text-sm">
          最小化联调页面：password login / email-2FA / verify-email-token。
        </p>
      </header>

      <Card>
        <CardHeader>
          <CardTitle>基础参数</CardTitle>
          <CardDescription>前端只收集 server url、email、master password。</CardDescription>
        </CardHeader>
        <CardContent className="grid gap-4 md:grid-cols-2">
          <FormField id="base-url" label="Base URL">
            <Input
              id="base-url"
              value={baseUrl}
              onChange={(e) => setBaseUrl(e.currentTarget.value)}
            />
          </FormField>
          <FormField id="email" label="Email">
            <Input id="email" value={email} onChange={(e) => setEmail(e.currentTarget.value)} />
          </FormField>
          <FormField id="password" label="Password (plaintext)" className="md:col-span-2">
            <Input
              id="password"
              type="password"
              value={password}
              onChange={(e) => setPassword(e.currentTarget.value)}
            />
          </FormField>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>2FA 参数（可选）</CardTitle>
        </CardHeader>
        <CardContent className="grid gap-4 md:grid-cols-2">
          <FormField id="two-factor-provider" label="twoFactorProvider">
            <Input
              id="two-factor-provider"
              value={twoFactorProvider}
              onChange={(e) => setTwoFactorProvider(e.currentTarget.value)}
              placeholder="例如 1（Email）"
            />
          </FormField>
          <FormField id="two-factor-token" label="twoFactorToken">
            <Input
              id="two-factor-token"
              value={twoFactorToken}
              onChange={(e) => setTwoFactorToken(e.currentTarget.value)}
              placeholder="验证码"
            />
          </FormField>
          <label className="flex items-center gap-2 text-sm md:col-span-2" htmlFor="two-factor-remember">
            <input
              id="two-factor-remember"
              type="checkbox"
              checked={twoFactorRemember}
              onChange={(e) => setTwoFactorRemember(e.currentTarget.checked)}
              className="border-input size-4 rounded border"
            />
            remember device
          </label>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>动作</CardTitle>
          <CardDescription>
            <span className="mr-2">Busy:</span>
            <Badge variant="outline">{busyAction ?? "idle"}</Badge>
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex flex-wrap gap-2">
            <Button type="button" disabled={!!busyAction} onClick={handleLogin}>
              Password Login
            </Button>
            <Button
              type="button"
              variant="outline"
              disabled={!!busyAction}
              onClick={handleAuthLogout}
            >
              Auth Logout
            </Button>
          </div>

          <Separator />

          <FormField
            id="email-2fa-password"
            label="Email 2FA Password Override (plaintext, optional)"
          >
            <Input
              id="email-2fa-password"
              type="password"
              value={email2faPasswordOverride}
              onChange={(e) => setEmail2faPasswordOverride(e.currentTarget.value)}
            />
          </FormField>
          <p className="text-muted-foreground text-sm">
            留空时默认使用上方 Password；Rust 端会自动 prelogin 并计算 master password hash。
          </p>
          <Button type="button" disabled={!!busyAction} onClick={handleSendEmailLogin}>
            Send Email Login 2FA
          </Button>

          <Separator />

          <div className="grid gap-4 md:grid-cols-2">
            <FormField id="verify-user-id" label="verifyEmailToken.userId">
              <Input
                id="verify-user-id"
                value={verifyUserId}
                onChange={(e) => setVerifyUserId(e.currentTarget.value)}
              />
            </FormField>
            <FormField id="verify-token" label="verifyEmailToken.token">
              <Input
                id="verify-token"
                value={verifyToken}
                onChange={(e) => setVerifyToken(e.currentTarget.value)}
              />
            </FormField>
          </div>
          <Button type="button" disabled={!!busyAction} onClick={handleVerifyEmailToken}>
            Verify Email Token
          </Button>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Vault Sync 联调</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <label className="flex items-center gap-2 text-sm" htmlFor="exclude-domains">
            <input
              id="exclude-domains"
              type="checkbox"
              checked={syncExcludeDomains}
              onChange={(e) => setSyncExcludeDomains(e.currentTarget.checked)}
              className="border-input size-4 rounded border"
            />
            exclude domains
          </label>
          <div className="flex flex-wrap gap-2">
            <Button type="button" disabled={!!busyAction} onClick={handleSyncNow}>
              Sync Now
            </Button>
            <Button
              type="button"
              variant="outline"
              disabled={!!busyAction}
              onClick={handleSyncStatus}
            >
              Sync Status
            </Button>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Vault 解锁与视图联调</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <FormField id="unlock-password" label="unlock.masterPassword">
            <Input
              id="unlock-password"
              type="password"
              value={unlockMasterPassword}
              onChange={(e) => setUnlockMasterPassword(e.currentTarget.value)}
            />
          </FormField>
          <div className="flex flex-wrap gap-2">
            <Button type="button" disabled={!!busyAction} onClick={handleVaultUnlock}>
              Unlock With Password
            </Button>
            <Button type="button" variant="outline" disabled={!!busyAction} onClick={handleVaultLock}>
              Lock
            </Button>
          </div>

          <div className="grid gap-4 md:grid-cols-2">
            <FormField id="view-page" label="viewData.page (optional)">
              <Input
                id="view-page"
                value={vaultPageInput}
                onChange={(e) => setVaultPageInput(e.currentTarget.value)}
                placeholder="1"
              />
            </FormField>
            <FormField id="view-page-size" label="viewData.pageSize (optional)">
              <Input
                id="view-page-size"
                value={vaultPageSizeInput}
                onChange={(e) => setVaultPageSizeInput(e.currentTarget.value)}
                placeholder="50"
              />
            </FormField>
          </div>
          <Button type="button" disabled={!!busyAction} onClick={handleVaultGetViewData}>
            Get View Data
          </Button>

          <Separator />

          <FormField id="cipher-id" label="cipherDetail.cipherId">
            <Input
              id="cipher-id"
              value={vaultCipherIdInput}
              onChange={(e) => setVaultCipherIdInput(e.currentTarget.value)}
              placeholder="cipher id"
            />
          </FormField>
          <Button type="button" disabled={!!busyAction} onClick={handleVaultGetCipherDetail}>
            Get Cipher Detail
          </Button>

          {vaultViewData && vaultViewData.ciphers.length > 0 ? (
            <div className="space-y-2">
              <p className="text-muted-foreground text-sm">快速选择一条 cipher 拉详情：</p>
              <div className="max-h-56 space-y-2 overflow-y-auto">
                {vaultViewData.ciphers.map((cipher) => (
                  <Button
                    key={cipher.id}
                    type="button"
                    variant="outline"
                    disabled={!!busyAction}
                    onClick={() => void handleSelectCipherAndGetDetail(cipher.id)}
                    className="h-auto w-full justify-start px-3 py-2 text-left"
                  >
                    <span className="font-semibold">{cipher.name || "(unnamed)"}</span>
                    <span className="text-muted-foreground ml-2 truncate font-mono text-xs">
                      {cipher.id}
                    </span>
                  </Button>
                ))}
              </div>
            </div>
          ) : null}
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>结果快照</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="grid gap-2 md:grid-cols-2">
            <SnapshotRow label="Auth Bootstrap" value={authBootstrapStatus} />
            <SnapshotRow label="Session" value={session ? "authenticated" : "none"} />
            <SnapshotRow label="2FA Challenge" value={challenge ? "required" : "none"} />
            <SnapshotRow label="Sync Status" value={syncStatus ? syncStatus.state : "none"} />
            <SnapshotRow label="Vault View Data" value={vaultViewData ? "loaded" : "none"} />
            <SnapshotRow
              label="Visible Ciphers"
              value={vaultViewData ? `${vaultViewData.ciphers.length}/${vaultViewData.totalCiphers}` : "none"}
            />
            <SnapshotRow
              label="Vault Cipher Detail"
              value={vaultCipherDetail ? "loaded" : "none"}
            />
            <SnapshotRow
              label="Cipher Detail Name"
              value={vaultCipherDetail ? (vaultCipherDetail.cipher.name ?? "(null)") : "none"}
            />
          </div>
          <Textarea
            readOnly
            className="min-h-80 font-mono text-xs leading-5"
            value={rawResult || "No response yet."}
          />
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>日志</CardTitle>
        </CardHeader>
        <CardContent>
          {logs.length === 0 ? (
            <p className="text-muted-foreground text-sm">No logs yet.</p>
          ) : (
            <ul className="space-y-1">
              {logs.map((log, index) => (
                <li
                  key={`${log.at}-${index}`}
                  className={log.level === "error" ? "text-destructive" : "text-muted-foreground"}
                >
                  [{log.at}] {log.message}
                </li>
              ))}
            </ul>
          )}
        </CardContent>
      </Card>
    </main>
  );
}

function FormField(props: {
  id: string;
  label: string;
  children: ReactNode;
  className?: string;
}) {
  return (
    <div className={props.className ? `space-y-2 ${props.className}` : "space-y-2"}>
      <Label htmlFor={props.id}>{props.label}</Label>
      {props.children}
    </div>
  );
}

function SnapshotRow(props: { label: string; value: string }) {
  return (
    <div className="flex items-center justify-between rounded-md border px-3 py-2 text-sm">
      <span className="text-muted-foreground">{props.label}</span>
      <Badge variant="outline">{props.value}</Badge>
    </div>
  );
}

function emptyToNull(value: string): string | null {
  const trimmed = value.trim();
  return trimmed ? trimmed : null;
}

function emptyToNullNumber(value: string): number | null {
  const trimmed = value.trim();
  if (!trimmed) {
    return null;
  }
  const parsed = Number(trimmed);
  return Number.isFinite(parsed) ? parsed : null;
}

function toPositiveNumberOrNull(value: string): number | null {
  const trimmed = value.trim();
  if (!trimmed) {
    return null;
  }

  const parsed = Number(trimmed);
  if (!Number.isFinite(parsed) || parsed <= 0) {
    return null;
  }

  return Math.floor(parsed);
}

export default App;
