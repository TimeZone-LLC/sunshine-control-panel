import { resolve, join } from 'path'
import vuePlugin from '@vitejs/plugin-vue'
import tailwindcss from '@tailwindcss/vite'
import { defineConfig } from 'vite'
import { fileURLToPath } from 'node:url'
import { dirname } from 'node:path'

const __filename = fileURLToPath(import.meta.url)
const __dirname = dirname(__filename)

const rendererSrcPath = join(__dirname, 'src', 'renderer')

/**
 * https://vitejs.dev/config
 */
export default defineConfig(() => ({
  root: rendererSrcPath,
  publicDir: 'public',
  server: {
    port: 8080,
    strictPort: true,
    host: '127.0.0.1',
    // HMR: connect the WebSocket straight to the Vite server.
    hmr: {
      protocol: 'ws',
      host: '127.0.0.1',
      port: 8080,
    },
  },
  envPrefix: ['VITE_', 'TAURI_'],
  open: false,
  build: {
    outDir: join(__dirname, 'dist'),
    emptyOutDir: true,
    target: ['es2021', 'chrome100', 'safari15'],
    minify: !process.env.TAURI_DEBUG ? 'esbuild' : false,
    sourcemap: !!process.env.TAURI_DEBUG,
    rollupOptions: {
      input: {
        app: resolve(rendererSrcPath, 'sunshine-frame.html'),
      },
    },
  },
  plugins: [
    tailwindcss(),
    vuePlugin(),
  ],
  resolve: {
    alias: {
      '@': rendererSrcPath,
    },
  },
  define: {
    // Environment flags derived from the build mode.
    __PROD__: JSON.stringify(process.env.NODE_ENV === 'production'),
    __DEV__: JSON.stringify(process.env.NODE_ENV === 'development'),
    __BETA__: JSON.stringify(process.env.BETA === 'true'),
  },
  css: {
    preprocessorOptions: {
      less: {
        math: 'always',
        relativeUrls: true,
        javascriptEnabled: true,
        // Prepended to EVERY .less block in the app, including each
        // component's <style lang="less">. These two files declare variables
        // and mixins only - they must never emit rules, or every component
        // would ship a duplicate copy of them.
        //
        // The design system itself (app/app.css) emits rules, so it is imported
        // once by the single renderer entry point rather than injected here.
        additionalData: `
          @import "@/styles/variables.less";
          @import "@/styles/mixins.less";
        `,
      },
    },
  },
  clearScreen: false,
}))
