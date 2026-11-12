import { mountInput, observeInputs } from './input'
import { mountSegmented } from './segmented'
import './uikit'
import './styles/islands.css'
// create， can section。
import './desktop-shell'
import './desktop-tooltip'
import './desktop-titlebar'
import './island-registry'

declare global {
  interface Window {
    wbSegmented?: { mount: typeof mountSegmented }
    wbInput?: { mount: typeof mountInput }
  }
}

window.wbSegmented = { mount: mountSegmented }
window.wbInput = { mount: mountInput }
observeInputs()

// in up ：`app.js` 1 `showPage` need etc DOMContentLoaded，
// by need after ， to that 1 will is empty 。 this by and 
// app.js `PAGE_KEY` localStorage before 1 ； after `showPage`
// when ，island-lazy for already is etc 。
const initialPage = (() => {
  try {
    return localStorage.getItem('workbuddy-desktop-page') || 'overview'
  } catch {
    return 'overview'
  }
})()
window.dispatchEvent(new CustomEvent('wb:page', { detail: initialPage }))
