import { Eye, EyeOff, KeyRound, Mail } from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  InputGroup,
  InputGroupAddon,
  InputGroupInput,
} from "@/components/ui/input-group";
import { Label } from "@/components/ui/label";

type LoginCredentialsFieldsProps = {
  email: string;
  masterPassword: string;
  showPassword: boolean;
  isSubmitting: boolean;
  onEmailChange: (value: string) => void;
  onMasterPasswordChange: (value: string) => void;
  onToggleShowPassword: () => void;
};

export function LoginCredentialsFields({
  email,
  masterPassword,
  showPassword,
  isSubmitting,
  onEmailChange,
  onMasterPasswordChange,
  onToggleShowPassword,
}: LoginCredentialsFieldsProps) {
  return (
    <>
      <div className="space-y-2.5">
        <Label htmlFor="email" className="text-sm font-medium text-slate-700">
          邮箱地址
        </Label>
        <InputGroup>
          <InputGroupAddon>
            <Mail className="h-5 w-5 text-slate-400" />
          </InputGroupAddon>
          <InputGroupInput
            id="email"
            type="email"
            autoComplete="email"
            placeholder="you@example.com"
            value={email}
            onChange={(event) => onEmailChange(event.target.value)}
            disabled={isSubmitting}
            className="h-12 text-base"
          />
        </InputGroup>
      </div>

      <div className="space-y-2.5">
        <Label
          htmlFor="master-password"
          className="text-sm font-medium text-slate-700"
        >
          主密码
        </Label>
        <InputGroup>
          <InputGroupAddon>
            <KeyRound className="h-5 w-5 text-slate-400" />
          </InputGroupAddon>
          <InputGroupInput
            id="master-password"
            type={showPassword ? "text" : "password"}
            autoComplete="current-password"
            placeholder="输入主密码"
            value={masterPassword}
            onChange={(event) => onMasterPasswordChange(event.target.value)}
            disabled={isSubmitting}
            className="h-12 text-base"
          />
          <InputGroupAddon align="inline-end" className="px-1.5">
            <Button
              type="button"
              variant="ghost"
              size="icon-sm"
              className="text-slate-400 hover:text-slate-700 transition-colors"
              onClick={onToggleShowPassword}
              disabled={isSubmitting}
              aria-label={showPassword ? "隐藏密码" : "显示密码"}
            >
              {showPassword ? (
                <EyeOff className="h-5 w-5" />
              ) : (
                <Eye className="h-5 w-5" />
              )}
            </Button>
          </InputGroupAddon>
        </InputGroup>
      </div>
    </>
  );
}
