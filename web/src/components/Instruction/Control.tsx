import { useState } from "react";
import SwapInstruction from "./Swap";
import CreatePositionInstruction from "./CreatePosition";
import ModifyPositionInstruction from "./ModifierPosition";

const tabs = [
  { key: "swap", label: "Swap" },
  { key: "create", label: "Create Position" },
  { key: "modify", label: "Modify Position" },
];

export default function InstructionTabs() {
  const [active, setActive] = useState("swap");

  return (
    <div className="box-middle mt-10">
      <h2 className="inst-title">Instruction Details</h2>

      {/* === Tabs header === */}
      <div className="inst-bar">
        {tabs.map((t) => (
          <button
            key={t.key}
            onClick={() => setActive(t.key)}
            className={`inst-btn ${active === t.key ? "inst-active" : ""}`}
          >
            {t.label}
          </button>
        ))}
      </div>

      {/* === Content === */}
      <div className="inst-content">
        {active === "swap" && <SwapInstruction />}
        {active === "create" && <CreatePositionInstruction />}
        {active === "modify" && <ModifyPositionInstruction />}
      </div>
    </div>
  );
}
