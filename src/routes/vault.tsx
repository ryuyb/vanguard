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
import { useCallback, useEffect, useMemo, useState } from "react";
import {
  commands,
  type RestoreAuthStateResponseDto,
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

function sortFolders(folders: VaultFolderItemDto[]) {
  return [...folders].sort((left, right) =>
    (left.name ?? "").localeCompare(right.name ?? "", "zh-Hans-CN"),
  );
}

function VaultPage() {
  const [pageState, setPageState] = useState<VaultPageState>("loading");
  const [restoreState, setRestoreState] =
    useState<RestoreAuthStateResponseDto | null>(null);
  const [viewData, setViewData] = useState<VaultViewDataResponseDto | null>(
    null,
  );
  const [selectedFolderId, setSelectedFolderId] = useState(ALL_FOLDERS_ID);
  const [errorText, setErrorText] = useState("");
  const [isRefreshing, setIsRefreshing] = useState(false);
  const [isLocking, setIsLocking] = useState(false);
  const [isLoggingOut, setIsLoggingOut] = useState(false);

  const loadVaultData = useCallback(async () => {
    setIsRefreshing(true);
    setErrorText("");
    setPageState("loading");

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

                {filteredCiphers.length === 0 && (
                  <div className="rounded-lg border border-slate-200 bg-slate-50 px-3 py-2 text-sm text-slate-600">
                    当前 folder 下没有 cipher。
                  </div>
                )}

                {filteredCiphers.map((cipher) => (
                  <CipherRow key={cipher.id} cipher={cipher} />
                ))}
              </section>
            </CardContent>
          </Card>
        )}
      </section>
    </main>
  );
}

function CipherRow({ cipher }: { cipher: VaultCipherItemDto }) {
  return (
    <div className="rounded-lg border border-slate-200 bg-slate-50/60 p-3">
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
    </div>
  );
}
