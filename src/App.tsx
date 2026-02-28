import { type CSSProperties, type ReactNode, useState } from "react";
import {
  commands,
  type PasswordLoginResponseDto,
  type PreloginResponseDto,
  type SessionResponseDto,
  type TwoFactorChallengeDto,
} from "./bindings";
import "./App.css";

type LogLevel = "info" | "error";

type LogEntry = {
  at: string;
  level: LogLevel;
  message: string;
};

function App() {
  const [baseUrl, setBaseUrl] = useState("http://127.0.0.1:8080");
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [twoFactorProvider, setTwoFactorProvider] = useState("");
  const [twoFactorToken, setTwoFactorToken] = useState("");
  const [twoFactorRemember, setTwoFactorRemember] = useState(true);

  const [email2faPasswordOverride, setEmail2faPasswordOverride] = useState("");
  const [verifyUserId, setVerifyUserId] = useState("");
  const [verifyToken, setVerifyToken] = useState("");
  const [refreshTokenInput, setRefreshTokenInput] = useState("");

  const [prelogin, setPrelogin] = useState<PreloginResponseDto | null>(null);
  const [session, setSession] = useState<SessionResponseDto | null>(null);
  const [challenge, setChallenge] = useState<TwoFactorChallengeDto | null>(null);
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

  const handlePrelogin = async () => {
    await run("prelogin", async () => {
      const result = await commands.authPrelogin({
        baseUrl,
        email,
      });

      if (result.status === "error") {
        setRaw(result);
        throw new Error(result.error);
      }

      setPrelogin(result.data);
      setRaw(result.data);
    });
  };

  const handleLogin = async () => {
    await run("password-login", async () => {
      const result = await commands.authLoginWithPassword({
        baseUrl,
        username: email,
        password,
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

  const handleRefreshToken = async () => {
    await run("refresh-token", async () => {
      const token = refreshTokenInput.trim();
      if (!token) {
        throw new Error("refresh token is empty");
      }

      const result = await commands.authRefreshToken({
        baseUrl,
        refreshToken: token,
      });

      if (result.status === "error") {
        setRaw(result);
        throw new Error(result.error || "backend returned an empty error message");
      }

      setSession(result.data);
      setChallenge(null);
      setRaw(result.data);
    });
  };

  const handleSendEmailLogin = async () => {
    await run("send-email-login", async () => {
      const result = await commands.authSendEmailLogin({
        baseUrl,
        email: emptyToNull(email),
        masterPasswordHash: emptyToNull(
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

  const handleLoginPayload = (payload: PasswordLoginResponseDto) => {
    if (payload.status === "authenticated") {
      const { status, ...sessionResult } = payload;
      void status;
      setSession(sessionResult);
      setChallenge(null);
      setRefreshTokenInput(sessionResult.refreshToken ?? "");
      return;
    }

    const { status, ...challengeResult } = payload;
    void status;
    setChallenge(challengeResult);
    setSession(null);

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
      <p>最小化联调页面：prelogin / password login / refresh / email-2FA / verify-email-token。</p>

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
          <button type="button" disabled={!!busyAction} onClick={handlePrelogin}>
            1) Prelogin
          </button>
          <button type="button" disabled={!!busyAction} onClick={handleLogin}>
            2) Password Login
          </button>
          <button type="button" disabled={!!busyAction} onClick={handleRefreshToken}>
            3) Refresh Token
          </button>
        </div>

        <Field label="Refresh Token">
          <input
            value={refreshTokenInput}
            onChange={(e) => setRefreshTokenInput(e.currentTarget.value)}
          />
        </Field>

        <hr />

        <Field label="Email 2FA Password Override (plaintext, optional)">
          <input
            type="password"
            value={email2faPasswordOverride}
            onChange={(e) => setEmail2faPasswordOverride(e.currentTarget.value)}
          />
        </Field>
        <p style={{ marginTop: -6, color: "#475467" }}>
          留空时默认使用上方 Password；Rust 端会自动 prelogin 并计算 masterPasswordHash。
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
        <h2>结果快照</h2>
        <p>Prelogin: {prelogin ? "done" : "none"}</p>
        <p>Session: {session ? "authenticated" : "none"}</p>
        <p>2FA Challenge: {challenge ? "required" : "none"}</p>
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
