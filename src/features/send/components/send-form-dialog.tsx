import { useForm } from "@tanstack/react-form";
import { ChevronDown, ChevronRight, LoaderCircle } from "lucide-react";
import { useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import type { SendItemDto, SyncSend } from "@/bindings";
import { TextInput } from "@/components/text-input";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";
import { Button } from "@/components/ui/button";
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "@/components/ui/collapsible";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Switch } from "@/components/ui/switch";
import { Textarea } from "@/components/ui/textarea";

type SendFormDialogProps = {
  open: boolean;
  mode: "create" | "edit";
  initialSend?: SendItemDto | null;
  onOpenChange: (open: boolean) => void;
  onConfirm: (send: SyncSend, fileData?: number[] | null) => void;
  onRemovePassword?: (sendId: string) => Promise<void>;
  isLoading?: boolean;
};

function defaultDeletionDate(): string {
  const d = new Date();
  d.setDate(d.getDate() + 7);
  // datetime-local format: YYYY-MM-DDTHH:mm
  return d.toISOString().slice(0, 16);
}

export function SendFormDialog({
  open,
  mode,
  initialSend,
  onOpenChange,
  onConfirm,
  onRemovePassword,
  isLoading = false,
}: SendFormDialogProps) {
  const { t } = useTranslation();
  const [advancedOpen, setAdvancedOpen] = useState(false);
  const [selectedFile, setSelectedFile] = useState<File | null>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);
  const [sendType, setSendType] = useState(initialSend?.type ?? 0);
  const hasExistingPassword =
    mode === "edit" && initialSend?.hasPassword === true;
  const [passwordRemoved, setPasswordRemoved] = useState(false);
  const [isRemovingPassword, setIsRemovingPassword] = useState(false);
  const [confirmRemovePassword, setConfirmRemovePassword] = useState(false);

  const form = useForm({
    defaultValues: {
      type: initialSend?.type ?? 0,
      name: initialSend?.name ?? "",
      textContent: initialSend?.text?.text ?? "",
      textHidden: initialSend?.text?.hidden ?? false,
      notes: initialSend?.notes ?? "",
      password: "",
      maxAccessCount:
        initialSend?.maxAccessCount != null
          ? String(initialSend.maxAccessCount)
          : "",
      expirationDate: initialSend?.expirationDate?.slice(0, 16) ?? "",
      deletionDate:
        initialSend?.deletionDate?.slice(0, 16) ?? defaultDeletionDate(),
      hideEmail: initialSend?.hideEmail ?? false,
      disabled: initialSend?.disabled ?? false,
    },
    onSubmit: async ({ value }) => {
      let fileData: number[] | null = null;
      if (value.type === 1 && selectedFile) {
        const buf = await selectedFile.arrayBuffer();
        fileData = Array.from(new Uint8Array(buf));
      }

      const send: SyncSend = {
        id: mode === "edit" && initialSend ? initialSend.id : "",
        type: value.type,
        name: value.name || null,
        notes: value.notes || null,
        key: initialSend?.key ?? null,
        password:
          hasExistingPassword && !passwordRemoved
            ? null
            : value.password || null,
        text:
          value.type === 0
            ? { text: value.textContent || null, hidden: value.textHidden }
            : null,
        file:
          value.type === 1 && selectedFile
            ? {
                id: null,
                file_name: selectedFile.name,
                size: null,
                size_name: null,
              }
            : null,
        max_access_count: value.maxAccessCount
          ? Number(value.maxAccessCount)
          : null,
        access_count: null,
        disabled: value.disabled,
        hide_email: value.hideEmail,
        expiration_date: value.expirationDate
          ? new Date(value.expirationDate).toISOString()
          : null,
        deletion_date: value.deletionDate
          ? new Date(value.deletionDate).toISOString()
          : null,
        revision_date: null,
        object: null,
        access_id: null,
        emails: null,
        auth_type: null,
      };

      onConfirm(send, fileData);
    },
  });

  const isFile = sendType === 1;

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-md max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>
            {mode === "create"
              ? t("send.form.createTitle")
              : t("send.form.editTitle")}
          </DialogTitle>
          <DialogDescription>
            {mode === "create"
              ? t("send.form.createDescription")
              : t("send.form.editDescription")}
          </DialogDescription>
        </DialogHeader>

        <form
          onSubmit={(e) => {
            e.preventDefault();
            void form.handleSubmit();
          }}
          className="space-y-4"
        >
          {/* Type */}
          <div className="space-y-1.5">
            <Label>{t("send.form.type")}</Label>
            <form.Field name="type">
              {(field) => (
                <Select
                  value={String(field.state.value)}
                  onValueChange={(v) => {
                    field.handleChange(Number(v));
                    setSendType(Number(v));
                  }}
                  disabled={mode === "edit"}
                >
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="0">{t("send.types.text")}</SelectItem>
                    <SelectItem value="1">{t("send.types.file")}</SelectItem>
                  </SelectContent>
                </Select>
              )}
            </form.Field>
          </div>

          {/* Name */}
          <div className="space-y-1.5">
            <Label>{t("send.form.name")} *</Label>
            <form.Field name="name">
              {(field) => (
                <TextInput
                  value={field.state.value}
                  onChange={(e) => field.handleChange(e.target.value)}
                  placeholder={t("send.form.name")}
                />
              )}
            </form.Field>
          </div>

          {/* Text content */}
          {!isFile && (
            <div className="space-y-1.5">
              <Label>{t("send.form.textContent")}</Label>
              <form.Field name="textContent">
                {(field) => (
                  <Textarea
                    value={field.state.value}
                    onChange={(e) => field.handleChange(e.target.value)}
                    rows={4}
                    placeholder={t("send.form.textContent")}
                  />
                )}
              </form.Field>
              <form.Field name="textHidden">
                {(field) => (
                  <div className="flex items-center gap-2">
                    <Switch
                      checked={field.state.value}
                      onCheckedChange={field.handleChange}
                      id="textHidden"
                    />
                    <Label htmlFor="textHidden" className="cursor-pointer">
                      {t("send.form.hideText")}
                    </Label>
                  </div>
                )}
              </form.Field>
            </div>
          )}

          {/* File */}
          {isFile && (
            <div className="space-y-1.5">
              <Label>{t("send.form.file")}</Label>
              {mode === "edit" && initialSend?.fileName ? (
                <div className="text-sm text-slate-600">
                  <span>{initialSend.fileName}</span>
                  {initialSend.sizeName && (
                    <span className="ml-2 text-slate-400">
                      ({initialSend.sizeName})
                    </span>
                  )}
                </div>
              ) : (
                <>
                  <div className="flex items-center gap-2">
                    <Button
                      type="button"
                      variant="outline"
                      size="sm"
                      onClick={() => fileInputRef.current?.click()}
                    >
                      {t("send.form.chooseFile")}
                    </Button>
                    {selectedFile && (
                      <span className="text-xs text-slate-600 truncate">
                        {selectedFile.name}
                      </span>
                    )}
                  </div>
                  <input
                    ref={fileInputRef}
                    type="file"
                    className="hidden"
                    onChange={(e) =>
                      setSelectedFile(e.target.files?.[0] ?? null)
                    }
                  />
                </>
              )}
            </div>
          )}

          {/* Notes */}
          <div className="space-y-1.5">
            <Label>{t("send.form.notes")}</Label>
            <form.Field name="notes">
              {(field) => (
                <Textarea
                  value={field.state.value}
                  onChange={(e) => field.handleChange(e.target.value)}
                  rows={2}
                  placeholder={t("send.form.notes")}
                />
              )}
            </form.Field>
          </div>

          {/* Advanced */}
          <Collapsible open={advancedOpen} onOpenChange={setAdvancedOpen}>
            <CollapsibleTrigger asChild>
              <Button
                type="button"
                variant="ghost"
                size="sm"
                className="gap-1 px-0 text-slate-600"
              >
                {advancedOpen ? (
                  <ChevronDown className="size-4" />
                ) : (
                  <ChevronRight className="size-4" />
                )}
                {t("send.form.advanced")}
              </Button>
            </CollapsibleTrigger>
            <CollapsibleContent className="space-y-4 pt-2">
              <div className="space-y-1.5">
                <Label>{t("send.form.password")}</Label>
                {hasExistingPassword && !passwordRemoved ? (
                  <div className="flex items-center gap-2">
                    <TextInput
                      type="password"
                      value="••••••••"
                      disabled
                      className="flex-1"
                    />
                    <Button
                      type="button"
                      variant="outline"
                      size="sm"
                      disabled={isRemovingPassword}
                      onClick={() => setConfirmRemovePassword(true)}
                    >
                      {isRemovingPassword ? (
                        <LoaderCircle className="size-4 animate-spin" />
                      ) : (
                        t("send.form.removePassword")
                      )}
                    </Button>
                  </div>
                ) : (
                  <form.Field name="password">
                    {(field) => (
                      <TextInput
                        type="password"
                        value={field.state.value}
                        onChange={(e) => field.handleChange(e.target.value)}
                      />
                    )}
                  </form.Field>
                )}
              </div>

              <div className="space-y-1.5">
                <Label>{t("send.form.maxAccessCount")}</Label>
                <form.Field name="maxAccessCount">
                  {(field) => (
                    <TextInput
                      type="number"
                      min="1"
                      value={field.state.value}
                      onChange={(e) => field.handleChange(e.target.value)}
                    />
                  )}
                </form.Field>
              </div>

              <div className="space-y-1.5">
                <Label>{t("send.form.expirationDate")}</Label>
                <form.Field name="expirationDate">
                  {(field) => (
                    <TextInput
                      type="datetime-local"
                      value={field.state.value}
                      onChange={(e) => field.handleChange(e.target.value)}
                    />
                  )}
                </form.Field>
              </div>

              <div className="space-y-1.5">
                <Label>{t("send.form.deletionDate")} *</Label>
                <form.Field name="deletionDate">
                  {(field) => (
                    <TextInput
                      type="datetime-local"
                      value={field.state.value}
                      onChange={(e) => field.handleChange(e.target.value)}
                    />
                  )}
                </form.Field>
              </div>

              <form.Field name="hideEmail">
                {(field) => (
                  <div className="flex items-center gap-2">
                    <Switch
                      checked={field.state.value}
                      onCheckedChange={field.handleChange}
                      id="hideEmail"
                    />
                    <Label htmlFor="hideEmail" className="cursor-pointer">
                      {t("send.form.hideEmail")}
                    </Label>
                  </div>
                )}
              </form.Field>

              <form.Field name="disabled">
                {(field) => (
                  <div className="flex items-center gap-2">
                    <Switch
                      checked={field.state.value ?? false}
                      onCheckedChange={field.handleChange}
                      id="disabled"
                    />
                    <Label htmlFor="disabled" className="cursor-pointer">
                      {t("send.form.disable")}
                    </Label>
                  </div>
                )}
              </form.Field>
            </CollapsibleContent>
          </Collapsible>

          <DialogFooter>
            <Button
              type="button"
              variant="outline"
              onClick={() => onOpenChange(false)}
            >
              {t("common.actions.cancel")}
            </Button>
            <Button type="submit" disabled={isLoading}>
              {isLoading ? (
                <LoaderCircle className="size-4 animate-spin" />
              ) : mode === "create" ? (
                t("send.form.submit.create")
              ) : (
                t("send.form.submit.save")
              )}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>

      <AlertDialog
        open={confirmRemovePassword}
        onOpenChange={setConfirmRemovePassword}
      >
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>
              {t("send.dialogs.removePassword.title")}
            </AlertDialogTitle>
            <AlertDialogDescription>
              {t("send.dialogs.removePassword.description")}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel disabled={isRemovingPassword}>
              {t("common.actions.cancel")}
            </AlertDialogCancel>
            <AlertDialogAction
              disabled={isRemovingPassword}
              onClick={async (e) => {
                e.preventDefault();
                if (!onRemovePassword || !initialSend) return;
                setIsRemovingPassword(true);
                try {
                  await onRemovePassword(initialSend.id);
                  setPasswordRemoved(true);
                  setConfirmRemovePassword(false);
                } finally {
                  setIsRemovingPassword(false);
                }
              }}
            >
              {isRemovingPassword ? (
                <LoaderCircle className="size-4 animate-spin" />
              ) : (
                t("send.dialogs.removePassword.confirm")
              )}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </Dialog>
  );
}
