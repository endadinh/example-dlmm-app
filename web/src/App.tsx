import { useState, useEffect, useRef } from "react";
import PoolSelector from "./components/PoolSelector";
import SwapPanel from "./components/SwapPanel";
import QuoteResult from "./components/QuoteResult";
import { useDebounce } from "./hooks/useDebounce";
import InstructionTabs from "./components/Instruction/Control";
import SwapInstruction from "./components/Instruction/Swap";
import CreatePositionInstruction from "./components/Instruction/CreatePosition";
import ModifyPositionInstruction from "./components/Instruction/ModifierPosition";

type TokenInfo = { symbol: string; address: string; decimals: number };

export default function App() {
  const [loading, setLoading] = useState(false);
  const [pair, setPair] = useState("");
  const [pairStatus, setPairStatus] = useState("");
  const [tokens, setTokens] = useState<{ a: TokenInfo; b: TokenInfo }>({
    a: { symbol: "USDC", address: "", decimals: 6 },
    b: { symbol: "SOL", address: "", decimals: 9 },
  });

  const [isReversed, setIsReversed] = useState(false);
  const [amountIn, setAmountIn] = useState("0");
  const debouncedAmount = useDebounce(amountIn, 500); // 1s delay
  const [amountOut, setAmountOut] = useState("0");
  const [quote, setQuote] = useState<any>(null);

  const currentPairRef = useRef(pair);

  useEffect(() => {
    console.log("Updating currentPairRef to:", pair);
    currentPairRef.current = pair;
  }, [pair]);

  const fetchPair = async (address: string) => {
    setPairStatus("⏳ Loading pool...");
    try {
      const res = await fetch(`/api/pair?address=${address}`);
      const response = await res.json();
      console.log("Fetched pair data:", response);
      if (response.status === "error") throw new Error(response.message);
      const data = response.data;
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
      setPair(address);
      setPairStatus(
        `Loaded pool: ${data.token_a.symbol}-${data.token_b.symbol}`
      );
    } catch (err: any) {
      console.log("catch e ", err);
      setPairStatus(`${err.message}`);
      throw err;
      // throw new Error(err.message);
    }
  };

  const getQuote = async (value: number) => {
    if (!pair || !value) return;
    setLoading(true);
    const thisPair = pair; // snapshot current pair
    // 💡 check pair still valid
    if (currentPairRef.current !== thisPair) {
      console.warn("Stale quote ignored for old pair:", thisPair);
      return;
    }

    try {
      console.info("Fetching quote for value:", value);

      const base = isReversed ? tokens.b : tokens.a;
      const quoteToken = isReversed ? tokens.a : tokens.b;
      const res = await fetch("/api/quote", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          pair_address: pair,
          source_mint: base.address,
          destination_mint: quoteToken.address,
          amount_in: value * 10 ** base.decimals,
        }),
      });
      const response = await res.json();
      if (response.status === "error") {
        setQuote({ error: response.message });
      } else {
        const data = response.data;
        const quoteData = {
          input: base.symbol,
          output: quoteToken.symbol,
          in_amount: data.in_amount,
          out_amount: data.out_amount,
          fee_amount: data.fee_amount,
        };

        setAmountOut(String(data.out_amount / 10 ** quoteToken.decimals));
        setQuote(quoteData);
      }
    } catch {
      setQuote({ error: "Failed to fetch quote" });
    } finally {
      if (currentPairRef.current === thisPair) setLoading(false);
    }
  };

  const toggleDirection = () => {
    setIsReversed((prev) => !prev);
  };

  useEffect(() => {
    console.log("Debounced amount or pair/direction changed:", {
      debouncedAmount,
      pair: currentPairRef.current,
      isReversed,
    });
    if (debouncedAmount && pair) {
      getQuote(Number(debouncedAmount));
    }
  }, [debouncedAmount, tokens, isReversed]);

  const tokenSell = isReversed ? tokens.b : tokens.a;
  const tokenBuy = isReversed ? tokens.a : tokens.b;

  const tabs = [
    { key: "quote", label: "Quote" },
    { key: "instruction", label: "Instructions" },
    { key: "connect", label: "Connect" },
  ];

  const [active, setActive] = useState("quote");

  return (
    <div className="min-h-screen flex flex-col bg-cosmos text-white">
      <div className="mt-10 bg-gray-900/40 border border-gray-800 rounded-xl p-4 backdrop-blur-lg">
        {/* === Tab header === */}
        {/* <div className="flex gap-6 border-b border-gray-800 pb-2 mb-4">

        </div> */}
        <nav className="flex gap-6 border-b border-gray-800 pb-2 mb-4">
          <h1 className="text-xl font-bold text-neon">Saros DLMM App</h1>
          {tabs.map((t) => (
            <button
              key={t.key}
              onClick={() => setActive(t.key)}
              className={`pb-2 text-sm font-medium transition-colors ${
                active === t.key
                  ? "text-neon border-b-2 border-neon"
                  : "text-gray-400 hover:text-gray-200"
              }`}
            >
              {t.label}
            </button>
          ))}
        </nav>
        {/* <nav className="flex justify-between items-center px-8 py-4 border-b border-gray-800 bg-plasma/60 backdrop-blur-md"> */}
        {/* <button className="px-4 py-2 bg-neon text-black rounded-lg font-semibold hover:shadow-glow transition">
            Connect
          </button> */}
        {/* </nav> */}
      </div>

      <main className="flex-grow flex items-center justify-center p-6">
        {active === "quote" && (
          <div className="box-middle">
            <h2 className="inst-title">Quote</h2>

            <PoolSelector
              pair={pair}
              setPair={setPair}
              fetchPair={fetchPair}
              status={pairStatus}
            />

            <SwapPanel
              tokenSell={tokenSell}
              tokenBuy={tokenBuy}
              amountIn={amountIn}
              setAmountIn={setAmountIn}
              amountOut={amountOut}
              toggleDirection={toggleDirection}
              isReversed={isReversed}
            />

            {quote != null && <QuoteResult quote={quote} loading={loading} />}
          </div>
        )}
        {active === "instruction" && <InstructionTabs />}
        {/* {active === "connect" && <ConnectInstruction />} */}
      </main>

      <footer className="text-center py-6 text-gray-500 text-xs">
        Built with 🦀 Rust • Powered by{" "}
        <span className="text-neon">Saros DLMM SDK</span>
      </footer>
    </div>
  );
}
