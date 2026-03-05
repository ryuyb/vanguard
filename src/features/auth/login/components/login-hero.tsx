import loginIllustration from "@/assets/login.svg";
import { Badge } from "@/components/ui/badge";

export function LoginHero() {
  return (
    <div className="hidden rounded-3xl border border-white/70 bg-white/75 p-10 shadow-sm backdrop-blur md:flex md:flex-col md:gap-8">
      <Badge
        variant="outline"
        className="w-fit border-blue-200 bg-blue-50 text-blue-700"
      >
        Vanguard Vault
      </Badge>
      <div className="space-y-4">
        <h1 className="text-4xl leading-tight font-semibold tracking-tight text-slate-900">
          欢迎回来，继续管理你的密码库
        </h1>
        <p className="text-base leading-relaxed text-slate-600">
          输入服务地址、邮箱和主密码后，即可完成登录并自动准备好你的密码库。
        </p>
      </div>
      <img
        src={loginIllustration}
        alt="Vault login illustration"
        className="h-72 w-full object-contain"
      />
    </div>
  );
}
