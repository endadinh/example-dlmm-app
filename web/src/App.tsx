import React, { useState, useEffect } from "react";

type TokenInfo = {
  symbol: string;
  address: string;
  decimals: number;
};

export default function App() {
  const [pair, setPair] = useState("");
  const [pairStatus, setPairStatus] = useState("");
  const [pairTimer, setPairTimer] = useState<NodeJS.Timeout | null>(null);
  const [tokens, setTokens] = useState<{ a: TokenInfo; b: TokenInfo }>({
    a: {
      symbol: "USDC",
      address: "",
      decimals: 6,
    },
    b: {
      symbol: "SOL",
      address: "",
      decimals: 9,
    },
  });
  const [isReversed, setIsReversed] = useState(false);
  const [amountIn, setAmountIn] = useState("");
  const [quote, setQuote] = useState<any>(null);
  const [debounceTimer, setDebounceTimer] = useState<NodeJS.Timeout | null>(
    null
  );

  const fetchPair = async (address: string) => {
    setPairStatus("⏳ Loading pool...");
    try {
      const res = await fetch(`/api/pair?address=${address}`);
      const data = await res.json();
      console.log("pair response", data);
      if (data.error) throw new Error("Failed to load pair: " + address);
      setTokens({
        a: {
          symbol: data.token_a.symbol,
          address: data.token_a.mint,
          decimals: data.token_a.decimals,
        },
        b: {
          symbol: data.token_b.symbol,
          address: data.token_b.mint,
          decimals: data.token_b.decimals,
        },
      });
      setPairStatus(
        `✅ Loaded pool: ${data.token_a.symbol}-${data.token_b.symbol}`
      );
    } catch (err: any) {
      setPairStatus(`❌ ${err.message}`);
    }
  };

  const getQuote = async (value: string) => {
    if (!pair || !value) return;
    try {
      const base = isReversed ? tokens.b : tokens.a;
      console.info("fetching quote", base);
      const res = await fetch("/api/quote", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          pair_address: pair,
          source_mint: base.address,
          amount_in: parseFloat(value) * 10 ** base.decimals,
        }),
      });
      const data = await res.json();
      setQuote(data);
    } catch {
      setQuote({ error: "❌ Failed to fetch quote" });
    }
  };

  // Debounce auto-quote
  useEffect(() => {
    if (!amountIn) return;
    if (debounceTimer) clearTimeout(debounceTimer);
    const timer = setTimeout(() => getQuote(amountIn), 500);
    setDebounceTimer(timer);
    return () => clearTimeout(timer);
  }, [amountIn, isReversed, pair]);

  const toggleDirection = () => {
    setIsReversed((prev) => !prev);
    if (quote?.amount_out) {
      setAmountIn(String(quote.amount_out));
    }

    // Delay 400–500ms cho animation swap xong rồi mới quote lại
    setTimeout(() => {
      if (pair && amountIn) {
        getQuote(amountIn);
      }
    }, 500);
  };

  const tokenSell = isReversed ? tokens.b : tokens.a;
  const tokenBuy = isReversed ? tokens.a : tokens.b;

  return (
    <div className="min-h-screen flex flex-col bg-cosmos text-white">
      <nav className="flex justify-between items-center px-8 py-4 border-b border-gray-800 bg-plasma/60 backdrop-blur-md">
        <h1 className="text-xl font-bold text-neon">Saros DLMM SDK</h1>
        <div className="flex gap-6 text-gray-400 text-sm">
          <a href="#" className="hover:text-neon transition">
            Pool
          </a>
          <a href="#" className="hover:text-neon transition">
            Docs
          </a>
          <a href="#" className="hover:text-neon transition">
            Support
          </a>
        </div>
        <button className="px-4 py-2 bg-neon text-black rounded-lg font-semibold hover:shadow-glow transition">
          Connect
        </button>
      </nav>

      <main className="flex-grow flex items-center justify-center p-6">
        <div className="swap-card">
          <h2 className="text-lg font-semibold mb-6 text-center text-gray-300">
            Swap & Quote Simulator
          </h2>

          {/* Pool Address */}
          <div className="space-y-2 mb-8">
            <label className="text-sm text-gray-400">Pool Address</label>
            <input
              type="text"
              placeholder="Enter pool pubkey (e.g. 7uLoV...xYVb)"
              className="input-box"
              value={pair}
              onChange={(e) => {
                setPair(e.target.value);
                if (pairTimer) clearTimeout(pairTimer);
                const timer = setTimeout(() => {
                  if (e.target.value.trim()) fetchPair(e.target.value.trim());
                }, 1500); // ⏳ 1.5s delay
                setPairTimer(timer);
              }}
            />
            <p className="text-xs text-gray-500">{pairStatus}</p>
          </div>

          {/* === Swap Section === */}
          <div className="w-full overflow-hidden">
            <div
              className={`transition-transform duration-500 ease-out ${
                isReversed ? "-translate-y-[5.5rem]" : "translate-y-0"
              }`}
            >
              {/* Sell box */}
              <div className="mb-12">
                <label className="block text-sm text-gray-400 mb-2">
                  You Sell
                </label>
                <div className="input-row transition-all duration-500">
                  <input
                    type="number"
                    value={amountIn}
                    onChange={(e) => setAmountIn(e.target.value)}
                    placeholder="0.0"
                  />
                  <div className="token-display">{tokenSell.symbol}</div>
                </div>
              </div>
              <div className="">
                <button
                  onClick={() => {
                    toggleDirection();
                  }}
                  className={`swap-toggle group
                ${isReversed ? "rotate-180 scale-110" : "rotate-0"}`}
                >
                  <span className="z-10 text-cyan-300 text-xl font-bold drop-shadow-[0_0_6px_#00f5ff] select-none margin-auto block">
                    ⇅
                  </span>
                </button>
              </div>

              {/* Buy box */}
              <div className="mt-12">
                <label className="block text-sm text-gray-400 mb-2">
                  You Get
                </label>
                <div className="input-row transition-all duration-500">
                  <input
                    type="number"
                    value={quote?.amount_out || ""}
                    readOnly
                    placeholder="0.0"
                  />
                  <div className="token-display">{tokenBuy.symbol}</div>
                </div>
              </div>
            </div>
          </div>

          {/* Quote preview */}
          {quote && (
            <pre className="mt-8 text-xs bg-gray-900/60 p-3 rounded-lg border border-gray-800 font-mono whitespace-pre-wrap">
              {JSON.stringify(quote, null, 2)}
            </pre>
          )}
        </div>
      </main>

      <footer className="text-center py-6 text-gray-500 text-xs">
        Built with 🦀 Rust • Powered by{" "}
        <span className="text-neon">Saros DLMM SDK</span>
      </footer>
    </div>
  );
}
