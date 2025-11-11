type Props = {
  tokenSell: any;
  tokenBuy: any;
  amountIn: string;
  setAmountIn: (v: string) => void;
  amountOut: string;
  toggleDirection: () => void;
  isReversed: boolean;
};

export default function SwapPanel({
  tokenSell,
  tokenBuy,
  amountIn,
  setAmountIn,
  amountOut,
  toggleDirection,
  isReversed,
}: Props) {
  console.log("Rendering SwapPanel with:");
  return (
    <div className="w-full overflow-hidden">
      <div
        className={`transition-transform duration-500 ease-out ${
          isReversed ? "-translate-y-[5.5rem]" : "translate-y-0"
        }`}
      >
        {/* Sell */}
        <div className="mb-12">
          <label className="block text-sm text-gray-400 mb-2">You Sell</label>
          <div className="input-row">
            <input
              type="number"
              value={amountIn}
              onChange={(e) => setAmountIn(e.target.value)}
              placeholder="0.0"
            />
            <div className="token-display">{tokenSell.symbol}</div>
          </div>
        </div>

        {/* Toggle */}
        <div className="text-center">
          <button
            onClick={toggleDirection}
            className={`swap-toggle group transition-transform duration-500 ${
              isReversed ? "rotate-180 scale-110" : "rotate-0"
            }`}
          >
            <span className="z-10 text-cyan-300 text-xl font-bold drop-shadow-[0_0_6px_#00f5ff]">
              ⇅
            </span>
          </button>
        </div>

        {/* Buy */}
        <div className="mt-12">
          <label className="block text-sm text-gray-400 mb-2">You Get</label>
          <div className="input-row">
            <input
              type="number"
              value={amountOut || ""}
              readOnly
              placeholder="0.0"
            />
            <div className="token-display">{tokenBuy.symbol}</div>
          </div>
        </div>
      </div>
    </div>
  );
}
