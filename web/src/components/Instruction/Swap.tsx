export default function SwapInstruction() {
  const swapAccounts = [
    { name: "User", address: "Your connected wallet" },
    { name: "Pool", address: "Loaded pool pubkey" },
    { name: "Token A", address: "Mint of source token" },
    { name: "Token B", address: "Mint of destination token" },
    { name: "Vault A", address: "Token A vault" },
    { name: "Vault B", address: "Token B vault" },
  ];

  return (
    <div className="swap-card">
      <div className="space-y-3 text-sm text-gray-300">
        <h3 className="font-semibold text-gray-200 mb-2">Swap Instruction</h3>

        <div className="grid grid-cols-1 sm:grid-cols-2 gap-2">
          {swapAccounts.map((acc) => (
            <div
              key={acc.name}
              className="bg-gray-800/40 border border-gray-700/60 rounded-lg p-3 flex flex-col"
            >
              <span className="text-xs text-gray-400">{acc.name}</span>
              <span className="font-mono text-cyan-300 text-xs truncate">
                {acc.address}
              </span>
            </div>
          ))}
        </div>

        <div className="mt-4 text-gray-400 text-xs italic">
          💡 Swap requires user account, pool state, token mints & vaults.
        </div>
      </div>
    </div>
  );
}
