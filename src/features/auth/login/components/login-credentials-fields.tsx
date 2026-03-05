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
      <div className="space-y-2">
        <Label htmlFor="email">登录邮箱</Label>
        <InputGroup>
          <InputGroupAddon>
            <Mail className="text-slate-500" />
          </InputGroupAddon>
          <InputGroupInput
            id="email"
            type="email"
            autoComplete="email"
            placeholder="you@example.com"
            value={email}
            onChange={(event) => onEmailChange(event.target.value)}
            disabled={isSubmitting}
          />
        </InputGroup>
      </div>

      <div className="space-y-2">
        <Label htmlFor="master-password">主密码</Label>
        <InputGroup>
          <InputGroupAddon>
            <KeyRound className="text-slate-500" />
          </InputGroupAddon>
          <InputGroupInput
            id="master-password"
            type={showPassword ? "text" : "password"}
            autoComplete="current-password"
            placeholder="输入主密码"
            value={masterPassword}
            onChange={(event) => onMasterPasswordChange(event.target.value)}
            disabled={isSubmitting}
          />
          <InputGroupAddon align="inline-end" className="px-1.5">
            <Button
              type="button"
              variant="ghost"
              size="icon-sm"
              className="text-slate-500 hover:text-slate-900"
              onClick={onToggleShowPassword}
              disabled={isSubmitting}
              aria-label={showPassword ? "隐藏密码" : "显示密码"}
            >
              {showPassword ? <EyeOff /> : <Eye />}
            </Button>
          </InputGroupAddon>
        </InputGroup>
      </div>
    </>
  );
}
