import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
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

type FolderDialogProps = {
  open: boolean;
  mode: "create" | "rename";
  initialName?: string;
  parentFolderName?: string | null;
  onOpenChange: (open: boolean) => void;
  onConfirm: (name: string) => void;
  isLoading?: boolean;
};

export function FolderDialog({
  open,
  mode,
  initialName = "",
  parentFolderName = null,
  onOpenChange,
  onConfirm,
  isLoading = false,
}: FolderDialogProps) {
  const { t } = useTranslation();
  const [name, setName] = useState(initialName);

  useEffect(() => {
    if (open) {
      setName(initialName);
    }
  }, [open, initialName]);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (name.trim()) {
      onConfirm(name.trim());
    }
  };

  const isCreatingSubFolder = mode === "create" && parentFolderName;

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[425px]">
        <form onSubmit={handleSubmit}>
          <DialogHeader>
            <DialogTitle>
              {isCreatingSubFolder
                ? t("vault.dialogs.folder.createSubFolderTitle")
                : mode === "create"
                  ? t("vault.dialogs.folder.createTitle")
                  : t("vault.dialogs.folder.renameTitle")}
            </DialogTitle>
            <DialogDescription>
              {isCreatingSubFolder
                ? t("vault.dialogs.folder.createSubFolderDescription", {
                    parentFolderName,
                  })
                : mode === "create"
                  ? t("vault.dialogs.folder.createDescription")
                  : t("vault.dialogs.folder.renameDescription")}
            </DialogDescription>
          </DialogHeader>
          <div className="grid gap-4 py-4">
            {isCreatingSubFolder && (
              <div className="rounded-md bg-blue-50 px-3 py-2 text-sm text-blue-700">
                {t("vault.dialogs.folder.fullPathLabel")}{" "}
                <strong>
                  {parentFolderName}/{name || "..."}
                </strong>
              </div>
            )}
            <div className="grid gap-2">
              <Label htmlFor="folder-name">
                {isCreatingSubFolder
                  ? t("vault.dialogs.folder.subFolderNameLabel")
                  : t("vault.dialogs.folder.folderNameLabel")}
              </Label>
              <Input
                id="folder-name"
                value={name}
                onChange={(e) => setName(e.target.value)}
                placeholder={
                  isCreatingSubFolder
                    ? t("vault.dialogs.folder.subFolderNamePlaceholder")
                    : t("vault.dialogs.folder.folderNamePlaceholder")
                }
                autoFocus
                disabled={isLoading}
              />
            </div>
          </div>
          <DialogFooter>
            <Button
              type="button"
              variant="outline"
              onClick={() => onOpenChange(false)}
              disabled={isLoading}
            >
              {t("common.actions.cancel")}
            </Button>
            <Button type="submit" disabled={!name.trim() || isLoading}>
              {isLoading
                ? t("vault.dialogs.folder.processing")
                : mode === "create"
                  ? t("vault.page.actions.create")
                  : t("vault.page.actions.rename")}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}
