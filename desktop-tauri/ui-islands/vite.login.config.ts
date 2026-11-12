import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import tailwindcss from '@tailwindcss/vite'
import { fileURLToPath } from 'node:url'

/**
 * config。
 *
 * ：`login.html` need need 1 ， before use is 
 * `islands/ui.js`（2.3 MB， and ）。 when this 1 
 * to seconds。 this `src/login-entry.tsx` as ，
 * `ui/islands/login.js`， and use 1 React / 。
 *
 * its config and `vite.config.ts` 1 （alias、dedupe、production 、
 * esbuild 、 es2020）， in when as up 。
 * not in this ： Tailwind by `ui.css` ，
 * `login.html` use that 1 。
 */
export default defineConfig({
  plugins: [react(), tailwindcss()],
  resolve: {
    alias: {
      '@ui': fileURLToPath(new URL('../ui-kit/src', import.meta.url)),
    },
    dedupe: ['react', 'react-dom', '@base-ui/react'],
  },
  define: {
    'process.env.NODE_ENV': JSON.stringify('production'),
  },
  build: {
    outDir: '../ui/islands',
    // and use directory， ui.js / ui.css 
    emptyOutDir: false,
    lib: {
      entry: 'src/login-entry.tsx',
      name: 'WbLogin',
      formats: ['iife'],
      fileName: () => 'login.js',
    },
    minify: 'esbuild',
    sourcemap: false,
    target: 'es2020',
  },
})
