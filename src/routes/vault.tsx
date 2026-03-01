import { createFileRoute, Link } from "@tanstack/react-router";
import {
  FolderOpen,
  KeyRound,
  LoaderCircle,
  Lock,
  LogIn,
  LogOut,
  RefreshCw,
  ShieldAlert,
} from "lucide-react";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import {
  commands,
  type RestoreAuthStateResponseDto,
  type VaultCipherDetailDto,
  type VaultCipherItemDto,
  type VaultFolderItemDto,
  type VaultViewDataResponseDto,
} from "@/bindings";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Separator } from "@/components/ui/separator";

export const Route = createFileRoute("/vault")({
  component: VaultPage,
});

type VaultPageState = "loading" | "needsLogin" | "locked" | "ready" | "error";

const ALL_FOLDERS_ID = "__all__";

function errorToText(error: unknown) {
  if (typeof error === "string") {
    return error;
  }
  if (error instanceof Error) {
    return error.message;
  }
  return "加载密码库失败，请稍后重试。";
}

function toCipherTypeLabel(type: number | null) {
  if (type === 1) {
    return "Login";
  }
  if (type === 2) {
    return "Secure Note";
  }
  if (type === 3) {
    return "Card";
  }
  if (type === 4) {
    return "Identity";
  }
  if (type === 5) {
    return "SSH Key";
  }
  return "Unknown";
}

function formatRevisionDate(value: string | null) {
  if (!value) {
    return "Unknown";
  }
  const parsed = new Date(value);
  if (Number.isNaN(parsed.getTime())) {
    return value;
  }
  return parsed.toLocaleString();
}

function formatRevisionDateOrNull(value: string | null | undefined) {
  if (!value) {
    return null;
  }
  return formatRevisionDate(value);
}

function sortFolders(folders: VaultFolderItemDto[]) {
  return [...folders].sort((left, right) =>
    (left.name ?? "").localeCompare(right.name ?? "", "zh-Hans-CN"),
  );
}

function isNonEmptyText(value: string | null | undefined): value is string {
  return typeof value === "string" && value.trim().length > 0;
}

function firstNonEmptyText(
  ...values: Array<string | null | undefined>
): string | null {
  for (const value of values) {
    if (isNonEmptyText(value)) {
      return value;
    }
  }
  return null;
}

function compactText(values: Array<string | null | undefined>) {
  return values.filter(isNonEmptyText);
}

function VaultPage() {
  const [pageState, setPageState] = useState<VaultPageState>("loading");
  const [restoreState, setRestoreState] =
    useState<RestoreAuthStateResponseDto | null>(null);
  const [viewData, setViewData] = useState<VaultViewDataResponseDto | null>(
    null,
  );
  const [selectedFolderId, setSelectedFolderId] = useState(ALL_FOLDERS_ID);
  const [selectedCipherId, setSelectedCipherId] = useState<string | null>(null);
  const [selectedCipherDetail, setSelectedCipherDetail] =
    useState<VaultCipherDetailDto | null>(null);
  const [cipherDetailError, setCipherDetailError] = useState("");
  const [isCipherDetailLoading, setIsCipherDetailLoading] = useState(false);
  const [errorText, setErrorText] = useState("");
  const [isRefreshing, setIsRefreshing] = useState(false);
  const [isLocking, setIsLocking] = useState(false);
  const [isLoggingOut, setIsLoggingOut] = useState(false);
  const detailRequestSeqRef = useRef(0);

  const loadVaultData = useCallback(async () => {
    detailRequestSeqRef.current += 1;
    setIsRefreshing(true);
    setErrorText("");
    setPageState("loading");
    setSelectedCipherId(null);
    setSelectedCipherDetail(null);
    setCipherDetailError("");
    setIsCipherDetailLoading(false);

    try {
      const restore = await commands.authRestoreState({});
      if (restore.status === "error") {
        setPageState("error");
        setErrorText(errorToText(restore.error));
        return;
      }

      setRestoreState(restore.data);
      if (restore.data.status === "needsLogin") {
        setPageState("needsLogin");
        return;
      }
      if (restore.data.status === "locked") {
        setPageState("locked");
        return;
      }

      const vaultViewData = await commands.vaultGetViewData({
        page: 1,
        pageSize: 200,
      });

      if (vaultViewData.status === "error") {
        const text = errorToText(vaultViewData.error);
        if (text.toLowerCase().includes("vault is locked")) {
          setPageState("locked");
          return;
        }
        setPageState("error");
        setErrorText(text);
        return;
      }

      setViewData(vaultViewData.data);
      setPageState("ready");
    } catch (error) {
      setPageState("error");
      setErrorText(errorToText(error));
    } finally {
      setIsRefreshing(false);
    }
  }, []);

  useEffect(() => {
    loadVaultData();
  }, [loadVaultData]);

  const onLock = async () => {
    setIsLocking(true);
    setErrorText("");
    try {
      const result = await commands.vaultLock({});
      if (result.status === "error") {
        setErrorText(errorToText(result.error));
        return;
      }
      await loadVaultData();
    } catch (error) {
      setErrorText(errorToText(error));
    } finally {
      setIsLocking(false);
    }
  };

  const onLogout = async () => {
    setIsLoggingOut(true);
    setErrorText("");
    try {
      const result = await commands.authLogout({});
      if (result.status === "error") {
        setErrorText(errorToText(result.error));
        return;
      }
      await loadVaultData();
    } catch (error) {
      setErrorText(errorToText(error));
    } finally {
      setIsLoggingOut(false);
    }
  };

  const loadCipherDetail = useCallback(async (cipherId: string) => {
    const normalizedCipherId = cipherId.trim();
    if (!normalizedCipherId) {
      return;
    }

    setSelectedCipherId(normalizedCipherId);
    setSelectedCipherDetail(null);
    setCipherDetailError("");
    setIsCipherDetailLoading(true);

    const requestSeq = detailRequestSeqRef.current + 1;
    detailRequestSeqRef.current = requestSeq;

    try {
      const detail = await commands.vaultGetCipherDetail({
        cipherId: normalizedCipherId,
      });

      if (requestSeq !== detailRequestSeqRef.current) {
        return;
      }

      if (detail.status === "error") {
        setCipherDetailError(errorToText(detail.error));
        return;
      }

      setSelectedCipherDetail(detail.data.cipher);
    } catch (error) {
      if (requestSeq !== detailRequestSeqRef.current) {
        return;
      }
      setCipherDetailError(errorToText(error));
    } finally {
      if (requestSeq === detailRequestSeqRef.current) {
        setIsCipherDetailLoading(false);
      }
    }
  }, []);

  const sortedFolders = useMemo(
    () => sortFolders(viewData?.folders ?? []),
    [viewData?.folders],
  );

  useEffect(() => {
    if (selectedFolderId === ALL_FOLDERS_ID) {
      return;
    }
    if (!sortedFolders.find((folder) => folder.id === selectedFolderId)) {
      setSelectedFolderId(ALL_FOLDERS_ID);
    }
  }, [selectedFolderId, sortedFolders]);

  const currentFolderName = useMemo(() => {
    if (selectedFolderId === ALL_FOLDERS_ID) {
      return "All";
    }
    return (
      sortedFolders.find((folder) => folder.id === selectedFolderId)?.name ??
      "Unknown Folder"
    );
  }, [selectedFolderId, sortedFolders]);

  const filteredCiphers = useMemo(() => {
    const all = viewData?.ciphers ?? [];
    if (selectedFolderId === ALL_FOLDERS_ID) {
      return all;
    }
    return all.filter((cipher) => cipher.folderId === selectedFolderId);
  }, [selectedFolderId, viewData?.ciphers]);

  useEffect(() => {
    if (!selectedCipherId) {
      return;
    }
    const existsInFilteredList = filteredCiphers.some(
      (cipher) => cipher.id === selectedCipherId,
    );
    if (existsInFilteredList) {
      return;
    }
    detailRequestSeqRef.current += 1;
    setSelectedCipherId(null);
    setSelectedCipherDetail(null);
    setCipherDetailError("");
    setIsCipherDetailLoading(false);
  }, [filteredCiphers, selectedCipherId]);

  const folderCipherCount = useMemo(() => {
    const map = new Map<string, number>();
    for (const cipher of viewData?.ciphers ?? []) {
      if (!cipher.folderId) {
        continue;
      }
      map.set(cipher.folderId, (map.get(cipher.folderId) ?? 0) + 1);
    }
    return map;
  }, [viewData?.ciphers]);

  return (
    <main className="min-h-dvh bg-[radial-gradient(circle_at_12%_8%,_hsl(212_95%_96%),_transparent_36%),radial-gradient(circle_at_92%_92%,_hsl(215_95%_97%),_transparent_40%),linear-gradient(145deg,_hsl(216_55%_98%),_hsl(0_0%_100%))] p-4 md:p-8">
      <section className="mx-auto flex w-full max-w-7xl flex-col gap-4">
        <Card className="border-white/80 bg-white/90 shadow-sm">
          <CardHeader className="flex flex-row items-start justify-between gap-3">
            <div className="space-y-1">
              <CardTitle className="text-2xl">Vault Browser</CardTitle>
              <CardDescription>
                左侧为 folders（All + 所有 folder），右侧为当前 folder 的
                ciphers
              </CardDescription>
            </div>
            <div className="flex flex-wrap items-center gap-2">
              <Button
                type="button"
                variant="outline"
                onClick={onLock}
                disabled={isLocking || isLoggingOut || isRefreshing}
              >
                {isLocking ? (
                  <LoaderCircle className="animate-spin" />
                ) : (
                  <Lock />
                )}
                锁定
              </Button>
              <Button
                type="button"
                variant="outline"
                onClick={onLogout}
                disabled={isLoggingOut || isLocking || isRefreshing}
              >
                {isLoggingOut ? (
                  <LoaderCircle className="animate-spin" />
                ) : (
                  <LogOut />
                )}
                登出
              </Button>
              <Button
                type="button"
                variant="outline"
                onClick={loadVaultData}
                disabled={isRefreshing || isLocking || isLoggingOut}
              >
                {isRefreshing ? (
                  <LoaderCircle className="animate-spin" />
                ) : (
                  <RefreshCw />
                )}
                刷新
              </Button>
            </div>
          </CardHeader>
        </Card>

        {pageState === "loading" && (
          <Card className="border-white/80 bg-white/90 shadow-sm">
            <CardContent className="flex items-center gap-2 py-6 text-sm text-slate-700">
              <LoaderCircle className="animate-spin" />
              正在加载密码库数据...
            </CardContent>
          </Card>
        )}

        {pageState === "needsLogin" && (
          <Card className="border-white/80 bg-white/90 shadow-sm">
            <CardContent className="space-y-4 py-6">
              <div className="rounded-lg border border-amber-200 bg-amber-50 px-3 py-2 text-sm text-amber-800">
                未检测到登录会话，请先登录。
              </div>
              <Button asChild>
                <Link to="/">
                  <LogIn />
                  前往登录页
                </Link>
              </Button>
            </CardContent>
          </Card>
        )}

        {pageState === "locked" && (
          <Card className="border-white/80 bg-white/90 shadow-sm">
            <CardContent className="space-y-4 py-6">
              <div className="rounded-lg border border-blue-200 bg-blue-50 px-3 py-2 text-sm text-blue-800">
                当前会话已锁定，请先输入 master password 解锁后再查看数据。
              </div>
              <Button asChild>
                <Link to="/unlock">
                  <KeyRound />
                  前往解锁页
                </Link>
              </Button>
            </CardContent>
          </Card>
        )}

        {pageState === "error" && (
          <Card className="border-white/80 bg-white/90 shadow-sm">
            <CardContent className="space-y-4 py-6">
              <div className="rounded-lg border border-red-200 bg-red-50 px-3 py-2 text-sm text-red-700">
                <ShieldAlert className="mr-1 inline size-4" />
                {errorText || "读取密码库时发生错误。"}
              </div>
              <Button type="button" variant="outline" onClick={loadVaultData}>
                重试
              </Button>
            </CardContent>
          </Card>
        )}

        {pageState === "ready" && viewData && (
          <Card className="border-white/80 bg-white/90 shadow-sm">
            <CardHeader className="space-y-2">
              <div className="flex flex-wrap items-center gap-2">
                <Badge variant="secondary">
                  account: {restoreState?.accountId ?? viewData.accountId}
                </Badge>
                <Badge variant="outline">folders: {sortedFolders.length}</Badge>
                <Badge variant="outline">
                  ciphers: {viewData.totalCiphers}
                </Badge>
                <Badge variant="outline">
                  sync: {viewData.syncStatus.state}/
                  {viewData.syncStatus.wsStatus}
                </Badge>
              </div>
            </CardHeader>
            <CardContent className="grid gap-4 md:grid-cols-[280px_minmax(0,1fr)]">
              <aside className="space-y-2 rounded-xl border border-slate-200 bg-white p-3">
                <div className="flex items-center gap-2 text-sm font-medium text-slate-700">
                  <FolderOpen className="size-4" />
                  Folders
                </div>
                <Separator />

                <Button
                  type="button"
                  variant={
                    selectedFolderId === ALL_FOLDERS_ID ? "secondary" : "ghost"
                  }
                  className="w-full justify-between"
                  onClick={() => setSelectedFolderId(ALL_FOLDERS_ID)}
                >
                  <span>All</span>
                  <Badge variant="outline">{viewData.ciphers.length}</Badge>
                </Button>

                {sortedFolders.map((folder) => (
                  <Button
                    key={folder.id}
                    type="button"
                    variant={
                      selectedFolderId === folder.id ? "secondary" : "ghost"
                    }
                    className="w-full justify-between"
                    onClick={() => setSelectedFolderId(folder.id)}
                  >
                    <span className="max-w-[180px] truncate text-left">
                      {folder.name ?? "Untitled Folder"}
                    </span>
                    <Badge variant="outline">
                      {folderCipherCount.get(folder.id) ?? 0}
                    </Badge>
                  </Button>
                ))}
              </aside>

              <section className="space-y-3 rounded-xl border border-slate-200 bg-white p-3">
                <div className="flex items-center justify-between gap-2">
                  <div className="text-sm font-medium text-slate-700">
                    Ciphers in: {currentFolderName}
                  </div>
                  <Badge variant="secondary">
                    {filteredCiphers.length} items
                  </Badge>
                </div>
                <Separator />

                <div className="grid gap-3 lg:grid-cols-[minmax(0,1fr)_minmax(0,1fr)]">
                  <div className="space-y-2">
                    {filteredCiphers.length === 0 && (
                      <div className="rounded-lg border border-slate-200 bg-slate-50 px-3 py-2 text-sm text-slate-600">
                        当前 folder 下没有 cipher。
                      </div>
                    )}

                    {filteredCiphers.map((cipher) => (
                      <CipherRow
                        key={cipher.id}
                        cipher={cipher}
                        selected={cipher.id === selectedCipherId}
                        loading={
                          isCipherDetailLoading &&
                          cipher.id === selectedCipherId
                        }
                        onClick={() => loadCipherDetail(cipher.id)}
                      />
                    ))}
                  </div>

                  <div className="rounded-lg border border-slate-200 bg-slate-50/40 p-3">
                    {!selectedCipherId && (
                      <div className="text-sm text-slate-600">
                        点击左侧 cipher 查看详情。
                      </div>
                    )}

                    {selectedCipherId && isCipherDetailLoading && (
                      <div className="flex items-center gap-2 text-sm text-slate-700">
                        <LoaderCircle className="size-4 animate-spin" />
                        正在加载 cipher 详情...
                      </div>
                    )}

                    {selectedCipherId &&
                      !isCipherDetailLoading &&
                      cipherDetailError && (
                        <div className="rounded-lg border border-red-200 bg-red-50 px-3 py-2 text-sm text-red-700">
                          <ShieldAlert className="mr-1 inline size-4" />
                          {cipherDetailError}
                        </div>
                      )}

                    {selectedCipherId &&
                      !isCipherDetailLoading &&
                      !cipherDetailError &&
                      selectedCipherDetail && (
                        <CipherDetailPanel cipher={selectedCipherDetail} />
                      )}
                  </div>
                </div>
              </section>
            </CardContent>
          </Card>
        )}
      </section>
    </main>
  );
}

function CipherRow({
  cipher,
  selected,
  loading,
  onClick,
}: {
  cipher: VaultCipherItemDto;
  selected: boolean;
  loading: boolean;
  onClick: () => void;
}) {
  return (
    <button
      type="button"
      className={[
        "w-full rounded-lg border p-3 text-left transition-colors",
        selected
          ? "border-sky-300 bg-sky-50/70"
          : "border-slate-200 bg-slate-50/60 hover:border-slate-300",
      ].join(" ")}
      onClick={onClick}
    >
      <div className="flex flex-wrap items-center justify-between gap-2">
        <div className="min-w-0">
          <div className="truncate text-sm font-medium text-slate-900">
            {cipher.name ?? "Untitled Cipher"}
          </div>
          <div className="mt-1 text-xs text-slate-600">id: {cipher.id}</div>
        </div>
        <div className="flex items-center gap-1.5">
          <Badge variant="outline">{toCipherTypeLabel(cipher.type)}</Badge>
          {cipher.attachmentCount > 0 && (
            <Badge variant="outline">
              attachments: {cipher.attachmentCount}
            </Badge>
          )}
        </div>
      </div>
      <div className="mt-2 text-xs text-slate-600">
        revision: {formatRevisionDate(cipher.revisionDate)}
      </div>
      {loading && (
        <div className="mt-2 inline-flex items-center gap-1.5 rounded border border-sky-200 bg-white/70 px-2 py-1 text-xs text-sky-700">
          <LoaderCircle className="size-3 animate-spin" />
          正在获取详情
        </div>
      )}
    </button>
  );
}

function CipherDetailPanel({ cipher }: { cipher: VaultCipherDetailDto }) {
  const username = firstNonEmptyText(
    cipher.login?.username,
    cipher.data?.username,
  );
  const password = firstNonEmptyText(
    cipher.login?.password,
    cipher.data?.password,
  );
  const totp = firstNonEmptyText(cipher.login?.totp, cipher.data?.totp);
  const singleUri = firstNonEmptyText(cipher.login?.uri, cipher.data?.uri);
  const uriList = compactText([
    ...((cipher.login?.uris ?? []).map((item) => item.uri) ?? []),
    ...((cipher.data?.uris ?? []).map((item) => item.uri) ?? []),
  ]);
  const notes = firstNonEmptyText(cipher.notes, cipher.data?.notes);
  const customFields = [
    ...(cipher.fields ?? []),
    ...(cipher.data?.fields ?? []),
  ].filter(
    (field) => isNonEmptyText(field.name) || isNonEmptyText(field.value),
  );
  const attachments = cipher.attachments ?? [];
  const passwordHistory = [
    ...(cipher.passwordHistory ?? []),
    ...(cipher.data?.passwordHistory ?? []),
  ].filter((item) => isNonEmptyText(item.password));

  const cardRows = [
    {
      label: "持卡人",
      value: firstNonEmptyText(
        cipher.card?.cardholderName,
        cipher.data?.cardholderName,
      ),
    },
    {
      label: "品牌",
      value: firstNonEmptyText(cipher.card?.brand, cipher.data?.brand),
    },
    {
      label: "卡号",
      value: firstNonEmptyText(cipher.card?.number, cipher.data?.number),
    },
    {
      label: "到期月",
      value: firstNonEmptyText(cipher.card?.expMonth, cipher.data?.expMonth),
    },
    {
      label: "到期年",
      value: firstNonEmptyText(cipher.card?.expYear, cipher.data?.expYear),
    },
    {
      label: "安全码",
      value: firstNonEmptyText(cipher.card?.code, cipher.data?.code),
    },
  ];

  const identityRows = [
    {
      label: "姓名",
      value: compactText([
        cipher.identity?.firstName ?? cipher.data?.firstName,
        cipher.identity?.middleName ?? cipher.data?.middleName,
        cipher.identity?.lastName ?? cipher.data?.lastName,
      ]).join(" "),
    },
    {
      label: "邮箱",
      value: firstNonEmptyText(cipher.identity?.email, cipher.data?.email),
    },
    {
      label: "电话",
      value: firstNonEmptyText(cipher.identity?.phone, cipher.data?.phone),
    },
    {
      label: "公司",
      value: firstNonEmptyText(cipher.identity?.company, cipher.data?.company),
    },
    {
      label: "地址",
      value: compactText([
        cipher.identity?.address1 ?? cipher.data?.address1,
        cipher.identity?.address2 ?? cipher.data?.address2,
        cipher.identity?.address3 ?? cipher.data?.address3,
        cipher.identity?.city ?? cipher.data?.city,
        cipher.identity?.state ?? cipher.data?.state,
        cipher.identity?.postalCode ?? cipher.data?.postalCode,
        cipher.identity?.country ?? cipher.data?.country,
      ]).join(", "),
    },
  ];

  const sshPrivateKey = firstNonEmptyText(
    cipher.sshKey?.privateKey,
    cipher.data?.privateKey,
  );
  const sshPublicKey = firstNonEmptyText(
    cipher.sshKey?.publicKey,
    cipher.data?.publicKey,
  );
  const sshFingerprint = firstNonEmptyText(
    cipher.sshKey?.keyFingerprint,
    cipher.data?.keyFingerprint,
  );

  return (
    <div className="space-y-4">
      <div className="space-y-1">
        <div className="text-sm font-semibold text-slate-900">
          {cipher.name ?? "Untitled Cipher"}
        </div>
        <div className="text-xs text-slate-600">id: {cipher.id}</div>
      </div>

      <DetailGrid
        items={[
          { label: "类型", value: toCipherTypeLabel(cipher.type) },
          { label: "Folder", value: cipher.folderId },
          { label: "Organization", value: cipher.organizationId },
          {
            label: "创建时间",
            value: formatRevisionDateOrNull(cipher.creationDate),
          },
          {
            label: "更新时间",
            value: formatRevisionDateOrNull(cipher.revisionDate),
          },
          {
            label: "收藏",
            value:
              cipher.favorite == null ? null : cipher.favorite ? "是" : "否",
          },
          {
            label: "附件数",
            value: String(cipher.attachments.length),
          },
          {
            label: "Collection 数",
            value: String(cipher.collectionIds.length),
          },
        ]}
      />

      {(username || password || totp || singleUri || uriList.length > 0) && (
        <div className="space-y-2">
          <div className="text-sm font-medium text-slate-800">登录信息</div>
          <DetailGrid
            items={[
              { label: "用户名", value: username },
              { label: "密码", value: password },
              { label: "TOTP", value: totp },
              { label: "主 URI", value: singleUri },
            ]}
          />
          {uriList.length > 0 && (
            <div className="space-y-1">
              <div className="text-xs text-slate-600">URI 列表</div>
              <div className="space-y-1">
                {uriList.map((uri) => (
                  <div
                    key={uri}
                    className="rounded border border-slate-200 bg-white px-2 py-1.5 font-mono text-xs break-all text-slate-700"
                  >
                    {uri}
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>
      )}

      {notes && (
        <div className="space-y-1">
          <div className="text-sm font-medium text-slate-800">备注</div>
          <pre className="whitespace-pre-wrap rounded border border-slate-200 bg-white px-2 py-1.5 text-xs text-slate-700">
            {notes}
          </pre>
        </div>
      )}

      {customFields.length > 0 && (
        <div className="space-y-2">
          <div className="text-sm font-medium text-slate-800">自定义字段</div>
          <div className="space-y-1.5">
            {customFields.map((field, index) => (
              <div
                key={`${field.name ?? "field"}-${index}`}
                className="rounded border border-slate-200 bg-white px-2 py-1.5 text-xs text-slate-700"
              >
                <span className="font-medium text-slate-900">
                  {field.name ?? "Unnamed"}
                </span>
                <span>: </span>
                <span className="break-all">{field.value ?? "—"}</span>
              </div>
            ))}
          </div>
        </div>
      )}

      {cardRows.some((item) => isNonEmptyText(item.value)) && (
        <div className="space-y-2">
          <div className="text-sm font-medium text-slate-800">银行卡信息</div>
          <DetailGrid items={cardRows} />
        </div>
      )}

      {identityRows.some((item) => isNonEmptyText(item.value)) && (
        <div className="space-y-2">
          <div className="text-sm font-medium text-slate-800">身份信息</div>
          <DetailGrid items={identityRows} />
        </div>
      )}

      {(sshPrivateKey || sshPublicKey || sshFingerprint) && (
        <div className="space-y-2">
          <div className="text-sm font-medium text-slate-800">SSH Key</div>
          <DetailGrid
            items={[{ label: "Fingerprint", value: sshFingerprint }]}
          />
          {sshPublicKey && (
            <pre className="whitespace-pre-wrap rounded border border-slate-200 bg-white px-2 py-1.5 text-xs text-slate-700">
              {sshPublicKey}
            </pre>
          )}
          {sshPrivateKey && (
            <pre className="whitespace-pre-wrap rounded border border-slate-200 bg-white px-2 py-1.5 text-xs text-slate-700">
              {sshPrivateKey}
            </pre>
          )}
        </div>
      )}

      {attachments.length > 0 && (
        <div className="space-y-2">
          <div className="text-sm font-medium text-slate-800">附件</div>
          <div className="space-y-1.5">
            {attachments.map((attachment) => (
              <div
                key={attachment.id}
                className="rounded border border-slate-200 bg-white px-2 py-1.5 text-xs text-slate-700"
              >
                <div className="font-medium text-slate-900">
                  {attachment.fileName ?? attachment.id}
                </div>
                <div className="mt-1 text-slate-600">
                  {firstNonEmptyText(attachment.sizeName, attachment.size) ??
                    "未知大小"}
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      {passwordHistory.length > 0 && (
        <div className="space-y-2">
          <div className="text-sm font-medium text-slate-800">密码历史</div>
          <div className="space-y-1.5">
            {passwordHistory.map((item, index) => (
              <div
                key={`${item.password ?? "password"}-${index}`}
                className="rounded border border-slate-200 bg-white px-2 py-1.5 text-xs text-slate-700"
              >
                <div className="font-mono break-all">{item.password}</div>
                <div className="mt-1 text-slate-600">
                  {formatRevisionDateOrNull(item.lastUsedDate) ?? "Unknown"}
                </div>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}

function DetailGrid({
  items,
}: {
  items: Array<{ label: string; value: string | null | undefined }>;
}) {
  const visibleItems = items.filter((item) => isNonEmptyText(item.value));

  if (visibleItems.length === 0) {
    return null;
  }

  return (
    <div className="grid gap-1.5 sm:grid-cols-2">
      {visibleItems.map((item) => (
        <div
          key={item.label}
          className="rounded border border-slate-200 bg-white px-2 py-1.5"
        >
          <div className="text-[11px] uppercase tracking-wide text-slate-500">
            {item.label}
          </div>
          <div className="mt-1 text-xs break-all text-slate-800">
            {item.value}
          </div>
        </div>
      ))}
    </div>
  );
}
