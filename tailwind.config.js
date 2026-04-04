/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{js,ts,jsx,tsx}"],
  theme: {
    extend: {
      colors: {
        island: {
          bg: "#0a0a0a",
          surface: "#1e1e1e",
          border: "#2c2c30",
          hover: "#333338",
          text: "#ffffff",
          muted: "#44444a",
          accent: "#06b6d4",
          green: "#22c55e",
          "green-bright": "#27c93f",
          amber: "#ffbd2e",
          red: "#ff5f56",
          orange: "#f97316",
          purple: "#c084fc",
          blue: "#3b82f6",
          // Vibe Island state colors (from original CSS vars)
          idle: "#22c55e",       // --vi-idle
          work: "#3b82f6",       // --vi-work
          alert: "#f97316",      // --vi-alert
          question: "#c084fc",   // --vi-question
          explore: "#06b6d4",    // --vi-explore
        },
      },
      borderRadius: {
        island: "24px",
      },
      animation: {
        "pulse-slow": "pulse 3s cubic-bezier(0.4, 0, 0.6, 1) infinite",
        "slide-down": "slideDown 0.3s ease-out",
        "slide-up": "slideUp 0.2s ease-in",
        "expand": "expand 0.3s ease-out",
      },
      keyframes: {
        slideDown: {
          "0%": { opacity: "0", transform: "translateY(-10px) scaleY(0.95)" },
          "100%": { opacity: "1", transform: "translateY(0) scaleY(1)" },
        },
        slideUp: {
          "0%": { opacity: "1", transform: "translateY(0)" },
          "100%": { opacity: "0", transform: "translateY(-10px)" },
        },
        expand: {
          "0%": { maxHeight: "48px", borderRadius: "24px" },
          "100%": { maxHeight: "400px", borderRadius: "20px" },
        },
      },
    },
  },
  plugins: [],
};
