import preact from "@preact/preset-vite";
import tailwindcss from "@tailwindcss/vite";
import { defineConfig } from "vite";

export default defineConfig({
  plugins: [preact(), tailwindcss()],
  resolve: {
    alias: [
      { find: "react/jsx-runtime", replacement: "preact/jsx-runtime" },
      { find: "react/jsx-dev-runtime", replacement: "preact/jsx-runtime" },
      { find: "react-dom/test-utils", replacement: "preact/test-utils" },
      { find: "react-dom/client", replacement: "preact/compat/client" },
      { find: "react-dom", replacement: "preact/compat" },
      { find: "react", replacement: "preact/compat" },
    ],
  },
  build: {
    outDir: "dist",
  },
  server: {
    port: 5173,
    strictPort: true,
    proxy: {
      "/api": {
        target: process.env.API_PROXY_TARGET ?? "http://127.0.0.1:8080",
        changeOrigin: false,
      },
    },
  },
  preview: {
    port: 5173,
    strictPort: true,
  },
});
