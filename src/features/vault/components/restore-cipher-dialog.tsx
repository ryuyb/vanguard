import { LoaderCircle, RotateCcw } from "lucide-react";
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

type RestoreCipherDialogProps = {
  open: boolean;
  cipherName: string;
  onOpenChange: (open: boolean) => void;
  onConfirm: () => void;
  isLoading?: boolean;
};

export function RestoreCipherDialog({
  open,
  cipherName,
  onOpenChange,
  onConfirm,
  isLoading = false,
}: RestoreCipherDialogProps) {
  const { t } = useTranslation();
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-md">
        <DialogHeader>
          <div className="flex items-center gap-3">
            <div className="flex size-10 items-center justify-center rounded-full bg-green-100">
              <RotateCcw className="size-5 text-green-600" />
            </div>
            <DialogTitle className="text-lg font-bold text-slate-900">
              {t("vault.dialogs.restoreCipher.title")}
            </DialogTitle>
          </div>
          <DialogDescription className="text-sm text-slate-600 pt-2">
            {t("vault.dialogs.restoreCipher.descriptionPrefix")}{" "}
            <span className="font-semibold text-slate-900">"{cipherName}"</span>{" "}
            {t("vault.dialogs.restoreCipher.descriptionSuffix")}
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
            {t("common.actions.cancel")}
          </Button>
          <Button
            type="button"
            onClick={onConfirm}
            disabled={isLoading}
            className="h-10"
          >
            {isLoading ? (
              <>
                <LoaderCircle className="size-4 animate-spin" />
                {t("vault.dialogs.restoreCipher.confirming")}
              </>
            ) : (
              t("vault.page.actions.restore")
            )}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
