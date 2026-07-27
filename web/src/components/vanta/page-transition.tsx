"use client";

import { motion, AnimatePresence } from "framer-motion";
import type { ReactNode } from "react";

/**
 * PageTransition — wraps view content with framer-motion entrance/exit.
 * Uses a subtle slide+fade that respects the manga aesthetic.
 * Exit animations require AnimatePresence with mode="wait" in the parent.
 */
export function PageTransition({
  viewKey,
  children,
}: {
  viewKey: string;
  children: ReactNode;
}) {
  return (
    <AnimatePresence mode="wait">
      <motion.div
        key={viewKey}
        initial={{ opacity: 0, y: 12 }}
        animate={{ opacity: 1, y: 0 }}
        exit={{ opacity: 0, y: -8 }}
        transition={{
          duration: 0.28,
          ease: [0.2, 0.8, 0.2, 1],
        }}
      >
        {children}
      </motion.div>
    </AnimatePresence>
  );
}
