import { useTranslation } from "react-i18next";
import lockedIllustration from "@/assets/locked.svg";
import { Badge } from "@/components/ui/badge";

export function UnlockHero() {
  const { t } = useTranslation();

  return (
    <div className="hidden rounded-3xl border border-white/80 bg-white/70 p-8 shadow-sm backdrop-blur md:flex md:flex-col md:gap-6">
      <Badge
        variant="outline"
        className="w-fit border-blue-200 bg-blue-50 text-blue-700"
      >
        {t("auth.unlock.hero.badge")}
      </Badge>
      <h1 className="text-3xl leading-tight font-semibold tracking-tight text-slate-900">
        {t("auth.unlock.hero.title")}
      </h1>
      <p className="text-sm leading-relaxed text-slate-600">
        {t("auth.unlock.hero.description")}
      </p>
      <img
        src={lockedIllustration}
        alt={t("auth.unlock.hero.illustrationAlt")}
        className="h-64 w-full object-contain"
      />
    </div>
  );
}
