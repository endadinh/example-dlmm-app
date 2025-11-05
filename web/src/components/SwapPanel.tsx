import React, { useState, useEffect } from "react";

export default function SwapPanel({ pair, onQuote }: { pair: string; onQuote: (q: any) => void }) {
  const [inputMint, setInputMint] = useState<string>("");
  const [outputMint, setOutputMint] = useState<string>("");
  const [amount, setAmount] = useState<string>("");

  async function fetchQuote() {
    if (!pair || !inputMint || !amount) return;
    const body = { pair_address: pair, source_mint: inputMint, amount_in: parseFloat(amount) };

    const res = await fetch("/api/quote", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(body),
    });
    const data = await res.json();
    onQuote(data);
  }

  return (
    <div className="card space-y-4">
      <div className="flex justify-between items-center">
        <label>Amount In</label>
        <input
          placeholder="Enter amount"
          value={amount}
          onChange={(e) => setAmount(e.target.value)}
          onBlur={fetchQuote}
        />
      </div>
      <button onClick={fetchQuote}>Get Quote</button>
    </div>
  );
}