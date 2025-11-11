import { motion, AnimatePresence } from "framer-motion";

type Props = {
  quote: any;
  loading?: boolean;
};

export default function QuoteResult({ quote, loading }: Props) {
  return (
    <div className="mt-8 min-h-[6rem]">
      {/* Loading spinner */}
      <AnimatePresence mode="wait">
        {loading && (
          <motion.div
            key="loading"
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            className="flex items-center justify-center py-6"
          >
            <div className="w-6 h-6 border-2 border-t-transparent border-neon rounded-full animate-spin" />
            <span className="ml-3 text-neon text-sm font-mono">
              Fetching quote...
            </span>
          </motion.div>
        )}

        {/* Error state */}
        {!loading && quote?.error && (
          <motion.div
            key="error"
            initial={{ opacity: 0, y: -8 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0 }}
            className="mt-6 p-4 bg-gradient-to-r from-red-900/60 to-red-700/30 border border-red-600/60 text-red-300 rounded-lg text-sm font-mono"
          >
            <div className="flex items-center gap-2">
              <span className="text-red-400">⚠️</span>
              <span>{quote.error}</span>
            </div>
          </motion.div>
        )}

        {/* Quote result */}
        {!loading && quote && !quote.error && (
          <motion.div
            key="quote"
            initial={{ opacity: 0, y: 8 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0 }}
            transition={{ duration: 0.25 }}
            className="mt-6 bg-gray-900/60 p-4 rounded-lg border border-gray-800 font-mono text-sm text-gray-300"
          >
            <div>
              <span className="text-gray-500">Input:</span>{" "}
              <span className="text-cyan-300">
                {quote.input} — {quote.in_amount}
              </span>
            </div>
            <div>
              <span className="text-gray-500">Output:</span>{" "}
              <span className="text-green-300">
                {quote.output} — {quote.out_amount}
              </span>
            </div>
            <div>
              <span className="text-gray-500">Fee:</span>{" "}
              <span className="text-yellow-300">{quote.fee_amount}</span>
            </div>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}
