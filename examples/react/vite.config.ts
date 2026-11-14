import react from "@vitejs/plugin-react";
import alien from "alien-vite";
import { defineConfig } from "vite";

export default defineConfig({
  plugins: [alien(), react()],
});
