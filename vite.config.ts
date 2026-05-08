import { defineConfig } from "vite";
import preact from "@preact/preset-vite";

export default defineConfig({
  plugins: [preact()],
  // Vite dev server port must match tauri.conf.json devUrl
  server: {
    port: 5173,
    strictPort: true,
  },
  // Serve data/icons/ as /icons/ — provides a static fallback when the Tauri
  // IPC is unavailable (plain browser dev) and matches the Rust icons_dir() path.
  publicDir: "data",
  build: {
    outDir: "dist",
    emptyOutDir: true,
  },
});
