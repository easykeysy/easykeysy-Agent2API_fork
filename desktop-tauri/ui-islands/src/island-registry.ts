// ： islands/ down 1 。 1 
// IIFE ，** not in when ** —— use mountOnPage ，
// etc app.js showPage 1 to that 1 create React root（
// src/island-lazy.ts）。 not and ， not will 。
//
// login-page not in its in ： ui/islands/login.js（
// src/login-entry.tsx and vite.login.config.ts）。if in ，
// as 1 need down 2.3 MB —— that is this need 。
const islandModules = import.meta.glob(['./islands/*.tsx', '!./islands/login-page.tsx'], {
  eager: true,
})
void islandModules
