import { createRoot } from 'react-dom/client'
import { App } from './App'
import { setAuthLostHandler } from './api/client'
import { store } from './state/store'
import { initPwa } from './lib/pwa'
import './styles/mobile.css'

/**
 * 。
 *
 * and ui-islands ： that is 「 1 to has up 」， this is ** 
 * ** use —— use createRoot to #root， not need need `display: contents`
 * that 。
 *
 * 401 ：client not （ not current in ）， by this 
 * ， to store 「 new / Key」。
 */
setAuthLostHandler(mode => {
  store.setAuthLost(mode)
})

/**
 * PWA when must in render before ， not can useEffect：
 * `beforeinstallprompt` in after not will ， for can 1 ——
 * etc React ， has can can already （ when ）。
 * this return is ， use not ，ignore can 。
 */
initPwa()

const host = document.getElementById('root')
if (host) {
  createRoot(host).render(<App />)
} else {
  // when to can error， not is 
  document.body.innerHTML =
    '<pre style="padding:20px;font:13px/1.6 monospace;color:#c00">手机端挂载失败：页面缺少 #root 容器。</pre>'
}
