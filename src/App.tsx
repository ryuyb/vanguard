import { type CSSProperties, type ReactNode, useEffect, useState } from "react";
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
import "./App.css";

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
    <main style={{ maxWidth: 900, margin: "0 auto", padding: "24px" }}>
      <h1>Vaultwarden 登录流程验证</h1>
      <p>最小化联调页面：password login / email-2FA / verify-email-token。</p>

      <section style={sectionStyle}>
        <h2>基础参数</h2>
        <Field label="Base URL">
          <input value={baseUrl} onChange={(e) => setBaseUrl(e.currentTarget.value)} />
        </Field>
        <Field label="Email">
          <input value={email} onChange={(e) => setEmail(e.currentTarget.value)} />
        </Field>
        <Field label="Password (plaintext)">
          <input
            type="password"
            value={password}
            onChange={(e) => setPassword(e.currentTarget.value)}
          />
        </Field>
      </section>

      <section style={sectionStyle}>
        <h2>2FA 参数（可选）</h2>
        <Field label="twoFactorProvider">
          <input
            value={twoFactorProvider}
            onChange={(e) => setTwoFactorProvider(e.currentTarget.value)}
            placeholder="例如 1（Email）"
          />
        </Field>
        <Field label="twoFactorToken">
          <input
            value={twoFactorToken}
            onChange={(e) => setTwoFactorToken(e.currentTarget.value)}
            placeholder="验证码"
          />
        </Field>
        <label style={{ display: "flex", alignItems: "center", gap: 8 }}>
          <input
            type="checkbox"
            checked={twoFactorRemember}
            onChange={(e) => setTwoFactorRemember(e.currentTarget.checked)}
          />
          remember device
        </label>
      </section>

      <section style={sectionStyle}>
        <h2>动作</h2>
        <div style={{ display: "flex", flexWrap: "wrap", gap: 8 }}>
          <button type="button" disabled={!!busyAction} onClick={handleLogin}>
            Password Login
          </button>
          <button type="button" disabled={!!busyAction} onClick={handleAuthLogout}>
            Auth Logout
          </button>
        </div>

        <hr />

        <Field label="Email 2FA Password Override (plaintext, optional)">
          <input
            type="password"
            value={email2faPasswordOverride}
            onChange={(e) => setEmail2faPasswordOverride(e.currentTarget.value)}
          />
        </Field>
        <p style={{ marginTop: -6, color: "#475467" }}>
          留空时默认使用上方 Password；Rust 端会自动 prelogin 并计算 master password hash。
        </p>
        <button type="button" disabled={!!busyAction} onClick={handleSendEmailLogin}>
          Send Email Login 2FA
        </button>

        <hr />

        <Field label="verifyEmailToken.userId">
          <input value={verifyUserId} onChange={(e) => setVerifyUserId(e.currentTarget.value)} />
        </Field>
        <Field label="verifyEmailToken.token">
          <input value={verifyToken} onChange={(e) => setVerifyToken(e.currentTarget.value)} />
        </Field>
        <button type="button" disabled={!!busyAction} onClick={handleVerifyEmailToken}>
          Verify Email Token
        </button>

        <p style={{ marginTop: 12 }}>
          <strong>Busy:</strong> {busyAction ?? "idle"}
        </p>
      </section>

      <section style={sectionStyle}>
        <h2>Vault Sync 联调</h2>
        <label style={{ display: "flex", alignItems: "center", gap: 8, marginBottom: 10 }}>
          <input
            type="checkbox"
            checked={syncExcludeDomains}
            onChange={(e) => setSyncExcludeDomains(e.currentTarget.checked)}
          />
          exclude domains
        </label>
        <div style={{ display: "flex", flexWrap: "wrap", gap: 8 }}>
          <button type="button" disabled={!!busyAction} onClick={handleSyncNow}>
            Sync Now
          </button>
          <button type="button" disabled={!!busyAction} onClick={handleSyncStatus}>
            Sync Status
          </button>
        </div>
      </section>

      <section style={sectionStyle}>
        <h2>Vault 解锁与视图联调</h2>
        <Field label="unlock.masterPassword">
          <input
            type="password"
            value={unlockMasterPassword}
            onChange={(e) => setUnlockMasterPassword(e.currentTarget.value)}
          />
        </Field>
        <div style={{ display: "flex", flexWrap: "wrap", gap: 8, marginBottom: 10 }}>
          <button type="button" disabled={!!busyAction} onClick={handleVaultUnlock}>
            Unlock With Password
          </button>
          <button type="button" disabled={!!busyAction} onClick={handleVaultLock}>
            Lock
          </button>
        </div>
        <Field label="viewData.page (optional)">
          <input
            value={vaultPageInput}
            onChange={(e) => setVaultPageInput(e.currentTarget.value)}
            placeholder="1"
          />
        </Field>
        <Field label="viewData.pageSize (optional)">
          <input
            value={vaultPageSizeInput}
            onChange={(e) => setVaultPageSizeInput(e.currentTarget.value)}
            placeholder="50"
          />
        </Field>
        <button type="button" disabled={!!busyAction} onClick={handleVaultGetViewData}>
          Get View Data
        </button>

        <hr />

        <Field label="cipherDetail.cipherId">
          <input
            value={vaultCipherIdInput}
            onChange={(e) => setVaultCipherIdInput(e.currentTarget.value)}
            placeholder="cipher id"
          />
        </Field>
        <button type="button" disabled={!!busyAction} onClick={handleVaultGetCipherDetail}>
          Get Cipher Detail
        </button>

        {vaultViewData && vaultViewData.ciphers.length > 0 ? (
          <>
            <p style={{ marginTop: 12, marginBottom: 6, color: "#475467" }}>
              快速选择一条 cipher 拉详情：
            </p>
            <div style={{ display: "flex", flexDirection: "column", gap: 8, maxHeight: 180, overflowY: "auto" }}>
              {vaultViewData.ciphers.map((cipher) => (
                <button
                  key={cipher.id}
                  type="button"
                  disabled={!!busyAction}
                  onClick={() => void handleSelectCipherAndGetDetail(cipher.id)}
                  style={{
                    textAlign: "left",
                    padding: "8px 10px",
                    border: "1px solid #d0d5dd",
                    borderRadius: 8,
                    background: "#fff",
                    cursor: busyAction ? "not-allowed" : "pointer",
                  }}
                >
                  <strong>{cipher.name || "(unnamed)"}</strong> · <code>{cipher.id}</code>
                </button>
              ))}
            </div>
          </>
        ) : null}
      </section>

      <section style={sectionStyle}>
        <h2>结果快照</h2>
        <p>Auth Bootstrap: {authBootstrapStatus}</p>
        <p>Session: {session ? "authenticated" : "none"}</p>
        <p>2FA Challenge: {challenge ? "required" : "none"}</p>
        <p>Sync Status: {syncStatus ? syncStatus.state : "none"}</p>
        <p>Vault View Data: {vaultViewData ? "loaded" : "none"}</p>
        <p>
          Visible Ciphers:{" "}
          {vaultViewData ? `${vaultViewData.ciphers.length}/${vaultViewData.totalCiphers}` : "none"}
        </p>
        <p>Vault Cipher Detail: {vaultCipherDetail ? "loaded" : "none"}</p>
        <p>
          Cipher Detail Name:{" "}
          {vaultCipherDetail ? (vaultCipherDetail.cipher.name ?? "(null)") : "none"}
        </p>
        <pre style={preStyle}>{rawResult || "No response yet."}</pre>
      </section>

      <section style={sectionStyle}>
        <h2>日志</h2>
        {logs.length === 0 ? (
          <p>No logs yet.</p>
        ) : (
          <ul style={{ margin: 0, paddingLeft: 20 }}>
            {logs.map((log, index) => (
              <li key={`${log.at}-${index}`} style={{ color: log.level === "error" ? "#b42318" : "#344054" }}>
                [{log.at}] {log.message}
              </li>
            ))}
          </ul>
        )}
      </section>
    </main>
  );
}

function Field(props: { label: string; children: ReactNode }) {
  return (
    <label style={{ display: "block", marginBottom: 10 }}>
      <div style={{ marginBottom: 4, fontSize: 13, color: "#475467" }}>{props.label}</div>
      {props.children}
    </label>
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

const sectionStyle: CSSProperties = {
  border: "1px solid #d0d5dd",
  borderRadius: 10,
  padding: 14,
  marginBottom: 16,
};

const preStyle: CSSProperties = {
  margin: 0,
  overflowX: "auto",
  maxHeight: 320,
  background: "#f9fafb",
  borderRadius: 8,
  padding: 12,
  border: "1px solid #eaecf0",
};

export default App;
