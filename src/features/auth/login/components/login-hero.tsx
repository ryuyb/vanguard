import { useTranslation } from "react-i18next";
import loginIllustration from "@/assets/login.svg";
import { Badge } from "@/components/ui/badge";

export function LoginHero() {
  const { t } = useTranslation();

  return (
    <div className="hidden rounded-3xl border border-white/70 bg-white/75 p-10 shadow-sm backdrop-blur md:flex md:flex-col md:gap-8">
      <Badge
        variant="outline"
        className="w-fit border-blue-200 bg-blue-50 text-blue-700"
      >
        {t("auth.login.hero.badge")}
      </Badge>
      <div className="space-y-4">
        <h1 className="text-4xl leading-tight font-semibold tracking-tight text-slate-900">
          {t("auth.login.hero.title")}
        </h1>
        <p className="text-base leading-relaxed text-slate-600">
          {t("auth.login.hero.description")}
        </p>
      </div>
      <img
        src={loginIllustration}
        alt={t("auth.login.hero.illustrationAlt")}
        className="h-72 w-full object-contain"
      />
    </div>
  );
}
