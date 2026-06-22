import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import { VitePWA } from "vite-plugin-pwa";

export default defineConfig(async () => ({
  plugins: [
    react(),
    VitePWA({
      registerType: "autoUpdate",
      includeAssets: ["icons/*.png", "favicon.ico"],
      manifest: {
        name: "Morn",
        short_name: "Morn",
        description: "Desktop AI Operating System",
        theme_color: "#1a1a2e",
        background_color: "#1a1a2e",
        display: "standalone",
        orientation: "any",
        start_url: "/",
        scope: "/",
        icons: [
          { src: "/icons/icon-192x192.png", sizes: "192x192", type: "image/png" },
          { src: "/icons/icon-512x512.png", sizes: "512x512", type: "image/png" },
          { src: "/icons/icon-512x512.png", sizes: "512x512", type: "image/png", purpose: "maskable" },
        ],
      },
      workbox: {
        globPatterns: ["**/*.{js,css,html,ico,png,svg,json}"],
        runtimeCaching: [
          {
            urlPattern: /^https?:\/\/.*\/api\/.*/i,
            handler: "NetworkFirst",
            options: {
              cacheName: "api-cache",
              expiration: { maxEntries: 50, maxAgeSeconds: 60 * 60 * 24 },
              networkTimeoutSeconds: 10,
            },
          },
        ],
      },
    }),
  ],
  clearScreen: false,
  server: {
    port: 5173,
    strictPort: true,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
}));