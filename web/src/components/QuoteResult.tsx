import React from "react";

export default function QuoteResult({ data }: { data: any }) {
  if (!data) return null;
  return (
    <div className="card">
      <p>💰 {data.amount_in} → {data.amount_out}</p>
      <p>📈 Price: {data.price ?? "?"}</p>
      <p>💸 Fee: {(data.fee * 100).toFixed(2)}%</p>
    </div>
  );
}