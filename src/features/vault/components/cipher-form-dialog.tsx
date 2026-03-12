import { LoaderCircle, Plus, X } from "lucide-react";
import { useEffect, useState } from "react";
import type { SyncCipher, VaultFolderItemDto } from "@/bindings";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Textarea } from "@/components/ui/textarea";

type CipherFormDialogProps = {
  open: boolean;
  mode: "create" | "edit";
  initialCipher?: SyncCipher | null;
  folderId?: string | null;
  folders?: VaultFolderItemDto[];
  onOpenChange: (open: boolean) => void;
  onConfirm: (cipher: SyncCipher) => void;
  isLoading?: boolean;
};

const CIPHER_TYPE_LOGIN = 1;
const CIPHER_TYPE_NOTE = 2;
const CIPHER_TYPE_CARD = 3;
const CIPHER_TYPE_IDENTITY = 4;

const CUSTOM_FIELD_TYPE_TEXT = 0;
const CUSTOM_FIELD_TYPE_HIDDEN = 1;
const CUSTOM_FIELD_TYPE_BOOLEAN = 2;
const CUSTOM_FIELD_TYPE_LINKED = 3;

export function CipherFormDialog({
  open,
  mode,
  initialCipher,
  folderId,
  folders = [],
  onOpenChange,
  onConfirm,
  isLoading = false,
}: CipherFormDialogProps) {
  const [cipherType, setCipherType] = useState<number>(CIPHER_TYPE_LOGIN);
  const [name, setName] = useState("");
  const [selectedFolderId, setSelectedFolderId] = useState<string>("");
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [uri, setUri] = useState("");
  const [notes, setNotes] = useState("");
  const [customFields, setCustomFields] = useState<
    Array<{ name: string; value: string; type: number }>
  >([]);

  useEffect(() => {
    if (open) {
      if (mode === "edit" && initialCipher) {
        setCipherType(initialCipher.type ?? CIPHER_TYPE_LOGIN);
        setName(initialCipher.name ?? "");
        setSelectedFolderId(initialCipher.folder_id ?? "");
        setUsername(
          initialCipher.login?.username ?? initialCipher.data?.username ?? "",
        );
        setPassword(
          initialCipher.login?.password ?? initialCipher.data?.password ?? "",
        );
        setUri(initialCipher.login?.uri ?? initialCipher.data?.uri ?? "");
        setNotes(initialCipher.notes ?? initialCipher.data?.notes ?? "");
        setCustomFields(
          (initialCipher.fields ?? []).map((f) => ({
            name: f.name ?? "",
            value: f.value ?? "",
            type: f.type ?? 0,
          })),
        );
      } else {
        setCipherType(CIPHER_TYPE_LOGIN);
        setName("");
        setSelectedFolderId(folderId ?? "");
        setUsername("");
        setPassword("");
        setUri("");
        setNotes("");
        setCustomFields([]);
      }
    }
  }, [open, mode, initialCipher, folderId]);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();

    const cipher: SyncCipher = {
      id: mode === "edit" && initialCipher ? initialCipher.id : "",
      organization_id: initialCipher?.organization_id ?? null,
      folder_id: selectedFolderId || null,
      type: cipherType,
      name: name || null,
      notes: notes || null,
      favorite: initialCipher?.favorite ?? false,
      fields: customFields.map((f) => ({
        name: f.name || null,
        value: f.value || null,
        type: f.type,
        linked_id: null,
      })),
      login:
        cipherType === CIPHER_TYPE_LOGIN
          ? {
              uri: uri || null,
              uris: uri
                ? [{ uri: uri || null, match: null, uri_checksum: null }]
                : [],
              username: username || null,
              password: password || null,
              password_revision_date: null,
              totp: null,
              autofill_on_page_load: null,
              fido2_credentials: [],
            }
          : null,
      data:
        cipherType === CIPHER_TYPE_LOGIN
          ? {
              name: name || null,
              notes: notes || null,
              fields: customFields.map((f) => ({
                name: f.name || null,
                value: f.value || null,
                type: f.type,
                linked_id: null,
              })),
              password_history: [],
              uri: uri || null,
              uris: uri
                ? [{ uri: uri || null, match: null, uri_checksum: null }]
                : [],
              username: username || null,
              password: password || null,
              password_revision_date: null,
              totp: null,
              autofill_on_page_load: null,
              fido2_credentials: [],
              type: null,
              cardholder_name: null,
              brand: null,
              number: null,
              exp_month: null,
              exp_year: null,
              code: null,
              title: null,
              first_name: null,
              middle_name: null,
              last_name: null,
              address1: null,
              address2: null,
              address3: null,
              city: null,
              state: null,
              postal_code: null,
              country: null,
              company: null,
              email: null,
              phone: null,
              ssn: null,
              passport_number: null,
              license_number: null,
              private_key: null,
              public_key: null,
              key_fingerprint: null,
            }
          : null,
      secure_note: cipherType === CIPHER_TYPE_NOTE ? { type: 0 } : null,
      card: null,
      identity: null,
      ssh_key: null,
      password_history: [],
      attachments: [],
      collection_ids: [],
      creation_date: initialCipher?.creation_date ?? null,
      deleted_date: null,
      revision_date: initialCipher?.revision_date ?? null,
      archived_date: null,
      key: null,
      edit: true,
      view_password: true,
      organization_use_totp: false,
      reprompt: null,
      permissions: null,
      object: null,
    };

    onConfirm(cipher);
  };

  const addCustomField = () => {
    setCustomFields([
      ...customFields,
      { name: "", value: "", type: CUSTOM_FIELD_TYPE_TEXT },
    ]);
  };

  const removeCustomField = (index: number) => {
    setCustomFields(customFields.filter((_, i) => i !== index));
  };

  const updateCustomField = (
    index: number,
    field: "name" | "value" | "type",
    value: string | number,
  ) => {
    const updated = [...customFields];
    updated[index] = { ...updated[index], [field]: value };
    setCustomFields(updated);
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-2xl max-h-[85vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle className="text-xl font-bold text-slate-900">
            {mode === "create" ? "新建项目" : "编辑项目"}
          </DialogTitle>
          <DialogDescription className="text-sm text-slate-600">
            {mode === "create" ? "创建一个新的密码或安全笔记" : "修改项目信息"}
          </DialogDescription>
        </DialogHeader>

        <form onSubmit={handleSubmit} className="space-y-5">
          <div className="space-y-2">
            <Label htmlFor="cipher-type" className="text-sm font-semibold">
              类型
            </Label>
            <Select
              value={String(cipherType)}
              onValueChange={(value) => setCipherType(Number(value))}
              disabled={mode === "edit"}
            >
              <SelectTrigger id="cipher-type" className="h-10 w-full">
                <SelectValue />
              </SelectTrigger>
              <SelectContent className="w-full">
                <SelectItem value={String(CIPHER_TYPE_LOGIN)}>登录</SelectItem>
                <SelectItem value={String(CIPHER_TYPE_NOTE)}>
                  安全笔记
                </SelectItem>
                <SelectItem value={String(CIPHER_TYPE_CARD)}>支付卡</SelectItem>
                <SelectItem value={String(CIPHER_TYPE_IDENTITY)}>
                  身份
                </SelectItem>
              </SelectContent>
            </Select>
          </div>

          <div className="space-y-2">
            <Label htmlFor="cipher-name" className="text-sm font-semibold">
              名称 <span className="text-red-500">*</span>
            </Label>
            <Input
              id="cipher-name"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="例如：GitHub、Gmail"
              required
              className="h-10"
            />
          </div>

          <div className="space-y-2">
            <Label htmlFor="cipher-folder" className="text-sm font-semibold">
              文件夹
            </Label>
            <Select
              value={selectedFolderId || "no-folder"}
              onValueChange={(value) => setSelectedFolderId(value === "no-folder" ? "" : value)}
            >
              <SelectTrigger id="cipher-folder" className="h-10 w-full">
                <SelectValue placeholder="无文件夹" />
              </SelectTrigger>
              <SelectContent className="w-full max-h-60">
                <SelectItem value="no-folder">无文件夹</SelectItem>
                {folders.map((folder) => (
                  <SelectItem key={folder.id} value={folder.id}>
                    {folder.name || "未命名文件夹"}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          {cipherType === CIPHER_TYPE_LOGIN && (
            <>
              <div className="space-y-2">
                <Label
                  htmlFor="cipher-username"
                  className="text-sm font-semibold"
                >
                  用户名
                </Label>
                <Input
                  id="cipher-username"
                  value={username}
                  onChange={(e) => setUsername(e.target.value)}
                  placeholder="用户名或邮箱"
                  className="h-10"
                />
              </div>

              <div className="space-y-2">
                <Label
                  htmlFor="cipher-password"
                  className="text-sm font-semibold"
                >
                  密码
                </Label>
                <Input
                  id="cipher-password"
                  type="password"
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  placeholder="密码"
                  className="h-10"
                />
              </div>

              <div className="space-y-2">
                <Label htmlFor="cipher-uri" className="text-sm font-semibold">
                  网站地址
                </Label>
                <Input
                  id="cipher-uri"
                  type="url"
                  value={uri}
                  onChange={(e) => setUri(e.target.value)}
                  placeholder="https://example.com"
                  className="h-10"
                />
              </div>
            </>
          )}

          <div className="space-y-2">
            <Label htmlFor="cipher-notes" className="text-sm font-semibold">
              备注
            </Label>
            <Textarea
              id="cipher-notes"
              value={notes}
              onChange={(e) => setNotes(e.target.value)}
              placeholder="添加备注信息..."
              rows={4}
              className="resize-none"
            />
          </div>

          {customFields.length > 0 && (
            <div className="space-y-3">
              <Label className="text-sm font-semibold">自定义字段</Label>
              {customFields.map((field, index) => (
                <div
                  key={`custom-field-${index}`}
                  className="space-y-2 rounded-lg border border-slate-200 bg-slate-50 p-3"
                >
                  <div className="flex gap-2 items-start">
                    <Input
                      value={field.name}
                      onChange={(e) =>
                        updateCustomField(index, "name", e.target.value)
                      }
                      placeholder="字段名"
                      className="h-10 flex-1"
                    />
                    <Select
                      value={String(field.type)}
                      onValueChange={(value) =>
                        updateCustomField(index, "type", Number(value))
                      }
                    >
                      <SelectTrigger className="h-10 w-32">
                        <SelectValue />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value={String(CUSTOM_FIELD_TYPE_TEXT)}>
                          文本
                        </SelectItem>
                        <SelectItem value={String(CUSTOM_FIELD_TYPE_HIDDEN)}>
                          隐藏
                        </SelectItem>
                        <SelectItem value={String(CUSTOM_FIELD_TYPE_BOOLEAN)}>
                          复选框
                        </SelectItem>
                        <SelectItem value={String(CUSTOM_FIELD_TYPE_LINKED)}>
                          链接
                        </SelectItem>
                      </SelectContent>
                    </Select>
                    <Button
                      type="button"
                      variant="ghost"
                      size="sm"
                      onClick={() => removeCustomField(index)}
                      className="h-10 px-3 shrink-0"
                    >
                      <X className="size-4" />
                    </Button>
                  </div>
                  {field.type === CUSTOM_FIELD_TYPE_BOOLEAN ? (
                    <Select
                      value={field.value}
                      onValueChange={(value) =>
                        updateCustomField(index, "value", value)
                      }
                    >
                      <SelectTrigger className="h-10 w-full">
                        <SelectValue placeholder="选择值" />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="true">是</SelectItem>
                        <SelectItem value="false">否</SelectItem>
                      </SelectContent>
                    </Select>
                  ) : (
                    <Input
                      value={field.value}
                      onChange={(e) =>
                        updateCustomField(index, "value", e.target.value)
                      }
                      placeholder={
                        field.type === CUSTOM_FIELD_TYPE_HIDDEN
                          ? "隐藏值"
                          : field.type === CUSTOM_FIELD_TYPE_LINKED
                            ? "链接地址"
                            : "字段值"
                      }
                      type={
                        field.type === CUSTOM_FIELD_TYPE_HIDDEN
                          ? "password"
                          : "text"
                      }
                      className="h-10"
                    />
                  )}
                </div>
              ))}
            </div>
          )}

          <Button
            type="button"
            variant="outline"
            onClick={addCustomField}
            className="w-full h-10"
          >
            <Plus className="size-4" />
            添加自定义字段
          </Button>

          <DialogFooter className="gap-2">
            <Button
              type="button"
              variant="outline"
              onClick={() => onOpenChange(false)}
              disabled={isLoading}
              className="h-10"
            >
              取消
            </Button>
            <Button
              type="submit"
              disabled={isLoading || !name.trim()}
              className="h-10 bg-blue-600 hover:bg-blue-700"
            >
              {isLoading ? (
                <>
                  <LoaderCircle className="size-4 animate-spin" />
                  {mode === "create" ? "创建中..." : "保存中..."}
                </>
              ) : mode === "create" ? (
                "创建"
              ) : (
                "保存"
              )}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}
