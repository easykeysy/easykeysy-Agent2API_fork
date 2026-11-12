import { defineConfig, type Plugin } from 'vite'
import react from '@vitejs/plugin-react'
import tailwindcss from '@tailwindcss/vite'
import { fileURLToPath } from 'node:url'
import { createHash } from 'node:crypto'
import fs from 'node:fs'
import path from 'node:path'

/**
 * sw.js from directory to `ui/` ， and ****。
 *
 * ── Why to ────────────────────────────────────────────
 * SW default scope limit is in directory：`ui/m/sw.js` can 
 * `/m/*`， is `/m.html` —— in scope 。 `scope: '/'`
 * will （`The path of the provided scope is not under the max
 * scope`）， `/m/` not to 。directory no has this 。
 *
 * ── Why is ， not is when ────────────────────────────
 * SW update is **file**： sw.js 1 when ， not will 
 * new SW， old mobile.js will 1 —— use to up 1 
 * 。must。
 *
 * not can use `Date.now()`：CI has 1 「must can by 」 
 * （.github/workflows/frontend-artifacts.yml）， when will 
 * not sw.js， that 。 use mobile.js + mobile.css 
 * ：， no 1 ，。
 *
 * ── and must 1 ──────────────────────────
 * `public/sw.js` is ** file**， not is ： is CRLF
 * is LF （ autocrlf=true to CRLF，CI Linux
 * to LF）。， 1 in will not ，
 * is CI sw.js 、 that （2026-10 to ）。
 * before 1 CRLF LF， LF —— by 
 * file， and in none。
 *
 * ── in web down ───────────────────────────────────────
 * APK （`--mode apk`） no has Service Worker（ in ， not HTTP），
 * this in that none can 。
 */
function swBuildStamp(): Plugin {
  return {
    name: 'sw-build-stamp',
    apply: 'build',
    closeBundle() {
      const built = path.resolve(__dirname, '../ui/m/sw.js')
      const target = path.resolve(__dirname, '../ui/sw.js')
      if (!fs.existsSync(built)) return

      // CRLF → LF。，（file ）
      const normalizeEol = (buf: Buffer): Buffer => {
        const out = Buffer.allocUnsafe(buf.length)
        let n = 0
        for (let i = 0; i < buf.length; i++) {
          if (buf[i] === 0x0d && buf[i + 1] === 0x0a) continue
          out[n++] = buf[i]
        }
        return out.subarray(0, n)
      }

      const hash = createHash('sha256')
      // sw.js ：、 not JS/CSS when ， need ，
      // otherwise not SW， new not 。
      hash.update(normalizeEol(fs.readFileSync(path.resolve(__dirname, 'public', 'sw.js'))))
      for (const name of ['mobile.js', 'mobile.css']) {
        const file = path.resolve(__dirname, '../ui/m', name)
        if (fs.existsSync(file)) hash.update(normalizeEol(fs.readFileSync(file)))
      }
      hash.update(normalizeEol(fs.readFileSync(path.resolve(__dirname, '../ui/aliyun-captcha.js'))))
      // 12 ， not will 
      const stamp = hash.digest('hex').slice(0, 12)

      // 1 ：otherwise（CRLF ） will CRLF sw.js，
      // and CI LF in up for not up 
      const text = normalizeEol(fs.readFileSync(built)).toString('utf8').replace(/__BUILD__/g, stamp)
      fs.writeFileSync(target, text)
      // directory not ： in that will 「 can SW」， scope not for 
      fs.rmSync(built)
    },
  }
}

/**
 * APK ： HTML and to 1 directory， to Capacitor `webDir`。
 *
 * ── Why 1 ， not use ui/m/ ─────────────────────────────
 * PWA is 「 1 directory」，path for `/m/`；APK WebView
 * from file，`/m/mobile.js` this for path will to file。
 * APK need use ** for path**、 is directory index.html， not can and PWA use 。
 *
 * ── Why need i18n and ───────────────────────────
 * down and by （`ui/i18n/*`、`ui/assets/*`）；
 * APK no has ， need can ， this must。
 */
function apkBundle(): Plugin {
  return {
    name: 'apk-bundle',
    apply: 'build',
    closeBundle() {
      const root = path.resolve(__dirname, 'android-www')
      const source = path.resolve(__dirname, '../ui')

      // ： not HTML， this apk.html index.html。
      // Capacitor webDir index.html。
      fs.copyFileSync(path.resolve(__dirname, 'apk.html'), path.join(root, 'index.html'))

      // ： when use to that file， manifest.js
      const i18nSrc = path.join(source, 'i18n')
      const i18nOut = path.join(root, 'i18n')
      if (fs.existsSync(i18nSrc)) {
        // combined/ （ and by ）；APK use not to ，
        // not ， 700 KB and 。
        fs.cpSync(i18nSrc, i18nOut, {
          recursive: true,
          filter: src => path.basename(src) !== 'combined',
        })
      }
      for (const name of ['i18n.js', 'aliyun-captcha.js']) {
        fs.copyFileSync(path.join(source, name), path.join(root, name))
      }
      // 
      const providersSrc = path.join(source, 'assets', 'providers')
      const providersOut = path.join(root, 'assets', 'providers')
      if (fs.existsSync(providersSrc)) {
        fs.mkdirSync(providersOut, { recursive: true })
        fs.cpSync(providersSrc, providersOut, { recursive: true })
      }
      // use （ / empty can can use to ）
      for (const name of ['app-icon-192.png', 'app-icon-512.png']) {
        const file = path.join(source, 'assets', name)
        if (fs.existsSync(file)) {
          fs.mkdirSync(path.join(root, 'assets'), { recursive: true })
          fs.copyFileSync(file, path.join(root, 'assets', name))
        }
      }
    },
  }
}

/**
 * config。 ：
 *
 * `npm run build` → web（PWA）， `ui/m/`， `ui/m.html`
 * `npm run build:apk` → APK， `android-www/`， index.html
 *
 * ── Why web ui/ ───────────────────────────────
 * `ui/` is Tauri frontendDist、Docker COPY 、server static_files
 * —— and use 1 directory， 1 not use 。
 *
 * Why is `ui/m.html` not is `ui/m/index.html`：server static_files
 * path is 「none .html」（`/login` → `login.html`），unsupporteddirectory
 * index —— `ui/m/index.html` when `/m` not to file。`ui/m.html` 
 * `/m` in ，Rust 。
 *
 * ── Why is iife ───────────────────────────────────────────
 * and ui-islands ： has `i18n.js` etc ，module is defer
 * ， will 1 when 。 is ， by 。
 * APK is —— Capacitor WebView from file， not and 
 * 。
 *
 * ── ──────────────────────────────────────────────────
 * `../ui-kit` （ not npm ），dedupe React to this 1 。
 */
export default defineConfig(({ mode }) => {
  const apk = mode === 'apk'
  // ： and android/variables.gradle is 1 package.json 
  // （APK versionName ），set「」 and 「checkupdate」 use this 。
  const pkgVersion = JSON.parse(
    fs.readFileSync(path.resolve(__dirname, 'package.json'), 'utf8'),
  ).version as string
  return {
    plugins: apk
      ? [react(), tailwindcss(), apkBundle()]
      : [react(), tailwindcss(), swBuildStamp()],
    resolve: {
      alias: {
        '@ui': fileURLToPath(new URL('../ui-kit/src', import.meta.url)),
      },
      dedupe: ['react', 'react-dom', '@base-ui/react'],
    },
    define: {
      'process.env.NODE_ENV': JSON.stringify('production'),
      // ：request package.json（APK no has ）
      __APP_VERSION__: JSON.stringify(pkgVersion),
    },
    build: {
      outDir: apk ? 'android-www' : '../ui/m',
      emptyOutDir: true,
      // web need public/（manifest and sw.js）；APK not need 
      // —— Service Worker in file no has 
      copyPublicDir: !apk,
      lib: {
        entry: 'src/main.tsx',
        name: 'A2aMobile',
        formats: ['iife'],
        fileName: () => 'mobile.js',
        cssFileName: 'mobile',
      },
      minify: 'esbuild',
      sourcemap: false,
      target: 'es2020',
    },
  }
})
