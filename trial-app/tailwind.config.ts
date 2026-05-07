import type { Config } from "tailwindcss";

const config: Config = {
  content: [
    "./app/**/*.{ts,tsx,js,jsx,mdx}",
    "./lib/**/*.{ts,tsx,js,jsx,mdx}",
  ],
  theme: {
    extend: {
      colors: {
        background: "var(--background)",
        foreground: "var(--foreground)",
      },
      keyframes: {
        cardEnter: {
          "0%": {
            opacity: "0",
            transform: "translateY(8px) scale(0.96)",
          },
          "100%": {
            opacity: "1",
            transform: "translateY(0) scale(1)",
          },
        },
      },
      animation: {
        "card-enter": "cardEnter 300ms ease-out",
      },
    },
  },
  plugins: [],
};

export default config;
