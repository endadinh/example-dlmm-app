/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  safelist: [
    "swap-card",
    "btn-neon",
    "token-btn",
    "input-box",
    "swap-arrow"
  ],
  theme: {
    extend: {
      colors: {
        cosmos: "#0a0b0f",
        plasma: "#13151c",
        neon: "#00f5ff",
      },
      boxShadow: {
        glow: "0 0 15px 2px rgba(0, 245, 255, 0.2)",
      },
      fontFamily: {
        sans: ["Inter", "system-ui", "sans-serif"],
      },
    },
  },
  plugins: [],
};