import { useState } from "react";
import { RotateCw } from "lucide-react"; // optional icon lib

type Props = {
  pair: string;
  setPair: (v: string) => void;
  fetchPair: (address: string) => Promise<void>;
  status: string;
};

export default function PoolSelector({
  pair,
  setPair,
  fetchPair,
  status,
}: Props) {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState(false);

  const handleFetch = async (addr: string) => {
    if (!addr.trim()) return;
    setLoading(true);
    setError(false);
    try {
      await fetchPair(addr.trim());
    } catch (err: any) {
      setError(true);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="space-y-2 mb-8">
      <label className="text-sm text-gray-400">Pool Address</label>

      {/* Input */}
      <div className="flex gap-2">
        <input
          type="text"
          placeholder="Enter pool pubkey (e.g. 7uLoV...xYVb)"
          className="input-box flex-1"
          value={pair}
          onChange={(e) => {
            const v = e.target.value;
            setPair(v);
            if (v.trim()) {
              const timer = setTimeout(() => handleFetch(v.trim()), 1200);
              return () => clearTimeout(timer);
            }
          }}
        />

        {/* Retry button only when error */}
        {error && (
          <button
            onClick={() => handleFetch(pair)}
            className="p-2 text-red-400 hover:text-red-200 transition"
            title="Retry"
          >
            <RotateCw className="w-4 h-4 animate-spin-once" />
          </button>
        )}
      </div>

      {/* Status messages */}
      {loading && (
        <div className="flex items-center gap-2 text-sm text-cyan-300">
          <div className="w-3 h-3 border-2 border-t-transparent border-cyan-400 rounded-full animate-spin" />
          <span>Loading pool...</span>
        </div>
      )}

      {!loading && !error && status && (
        <div className="text-sm text-green-400 flex items-center gap-1">
          <span>✅</span>
          <span>{status}</span>
        </div>
      )}

      {!loading && error && (
        <div className="text-sm text-red-400 flex items-center gap-2">
          <span>❌</span>
          <span>{status}</span>
        </div>
      )}
    </div>
  );
}
