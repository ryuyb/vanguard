import { AlertTriangle, LoaderCircle } from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";

type DeleteCipherDialogProps = {
  open: boolean;
  cipherName: string;
  onOpenChange: (open: boolean) => void;
  onConfirm: () => void;
  isLoading?: boolean;
};

export function DeleteCipherDialog({
  open,
  cipherName,
  onOpenChange,
  onConfirm,
  isLoading = false,
}: DeleteCipherDialogProps) {
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-md">
        <DialogHeader>
          <div className="flex items-center gap-3">
            <div className="flex size-10 items-center justify-center rounded-full bg-red-100">
              <AlertTriangle className="size-5 text-red-600" />
            </div>
            <DialogTitle className="text-lg font-bold text-slate-900">
              删除项目
            </DialogTitle>
          </div>
          <DialogDescription className="text-sm text-slate-600 pt-2">
            确定要删除{" "}
            <span className="font-semibold text-slate-900">"{cipherName}"</span>{" "}
            吗？ 此操作无法撤销。
          </DialogDescription>
        </DialogHeader>

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
            type="button"
            variant="destructive"
            onClick={onConfirm}
            disabled={isLoading}
            className="h-10"
          >
            {isLoading ? (
              <>
                <LoaderCircle className="size-4 animate-spin" />
                删除中...
              </>
            ) : (
              "删除"
            )}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
