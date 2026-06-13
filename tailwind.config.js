/** @type {import('tailwindcss').Config} */
module.exports = {
  mode: "all",
  content: ["./src/**/*.{rs,html}", "./dist/**/*.html"],
  theme: {
    extend: {
      colors: {
        base: "#080b10",
        surface: "#0d131b",
        panel: "#111a24",
        elevated: "#16212d",
        line: "#1e2b39",
        ink: "#e7eef6",
        muted: "#8a9bb0",
        faint: "#5a6b7e",
        aurora: {
          green: "#5ef2a6",
          teal: "#34d3e0",
          violet: "#a78bfa",
        },
        tier: {
          low: "#43e08a",
          mid: "#f5c451",
          high: "#fb7185",
        },
      },
      fontFamily: {
        display: ['"Schibsted Grotesk"', "system-ui", "sans-serif"],
        sans: ['"Schibsted Grotesk"', "system-ui", "sans-serif"],
        mono: ['"IBM Plex Mono"', "ui-monospace", "SFMono-Regular", "monospace"],
      },
      letterSpacing: {
        label: "0.18em",
      },
      boxShadow: {
        glow: "0 0 0 1px rgba(94,242,166,0.25), 0 0 32px -8px rgba(94,242,166,0.35)",
        panel: "0 1px 0 0 rgba(255,255,255,0.03) inset, 0 24px 48px -24px rgba(0,0,0,0.8)",
      },
      keyframes: {
        "fade-up": {
          "0%": { opacity: "0", transform: "translateY(10px)" },
          "100%": { opacity: "1", transform: "translateY(0)" },
        },
        shimmer: {
          "100%": { transform: "translateX(100%)" },
        },
        "glow-pulse": {
          "0%, 100%": { opacity: "0.5" },
          "50%": { opacity: "1" },
        },
      },
      animation: {
        "fade-up": "fade-up 0.55s cubic-bezier(0.22,1,0.36,1) both",
        "glow-pulse": "glow-pulse 4.5s ease-in-out infinite",
      },
    },
  },
  plugins: [],
};
