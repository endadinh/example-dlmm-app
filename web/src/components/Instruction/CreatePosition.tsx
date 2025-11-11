export default function CreatePositionInstruction() {
  return (
    <div className="space-y-3 text-sm text-gray-300">
      <h3 className="font-semibold text-gray-200 mb-2">Create Position Instruction</h3>
      <p className="text-gray-400 text-xs">
        Used when initializing a new liquidity position with a range of bins.
      </p>
      <ul className="list-disc ml-4 text-xs text-gray-400">
        <li>User Account</li>
        <li>Pool</li>
        <li>Position PDA</li>
        <li>Bin arrays (lower / middle / upper)</li>
        <li>Vaults for both tokens</li>
      </ul>
    </div>
  );
}