/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{js,ts,jsx,tsx}"],
  theme: {
    extend: {
      colors: {
        island: {
          bg: "#1a1a1a",
          surface: "#2a2a2a",
          border: "#3a3a3a",
          text: "#e0e0e0",
          muted: "#888888",
          accent: "#00D4FF",
          green: "#34D399",
          amber: "#FBBF24",
          red: "#F87171",
          purple: "#A78BFA",
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
