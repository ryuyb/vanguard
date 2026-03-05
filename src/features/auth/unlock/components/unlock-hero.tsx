import lockedIllustration from "@/assets/locked.svg";
import { Badge } from "@/components/ui/badge";

export function UnlockHero() {
  return (
    <div className="hidden rounded-3xl border border-white/80 bg-white/70 p-8 shadow-sm backdrop-blur md:flex md:flex-col md:gap-6">
      <Badge
        variant="outline"
        className="w-fit border-blue-200 bg-blue-50 text-blue-700"
      >
        Vault Unlock
      </Badge>
      <h1 className="text-3xl leading-tight font-semibold tracking-tight text-slate-900">
        会话已锁定，请输入主密码解锁
      </h1>
      <p className="text-sm leading-relaxed text-slate-600">
        输入主密码后即可解锁，继续安全访问你的密码库。
      </p>
      <img
        src={lockedIllustration}
        alt="Vault locked illustration"
        className="h-64 w-full object-contain"
      />
    </div>
  );
}
