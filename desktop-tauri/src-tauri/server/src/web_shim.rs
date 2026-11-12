//! （headless when `window.workbuddyDesktop` ）。
//!
//! ── Why need need ────────────────────────────────────────────
//! （desktop-tauri/ui/） `window.workbuddyDesktop` this ，
//! by `bridge.rs` in before for （Tauri IPC → → ）。
//! headless no has ： 1 ** HTTP** ， by 
//! `static_files` index.html —— in ，
//! 「 not 」 has in this 。
//!
//! ── and bridge （ its 1 1 for ）─────────────
//!   · 传输：`call` 直接 `fetch` 同源 `/api/*`， `x-api-key`；
//! （`{success, data}` / 401） and `gateway::unwrap_envelope`
//! for ， try/catch not 。
//! · ： 1 Key（）； by ** use **
//! （401 when ， localStorage after 1 ）。
//! · ：Tauri push not in ， as **** ——
//! `accounts:state-changed` in requestsuccess after 1 /api/session 
//! （300ms ， and ）；`login:state` by file 
//! ；`backend:error` / `accounts:auto-maintained` no has for ，
//! returnempty（ is 「 1 」 ）。
//!
//! ── in ──────────────────────────────────
//! startLogin is 「 + 」； as 
//! `POST /api/session/login/start` authUrl → new → 
//! `GET /api/session/login/wait?state=` to done（ and etc 1 ，
//! return session；/error up ）。 use （ for up ）：
//! · WorkBuddy / Qoder / Cline：，available；
//! · AutoClaw（OAuth）： loopback ， by 
//! ；CatPaw by server poll-token ；
//! · ： to `redirect` ， to 
//! HTTP —— before is （ up 
//! http://127.0.0.1 / localhost / [::1] + ，
//! `raccoonCallbackOriginProblem`）， not when before down ；
//! Trae / Accio / CodeArts / Antigravity：“”。
//!
//! ── has ────────────────────────────────────────
//! 、、、update、set、file for 
//! this has can by 「 can then ， not can then 」：
//! to can （ will toast ）， not success。

/// index.html （ in all before ）。
///
/// use `r#"…"#` ： not `"#` （ 1 ），
/// not need need ； no has —— `'web'`。
pub fn shim_js() -> &'static str {
    r#"(function () {
  'use strict';

  // ── （`?from=login`）─────────────
  // server in `/` up will 「 will invalid 」 and （
  // server::http::panel_boot_redirect）。 in will in localStorage
  // `x-panel-token` ， not ， when this 
  // 、server this 1 。must in after from ：，
  // will after use new will that ， is 1 401。
  (function () {
    try {
      var url = new URL(window.location.href);
      if (!url.searchParams.has('from')) return;
      url.searchParams.delete('from');
      var search = url.searchParams.toString();
      window.history.replaceState(null, '',
        url.pathname + (search ? '?' + search : '') + url.hash);
    } catch (e) { /* / use ： is 1 ， not */ }
  })();

  // ── API Key ────────────────────────────────────────
  var KEY_STORAGE = 'agent2api.webKey';
  function readStoredKey() {
    try { return localStorage.getItem(KEY_STORAGE) || ''; } catch (e) { return ''; }
  }
  var apiKey = readStoredKey();
  function storeKey(value) {
    apiKey = value;
    try {
      if (value) localStorage.setItem(KEY_STORAGE, value);
      else localStorage.removeItem(KEY_STORAGE);
    } catch (e) { /* etc ： in can */ }
  }

  // ── （PANEL_AUTH）──────────────────────
  // HttpOnly cookie in 「 in 」（fnOS docker this ）
  // down / will ： POST can to 、Set-Cookie not ， 1 
  // request 401 。/ new when response
  // （ panel.rs issue_response）， before this ， after request
  // `x-panel-token` / `x-panel-refresh` 。 down cookie down 
  // and server， this is ， not 。
  var PANEL_AUTH = 'agent2api.panelAuth';
  /**
   * down access to when （ms；0 = not ）。
   *
   * ** not is **：「 this not this in down request before 」（ `ensureFreshSession`）。
   * server by will valid， this will 「401 → 」 path。
   * in 1 PANEL_AUTH not is 1 ： will when 1 。
   *
   * this is **unavailable when **（）： down 1 
   * localStorage as —— use 1 ， 1 after this must
   * to new to when ， not can by when that old 。
   */
  var panelExpiresAt = 0;
  function readPanelAuth() {
    try { return JSON.parse(localStorage.getItem(PANEL_AUTH) || 'null'); }
    catch (e) { return null; }
  }
  function storedPanelToken(kind) {
    var auth = readPanelAuth();
    var value = auth && auth[kind];
    if (typeof value === 'string' && value) return value;
    return '';
  }
  /** current to when ：localStorage ，unavailable when to */
  function currentExpiry() {
    var auth = readPanelAuth();
    var stored = Number(auth && auth.expiresAt);
    if (isFinite(stored) && stored > 0) return stored;
    return panelExpiresAt;
  }

  /** down to when 。`expiresIn` is server down remainingseconds（ panel.rs issue_response） */
  function rememberExpiry(expiresIn) {
    var seconds = Number(expiresIn);
    if (!isFinite(seconds) || seconds <= 0) return;
    panelExpiresAt = Date.now() + seconds * 1000;
    var auth = readPanelAuth() || {};
    auth.expiresAt = panelExpiresAt;
    try { localStorage.setItem(PANEL_AUTH, JSON.stringify(auth)); }
    catch (e) { /* etc ： will valid */ }
  }

  function storePanelAuth(access, refresh) {
    try {
      localStorage.setItem(PANEL_AUTH, JSON.stringify({
        access: access, refresh: refresh, expiresAt: panelExpiresAt || currentExpiry(),
      }));
    } catch (e) { /* etc ： in use can */ }
  }
  function clearPanelAuth() {
    panelExpiresAt = 0;
    try { localStorage.removeItem(PANEL_AUTH); } catch (e) { /* none */ }
  }

  // ── （Key / / ）： DOM， not ──
  // options（optional）：
  // cancelText to 1 need by ； or by Esc null ——
  // use 「」（ not is null） and 「」。
  // onMount(body, controls) up DOM after 1 ： is HTML ，
  // need to by has this 1 will ；controls.cancel()
  // （e.g. when ） can 。
  //
  // cancelText will to activeOverlay：`cancelLogin` and 
  // `closeWebOverlay` need can to （ before use 「 etc 」 when ，
  // after in when document up Esc —— not ， that 
  // will in already after response Esc， up 1 Promise ）。
  var activeOverlay = null;
  // use this ： not is use ， not 
  var DISPOSED = {};
  function ensureOverlay(titleText, bodyHtml, confirmText, options) {
    options = options || {};
    // up 1 （ can can cancelText and document Esc ）。
    // use dispose not is cancel： new ≠ use ， not can 
    // up 1 （：401 API Key ）。
    closeWebOverlay(true);
    return new Promise(function (resolve) {
      var overlay = document.createElement('div');
      overlay.id = 'a2a-web-overlay';
      overlay.style.cssText = 'position:fixed;inset:0;z-index:2147483647;display:flex;'
        + 'align-items:center;justify-content:center;background:rgba(0,0,0,.55);';
      var card = document.createElement('div');
      card.style.cssText = 'background:#1e1f22;color:#e8e8e8;border-radius:10px;'
        + 'padding:22px 24px;width:min(420px,86vw);box-shadow:0 12px 40px rgba(0,0,0,.4);'
        + 'font:14px/1.6 system-ui,-apple-system,"Segoe UI",sans-serif;';
      var title = document.createElement('div');
      title.textContent = titleText;
      title.style.cssText = 'font-size:15px;font-weight:600;margin-bottom:10px;';
      var body = document.createElement('div');
      body.innerHTML = bodyHtml;
      card.appendChild(title);
      card.appendChild(body);
      var input = body.querySelector('input');
      var cancelText = options.cancelText || '';
      var done = false;
      var actions = document.createElement('div');
      actions.style.cssText = 'display:flex;gap:10px;margin-top:14px;';
      if (cancelText) {
        var cancelButton = document.createElement('button');
        cancelButton.textContent = cancelText;
        cancelButton.style.cssText = 'flex:1;padding:8px 0;border:1px solid #3a3b3f;'
          + 'border-radius:6px;background:transparent;color:#c9c9c9;font-size:14px;cursor:pointer;';
        cancelButton.addEventListener('click', function () { finish(null); });
        actions.appendChild(cancelButton);
      }
      var button = document.createElement('button');
      button.textContent = confirmText;
      button.style.cssText = 'flex:1;padding:8px 0;border:0;border-radius:6px;'
        + 'background:#4c7dff;color:#fff;font-size:14px;cursor:pointer;';
      actions.appendChild(button);
      card.appendChild(actions);
      overlay.appendChild(card);
      // Esc and by 。 down and stopPropagation： down 「」
      // Esc， not will ， 1 。
      var onKeydown = function (event) {
        if (event.key !== 'Escape' || !cancelText) return;
        event.preventDefault();
        event.stopPropagation();
        finish(null);
      };
      // disposeOnly： and DOM， use 1 etc 。
      // path when （pollWait Promise.race）， this 1 
      // must —— not can resolve(null)（ will 「 use 」server），
      // not can （ that will in ）。
      var finish = function (value, disposeOnly) {
        if (done) return;
        done = true;
        document.removeEventListener('keydown', onKeydown, true);
        overlay.remove();
        if (activeOverlay && activeOverlay.overlay === overlay) activeOverlay = null;
        resolve(disposeOnly ? DISPOSED : value);
      };
      document.body.appendChild(overlay);
      // has by Esc ： its （API Key 、）
      // not this Esc ， not this in after down to already document 
      if (cancelText) document.addEventListener('keydown', onKeydown, true);
      button.addEventListener('click', function () { finish(input ? input.value : true); });
      if (input) {
        input.addEventListener('keydown', function (event) {
          if (event.key === 'Enter') finish(input.value);
        });
        setTimeout(function () { input.focus(); }, 60);
      }
      if (cancelText) {
        activeOverlay = {
          overlay: overlay,
          cancelable: true,
          cancel: function () { finish(null); },
          dispose: function () { finish(undefined, true); },
        };
      } else {
        // not ： when need 「」 this 1 
        activeOverlay = {
          overlay: overlay,
          cancelable: false,
          cancel: function () { /* no has */ },
          dispose: function () { finish(undefined, true); },
        };
      }
      if (options.onMount) {
        try { options.onMount(body, { cancel: function () { finish(null); } }); }
        catch (e) { /* failure not this */ }
      }
    });
  }

  /** ： http （ up down ），clipboard API unavailable，
   * textarea + execCommand old path。 */
  function copyText(text) {
    if (navigator.clipboard && window.isSecureContext) {
      return navigator.clipboard.writeText(text);
    }
    return new Promise(function (resolve, reject) {
      var area = document.createElement('textarea');
      area.value = text;
      area.setAttribute('readonly', '');
      area.style.cssText = 'position:fixed;top:0;left:0;opacity:0;pointer-events:none;';
      document.body.appendChild(area);
      area.select();
      var ok = false;
      try { ok = document.execCommand('copy'); } catch (e) { ok = false; }
      area.remove();
      if (ok) resolve();
      else reject(new Error('复制失败'));
    });
  }

  function askForKey() {
    if (!askForKey.promise) {
      askForKey.promise = ensureOverlay(
        '需要网关 API Key',
        '<div style="margin-bottom:10px;">此面板受网关鉴权保护，请输入一把已启用的 API Key'
        + '（在「API Keys」页创建）。</div>'
        + '<input type="password" placeholder="sk-…" style="width:100%;box-sizing:border-box;'
        + 'padding:8px 10px;border:1px solid #3a3b3f;border-radius:6px;background:#26272b;'
        + 'color:#e8e8e8;">',
        '保存并继续'
      ).then(function (value) {
        askForKey.promise = null;
        // （ DISPOSED）： no ， this Key 
        if (value === DISPOSED) return '';
        if (value) storeKey(String(value).trim());
        return value;
      });
    }
    return askForKey.promise;
  }

  function showLinkFallback(url) {
    // url already ；， can /， not can HTML。
    ensureOverlay(
      '浏览器拦截了弹出窗口',
      '<div>请点击下面的链接打开授权页：</div>'
      + '<a href="' + escapeHtml(url) + '" target="_blank" rel="noopener noreferrer" '
      + 'style="color:#7fa7ff;word-break:break-all;">' + escapeHtml(url) + '</a>',
      '已完成，关闭'
    );
  }

  // current。silent=false when etc in Promise 1 「」 ——
  // 「 etc 」 by and this ；（silent=true）
  // when DOM， not resolve 「 use 」， already success 。
  function closeWebOverlay(silent) {
    if (activeOverlay) {
      if (!silent && activeOverlay.cancelable) activeOverlay.cancel();
      else activeOverlay.dispose();
      return;
    }
    var overlay = document.getElementById('a2a-web-overlay');
    if (overlay) overlay.remove();
  }

  // this will to loopback 。
  // Docker this in ， not can to ，
  // use URL 。 use 
  // supports redirect ， authorization_code 。
  // path ： to 「」 that 1 when ， need not need 
  // to use 。 runLoginFlow that 。
  var MANUAL_CALLBACK_GRACE_MS = 10000;

  // no has options old use use ； method metadata。
  function legacyNeedsManualCallback(provider) {
    return provider === 'trae'
      || provider === 'accio' || provider === 'accio-cn'
      || provider === 'codearts' || provider === 'autoclaw-intl'
      // LobsterAI this 1 ： after 302 
      // `http://127.0.0.1:<>/auth/callback`， in Docker that 
      // in 、 not to ， can use URL 。
      || provider === 'lobsterai'
      // Antigravity is this 1 ： `redirect_uri` by 
      // `http://localhost:<>/oauth-callback`（Google desktop 
      // client loopback， `providers/antigravity/oauth.rs`）。
      // and ， 1 ； / Docker when that localhost
      // in ， to not —— use can URL
      // ， by after `submit_login_callback` Antigravity 。
      // not need this can ： `redirect_uri` and when 
      // （ `login_redirect_uri`）， 1 。
      || provider === 'antigravity';
  }

  function escapeHtml(value) {
    return String(value == null ? '' : value)
      .replace(/&/g, '&amp;').replace(/</g, '&lt;')
      .replace(/>/g, '&gt;').replace(/"/g, '&quot;');
  }

  function hasMethodIndex(options) {
    return !!(options && Object.prototype.hasOwnProperty.call(options, 'methodIndex'));
  }

  // GET when when provider/， not BUILTIN or any id。
  async function selectedLoginMethod(provider, options) {
    if (!Number.isSafeInteger(options.methodIndex) || options.methodIndex < 0) {
      throw new Error('登录方式索引必须是非负整数');
    }
    var metadata = await call('GET', '/api/session/login/methods?provider=' + encodeURIComponent(provider));
    var method = metadata && Array.isArray(metadata.methods)
      && metadata.methods.find(function (item) { return item && item.methodIndex === options.methodIndex; });
    if (!method) throw new Error('该登录方式已不在提供商的运行时清单中，请重新选择');
    return {
      manualCallback: method.manualCallback === true,
      callbackInput: method.callbackInput === 'code' ? 'code' : 'url',
      localOnly: method.localOnly === true,
      instructions: typeof metadata.instructions === 'string' ? metadata.instructions : '',
    };
  }

  function loginCallbackTargets(authUrl) {
    if (!authUrl) return [];
    var auth;
    try { auth = new URL(authUrl); } catch (e) { return []; }
    var targets = [];
    var collect = function (value, key) {
      if (['redirect_uri', 'redirect', 'redirect_url', 'callback_url', 'auth_callback_url',
        'navigate_uri', 'return_url'].indexOf(key) < 0) return;
      try {
        var target = new URL(value);
        if (target.hostname && !target.username && !target.password) targets.push(target);
      } catch (e) { /* URL parameter not callback */ }
    };
    auth.searchParams.forEach(collect);
    var question = auth.hash.indexOf('?');
    if (question >= 0) new URLSearchParams(auth.hash.slice(question + 1)).forEach(collect);
    return targets;
  }

  function checkedLoginUrl(value, started, targets) {
    if (typeof value !== 'string' || /[\u0000-\u0020\u007f]/.test(value)) {
      throw new Error('登录链接包含无效字符');
    }
    var parsed;
    try { parsed = new URL(value); } catch (e) { throw new Error('登录链接无效'); }
    if (!parsed.hostname || parsed.username || parsed.password) throw new Error('登录链接无效');
    var hosts = started && started.allowedHosts;
    if (hosts != null && (!Array.isArray(hosts) || hosts.some(function (host) {
      return typeof host !== 'string' || !host.trim().replace(/^\*\./, '').replace(/\.$/, '');
    }))) throw new Error('登录域名白名单格式无效');
    var regularHosts = [];
    var loopbackPages = [];
    (Array.isArray(hosts) ? hosts : []).forEach(function (host) {
      if (typeof host !== 'string') return;
      var text = host.trim().toLowerCase();
      // （，）
      var page = /^(127\.0\.0\.1|localhost|\[::1\]):(\d+)$/.exec(text);
      if (page) { loopbackPages.push({ host: page[1], port: page[2] }); return; }
      var allowed = text.replace(/^\*\./, '').replace(/\.$/, '');
      if (allowed) regularHosts.push(allowed);
    });
    var permittedHost = !Array.isArray(hosts) || regularHosts.length === 0 || regularHosts.some(function (allowed) {
      return parsed.hostname.toLowerCase() === allowed
        || parsed.hostname.toLowerCase().endsWith('.' + allowed);
    });
    var callbackHttp = parsed.protocol === 'http:'
      && ['localhost', '127.0.0.1', '[::1]'].indexOf(parsed.hostname) >= 0
      && (targets || []).some(function (target) {
        return target.protocol === parsed.protocol && target.hostname === parsed.hostname
          && target.port === parsed.port && target.pathname === parsed.pathname;
      });
    var loopbackPage = parsed.protocol === 'http:'
      && loopbackPages.some(function (page) {
        return parsed.hostname.toLowerCase() === page.host && (parsed.port || '80') === page.port;
      });
    if (!permittedHost || (parsed.protocol !== 'https:' && !callbackHttp && !loopbackPage)) {
      throw new Error('登录链接不是允许的 HTTPS 站点或合法本机回调地址');
    }
    return value; // 校验不重写原文，保留授权地址中的编码、顺序与签名。
  }

  // in login_source=desktop when office-raccoon://，
  // not will 。 this supports redirect ： desktop
  // after ， will authorization_code to redirect URL and 。
  // this in current Origin， can Docker 。
  //
  // ── Why before need check（raccoonCallbackOriginProblem）──
  // up for redirect parameter has 1 （ 2026-10）：http 
  // + （127.0.0.1 / localhost / [::1]）+ ；、 IP、
  // https、 1 —— server up down empty，
  // 「invalid， from client new 。」 this for use none
  // （ is ， use no parameter）， in before by 1 
  // check， not down ， not use that 。
  function raccoonCallbackOriginProblem() {
    var hostname = window.location.hostname;
    var port = String(window.location.port || '');
    var loopback = hostname === '127.0.0.1' || hostname === 'localhost' || hostname === '[::1]';
    if (window.location.protocol === 'http:' && loopback && port !== '') return null;
    var reason;
    if (window.location.protocol !== 'http:') {
      reason = '小浣熊授权页只接受 http:// 的回调地址，当前面板以 '
        + window.location.protocol + ' 访问';
    } else if (!loopback) {
      reason = '小浣熊授权页只接受 127.0.0.1 / localhost 的回调地址，当前面板主机是 ' + hostname;
    } else {
      reason = '小浣熊授权页要求回调地址带显式端口，当前面板地址未带（默认端口访问）';
    }
    var displayPort = port || (window.location.protocol === 'https:' ? '443' : '80');
    var html = '<div style="margin-bottom:10px;">小浣熊官方授权页只接受<strong>浏览器本机的回环地址</strong>'
      + '（127.0.0.1 / localhost，http 且带端口）作为登录回调；当前面板地址是 <code>'
      + escapeHtml(window.location.origin) + '</code>，继续发起只会看到「授权链接无效」。</div>'
      + '<div style="margin-bottom:6px;">可行的做法：</div>'
      + '<div style="margin-bottom:6px;">① 在打开面板的这台电脑上，改用 <code>http://localhost:'
      + escapeHtml(displayPort) + '</code> 访问面板后再登录；面板部署在别的机器时，先做端口转发'
      + '（例如 <code>ssh -L ' + escapeHtml(displayPort) + ':127.0.0.1:' + escapeHtml(displayPort)
      + ' 用户@面板主机</code>，端口按实际改）。</div>'
      + '<div>② 改用「粘贴 token / refreshToken」方式添加小浣熊账号，不走授权页。</div>';
    return { title: '小浣熊网页登录无法在当前地址发起', html: html, reason: reason };
  }

  // not ：（， in this ）， use after 
  // reject —— will toast 1 reason （，toast ）。
  async function reportRaccoonCallbackProblem(problem) {
    await ensureOverlay(problem.title, problem.html, '知道了');
    throw new Error(problem.reason);
  }

  function buildRaccoonWebAuthUrl(provider, state, authUrl) {
    if (provider !== 'raccoon') return authUrl;
    // ：startLogin already （ and down ）。 this 1 ， is as 
    // is redirect 1 —— add use not 「must
    // 」 this before ； to this failure when ，runLoginFlow finally will ，
    // already after none will when 。
    var problem = raccoonCallbackOriginProblem();
    if (problem) throw new Error(problem.reason);
    var authorizeUrl;
    try {
      authorizeUrl = new URL(authUrl, window.location.href);
    } catch (error) {
      throw new Error('小浣熊授权地址无效，无法建立远程回调');
    }
    // use in by ： to ，parameter and by 
    var callbackUrl = new URL('/api/session/login/remote-callback', window.location.origin);
    callbackUrl.searchParams.set('state', state);
    authorizeUrl.searchParams.set('login_source', 'web');
    authorizeUrl.searchParams.set('redirect', callbackUrl.toString());
    return authorizeUrl.toString();
  }

  async function waitForManualCallback(provider, flow, authUrl, waitPromise) {
    var label = window.wbProviders && window.wbProviders.labelOf
      ? window.wbProviders.labelOf(provider) : provider;
    var codeInput = flow.callbackInput === 'code';
    var callbackInstruction = codeInput
      ? '仅粘贴此登录方式要求的<strong>本人已授权令牌或一次性 code</strong>；不是设备码或完整回调地址。'
        + '正文将原样通过受保护的 POST 提交；接受后仍须等待网关确认并保存账号。'
      : '<strong>如果授权完成后账号没有自动出现</strong>，'
        + '复制浏览器地址栏中的<strong>完整回调地址</strong>粘贴提交；'
        + '不要改动参数。回调被接受后仍须等待网关确认并保存账号。';
    var linkStyle = 'flex:1;padding:7px 0;border:1px solid #3a3b3f;border-radius:6px;'
      + 'background:transparent;color:#c9c9c9;font-size:13px;cursor:pointer;';
    while (currentFlow === flow && !flow.canceled) {
      // empty URL can code； not empty， not can 。
      var authLinkHtml = authUrl ? '<div style="margin-bottom:6px;color:#9a9a9a;">授权页地址</div>'
        + '<code style="display:block;margin-bottom:8px;padding:8px 10px;border-radius:6px;'
        + 'background:#26272b;color:#b8c7e6;font-size:12px;line-height:1.5;'
        + 'word-break:break-all;max-height:96px;overflow:auto;">' + escapeHtml(authUrl) + '</code>'
        + '<div style="display:flex;gap:8px;margin-bottom:14px;">'
        + '<button type="button" data-a2a-act="open" style="' + linkStyle + '">打开授权页</button>'
        + '<button type="button" data-a2a-act="copy" style="' + linkStyle + '">复制链接</button>'
        + '</div>' : '';
      var callbackUrl = await ensureOverlay(
        label + (codeInput ? '需要粘贴授权令牌或 code' : '需要粘贴回调地址'),
        '<div style="margin-bottom:10px;">' + callbackInstruction + '</div>'
        + '<div data-a2a-login-progress style="white-space:pre-wrap;margin-bottom:10px;">'
        + escapeHtml(loginProgressText(flow)) + '</div>'
        + authLinkHtml
        + '<input type="password" spellcheck="false" autocomplete="off" autocapitalize="off" autocorrect="off" '
        + 'placeholder="' + (codeInput ? '此方式要求的授权令牌或一次性 code' : '本次登录的完整回调地址') + '" '
        + 'style="width:100%;box-sizing:border-box;padding:8px 10px;border:1px solid #3a3b3f;'
        + 'border-radius:6px;background:#26272b;color:#e8e8e8;">',
        codeInput ? '提交授权令牌或 code' : '提交回调地址',
        {
          cancelText: '取消登录',
          onMount: function (body) {
            var openBtn = body.querySelector('[data-a2a-act="open"]');
            var copyBtn = body.querySelector('[data-a2a-act="copy"]');
            if (openBtn) openBtn.addEventListener('click', function () {
              if (currentFlow !== flow || flow.canceled) return;
              try {
                var checked = checkedLoginUrl(authUrl, flow.dynamic ? flow.started : null, flow.callbackTargets);
                window.open(checked, '_blank', 'noopener,noreferrer');
              } catch (e) {
                flow.hint = '浏览器无法打开授权页，请复制上方链接手动打开。';
                emitLogin();
              }
            });
            if (copyBtn) copyBtn.addEventListener('click', function () {
              copyText(authUrl).then(function () {
                copyBtn.textContent = '已复制';
                setTimeout(function () {
                  if (copyBtn.isConnected) copyBtn.textContent = '复制链接';
                }, 1500);
              }, function () { copyBtn.textContent = '请手动复制上方链接'; });
            });
          },
        }
      );
      if (callbackUrl === DISPOSED || currentFlow !== flow) {
        var takenOver = canceledError();
        takenOver.silent = true;
        throw takenOver;
      }
      if (callbackUrl == null) return cancelWebLogin(flow.state);
      // URL and code ， use trim empty， not //。
      callbackUrl = String(callbackUrl);
      if (!callbackUrl.trim()) continue;
      var submitted;
      try {
        if (currentFlow !== flow || flow.canceled) throw canceledError();
        submitted = await call('POST', '/api/session/login/callback', {
          state: flow.state, callbackUrl: callbackUrl,
        });
        callbackUrl = '';
        if (currentFlow !== flow || flow.canceled) throw canceledError();
        if (submitted && typeof submitted.nextUrl === 'string' && submitted.nextUrl) {
          authUrl = checkedLoginUrl(submitted.nextUrl, flow.dynamic ? flow.started : null, flow.callbackTargets);
          continue;
        }
      } catch (error) {
        callbackUrl = '';
        if (error && error.canceled) throw error;
        var failureMessage = codeInput ? '授权令牌或 code 未被接受；请检查此方式的要求。'
          : error && error.message ? error.message : error;
        var retry = await ensureOverlay(
          '回调提交失败',
          '<div style="margin-bottom:4px;">' + escapeHtml(failureMessage)
          + '</div><div>' + (codeInput
            ? '请确认粘贴的是此方式要求的本人授权令牌或一次性 code，不是设备码或回调地址，再重新提交。'
            : '请确认复制的是本次授权完成后的完整回调地址，再重新提交。') + '</div>',
          '重新填写', { cancelText: '取消登录' }
        );
        if (retry === DISPOSED || currentFlow !== flow) {
          var retryTakenOver = canceledError();
          retryTakenOver.silent = true;
          throw retryTakenOver;
        }
        if (retry == null) return cancelWebLogin(flow.state);
        continue;
      }
      var pendingHint = submitted && (typeof submitted.hint === 'string' ? submitted.hint
        : submitted.pending && submitted.pending.hint);
      if (typeof pendingHint === 'string') flow.hint = pendingHint;
      else flow.hint = '回调已提交，正在等待网关确认登录并保存账号…';
      emitLogin();
      // accepted/Pending/ callback Done not is this success： use 
      // 1 /wait Promise， or 「 already 」。
      return await waitPromise;
    }
    throw canceledError();
  }

  /** use ：server this 1 ， and to up 1 
   * can as 「」 。
   *
   * Why need /cancel：Trae that in has ， not need 
   * 5 minutes when ， use new will not to （ and `LoginService::cancel`
   * AutoClaw / CodeArts 1 ）。 */
  async function cancelWebLogin(state) {
    if (currentFlow && currentFlow.state === state) currentFlow.canceled = true;
    var canceled = true;
    try {
      var result = await call('POST', '/api/session/login/cancel', { state: state });
      canceled = !!(result && result.canceled);
    } catch (error) { /* already when will 404： already */ }
    var err = new Error(canceled ? '已取消登录等待' : '登录已结束');
    err.canceled = true;
    throw err;
  }

  // ── （set「」 use ）────────────────────
  // by to WebView2 / WKWebView ； can from UA ：
  // Chrome/Edge/Opera is `Chrome/<ver>`、Firefox is `Firefox/<ver>`、
  // Safari is `Version/<ver> … Safari/`。 not to empty —— 「—」，
  // not 1 。
  function browserEngineVersion() {
    var ua = (typeof navigator !== 'undefined' && navigator.userAgent) || '';
    var patterns = [
      [/Edg(?:e|A|iOS)?\/(\d+(?:\.\d+)*)/, 'Edge '],
      [/OPR\/(\d+(?:\.\d+)*)/, 'Opera '],
      [/Chrome\/(\d+(?:\.\d+)*)/, 'Chrome '],
      [/Firefox\/(\d+(?:\.\d+)*)/, 'Firefox '],
      [/Version\/(\d+(?:\.\d+)*).*Safari/, 'Safari '],
    ];
    for (var i = 0; i < patterns.length; i += 1) {
      var match = patterns[i][0].exec(ua);
      if (match) return patterns[i][1] + match[1];
    }
    return '';
  }

  // ── error 1 （ and bridge asError ）─────────────
  function asError(failure) {
    if (failure instanceof Error) return failure;
    if (typeof failure === 'string') return new Error(failure);
    if (failure && typeof failure === 'object') {
      var message = failure.message != null ? failure.message
        : failure.error != null ? failure.error
        : failure.msg;
      if (typeof message === 'string' && message) return new Error(message);
      try { return new Error(JSON.stringify(failure)); } catch (e) { /* to down */ }
    }
    return new Error(String(failure == null ? '操作失败' : failure));
  }

  // ── HTTP use ： and gateway::unwrap_envelope for ──
  async function httpCall(method, path, body, retried, selectedKey) {
    // to before ： this 1 「access → 401 → 」 not ，
    // no has that 2 when 1 401（ `ensureFreshSession`）。
    // in ： that 1 （retried=true）， 1 。
    if (!retried) await ensureFreshSession();
    var headers = { 'Accept': 'application/json' };
    if (apiKey) headers['x-api-key'] = apiKey;
    // will （ PANEL_AUTH ）： has up ；
    // refresh/logout in /api/panel/ path refresh， for 
    // refresh cookie Path=/api/panel 。
    var panelAccess = storedPanelToken('access');
    if (panelAccess) {
      headers['x-panel-token'] = panelAccess;
      // Authorization is ：request in will 
      headers['Authorization'] = 'Bearer ' + panelAccess;
    }
    if (path.indexOf('/api/panel/') === 0) {
      var panelRefresh = storedPanelToken('refresh');
      if (panelRefresh) headers['x-panel-refresh'] = panelRefresh;
    }
    if (selectedKey !== undefined) headers['x-api-key'] = selectedKey;
    var init = { method: method, headers: headers };
    var wantsBody = method === 'POST' || method === 'PUT' || method === 'PATCH'
      || !(body === null || body === undefined);
    if (wantsBody) {
      headers['Content-Type'] = 'application/json';
      init.body = JSON.stringify(body === null || body === undefined ? {} : body);
    }
    var response;
    try {
      response = await fetch(path, init);
    } catch (error) {
      throw new Error('无法连接网关：' + (error && error.message ? error.message : error));
    }
    var text = await response.text();
    if (response.status === 401 && !retried) {
      // by error。panel_login_required： already 、 need ——
      // use refresh new （access when use none）， not 
      // ； its 401 = ， Key 。
      var probe = null;
      try { probe = JSON.parse(text); } catch (e) { /* JSON */ }
      var errorType = probe && probe.error && probe.error.type;
      if (errorType === 'panel_login_required') {
        var refreshed = await tryRefresh();
        if (refreshed) return httpCall(method, path, body, true, selectedKey);
        // failure = will no ：，
        // otherwise down token， is 1 failure。
        clearPanelAuth();
        window.location.href = '/login';
        // will ，return 1 
        return new Promise(function () {});
      }
      var key = selectedKey === undefined ? await askForKey() : null;
      if (key) {
        storeKey(key);
        return httpCall(method, path, body, true, selectedKey);
      }
    }
    var payload;
    try { payload = JSON.parse(text); } catch (e) {
      throw new Error(text.trim() || '本地代理返回了空响应（HTTP ' + response.status + '）');
    }
    var ok = response.status >= 200 && response.status < 300;
    var success = typeof payload.success === 'boolean' ? payload.success : undefined;
    if (!ok || success === false) {
      var detail = payload.error != null ? payload.error
        : payload.message != null ? payload.message
        : payload.msg != null ? payload.msg
        : 'HTTP ' + response.status;
      throw asError(typeof detail === 'string' ? detail : (detail && detail.message) || JSON.stringify(detail));
    }
    return payload && Object.prototype.hasOwnProperty.call(payload, 'data') ? payload.data : payload;
  }

  function call(method, path, body) {
    return httpCall(method, path, body === undefined ? null : body);
  }

  // access after 。 in refresh cookie to not 
  // server， as `x-panel-refresh` ， and `x-panel-auth-mode: body`
  // need new response；response new PANEL_AUTH（ after old refresh already 
  // ， not down failure）。cookie （ no ， not 
  // x-panel-refresh） down Set-Cookie ，body no has ——
  // resp.ok success， not can by failure。
  //
  // ⚠️ ** in when **（ and query 1 ， 1 ）：
  // this `?auth-mode=body` none， is when 1 
  // will response、 and `storePanelAuth` localStorage ——
  // 「 not JS can 」 this in 1 after no 。server 
  // ****（ `api::panel::tokens_in_body`）， not when will by 
  // cookie ： → cookie（responsenone）； in → not ，
  // response。 as not ， has 1 。
  // ── and 401 ───────────────────────────────────────
  // access 过期那一刻，面板首屏一次会并发十几条 /api/*， when to 401、
  // when 1 refresh —— ， to success， its up 
  // 「 already 」。server for 1 minutes has （access::ROTATION_GRACE_MS），
  // that up down （）； 1 and in this and ：
  // 1 when 1 request， its etc 1 。
  var refreshInFlight = null;
  async function tryRefresh() {
    if (refreshInFlight) return refreshInFlight;
    refreshInFlight = doRefresh().finally(function () { refreshInFlight = null; });
    return refreshInFlight;
  }

  /**
   * before （ms）： to before this 。
   *
   * minutes「 this 1 request in to 」 ， 2 when 
   * —— not will as when or 1 request before （ that will 
   * 1 ， and server ）。
   */
  var REFRESH_AHEAD_MS = 2 * 60 * 1000;

  /**
   * to before ： 「access → 401 → 」 this path before to before 。
   *
   * ── Why need has this 1 ────────────────────────────────────────
   * 401 when ，**** access 1 401。
   * is （2.5 seconds 1 ， ui/zcode-captcha-pool.js），
   * is 1 up that —— is 2 when when 1 401、 1 、 +1。
   * is success （ use none）， that by old error，。
   * server already by 「refresh 」 this 401 verbose ； this from 
   * request： use 1 ， not this 1 this 。
   *
   * ── failure ──────────────────────────────────────────────
   * 1 ****，request：`doRefresh` return false can can is will 
   * no （ this 401 → ）， can can is 1 （ down 1 will ）。
   * in this will 「 1 down 」 1 ， （ will valid）
   * has server —— to has 401 。
   *
   * not `expiresIn` when （ old server、 or from no ）`panelExpiresAt` is 0，
   * this ， as and before 1 。
   */
  async function ensureFreshSession() {
    // ： 1 can can ， by when that old will 1 
    var expiresAt = currentExpiry();
    if (!expiresAt) return;
    if (Date.now() < expiresAt - REFRESH_AHEAD_MS) return;
    // need this —— 401 will tryRefresh，
    // down no has （ and `/api/panel/refresh` is public， not will 401）
    await tryRefresh();
  }

  async function doRefresh() {
    try {
      var headers = { 'Accept': 'application/json' };
      var refreshToken = storedPanelToken('refresh');
      var url = '/api/panel/refresh';
      if (refreshToken) {
        headers['x-panel-refresh'] = refreshToken;
        headers['x-panel-auth-mode'] = 'body';
        // （ + query）： in request when query 
        url += '?auth-mode=body';
      }
      var resp = await fetch(url, { method: 'POST', headers: headers });
      if (!resp.ok) return false;
      var payload = await resp.json();
      var data = payload && payload.data;
      // to when has （server down expiresIn）： cookie down 
      // response no has ，「 when this 」 need —— otherwise
      // can 401 ， is need that 。
      if (data) rememberExpiry(data.expiresIn);
      if (data && typeof data.accessToken === 'string' && data.accessToken
        && typeof data.refreshToken === 'string' && data.refreshToken) {
        storePanelAuth(data.accessToken, data.refreshToken);
      }
      return true;
    } catch (e) {
      return false;
    }
  }

  // ── ──────────────────────────────────────────────
  var stateListeners = new Set();
  var stateEmitTimer = null;
  function emitStateSoon() {
    if (stateEmitTimer) return;
    stateEmitTimer = setTimeout(async function () {
      stateEmitTimer = null;
      try {
        var next = await call('GET', '/api/session');
        stateListeners.forEach(function (cb) { try { cb(next); } catch (e) { console.warn(e); } });
      } catch (e) { /* in failure */ }
    }, 300);
  }

  // ── （ for login:state {active, provider}）──
  var loginActive = false;
  var loginProvider = '';
  var loginListeners = new Set();
  var loginProgressListeners = new Set();
  function loginProgressText(flow) {
    return flow ? [flow.instructions].concat(flow.notices || [], [flow.hint]).filter(function (value) {
      return typeof value === 'string' && value;
    }).join('\n') : '';
  }
  function loginSnapshot() {
    var flow = loginActive && currentFlow;
    return {
      active: loginActive, provider: loginProvider,
      mode: flow ? flow.mode : null, edition: flow ? flow.edition : null,
      instructions: flow ? [flow.instructions].concat(flow.notices || []).filter(Boolean).join('\n') : '',
      hint: flow ? flow.hint || '' : '',
      state: flow && flow.dynamic && flow.state ? flow.state : null,
      manualCallback: !!(flow && flow.dynamic && flow.manualCallback),
      callbackInput: flow && flow.dynamic ? flow.callbackInput : 'url',
    };
  }
  function emitLogin() {
    var payload = loginSnapshot();
    var notify = function (cb) { try { cb(payload); } catch (e) { /* not can can URL */ } };
    loginListeners.forEach(notify);
    loginProgressListeners.forEach(notify);
    // use dialog ， not can as HTML 。
    if (activeOverlay) {
      var text = activeOverlay.overlay.querySelector('[data-a2a-login-progress]');
      if (text) text.textContent = loginProgressText(loginActive && currentFlow);
    }
  }

  var sleep = function (ms) { return new Promise(function (r) { setTimeout(r, ms); }); };

  // when must to down 1 。 for loopback ， Reason not is 
  // 「」， is and not in 1 up （ / Docker ）：
  // use ， to not ， can URL 。
  // 「 etc when 」， use not this 。
  var MANUAL_CALLBACK_TIMEOUT_HINT = '：如果浏览器与网关不在同一台机器上，'
    + '授权完成后的回调会落到你本机而不是网关 —— 请复制浏览器地址栏里的'
    + '完整回调地址，粘回面板重试。';

  async function pollWait(flow) {
    while (Date.now() < flow.deadline) {
      if (currentFlow !== flow || flow.canceled) throw canceledError();
      var wait;
      try {
        wait = await call('GET', '/api/session/login/wait?state=' + encodeURIComponent(flow.state));
      } catch (error) {
        if (currentFlow !== flow || flow.canceled) throw canceledError();
        throw error;
      }
      if (currentFlow !== flow || flow.canceled) throw canceledError();
      if (wait && wait.done === true && !wait.pending) {
        if (wait.error) throw asError(wait.error);
        return wait.session == null ? {} : wait.session;
      }
      if (wait) {
        var hint = typeof wait.hint === 'string' ? wait.hint : wait.pending && wait.pending.hint;
        var instructions = typeof wait.instructions === 'string' ? wait.instructions : flow.instructions;
        var nextHint = typeof hint === 'string' ? hint : flow.hint;
        if (instructions !== flow.instructions || nextHint !== flow.hint) {
          flow.instructions = instructions;
          flow.hint = nextHint;
          emitLogin();
        }
      }
      await sleep(flow.pollIntervalMs);
    }
    var timeoutHint = flow.callbackInput === 'code'
      ? '：请重新发起，并按此方式要求粘贴本人已授权的令牌或一次性 code。'
      : MANUAL_CALLBACK_TIMEOUT_HINT;
    throw new Error('登录等待超时（5 分钟）' + (flow.manualCallback ? timeoutHint : ''));
  }

  /** ： use `canceled` and failure（「 already 」） */
  function canceledError(message) {
    var err = new Error(message || '已取消登录等待');
    err.canceled = true;
    return err;
  }

  /** current this 1 state and 。 1 （/ 1 ），
   * new —— up 1 to state for 。 */
  var currentFlow = null;

  /** old use ； need URL empty， not empty browser。 */
  async function runLoginFlow(provider, startRequest, config) {
    if (loginActive || currentFlow) throw new Error('已有登录流程在进行，请先完成或取消当前登录');
    config = config || {};
    var dynamic = config.dynamic === true;
    var popup = null;
    var flow = {
      state: '', canceled: false, dynamic: dynamic, mode: config.mode || 'embedded', edition: config.edition || 'cn',
      instructions: '', hint: '', notices: [], manualCallback: false, callbackInput: 'url', started: null, callbackTargets: [],
      pollIntervalMs: 3000, deadline: 0,
    };
    currentFlow = flow;
    loginActive = true;
    loginProvider = provider;
    emitLogin();
    if (!dynamic) {
      try {
        popup = window.open('about:blank', 'a2a-login');
        if (popup) popup.opener = null;
      } catch (e) { /* when */ }
    }
    try {
      // after to after ， use POST 。
      var started = await (typeof startRequest === 'function' ? startRequest(flow) : startRequest);
      flow.started = started || {};
      flow.state = String(started && (started.state || started.flow) || '');
      if (!flow.state.trim()) throw new Error('网关未返回登录流程状态');
      if (flow.canceled || currentFlow !== flow) await cancelWebLogin(flow.state);
      var authUrl = started && Object.prototype.hasOwnProperty.call(started, 'url')
        ? started.url : started && started.authUrl;
      if (authUrl == null) authUrl = '';
      if (typeof authUrl !== 'string') throw new Error('网关返回的登录链接格式无效');
      var hasUrl = !!authUrl.trim();
      flow.manualCallback = dynamic && typeof started.manualCallback === 'boolean' ? started.manualCallback
        : dynamic ? !!(flow.method && flow.method.manualCallback) : legacyNeedsManualCallback(provider);
      flow.callbackInput = dynamic && flow.manualCallback
        && (started.callbackInput || flow.method && flow.method.callbackInput) === 'code' ? 'code' : 'url';
      flow.instructions = typeof started.instructions === 'string' && started.instructions
        ? started.instructions : flow.method && flow.method.instructions || '';
      if (flow.method && flow.method.localOnly) {
        flow.notices.push('此方式读取的是网关所在机器的本机客户端账号，不是当前浏览器所在电脑的账号；网页仍可发起。');
      }
      var onlyExternal = started.externalBrowser === true;
      flow.mode = !hasUrl ? 'auto' : 'external';
      if (dynamic && onlyExternal && config.mode === 'embedded' && hasUrl) {
        flow.notices.push('该供应商只支持外部浏览器，授权页将在浏览器标签页中打开，不会伪装成内嵌窗口。');
      }
      var interval = Number(started.pollIntervalMs);
      // pollIntervalMs=0 is ， not /wait； not used for manualCallback。
      flow.pollIntervalMs = Number.isFinite(interval) && interval > 0
        ? Math.max(1000, Math.min(30000, interval)) : 3000;
      flow.deadline = Date.now() + 5 * 60 * 1000;
      if (hasUrl) {
        if (!dynamic) authUrl = buildRaccoonWebAuthUrl(provider, flow.state, authUrl);
        flow.callbackTargets = loginCallbackTargets(authUrl);
        // old no has ， SSO as ； use allowedHosts。
        authUrl = checkedLoginUrl(authUrl, dynamic ? started : null, flow.callbackTargets);
        if (popup && !popup.closed) popup.location.href = authUrl;
        else {
          var opened = false;
          try {
            // empty opener； noopener will in success when return null，
            // not can 「 」。 this 1 in already URL empty and after 。
            var second = window.open('about:blank', '_blank');
            if (second) {
              second.opener = null;
              second.location.replace(authUrl);
              opened = true;
            }
          } catch (e) { /* */ }
          if (!opened) showLinkFallback(authUrl);
        }
      } else if (popup && !popup.closed) {
        popup.close();
        popup = null;
      }
      emitLogin();
      var waiting = pollWait(flow);
      if (!flow.manualCallback) return await waiting;
      // 1 /wait Promise and dialog ，accepted after etc 1 。
      var outcome = await Promise.race([
        waiting.then(function (session) { return { session: session }; },
          function (error) { return { error: error }; }),
        (async function () {
          await sleep(MANUAL_CALLBACK_GRACE_MS);
          if (currentFlow !== flow || flow.canceled) throw canceledError();
          return waitForManualCallback(provider, flow, authUrl, waiting);
        })().then(function (session) { return { session: session }; },
          function (error) { return { error: error }; }),
      ]);
      if (outcome.error) throw outcome.error;
      return outcome.session;
    } catch (error) {
      if (flow.canceled || (error && error.canceled)) {
        var quiet = flow.externalCancel || (error && error.silent);
        return quiet ? { canceled: true, silent: true } : { canceled: true };
      }
      if (flow.state) {
        call('POST', '/api/session/login/cancel', { state: flow.state }).catch(function () {});
      }
      throw error;
    } finally {
      if (popup && !popup.closed) { try { popup.close(); } catch (e) { /* none */ } }
      if (currentFlow === flow) {
        closeWebOverlay(true);
        currentFlow = null;
        loginActive = false;
        loginProvider = '';
        emitLogin();
      }
    }
  }

  // ── has ──────────────────────────────
  var SHELL_UNAVAILABLE = '该操作在网页端不可用（仅桌面端支持）';

  var shellCommands = {
    // and edition/mode/provider and ， invoke options。
    start_login: function (args) {
      args = args || {};
      return window.workbuddyDesktop.startLogin(args.edition, args.mode, args.provider,
        args.socialRestore, args.options);
    },
    login_state: function () { return window.workbuddyDesktop.getLoginState(); },
    cancel_login: function () { return window.workbuddyDesktop.cancelLogin(); },
    api_request: function (args) {
      var request = (args && args.request) || {};
      return httpCall(request.method || 'GET', request.path || '/', request.body, false,
        request.apiKey === undefined ? undefined : String(request.apiKey));
    },
    check_update: function () {
      // no has 「 」 this ： not current，server by 
      // （ api/update check ）。return selfUpdate /
      // serverAsset —— 「 1 update」 is 。
      return call('GET', '/api/update/check');
    },
    get_update_status: function () { return call('GET', '/api/update/status'); },
    // down ： down in server directory（/data/updates）， by 。
    // this （ need after ）， —— 
    // 1 ， not 。
    download_update: function (args) {
      var payload = args || {};
      return call('POST', '/api/update/download', {
        url: String(payload.url || ''),
        name: payload.name != null ? String(payload.name) : '',
      });
    },
    update_progress: function () { return call('GET', '/api/update/progress'); },
    cancel_update: function () { return call('POST', '/api/update/cancel', {}); },
    backend_status: function () {
      return Promise.resolve({ ready: true, port: null, portFromEnv: false, failure: null });
    },
    port_occupant: function () { return Promise.resolve(null); },
    get_app_settings: function () {
      // set（ to / / ） in no has ，default
      return Promise.resolve({ closeToTray: false, autostart: false, proxyPort: 0, lightweightMode: false });
    },
    open_release_page: function (args) {
      if (args && args.url) { try { window.open(args.url, '_blank'); } catch (e) { /* none */ } }
      return Promise.resolve({ url: (args && args.url) || '' });
    },
    export_accounts: function () {
      return downloadFile('GET', '/api/accounts/export', 'agent2api-accounts.json');
    },
    export_account_files: function (args) {
      return exportAccountFilesViaDownload(args && args.selection);
    },
    export_logs: function (args) {
      return downloadFile('GET', '/api/logs/download' + toQuery(args && args.query), 'agent2api-logs.jsonl');
    },
  };

  /** 「file for 」： Blob down */
  async function downloadFile(method, path, filename) {
    var headers = authHeaders({ 'Accept': '*/*' });
    var response = await fetch(path, { method: method, headers: headers });
    if (!response.ok) throw new Error('failure（HTTP ' + response.status + '）');
    var blob = await response.blob();
    // API response { success, data } ： before ， data ——
    // and file 1 ；， this file when no has 
    // accounts， will to 「 accounts 」。 to use 
    //（set use 「 already N 」， not to will 「 no has can 」）
    var summary = {};
    try {
      var body = JSON.parse(await blob.text());
      if (body && body.success === true && body.data && typeof body.data === 'object') {
        var accounts = Array.isArray(body.data.accounts) ? body.data.accounts : null;
        var providers = Array.isArray(body.data.customProviders) ? body.data.customProviders : null;
        if (accounts) summary.count = accounts.length;
        if (providers) summary.customProviders = providers.length;
        blob = new Blob([JSON.stringify(body.data, null, 2)], { type: 'application/json' });
      }
    } catch (e) { /* JSON response（ down ） by */ }
    saveBlob(blob, filename);
    summary.saved = true;
    return summary;
  }

  /** ：Blob → when object URL → <a download> */
  function saveBlob(blob, filename) {
    var url = URL.createObjectURL(blob);
    var link = document.createElement('a');
    link.href = url;
    link.download = filename;
    document.body.appendChild(link);
    link.click();
    link.remove();
    setTimeout(function () { URL.revokeObjectURL(url); }, 5000);
  }

  async function exportAccountFilesViaDownload(selection) {
    var data = await call('POST', '/api/accounts/export-files', selection || {});
    if (!data || !Array.isArray(data.files)) throw new Error('账号导出响应格式无效');
    var failed = Array.isArray(data.failed) ? data.failed.slice() : [];
    var count = 0;
    for (var index = 0; index < data.files.length; index++) {
      var item = data.files[index];
      try {
        var blob = new Blob([JSON.stringify(item.content, null, 2)], { type: 'application/json' });
        saveBlob(blob, item.fileName);
        count++;
      } catch (error) {
        failed.push({ id: item.id, name: item.name, error: asError(error).message });
      }
      if (index + 1 < data.files.length) {
        await new Promise(function (resolve) { setTimeout(resolve, 150); });
      }
    }
    return { total: data.total, count: count, failed: failed, downloadStarted: count > 0 };
  }

  /** / API Key request（ down 、 up this JSON request use ） */
  function authHeaders(extra) {
    var headers = extra || {};
    if (apiKey) headers['x-api-key'] = apiKey;
    var panelAccess = storedPanelToken('access');
    if (panelAccess) {
      headers['x-panel-token'] = panelAccess;
      headers['Authorization'] = 'Bearer ' + panelAccess;
    }
    return headers;
  }

  /**
   * JSON request（ down / up ） ，401 and `httpCall` 1 ：
   * panel_login_required → ， its → Key 。
   *
   * `buildInit` is not is for ： after already ，must new 。
   * 401 response（error JSON），`clone()` not after path。
   */
  async function fetchAuthed(path, buildInit, retried) {
    // and httpCall 1 to ： to before ， this JSON request 401
    if (!retried) await ensureFreshSession();
    var response = await fetch(path, buildInit());
    if (response.status !== 401 || retried) return response;
    var probe = null;
    try { probe = JSON.parse(await response.clone().text()); } catch (e) { /* JSON */ }
    var errorType = probe && probe.error && probe.error.type;
    if (errorType === 'panel_login_required') {
      var refreshed = await tryRefresh();
      if (refreshed) return fetchAuthed(path, buildInit, true);
      clearPanelAuth();
      window.location.href = '/login';
      return new Promise(function () {});
    }
    var key = await askForKey();
    if (!key) return response;
    storeKey(key);
    return fetchAuthed(path, buildInit, true);
  }

  /**
   * （）：POST parameter， down .a2ab 。
   *
   * return and `commands.rs::export_database` for 
   * （`{ bytes, tables, file }`），set success use 1 。
   * in after `X-Backup-Tables` response，response。
   */
  async function exportDatabase(includeLogs, password) {
    var response = await fetchAuthed('/api/backup/download', function () {
      return {
        method: 'POST',
        cache: 'no-store',
        headers: authHeaders({ 'Accept': '*/*', 'Content-Type': 'application/json' }),
        body: JSON.stringify({ includeLogs: includeLogs !== false, password: password || '' }),
      };
    });
    if (!response.ok) {
      // failure when response is （error in ）， to 1 error 1 
      var failure = null;
      try { failure = JSON.parse(await response.text()); } catch (e) { /* JSON */ }
      var detail = failure && failure.error != null ? failure.error : ('HTTP ' + response.status);
      throw asError(typeof detail === 'string' ? detail : (detail && detail.message) || '导出失败');
    }
    var tables = [];
    try { tables = JSON.parse(response.headers.get('x-backup-tables') || '[]'); } catch (e) { tables = []; }
    var includedKeys = [];
    try { includedKeys = JSON.parse(response.headers.get('x-backup-keys') || '[]'); } catch (e) { includedKeys = []; }
    var filename = fileNameFromDisposition(response.headers.get('content-disposition'))
      || ('agent2api-backup-' + downloadStamp() + '.a2ab');
    var blob = await response.blob();
    saveBlob(blob, filename);
    return { bytes: blob.size, tables: tables, file: filename, saved: true, encrypted: true, includedKeys: includedKeys };
  }

  /**
   * ： .a2ab or old .sql， and File multipart up 。
   * not can file.text()，otherwise will in 。
   * return and `commands.rs::import_database` 1 
   * （`{ tables, safetyBackup, warning? }`）。
   */
  async function importDatabase(password) {
    var picked = await pickFile('.a2ab,.sql');
    if (picked.canceled) return { canceled: true };
    var file = picked.file;
    var form = new FormData();
    form.append('password', password || '');
    form.append('backup', file);
    var response = await fetchAuthed('/api/backup/upload', function () {
      return {
        method: 'POST',
        cache: 'no-store',
        headers: authHeaders({ 'Accept': 'application/json' }),
        body: form,
      };
    });
    var text = await response.text();
    var payload = null;
    try { payload = JSON.parse(text); } catch (e) { /* JSON（ 413） */ }
    var ok = response.status >= 200 && response.status < 300;
    var success = payload && typeof payload.success === 'boolean' ? payload.success : undefined;
    if (!ok || success === false) {
      if (response.status === 413) {
        throw new Error('备份文件超过 2GB 上限，请改用精简导出（不含日志）后重试');
      }
      var detail = payload && payload.error != null ? payload.error : ('HTTP ' + response.status);
      throw asError(typeof detail === 'string' ? detail : (detail && detail.message) || '恢复失败');
    }
    return payload && Object.prototype.hasOwnProperty.call(payload, 'data') ? payload.data : payload;
  }

  /** `Content-Disposition` filename（ not to returnempty， by use ） */
  function fileNameFromDisposition(value) {
    if (!value) return '';
    var match = /filename\*?=(?:UTF-8'')?"?([^";]+)"?/i.exec(value);
    if (!match) return '';
    try { return decodeURIComponent(match[1]); } catch (e) { return match[1]; }
  }

  /** down file when （ and after `backup_api::file_stamp` ， when ） */
  function downloadStamp() {
    var now = new Date();
    function pad(value) { return (value < 10 ? '0' : '') + value; }
    return String(now.getFullYear()) + pad(now.getMonth() + 1) + pad(now.getDate())
      + '-' + pad(now.getHours()) + pad(now.getMinutes()) + pad(now.getSeconds());
  }

  /** total（ when UI internals.invoke('api_request')） */
  function invokeShell(command, args) {
    var handler = shellCommands[command];
    if (handler) return handler(args);
    // file for ： need file， to 1 
    if (command === 'import_accounts') return importAccountsViaFile();
    return Promise.reject(new Error(
      command === 'run_installer'
        ? '网页端不启动安装程序：容器内的更新请用「一键更新」（设置 → 软件更新）'
        : SHELL_UNAVAILABLE
    ));
  }

  /**
   * 「file」： no has file for ， use <input type=file> 。
   *
   * （， when ）：
   * · ** not can ** `display:none` —— iOS Safari will ignore for file input 
   * `click()`，「 no 」。（ + ）。
   * · in 「」 when `change` not ，Promise 、 by in 
   * 「 in …」。 use window focus / visibilitychange ： to after no to 
   * file by （ and return `{canceled:true}` 1 ）。
   * · `accept` ：Android file for MIME （application/json）
   * supports not 1 ， up can can .json 。
   *
   * 「 to file」， and to use —— need request
   * File（ up ）， need ， after no has 。
   * and `commands.rs::import_accounts` for ， buildImportPayload。
   */
  function pickFile(accept) {
    return new Promise(function (resolve, reject) {
      var input = document.createElement('input');
      input.type = 'file';
      input.accept = accept;
      input.setAttribute('aria-hidden', 'true');
      input.tabIndex = -1;
      // display:none（ by up ）
      input.style.position = 'fixed';
      input.style.left = '-9999px';
      input.style.top = '0';
      input.style.width = '1px';
      input.style.height = '1px';
      input.style.opacity = '0';

      var settled = false;      // 结果只落定一次
      var picked = false;       // change 是否已同步拿到文件（压过取消兜底）
      var settleTimer = null;

      function cleanup() {
        window.removeEventListener('focus', onReturn);
        document.removeEventListener('visibilitychange', onVisibility);
        if (settleTimer !== null) { window.clearTimeout(settleTimer); settleTimer = null; }
        input.remove();
      }
      function settleCancel() {
        if (settled) return;
        settled = true;
        cleanup();
        resolve({ canceled: true });
      }
      function onReturn() {
        if (settleTimer !== null) return;
        // to 1 when change，「 no 」= 
        settleTimer = window.setTimeout(function () {
          settleTimer = null;
          if (!picked) settleCancel();
        }, 400);
      }
      function onVisibility() {
        if (document.visibilityState === 'visible') onReturn();
      }

      input.addEventListener('change', function () {
        var file = input.files && input.files[0];
        if (!file) { settleCancel(); return; }
        picked = true;          // 同步置位：兜底那条链不再当取消
        settled = true;
        cleanup();
        resolve({ canceled: false, file: file });
      });

      document.body.appendChild(input);
      window.addEventListener('focus', onReturn);
      document.addEventListener('visibilitychange', onVisibility);
      input.click();
    });
  }

  /** 「」： .json → → `POST /api/accounts/import` */
  async function importAccountsViaFile() {
    var picked = await pickFile('.json');
    if (picked.canceled) return { canceled: true };
    var text = await picked.file.text();
    var payload;
    try {
      payload = buildImportPayload(JSON.parse(text));
    } catch (error) {
      throw asError(error);
    }
    return call('POST', '/api/accounts/import', payload);
  }

  /**
   * use file `POST /api/accounts/import` request。
   * and `commands.rs::import_accounts` + `unwrap_envelope_file` ：
   * 「file」（ accounts）、「」、 and old `{success,data}`
   * file —— ， 1 and up 。
   */
  function buildImportPayload(parsed) {
    var document_ = parsed;
    if (document_ && !Array.isArray(document_)
      && !document_.accounts && document_.data && document_.data.accounts) {
      document_ = document_.data;
    }
    var accounts = Array.isArray(document_) ? document_ : (document_ && document_.accounts);
    if (!Array.isArray(accounts) && document_ && typeof document_.id === 'string'
      && document_.id.trim() && !Object.prototype.hasOwnProperty.call(document_, 'accounts')) {
      accounts = [document_];
    }
    if (!Array.isArray(accounts)) {
      throw new Error('文件里没有 accounts 数组，也不是单账号导出文件');
    }
    var body = { accounts: accounts };
    if (document_ && !Array.isArray(document_) && document_.customProviders) {
      body.customProviders = document_.customProviders;
    }
    return body;
  }

  // ── for ─────────────────────────────────────────
  // __TAURI_INTERNALS__ ： internals.invoke('api_request')
  // （sms-login and postAccount） in 。
  window.__TAURI_INTERNALS__ = {
    invoke: function (command, args) {
      return invokeShell(command, args).catch(asError).then(function (value) {
        if (value instanceof Error) throw value;
        return value;
      });
    },
    transformCallback: function (callback) {
      // ： no has push ，return 1 none id can 
      return 0;
    },
  };

  window.workbuddyDesktop = {
    // ： can （ etc in web down ）
    platform: 'web',

    // ── will ──
    getState: function () { return call('GET', '/api/session'); },
    startLogin: function (edition, mode, provider, socialRestore, options) {
      var target = String(provider || 'workbuddy').trim();
      var dynamic = hasMethodIndex(options);
      if (target === 'monkeycode' || target === 'monkeycode-intl') {
        return Promise.reject(new Error('MonkeyCode 自动网页登录仅限桌面端；浏览器面板请登录对应官网后粘贴 session。'));
      }
      // has old redirect/； not can provider 。
      if (!dynamic && target === 'raccoon') {
        var problem = raccoonCallbackOriginProblem();
        if (problem) return reportRaccoonCallbackProblem(problem);
      }
      return runLoginFlow(target, async function (flow) {
        if (dynamic) flow.method = await selectedLoginMethod(target, options);
        if (flow.canceled) throw canceledError();
        var body = { edition: edition || 'cn', provider: target };
        // not socialRestore； for /（ proxy:null）。
        if (options !== undefined) body.options = options;
        if (dynamic) {
          body.mode = mode || 'embedded';
          body.socialRestore = socialRestore === true;
          // current API ；options to new 。
          body.methodIndex = options.methodIndex;
          if (Object.prototype.hasOwnProperty.call(options, 'inputs')) body.inputs = options.inputs;
          if (Object.prototype.hasOwnProperty.call(options, 'proxy')) body.proxy = options.proxy;
        } else if (options && Object.prototype.hasOwnProperty.call(options, 'proxy')) {
          // old （ ） can ： 
          // is `proxy`（ api::session legacy_proxy）。 this 1 ，
          // none up 。
          body.proxy = options.proxy;
        }
        return call('POST', '/api/session/login/start', body);
      }, { dynamic: dynamic, mode: mode || 'embedded', edition: edition || 'cn' });
    },
    submitLoginCallback: function (state, callbackUrl) {
      return call('POST', '/api/session/login/callback', {
        state: String(state == null ? '' : state),
        callbackUrl: String(callbackUrl == null ? '' : callbackUrl),
      });
    },
    getLoginState: function () {
      return Promise.resolve(loginSnapshot());
    },
    cancelLogin: async function () {
      // 1 ； by 。
      //
      // 1 not can （ 1 ， is 「 etc 」 after that 
      // in up ， use new none）：
      // ① to current this 1 —— pollWait down 1 （≤3 seconds） will 
      // canceled，10 seconds that 1 will in when when ；
      // ② ： has when cancel()， /cancel 
      // server（Trae this 1 ）；
      // ③ no has （ in ） when 1 /cancel。
      //
      // in this 1 （externalCancel）： that 1 in when 
      // already toast 1 ， this not need 。
      var wasActive = loginActive;
      var state = currentFlow && currentFlow.state;
      if (currentFlow) { currentFlow.canceled = true; currentFlow.externalCancel = true; }
      if (activeOverlay && activeOverlay.cancelable) {
        // will /cancel（ cancelWebLogin）， this not 
        closeWebOverlay();
      } else {
        closeWebOverlay(true);
        if (state) {
          call('POST', '/api/session/login/cancel', { state: state }).catch(function () {});
        }
      }
      loginActive = false;
      loginProvider = '';
      emitLogin();
      return wasActive;
    },
    startAutoclawOauthLogin: function (state, authUrl, mode) {
      if (!state || !authUrl) return Promise.reject(new Error('缺少授权参数（state / authUrl）'));
      // AutoClaw OAuth and bridge use `{ok, session}` 。
      // to is session，return will before as 。
      // use when runLoginFlow to is `{canceled:true}`：must
      // `ok:false`，otherwise will 「 to 1 canceled session」。
      return runLoginFlow('autoclaw-intl', function () { return Promise.resolve({ state: state, authUrl: authUrl }); },
        { mode: mode || 'embedded', edition: 'intl' })
        .then(function (session) {
          if (session && session.canceled) return { ok: false, canceled: true };
          return { ok: true, session: session };
        });
    },
    getAutoclawOauthCaptchaConfig: function (provider) {
      return call('POST', '/api/session/login/oauth/captcha-config',
        provider ? { provider: String(provider) } : {});
    },
    startAutoclawOauth: function (provider, vendor, captchaVerifyParam) {
      return call('POST', '/api/session/login/oauth/start', {
        provider: provider ? String(provider) : undefined,
        vendor: String(vendor || ''),
        captchaVerifyParam: String(captchaVerifyParam || ''),
      });
    },
    // ── ZCode when （ ，path）──────
    // and up AutoClaw 1 ：，path and request
    // by this for after `api::zcode_claim` 1 。
    //
    // （ `api/zcode_claim.rs` module header）：
    // · captchaConfig config（ before use SDK）；
    // return `{enabled:false}` up not need —— before ** not this **；
    // · preview ，return `{plans:[...], deployed}`；
    // `deployed:false` = （ before ， not is error）；
    // · claim 。`captchaVerifyParam` ** can as empty**： up not need 
    // when not that （empty will invalid， `claim::claim` ）。
    // **failure 200**， by `ok:false` + `failure` （ before 
    // ）；`claimedAt` is when ， before by 「 already 」。
    zcodeClaimCaptchaConfig: function (accountId) {
      return call('POST', '/api/accounts/' + encodeURIComponent(String(accountId || ''))
        + '/zcode-claim/captcha-config');
    },
    zcodeClaimPreview: function (accountId) {
      return call('POST', '/api/accounts/' + encodeURIComponent(String(accountId || ''))
        + '/zcode-claim/preview');
    },
    zcodeClaim: function (accountId, planId, captchaVerifyParam, captchaRegion) {
      return call('POST', '/api/accounts/' + encodeURIComponent(String(accountId || ''))
        + '/zcode-claim', {
        planId: planId ? String(planId) : '',
        captchaVerifyParam: String(captchaVerifyParam || ''),
        captchaRegion: captchaRegion ? String(captchaRegion) : '',
      });
    },
    // ── ZCode （ api/zcode_captcha.rs）──────
    // and up ** not is 1 **： that 「」（ use ）； this 
    // **** —— request need 1 ， can by 
    // after （ 1 use ui/zcode-captcha-pool.js）。
    //
    // ★ this **must** in this to web ， by is in by ：
    // `window.workbuddyDesktop`， by bridge.rs 、headless by this 
    // 。 to ， is headless（Docker / ） down 
    // to 「 no has this 」→ → is empty ，
    // unavailable（Issue #163）。 in ****（ SDK in 
    // up ）， headless need has can —— and 1 。
    zcodeClaimVerification: function (payload) {
      return call('POST', '/api/zcode/claim-verification', payload);
    },
    zcodeCaptchaStats: function () {
      return call('GET', '/api/zcode/captcha');
    },
    pushZcodeCaptchaTokens: function (tokens) {
      return call('POST', '/api/zcode/captcha', {
        tokens: Array.isArray(tokens) ? tokens : [],
      });
    },
    onLoginState: function (callback) {
      loginListeners.add(callback);
      return function () { loginListeners.delete(callback); };
    },
    onLoginProgress: function (callback) {
      loginProgressListeners.add(callback);
      return function () { loginProgressListeners.delete(callback); };
    },
    refreshSession: async function () {
      await call('POST', '/api/session/refresh', {});
      return call('GET', '/api/session');
    },
    logout: async function () {
      await call('POST', '/api/session/logout', {});
      return call('GET', '/api/session');
    },

    // ── config ──
    getConfig: function () { return call('GET', '/api/config'); },
    saveConfig: function (payload) { return call('POST', '/api/config', payload); },

    // ── ──
    // （ and bridge for ）：`{accounts: {providerId: accountId},
    // providers: [id, ...]}` —— 「」 and new in 。
    // this `{}`， optional 1 ：（ up not to 
    // ， 1 「」 ）， not 。
    refreshModels: function (payload) { return call('POST', '/api/models/refresh', payload || {}); },
    getModelManage: function () { return call('GET', '/api/models/manage'); },
    getClientIdentification: function () { return call('GET', '/api/client-identification'); },
    saveClientIdentification: function (payload) { return call('PUT', '/api/client-identification', payload); },
    previewClientIdentification: function (payload) { return call('POST', '/api/client-identification/preview', payload); },
    getCompactRoutes: function () { return call('GET', '/api/models/compact-routes'); },
    saveCompactRoutes: function (payload) { return call('PUT', '/api/models/compact-routes', payload); },
    setModelState: function (payload) { return call('POST', '/api/models/state', payload); },
    // 4 / 5 parameter（ etc / ） by 「 has no has 」 is request：
    // after by 「request has no has this 」，undefined not will JSON
    addModelMapping: function (alias, target, provider, reasoning, enabled, create) {
      var payload = { alias: alias, target: target, provider: provider };
      if (reasoning !== undefined) payload.reasoning = reasoning;
      if (enabled !== undefined) payload.enabled = enabled;
      if (create !== undefined) payload.create = create;
      return call('POST', '/api/models/mappings', payload);
    },
    removeModelMapping: function (alias, target, provider) {
      return call('POST', '/api/models/mappings/remove', { alias: alias, target: target, provider: provider });
    },
    addCustomModel: function (provider, id) { return call('POST', '/api/models/custom', { provider: provider, id: id }); },
    removeCustomModel: function (provider, id) { return call('POST', '/api/models/custom/remove', { provider: provider, id: id }); },
    // can （）：`capabilities` —— not to = not 、
    // null = 、 to = 。 /api/custom-providers/models
    // （ after handler ）， not this 。
    setModelCapabilities: function (provider, id, capabilities) {
      return call('POST', '/api/models/capabilities', { provider: provider, id: id, capabilities: capabilities });
    },
    // ： up 、 will ；payload （ and bridge for ），
    // optional（account_id / prompt / system_prompt / reasoning / stream / test_id）。
    // failurereturn 2xx —— up error in return status / error 
    testModel: function (payload, selectedKey) { return httpCall('POST', '/api/models/test', payload || {}, false, String(selectedKey || '')); },
    getModelApiKeys: function () { return call('GET', '/api/model-access/keys'); },

    // ── API Key（ ）──
    getKeys: function () { return call('GET', '/api/keys'); },
    createKey: function (payload) { return call('POST', '/api/keys', payload || {}); },
    updateKey: function (id, patch) { return call('PATCH', '/api/keys/' + encodeURIComponent(id), patch); },
    deleteKey: function (id) { return call('DELETE', '/api/keys/' + encodeURIComponent(id)); },

    // ── ──
    getAccounts: function () { return call('GET', '/api/accounts'); },
    getAccountSorting: function () { return call('GET', '/api/accounts/sorting'); },
    saveAccountSorting: function (payload) { return call('PUT', '/api/accounts/sorting', payload || {}); },
    switchAccount: function (id) { return call('POST', '/api/accounts/current', { id: id }); },
    refreshAccountToken: function (id) { return call('POST', '/api/accounts/refresh', id ? { id: id } : {}); },
    removeAccount: function (id) { return call('DELETE', '/api/accounts/' + encodeURIComponent(id)); },
    updateAccount: function (id, patch) { return call('PATCH', '/api/accounts/' + encodeURIComponent(id), patch); },
    // （ ； and bridge.rs for ）。
    // ★ this in ： use is `bridge?.reorderAccounts?.(ids)`
    // —— optional use ， this 1 not 、 is empty， is Docker / up 
    // 「✅ already 」， new old （ from no ）。
    // is 「 1 by 」（ account_store::priority），
    // / by before id 1 up ， after 0、1、2…，
    // this ， not 1 （empty and 1 ）。
    reorderAccounts: function (queue) {
      return call('POST', '/api/accounts/reorder', {
        queue: Array.isArray(queue) ? queue.map(function (item) { return String(item); }) : [],
      });
    },
    moveAccount: function (id, direction) {
      return call('POST', '/api/accounts/' + encodeURIComponent(id) + '/move', {
        direction: direction === 'down' ? 'down' : 'up',
      });
    },
    clearRateLimits: function (id, model) {
      return call('POST', '/api/accounts/' + encodeURIComponent(id) + '/rate-limits/clear',
        model ? { model: model } : {});
    },
    batchAccounts: function (payload) {
      payload = payload || {};
      var body = {
        action: String(payload.action || ''),
        ids: Array.isArray(payload.ids) ? payload.ids : [],
      };
      if (Object.prototype.hasOwnProperty.call(payload, 'proxy')) body.proxy = payload.proxy;
      if (Object.prototype.hasOwnProperty.call(payload, 'maxConcurrent')) {
        body.maxConcurrent = payload.maxConcurrent;
      }
      if (Object.prototype.hasOwnProperty.call(payload, 'usageQuery')) body.usageQuery = payload.usageQuery;
      if (Object.prototype.hasOwnProperty.call(payload, 'limiterGroups')) body.limiterGroups = payload.limiterGroups;
      if (Object.prototype.hasOwnProperty.call(payload, 'limiters')) body.limiters = payload.limiters;
      if (Object.prototype.hasOwnProperty.call(payload, 'limiterMode')) body.limiterMode = payload.limiterMode;
      return call('POST', '/api/accounts/batch', body);
    },

    // ── by （ and bridge.rs for ， 1 will 「」）──
    getRoutingConfig: function () { return call('GET', '/api/routing'); },
    getRoutingWorkspace: function (model) { return call('GET', '/api/routing/workspace' + toQuery({ model: model })); },
    saveRoutingProfile: function (payload) { return call('PUT', '/api/routing/profile', payload || {}); },
    getRoutingGroups: function () { return call('GET', '/api/routing/groups'); },
    saveRoutingGroup: function (payload) { return call('PUT', '/api/routing/groups', payload); },
    previewRouting: function (payload) { return call('POST', '/api/routing/preview', payload); },
    getRoutingSticky: function (model) { return call('GET', '/api/routing/sticky' + toQuery({ model: model })); },
    clearRoutingSticky: function (payload) { return call('POST', '/api/routing/sticky/clear', payload || {}); },
    getRoutingReplays: function (query) { return call('GET', '/api/routing/replays' + toQuery(query)); },
    getRoutingFlow: function (query) { return call('GET', '/api/routing/flow' + toQuery(query)); },
    getRoutingTrace: function (query) { return call('GET', '/api/routing/trace' + toQuery(query)); },
    getRoutingTraceRequest: function (query) { return call('GET', '/api/routing/trace/request' + toQuery(query)); },
    setRoutingConfig: function (patch) { return call('PUT', '/api/routing', patch || {}); },
    getRoutingQueue: function (model) {
      return call('GET', '/api/routing/queue' + (model ? '?model=' + encodeURIComponent(model) : ''));
    },

    // ── ──
    getProxies: function () { return call('GET', '/api/proxies'); },
    testProxy: function (payload) {
      var body = {};
      if (payload && typeof payload === 'object' && !Array.isArray(payload)) {
        if (typeof payload.id === 'string' && payload.id) body.id = payload.id;
        else if (Object.prototype.hasOwnProperty.call(payload, 'proxy')) body.proxy = payload.proxy;
      }
      return call('POST', '/api/proxies/test', body);
    },
    // Agent（ AI client config； and bridge.rs for 
    // in —— 1 when 1 down will 「」。
    // api::agents module header：return {clients}，）
    getAgents: function () { return call('GET', '/api/agents'); },
    saveAgentProfiles: function (payload) { return call('POST', '/api/agents/profiles', payload || {}); },
    applyAgentProfile: function (payload) { return call('POST', '/api/agents/apply', payload || {}); },
    previewAgentProfile: function (payload) { return call('POST', '/api/agents/preview', payload || {}); },
    restoreAgentClient: function (payload) { return call('POST', '/api/agents/restore', payload || {}); },
    importAgentClient: function (payload) { return call('POST', '/api/agents/import', payload || {}); },
    // （「」； and bridge.rs for in ，
    // api::proxies module header —— 1 when 1 down will 
    // 「」， that 1 has ）
    getProxyPool: function () { return call('GET', '/api/proxies/pool'); },
    createProxyPoolItem: function (payload) { return call('POST', '/api/proxies/pool', payload || {}); },
    updateProxyPoolItem: function (payload) { return call('POST', '/api/proxies/pool/update', payload || {}); },
    setProxyPoolItemEnabled: function (payload) { return call('POST', '/api/proxies/pool/enable', payload || {}); },
    removeProxyPoolItem: function (id) { return call('POST', '/api/proxies/pool/remove', { id: String(id || '') }); },
    testProxyPoolItem: function (id) { return call('POST', '/api/proxies/pool/test', { id: String(id || '') }); },
    syncClashToProxyPool: function () { return call('POST', '/api/proxies/pool/sync-clash', {}); },
    // set（「」 set； and bridge.rs 
    // for in ， api::proxies module header settings ）
    getProxySettings: function () { return call('GET', '/api/proxies/settings'); },
    saveProxySettings: function (payload) { return call('POST', '/api/proxies/settings', payload || {}); },
    // file is ** can **（web no has ）： `unsupported`，
    // path —— `canceled` ， 1 down by not will ， use will as 
    pickFolder: function () { return Promise.resolve({ canceled: true, unsupported: true }); },

    // ── / to ──
    getUsage: function () { return call('GET', '/api/usage'); },
    getCheckinStatus: function () { return call('GET', '/api/checkin/status'); },
    claimCheckin: function () { return call('POST', '/api/checkin', {}); },
    getAllBalances: function (id) {
      return call('GET', '/api/accounts/usage' + (id ? '?id=' + encodeURIComponent(id) : ''));
    },
    getBalancesSnapshot: function () { return call('GET', '/api/accounts/usage/snapshot'); },
    getAccountConnections: function () { return call('GET', '/api/accounts/connections'); },
    checkinAllAccounts: function (id) { return call('POST', '/api/accounts/checkin', id ? { id: id } : {}); },
    // ── Loomy new （ / 1 ）──
    // and `bridge.rs` for ： to after 、 has 
    // （accounts-dialog-onboarding）。
    // `refresh` = 「」→ `?refresh=1` up ； not then after 
    // （ 1 not up ， api::onboarding ）。
    getOnboardingTasks: function (id, refresh) {
      return call('GET', '/api/accounts/' + encodeURIComponent(String(id || '')) + '/onboarding'
        + (refresh ? '?refresh=1' : ''));
    },
    claimOnboardingTasks: function (id) {
      return call('POST', '/api/accounts/' + encodeURIComponent(String(id || '')) + '/onboarding/claim', {});
    },
    // ── WorkBuddy （： / / / ）──
    // and `bridge.rs` **must for in **（ by up Loomy that already ：
    // 1 when 1 will ）。 in server `api::growth`。
    // （60 seconds when ， api::growth module header）。
    getGrowthStatus: function (id) {
      return call('GET', '/api/accounts/' + encodeURIComponent(String(id || '')) + '/growth');
    },
    runGrowthTask: function (id, code, mode) {
      return call('POST', '/api/accounts/' + encodeURIComponent(String(id || '')) + '/growth/task',
        { code: code, mode: mode || 'full' });
    },
    runGrowthSchool: function (id, lottery) {
      return call('POST', '/api/accounts/' + encodeURIComponent(String(id || '')) + '/growth/school',
        { lottery: lottery === true });
    },
    runGrowthTravel: function (id) {
      return call('POST', '/api/accounts/' + encodeURIComponent(String(id || '')) + '/growth/travel', {});
    },

    // ── （AutoClaw / Loomy / Tabbit ）──
    // and `bridge.rs` **must for in **（ by down ZCode that 
    // ）：「 + in 」， by 。
    //
    // ── Why this need by provider （，issue #93）──────
    // Loomy is ** 1 **（ and ， in msgid not is 
    // deviceId）， in server is （`api::session::login_loomy_*`）。
    // that 、 this when ， loomy will AutoClaw
    // ： by AutoClaw 、 AutoClaw， as 
    // label 「Loomy already 」—— no has 1 。
    sendSmsCode: function (input) {
      var isObject = input && typeof input === 'object';
      var phone = String((isObject ? input.phone : input) || '');
      var provider = isObject && input.provider ? String(input.provider) : '';
      var path = provider === 'loomy'
        ? '/api/session/login/loomy/sms/send'
        : provider === 'tabbit'
          ? '/api/session/login/tabbit/sms/send'
          : '/api/session/login/sms/send';
      return call('POST', path, {
        phone: phone,
        provider: provider || undefined,
        // and ：「 before 」，
        // 200 + `{status:"captcha"}` not is 409；uuid used for when 
        // use 1 （ api::session::login_tabbit_sms_send ）。
        supportsCaptcha: isObject && input.supportsCaptcha ? true : undefined,
        uuid: isObject && input.uuid ? String(input.uuid) : undefined,
      });
    },
    // up to 。 no has available， new ：
    // this 1 down by use by ， not will 。 not use `noopener` parameter ——
    // will window.open return null，「 」； after opener。
    openVerifyPage: function (url) {
      var opened = window.open(String(url || ''), '_blank');
      // when reject not is return false： ，
      // not is as already 、 etc 1 not will 。
      if (!opened) return Promise.reject(new Error('浏览器拦截了弹窗'));
      try { opened.opener = null; } catch (error) { /* when not ，ignore */ }
      return Promise.resolve(true);
    },
    verifySmsLogin: function (payload) {
      payload = payload || {};
      var provider = payload.provider ? String(payload.provider) : '';
      var path = provider === 'loomy'
        ? '/api/session/login/loomy/sms/verify'
        : provider === 'tabbit'
          ? '/api/session/login/tabbit/sms/verify'
          : '/api/session/login/sms/verify';
      var body = {
        phone: String(payload.phone || ''),
        code: String(payload.code || ''),
      };
      // deviceId（AutoClaw）/ msgid（Loomy）/ uuid（Tabbit） by 「 has 」。
      // by by to （ ui/sms-login.js SMS_PROFILES），
      // return in must， not can ， not 1 。
      if (payload.deviceId) body.deviceId = String(payload.deviceId);
      if (payload.msgid) body.msgid = String(payload.msgid);
      if (payload.uuid) body.uuid = String(payload.uuid);
      if (payload.name) body.name = String(payload.name);
      if (provider) body.provider = provider;
      return call('POST', path, body);
    },

    // ── when to ──
    getAutoCheckin: function () { return call('GET', '/api/auto-checkin'); },
    saveAutoCheckin: function (patch) { return call('POST', '/api/auto-checkin', patch); },
    runAutoCheckinNow: function () { return call('POST', '/api/auto-checkin/run', {}); },
    // when to ****（ and bridge.rs for ）
    getCheckinSchedules: function () { return call('GET', '/api/checkin-schedules'); },
    saveCheckinSchedule: function (patch) { return call('POST', '/api/checkin-schedules', patch); },
    deleteCheckinSchedule: function (id) {
      return call('POST', '/api/checkin-schedules/' + encodeURIComponent(String(id || '')) + '/delete', {});
    },
    runCheckinSchedule: function (id) {
      return call('POST', '/api/checkin-schedules/' + encodeURIComponent(String(id || '')) + '/run', {});
    },
    // WorkBuddy （ and bridge.rs for ）
    getCheckinKeepalive: function () { return call('GET', '/api/checkin-keepalive'); },
    saveCheckinKeepalive: function (models, apiKeyId) { return call('POST', '/api/checkin-keepalive', { models: models, apiKeyId: apiKeyId }); },
    // when （； and bridge.rs for ）
    getTravelSchedule: function () { return call('GET', '/api/travel-schedule'); },
    saveTravelSchedule: function (patch) { return call('POST', '/api/travel-schedule', patch); },
    runTravelScheduleNow: function () { return call('POST', '/api/travel-schedule/run', {}); },
    // WorkBuddy （mode: full | claim | keepalive）
    runCheckinActivity: function (id, mode) { return call('POST', '/api/checkin-activity', { id: id, mode: mode }); },
    // to in （ and bridge.rs 1 path）
    getCheckinCenter: function () { return call('GET', '/api/checkin-center'); },
    // Qoder UMID （） and （ and bridge.rs for ）。
    // path protected：headless must， can 、 not can fetch。
    getQoderUmid: function () { return call('GET', '/api/qoder-umid'); },
    installQoderUmid: function () { return call('POST', '/api/qoder-umid/install', {}); },
    // to （ / empty；parameter for empty not ， and bridge.rs for ）
    getCheckinRecords: function (params) {
      const query = new URLSearchParams();
      for (const key of Object.keys(params || {})) {
        const value = params[key];
        if (value !== undefined && value !== null && value !== '') query.set(key, String(value));
      }
      const text = query.toString();
      return call('GET', '/api/checkin-records' + (text ? '?' + text : ''));
    },
    clearCheckinRecords: function (params) {
      const query = new URLSearchParams();
      for (const key of Object.keys(params || {})) {
        const value = params[key];
        if (value !== undefined && value !== null && value !== '') query.set(key, String(value));
      }
      const text = query.toString();
      return call('DELETE', '/api/checkin-records' + (text ? '?' + text : ''));
    },

    // ── when ──
    getScheduledTasks: function () { return call('GET', '/api/scheduled-tasks'); },
    saveScheduledTask: function (id, patch) {
      return call('PATCH', '/api/scheduled-tasks/' + encodeURIComponent(String(id || '')), patch);
    },
    runScheduledTask: function (id) {
      return call('POST', '/api/scheduled-tasks/' + encodeURIComponent(String(id || '')) + '/run', {});
    },

    // ── update（check / down / update； has 「」 is has ）──
    checkUpdate: function () { return invokeShell('check_update'); },
    getUpdateStatus: function () { return call('GET', '/api/update/status'); },
    // set「」： and bridge for in （must 1 ，
    // 1 when that 1 will 「」）。 no has 
    // package_info / WebView ， this is **** that ：
    // · runtime.webviewVersion ← （ from UA ， not to to empty）；
    // · app.version not —— no has 「 is 」 this ，
    // after is 1 。
    // parameter（ after ， api::system_info module header）。
    getSystemInfo: function (options) {
      var hide = !!(options && options.hideUsername);
      return call('GET', '/api/system-info' + (hide ? '?hideUsername=1' : ''))
        .then(function (info) {
          if (info && info.runtime) {
            info.runtime.webviewVersion = browserEngineVersion();
          }
          return info;
        });
    },
    // 「update」 （null = ）： and bridge for ， down 
    getUpdateProxy: function () { return call('GET', '/api/update/proxy'); },
    setUpdateProxy: function (payload) { return call('POST', '/api/update/proxy', payload); },
    getGithubAccess: function () { return call('GET', '/api/update/github'); },
    setGithubAccess: function (payload) { return call('POST', '/api/update/github', payload); },
    // GitHub ： {filled, origin}， {token: '…' | null}
    getUpdateToken: function () { return call('GET', '/api/update/token'); },
    setUpdateToken: function (payload) { return call('POST', '/api/update/token', payload); },
    // 「 beta 」： {enabled}， {enabled: true|false}（ and bridge
    // —— and must 1 ， in ）
    getUpdateBeta: function () { return call('GET', '/api/update/beta'); },
    setUpdateBeta: function (payload) { return call('POST', '/api/update/beta', payload); },
    getUpdateChannels: function () { return call('GET', '/api/update/channels'); },
    setUpdateChannels: function (payload) { return call('POST', '/api/update/channels', payload); },
    downloadUpdate: function (payload) { return invokeShell('download_update', payload); },
    updateProgress: function () { return invokeShell('update_progress'); },
    cancelUpdate: function () { return invokeShell('cancel_update'); },
    runInstaller: function (path, restart) {
      return invokeShell('run_installer', { path: String(path || ''), restart: restart !== false });
    },
    openReleasePage: function (url) { return invokeShell('open_release_page', { url: String(url || '') }); },
    // ── update（Linux headless / Docker）： → ──
    // ： down （ up already ）→ installUpdate（ sha256 after 
    // and ui directory）→ restartService（， by restart 
    // ， 1 、 can ）。 AGENT2API_SELF_UPDATE by 
    // after 400 ； by check return selfUpdate is this 。
    installUpdate: function () { return call('POST', '/api/update/install', {}); },
    startSystemdUpdate: function (payload) { return call('POST', '/api/update/systemd', payload); },
    startWindowsServiceUpdate: function (payload) { return call('POST', '/api/update/windows-service', payload); },
    restartService: function () { return call('POST', '/api/update/restart', {}); },

    // ── ──
    getSanitize: function () { return call('GET', '/api/sanitize'); },
    saveSanitize: function (patch) {
      return call('PUT', '/api/sanitize', typeof patch === 'boolean'
        ? { sanitizeBlacklistFingerprints: patch } : patch);
    },

    // ── ──
    // and （src/bridge.rs getDeveloperRoleCompat / saveDeveloperRoleCompat）
    // 1 ： and must 1 ， in 
    //（ 1 getClineHeaders that ）。
    getDeveloperRoleCompat: function () { return call('GET', '/api/developer-role-compat'); },
    saveDeveloperRoleCompat: function (enabled) {
      return call('PUT', '/api/developer-role-compat', { developerRoleCompat: enabled === true });
    },

    // ── Cline （）──
    // and （src/bridge.rs getClineHeaders / saveClineHeaders） 1 。
    // this in ：set use `getClineHeaders` 
    // 「 not is 」， 1 「unavailable」—— after is ，
    // is this 。
    getClineHeaders: function () { return call('GET', '/api/cline/headers'); },
    saveClineHeaders: function (overrides) {
      // （ merge）， and after PUT ； is 
      // 「 after 」。return and GET （ after PUT is that ），
      // 。
      return call('PUT', '/api/cline/headers', { overrides: overrides || {} });
    },

    // ── ZCode client ──
    // and （src/bridge.rs getZcodeVersion / saveZcodeVersion） 1 。
    getZcodeVersion: function () { return call('GET', '/api/zcode/version'); },
    saveZcodeVersion: function (version) {
      return call('PUT', '/api/zcode/version', { version: version == null ? '' : String(version) });
    },

    // ── 网关面跨域访问（/v1/*，default）──
    getCors: function () { return call('GET', '/api/cors'); },
    saveCors: function (enabled) {
      return call('PUT', '/api/cors', { corsEnabled: enabled === true });
    },

    // ── and ──
    getPrompt: function () { return call('GET', '/api/prompt'); },
    savePrompt: function (payload) {
      payload = payload || {};
      return call('PUT', '/api/prompt', {
        promptMode: payload.promptMode != null ? String(payload.promptMode) : null,
        promptFile: payload.promptFile != null ? String(payload.promptFile) : null,
        // （empty = this 1 、file / default）
        promptText: payload.promptText != null ? String(payload.promptText) : null,
        // by ：（ is for ， or null = this 1 ）。
        // by /api/prompt —— this 1 will 1 can can and after 
        promptProviders: payload.promptProviders != null ? payload.promptProviders : null,
        // （ 1 ， is or null）—— 
        promptGateway: payload.promptGateway != null ? payload.promptGateway : null,
        // （`{"<id>": {identity?, stable?, dynamic?} | null}`）
        promptGatewayText: payload.promptGatewayText != null ? payload.promptGatewayText : null,
        clearDegrade: !!payload.clearDegrade,
      });
    },

    // ── ──
    getLogs: function (query) { return call('GET', '/api/logs' + toQuery(query)); },
    getLogStats: function () { return call('GET', '/api/logs/stats'); },
    // （ use ）。 and desktop bridge.rs 
    // **must for ** —— 1 will headless 。
    getPlugins: function () { return call('GET', '/api/plugins'); },
    // （ / / ）。 and desktop bridge.rs 
    // **must for ** —— this 1 ， not to that ，
    // 「 in WebUI client」 this can etc not in 。
    // api::sidebar_layout： `{layout}`（null = no ），。
    getSidebarLayout: function () { return call('GET', '/api/sidebar-layout'); },
    saveSidebarLayout: function (layout) {
      return call('PUT', '/api/sidebar-layout', { layout: layout === undefined ? null : layout });
    },
    clearLogs: function (query) { return call('DELETE', '/api/logs' + toQuery(query)); },
    exportLogs: function (query) { return invokeShell('export_logs', { query: query }); },

    // ── requeststatistics / ──
    // dates（`{start, end}`，`YYYY-MM-DD`） in range=custom when to ，
    // and bridge.rs 1 （toQuery empty）
    getStatsSummary: function (range, dates, options) {
      return call('GET', '/api/stats/summary' + toQuery({
        range: range,
        startDate: dates && dates.start,
        endDate: dates && dates.end,
        apiKey: options && options.apiKey,
      }));
    },
    getStatsRequests: function (query) { return call('GET', '/api/stats/requests' + toQuery(query)); },
    getStatsRequestFilters: function () { return call('GET', '/api/stats/requests/filters'); },
    clearStatsRequests: function (query) { return call('DELETE', '/api/stats/requests' + toQuery(query)); },
    // by id request （「 for 」 ； not to to 404）
    getStatsRequestRaw: function (id) {
      return call('GET', '/api/stats/requests/raw' + toQuery({ id: id }));
    },
    // 1 in request（request 「」 and use ； 
    // invoke no has abort， this is before 1 can in in request ）。
    // ★ and up ZCode 1 ： has this 、web ，
    // use is `?.` optional use —— not will ， will 「 no 」。
    terminateStatsRequest: function (id) {
      return call('POST', '/api/stats/requests/terminate' + toQuery({ id: id }));
    },
    // statistics（ and DELETE use 1 ， and must）
    getStatsClearPreview: function (query) {
      return call('GET', '/api/stats/requests/clear-preview' + toQuery(query));
    },
    // after ： 409， clear-preview vacuumRunning
    compactStatsDb: function () { return call('POST', '/api/stats/requests/compact'); },
    getRetention: function () { return call('GET', '/api/retention'); },
    saveRetention: function (patch) { return call('PUT', '/api/retention', patch); },

    // ── （headless has 「」 ）──
    panelLogout: async function () {
      await call('POST', '/api/panel/logout', {});
      // server already will ； 1 and 。
      clearPanelAuth();
    },

    // ── （AI use ； bridge bridge.rs）
    // as for fetch 、 not call（ ui-islands 
    // assistant-state.ts）， that request by this —— must and 
    // httpCall 1 ，otherwise will 「set can 、 401」 this ：
    // · ： will in HttpOnly cookie ，request，
    // this as empty（ not JS is ）；
    // · in （cookie not ）： in localStorage，must
    // `x-panel-token`（ when has Authorization ）；
    // · ： use Key is 1 。
    // before use request before ；refresh=true used for 401。
    // localStorage not will ， is 1 。
    directCredentials: async function (refresh) {
      if (refresh) {
        if (!(await tryRefresh())) {
          clearPanelAuth();
          window.location.href = '/login';
          return new Promise(function () {});
        }
      } else {
        await ensureFreshSession();
      }
      return authHeaders({});
    },

    // ── （ / ALTCHA proof-of-work，）──
    // and bridge ：patch is `{captchaDesktop?, captchaWeb?}`
    // （update）； old when by 「 1 」（ old use ）。
    getCaptchaSetting: function () { return call('GET', '/api/captcha'); },
    saveCaptchaSetting: function (patch) {
      return call(
        'PUT',
        '/api/captcha',
        typeof patch === 'boolean' ? { captchaEnabled: patch } : (patch || {})
      );
    },

    // ── will （ by cookie / localStorage ）──
    // this 「 in tauri:// down 、 to 」 this 
    // ； cookie already use ， this to and ，
    // 。
    panelSessionStatus: function () {
      return call('GET', '/api/panel/status').then(function (state) {
        if (!state || state.registered !== true) return { registered: false, authenticated: false };
        return call('GET', '/api/session').then(
          function () { return { registered: true, authenticated: true }; },
          function () {
            return call('GET', '/api/panel/status').then(function () {
              return { registered: true, authenticated: false };
            }, function () { return { registered: true, authenticated: false }; });
          }
        );
      });
    },
    panelSessionStore: function () { return Promise.resolve({ saved: true }); },
    panelSessionClear: function () { return Promise.resolve({ cleared: true }); },

    // ── （set「」）──
    getPanelAccount: function () { return call('GET', '/api/panel/account'); },
    changePanelPassword: function (oldPassword, newPassword) {
      return call('POST', '/api/panel/change-password', {
        oldPassword: String(oldPassword || ''),
        newPassword: String(newPassword || ''),
      });
    },

    // ── models.dev （ can and ）──
    // and up ：GET （ + ）、PUT ；
    // refresh that will ，return when 1 and new after 。
    getModelsDev: function () { return call('GET', '/api/models/metadata'); },
    saveModelsDev: function (on) {
      return call('PUT', '/api/models/metadata', { modelsDevEnabled: on === true });
    },
    refreshModelsDev: function () { return call('POST', '/api/models/metadata/refresh'); },

    // ── ──
    getStorage: function () { return call('GET', '/api/storage'); },

    // ── （set「 → 」）──
    // up / down （ as when ），：
    // fetch no has total when ， by server 。
    getCloudSync: function () { return call('GET', '/api/cloud-sync'); },
    saveCloudSync: function (payload) { return call('PUT', '/api/cloud-sync', payload || {}); },
    testCloudSync: function (payload) { return call('POST', '/api/cloud-sync/test', payload || {}); },
    getCloudSyncRemote: function () { return call('GET', '/api/cloud-sync/remote'); },
    cloudSyncUpload: function () { return call('POST', '/api/cloud-sync/upload', {}); },
    cloudSyncDownload: function () { return call('POST', '/api/cloud-sync/download', {}); },
    importCloudSyncKey: function (password) {
      return call('POST', '/api/cloud-sync/import-key', { password: password });
    },

    // ── ──
    getUpgrade: function () { return call('GET', '/api/upgrade'); },
    runUpgrade: function () { return call('POST', '/api/upgrade/run', {}); },

    // ── request ──
    getRetry: function () { return call('GET', '/api/retry'); },
    saveRetry: function (patch) { return call('PUT', '/api/retry', patch); },
    // ── up request when （）──
    getTimeouts: function () { return call('GET', '/api/timeouts'); },
    saveTimeouts: function (patch) { return call('PUT', '/api/timeouts', patch); },
    // ── etc （ / seconds）──
    getQueue: function () { return call('GET', '/api/queue'); },
    saveQueue: function (patch) { return call('PUT', '/api/queue', patch); },
    // ── will （ + / / ）──
    // and （src/bridge.rs getImageCompress / saveImageCompress）。
    getImageCompress: function () { return call('GET', '/api/image-compress'); },
    saveImageCompress: function (patch) { return call('PUT', '/api/image-compress', patch); },
    // ── （ZCode ）──
    getCaptchaPool: function () { return call('GET', '/api/zcode/captcha-pool'); },
    saveCaptchaPool: function (patch) { return call('PUT', '/api/zcode/captcha-pool', patch); },

    // ── ──
    getDebug: function () { return call('GET', '/api/debug'); },
    saveDebug: function (enabled) { return call('PUT', '/api/debug', { debugMode: enabled }); },
    getDebugTraffic: function (id) {
      return call('GET', '/api/debug/traffic?id=' + encodeURIComponent(id));
    },

    // ── ──
    onStateChanged: function (callback) {
      stateListeners.add(callback);
      return function () { stateListeners.delete(callback); };
    },
    // after in no has for ： not 
    onAutoMaintained: function () { return function () {}; },
    onBackendError: function () { return function () {}; },

    // ── has ： after and ──
    getBackendStatus: function () { return invokeShell('backend_status'); },
    getPortOccupant: function () { return invokeShell('port_occupant'); },
    endPortOccupant: function () { return Promise.reject(new Error(SHELL_UNAVAILABLE)); },
    checkPort: function () {
      return Promise.reject(new Error(' not ： by AGENT2API_PROXY_PORT '));
    },
    changePort: function () {
      return Promise.reject(new Error('unsupported：set AGENT2API_PROXY_PORT after '));
    },
    restartApp: function () {
      return Promise.reject(new Error('unsupported：（docker compose restart）'));
    },

    // ── ： no has can ， to / ──
    setWindowTheme: function () { return Promise.resolve(); },
    // ── ： （Ctrl +/- and ）──
    // empty not is ：set this 1 in is use 、 not to ，
    // has app.js use will to this —— will in 1 none 。
    setZoom: function () { return Promise.resolve(); },

    // ── ： no has use ──
    // by 「 can then ， not can then 」 ： ，
    // isMaximized is ， backend_status return false；
    // onAutoMaintained returnempty。
    // up in not （titlebar.js platform ：
    // shim platform='web'）， this is 。
    windowMinimize: function () {
      return Promise.reject(new Error(SHELL_UNAVAILABLE + '： no has use '));
    },
    windowToggleMaximize: function () {
      return Promise.reject(new Error(SHELL_UNAVAILABLE + '： no has use '));
    },
    windowClose: function () {
      return Promise.reject(new Error(SHELL_UNAVAILABLE + '： no has use '));
    },
    windowIsMaximized: function () { return Promise.resolve(false); },
    onWindowResize: function () { return function () {}; },

    // ── use set and ──
    getAppSettings: function () { return invokeShell('get_app_settings'); },
    saveAppSettings: function (patch) {
      // none not this ：success can （ will ignore）
      return Promise.resolve(Object.assign({ closeToTray: false, autostart: false, proxyPort: 0, lightweightMode: false }, patch));
    },
    exportAccounts: function () { return invokeShell('export_accounts'); },
    exportAccountFiles: function (selection) { return invokeShell('export_account_files', { selection: selection }); },
    importAccounts: function () { return invokeShell('import_accounts'); },

    // ── / ──
    // file for （bridge.rs exportDatabase / importDatabase），
    // can 「 down 」 and 「 up file」， for after new 1 for HTTP 
    // （ api::backup_api module header）。return and 1 ，set
    // and use 。
    exportDatabase: function (includeLogs, password) { return exportDatabase(includeLogs, password); },
    importDatabase: function (password) { return importDatabase(password); },
    // ： and 1 after （ api::backup_api）
    getBackupProgress: function () { return call('GET', '/api/backup/progress'); },
  };

  // （ and bridge toQuery ：empty）
  function toQuery(query) {
    if (!query) return '';
    if (typeof query === 'string') {
      return query.indexOf('?') === 0 || query === '' ? query : '?' + query;
    }
    if (query instanceof URLSearchParams) {
      var text = query.toString();
      return text ? '?' + text : '';
    }
    var params = new URLSearchParams();
    for (var key in query) {
      if (!Object.prototype.hasOwnProperty.call(query, key)) continue;
      var value = query[key];
      if (value === undefined || value === null || value === '') continue;
      params.append(key, String(value));
    }
    var result = params.toString();
    return result ? '?' + result : '';
  }

  // requestsuccess after 1 （ accounts:state-changed ）：
  // 1 call， for API GET ，/v1/* and not 。
  var rawCall = call;
  call = function (method, path, body) {
    var promise = rawCall(method, path, body);
    if (method !== 'GET' && path.indexOf('/api/') === 0) {
      promise.then(function () { emitStateSoon(); }, function () {});
    }
    return promise;
  };
})();
"#
}
