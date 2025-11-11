export default function ModifyPositionInstruction() {
  return (
    <div className="space-y-3 text-sm text-gray-300">
      <h3 className="font-semibold text-gray-200 mb-2">Modify Position Instruction</h3>
      <p className="text-gray-400 text-xs">
        Allows increasing or decreasing liquidity of an existing position.
      </p>
      <ul className="list-disc ml-4 text-xs text-gray-400">
        <li>Position Account</li>
        <li>Pool</li>
        <li>User Authority</li>
        <li>Bin array references</li>
      </ul>
    </div>
  );
}