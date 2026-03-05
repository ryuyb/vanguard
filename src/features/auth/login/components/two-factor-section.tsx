import { LoaderCircle, Send, ShieldCheck } from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  InputGroup,
  InputGroupAddon,
  InputGroupInput,
} from "@/components/ui/input-group";
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import type { TwoFactorState } from "@/features/auth/login/types";
import { toProviderLabel } from "@/features/auth/login/utils";

type TwoFactorSectionProps = {
  state: TwoFactorState;
  isSubmitting: boolean;
  onProviderChange: (value: string) => void;
  onTokenChange: (value: string) => void;
  onSendEmailCode: () => void;
};

export function TwoFactorSection({
  state,
  isSubmitting,
  onProviderChange,
  onTokenChange,
  onSendEmailCode,
}: TwoFactorSectionProps) {
  const isDisabled = isSubmitting || state.isSendingEmailCode;

  return (
    <div className="space-y-4 rounded-xl border border-amber-200 bg-amber-50/80 p-4">
      <div className="flex items-center gap-2 text-sm font-medium text-amber-800">
        <ShieldCheck className="size-4" />
        二步验证
      </div>

      <div className="space-y-2">
        <Label htmlFor="two-factor-provider">验证方式</Label>
        <Select
          value={state.selectedProvider}
          onValueChange={onProviderChange}
          disabled={isDisabled}
        >
          <SelectTrigger id="two-factor-provider" className="w-full bg-white">
            <SelectValue placeholder="选择验证方式" />
          </SelectTrigger>
          <SelectContent>
            {state.providers.map((provider) => (
              <SelectItem key={provider} value={provider}>
                {toProviderLabel(provider)}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </div>

      <div className="space-y-2">
        <Label htmlFor="two-factor-token">二步验证码</Label>
        <InputGroup>
          <InputGroupAddon>
            <ShieldCheck className="text-slate-500" />
          </InputGroupAddon>
          <InputGroupInput
            id="two-factor-token"
            type="text"
            inputMode="numeric"
            autoComplete="one-time-code"
            placeholder="输入验证码"
            value={state.token}
            onChange={(event) => onTokenChange(event.target.value)}
            disabled={isDisabled}
          />
        </InputGroup>
      </div>

      {state.selectedProvider === "1" && (
        <Button
          type="button"
          variant="outline"
          className="w-full"
          disabled={isDisabled}
          onClick={onSendEmailCode}
        >
          {state.isSendingEmailCode && (
            <LoaderCircle className="animate-spin" />
          )}
          {!state.isSendingEmailCode && <Send />}
          {state.isSendingEmailCode
            ? "正在发送邮箱验证码..."
            : "发送邮箱验证码"}
        </Button>
      )}
    </div>
  );
}
