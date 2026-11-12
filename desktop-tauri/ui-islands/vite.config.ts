import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import tailwindcss from '@tailwindcss/vite'
import { fileURLToPath } from 'node:url'

/**
 * React config。
 *
 * to `ui/islands/`， by index.html <script> / <link> ——
 * Tauri is `ui/` this directory（ tauri.conf.json frontendDist），
 * up must is 「 already js / css file」， not is Tauri 。
 *
 * use iife esm： has 60 before is 、，
 * module is defer ， will 1 when 。
 *
 * （../ui-kit）， not npm ： 1 node_modules
 * typecheck and （globals.css tailwindcss / tw-animate-css /
 * shadcn @import need can from to ）。 when dedupe React
 * to this 1 —— React will not 。
 *
 * not is this 1 ： by `vite.login.config.ts` 
 * `ui/islands/login.js`（ this filemodule header）， down 。
 */
export default defineConfig({
  plugins: [react(), tailwindcss()],
  resolve: {
    alias: {
      '@ui': fileURLToPath(new URL('../ui-kit/src', import.meta.url)),
    },
    dedupe: ['react', 'react-dom', '@base-ui/react'],
  },
  /**
   * lib down Vite not `process.env.NODE_ENV`（ need 
   * ）， is to —— React will this ，
   * not will in when `process is not defined`、 not 。
   */
  define: {
    'process.env.NODE_ENV': JSON.stringify('production'),
  },
  build: {
    outDir: '../ui/islands',
    // directory in ， and after will has file，empty
    emptyOutDir: false,
    lib: {
      entry: 'src/index.tsx',
      name: 'WbIslands',
      formats: ['iife'],
      fileName: () => 'ui.js',
      // （） ， and ui.js and in ui/islands/
      cssFileName: 'ui',
    },
    // need ，； need need when sourcemap when 1 
    minify: 'esbuild',
    sourcemap: false,
    target: 'es2020',
  },
})
