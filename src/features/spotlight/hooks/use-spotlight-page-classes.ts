import { useEffect } from "react";

export function useSpotlightPageClasses(): void {
  useEffect(() => {
    const htmlClasses = ["h-full", "w-full", "overflow-hidden"];
    const bodyClasses = [
      "m-0",
      "h-full",
      "w-full",
      "overflow-hidden",
      "overscroll-none",
      "bg-transparent",
    ];

    for (const className of htmlClasses) {
      document.documentElement.classList.add(className);
    }
    for (const className of bodyClasses) {
      document.body.classList.add(className);
    }

    return () => {
      for (const className of htmlClasses) {
        document.documentElement.classList.remove(className);
      }
      for (const className of bodyClasses) {
        document.body.classList.remove(className);
      }
    };
  }, []);
}
