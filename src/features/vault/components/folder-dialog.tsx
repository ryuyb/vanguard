import { useEffect, useState } from "react";
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
                ? "新建子文件夹"
                : mode === "create"
                  ? "新建文件夹"
                  : "重命名文件夹"}
            </DialogTitle>
            <DialogDescription>
              {isCreatingSubFolder
                ? `在 "${parentFolderName}" 下创建子文件夹`
                : mode === "create"
                  ? "创建一个新文件夹来组织你的密码"
                  : "为文件夹输入新名称"}
            </DialogDescription>
          </DialogHeader>
          <div className="grid gap-4 py-4">
            {isCreatingSubFolder && (
              <div className="rounded-md bg-blue-50 px-3 py-2 text-sm text-blue-700">
                完整路径:{" "}
                <strong>
                  {parentFolderName}/{name || "..."}
                </strong>
              </div>
            )}
            <div className="grid gap-2">
              <Label htmlFor="folder-name">
                {isCreatingSubFolder ? "子文件夹名称" : "文件夹名称"}
              </Label>
              <Input
                id="folder-name"
                value={name}
                onChange={(e) => setName(e.target.value)}
                placeholder={
                  isCreatingSubFolder ? "输入子文件夹名称" : "输入文件夹名称"
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
              取消
            </Button>
            <Button type="submit" disabled={!name.trim() || isLoading}>
              {isLoading ? "处理中..." : mode === "create" ? "创建" : "重命名"}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}
