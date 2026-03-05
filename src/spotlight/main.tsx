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
      className="grid h-full w-full place-items-center overflow-hidden bg-white/[0.004] px-4 py-6 sm:px-8 sm:py-10 md:px-16 md:py-12"
    >
      <Card
        id="spotlight-card"
        className="block w-full max-w-180 gap-0 rounded-4xl border border-white/40 bg-linear-to-b from-[rgba(250,251,255,0.82)] to-[rgba(243,246,252,0.74)] p-3.5 pb-0 text-slate-900 ring-0 shadow-[0_36px_70px_rgba(15,23,42,0.36),inset_0_1px_0_rgba(255,255,255,0.42)] backdrop-blur-[20px] backdrop-saturate-[1.8]"
      >
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

        <Separator className="mt-2.5 bg-slate-400/30" />

        <CardFooter className="mt-0 flex min-h-9 flex-wrap items-center gap-2.5 px-1 pb-0">
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

ReactDOM.createRoot(spotlightRoot).render(
  <StrictMode>
    <SpotlightApp />
  </StrictMode>,
);
