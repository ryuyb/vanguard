import { motion } from "motion/react";
import { StrictMode } from "react";
import ReactDOM from "react-dom/client";
import { Card, CardFooter } from "@/components/ui/card";
import { Command } from "@/components/ui/command";
import { Separator } from "@/components/ui/separator";
import {
  SpotlightFooterHints,
  SpotlightResultsPanel,
  SpotlightSearchInput,
  useSpotlightCopyAction,
  useSpotlightDetailActions,
  useSpotlightHideOnOutsideClick,
  useSpotlightHotkeys,
  useSpotlightPageClasses,
  useSpotlightSession,
  useSpotlightViewModel,
} from "@/features/spotlight";
import { AppLocaleProvider, initializeAppI18n } from "@/i18n";
import "@/main.css";

function SpotlightApp() {
  useSpotlightPageClasses();

  const { hideSpotlight, isLoadingVault, vaultItems } = useSpotlightSession();

  const { copiedDetailField, copiedItemId, runCopyAction } =
    useSpotlightCopyAction({
      hideSpotlight,
    });

  const {
    detailItem,
    hasQuery,
    hasVisibleResults,
    query,
    setDetailItemId,
    setQuery,
    shouldShowResults,
    visibleItems,
  } = useSpotlightViewModel({
    isLoadingVault,
    vaultItems,
  });

  const {
    detailActionIndex,
    detailActions,
    detailHasTotp,
    setDetailActionIndex,
  } = useSpotlightDetailActions({ detailItem });

  const { onCommandInputKeyDown } = useSpotlightHotkeys({
    detailActionIndex,
    detailActions,
    detailHasTotp,
    detailItem,
    hasVisibleResults,
    hideSpotlight,
    query,
    runCopyAction,
    setDetailActionIndex,
    setDetailItemId,
    setQuery,
    visibleItems,
  });

  useSpotlightHideOnOutsideClick({
    cardId: "spotlight-card",
    onOutsideClick: () => {
      void hideSpotlight();
    },
  });

  return (
    <motion.main
      initial={{ opacity: 0, y: 8, scale: 0.985 }}
      animate={{ opacity: 1, y: 0, scale: 1 }}
      transition={{ duration: 0.18, ease: "easeOut" }}
      className="flex h-full w-full items-start justify-center overflow-hidden bg-transparent px-4 pt-20 sm:px-8 sm:pt-24 md:px-16 md:pt-28"
    >
      <Card
        id="spotlight-card"
        className="block w-full max-w-180 gap-0 overflow-hidden rounded-2xl border border-slate-200/40 bg-slate-100 p-0 text-slate-900 shadow-[0_20px_60px_-10px_rgba(0,0,0,0.4),0_10px_30px_-5px_rgba(0,0,0,0.25),0_0_0_1px_rgba(0,0,0,0.08)]"
      >
        <div className="bg-slate-200/60 p-4">
          <Command
            className="rounded-none! bg-transparent p-0! text-inherit"
            shouldFilter={false}
            loop
          >
            <div>
              <SpotlightSearchInput
                query={query}
                onQueryChange={setQuery}
                onKeyDown={onCommandInputKeyDown}
              />
            </div>
            <SpotlightResultsPanel
              shouldShowResults={shouldShowResults}
              isLoadingVault={isLoadingVault}
              hasQuery={hasQuery}
              detailItem={detailItem}
              detailActions={detailActions}
              detailActionIndex={detailActionIndex}
              copiedDetailField={copiedDetailField}
              visibleItems={visibleItems}
              copiedItemId={copiedItemId}
            />
          </Command>
        </div>

        <Separator className="bg-slate-300/60" />

        <CardFooter className="flex min-h-9 flex-wrap items-center gap-2.5 bg-slate-50 px-4 py-3">
          <SpotlightFooterHints
            hasVisibleResults={hasVisibleResults}
            detailItem={detailItem}
          />
        </CardFooter>
      </Card>
    </motion.main>
  );
}

const spotlightRoot = document.getElementById("spotlight-root") as HTMLElement;
spotlightRoot.classList.add("h-full", "w-full", "overflow-hidden");

async function bootstrap() {
  await initializeAppI18n();

  ReactDOM.createRoot(spotlightRoot).render(
    <StrictMode>
      <AppLocaleProvider>
        <SpotlightApp />
      </AppLocaleProvider>
    </StrictMode>,
  );
}

void bootstrap();
