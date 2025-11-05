import React, { useState, useEffect } from "react";
import { useDebounce } from "../hooks/useDebounce";

export default function PoolSelector({ onPairLoaded }: { onPairLoaded: (pair: string) => void }) {
  const [input, setInput] = useState("");
  const [status, setStatus] = useState("");
  const debounced = useDebounce(input, 1000);

  useEffect(() => {
    if (debounced.length < 10) return;
    fetchPair(debounced);
  }, [debounced]);

  async function fetchPair(address: string) {
    setStatus("Loading...");
    try {
      const res = await fetch(`/api/pair?address=${address}`);
      const data = await res.json();
      if (data.error) {
        setStatus(`❌ ${data.error}`);
        return;
      }
      setStatus(`✅ Loaded: ${data.token_a.symbol}-${data.token_b.symbol}`);
      onPairLoaded(address);
    } catch (e) {
      setStatus("❌ Failed to load pair");
    }
  }

  return (
    <div className="card space-y-2">
      <label>Pool Address</label>
      <input
        value={input}
        onChange={(e) => setInput(e.target.value)}
        placeholder="Enter pair pubkey"
      />
      <div className="text-sm text-gray-400">{status}</div>
    </div>
  );
}