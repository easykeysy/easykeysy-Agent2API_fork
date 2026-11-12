//! before 。
//!
//! （desktop-tauri/ui/） `window.workbuddyDesktop` this 。
//! this in before for ， to Tauri invoke。
//!
//! and is has as ： not Tauri， will not use 。
//! path and in in file， and after by for 。
//!
//! ：invoke in use when （ not is when ）， this not when ——
//! and Tauri after not will 。
//!
//! ── Why by not is before ──────────────────
//! has if 「 this can in macOS up unavailable」 （e.g.：
//! is Windows client file and DPAPI ）。 use UA in 
//! WebView and not can （ UA will ， and UA is 「」，
//! not is 「 」）， —— `cfg!(target_os)` is 。
//! this `platform`， by ， has 1 。

/// （`"macos"` / `"windows"` / `"linux"`）。
///
/// use `cfg!` not is ： need that is **** 
/// （ has DPAPI、 can `%APPDATA%`）， not is 。
fn target_platform() -> &'static str {
    if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "windows") {
        "windows"
    } else {
        "linux"
    }
}

/// ： `__AGENT2API_PLATFORM__` / `__AGENT2API_TITLE__`
/// [`target_platform`] and [`crate::app_title`]。
///
/// Why use not is `format!`： has `{}`（ for 、），
/// `format!` `{{}}`， 1 need for 1 。
/// 1 not will 。
pub fn bridge_js() -> String {
    BRIDGE_JS
        .replace("__AGENT2API_PLATFORM__", target_platform())
        .replace("__AGENT2API_TITLE__", crate::app_title())
}

const BRIDGE_JS: &str = r#"
(() => {
  const invokeRaw = (command, args) => {
    const internals = window.__TAURI_INTERNALS__;
    if (!internals || typeof internals.invoke !== 'function') {
      return Promise.reject(new Error('桌面运行时不可用（Tauri 未初始化）'));
    }
    return internals.invoke(command, args);
  };

  /**
   * failure 1 Error。
   *
   * Tauri in return `Err` when ，rejection is ** after error**，
   * not is Error for —— error is String， is to is 1 
   * ，`error.message` as undefined，all `failure：${error.message}`
   * 「failure：undefined」（： new token up 401
   * when ， undefined， use not to 「authorization_verify_error」）。
   *
   * has 50 `error.message`， ； 1 in 
   * 1 ， and Electron when 1 （ that when preload is Error）。
   * failure（ will if error for ）， to 。
   */
  const asError = failure => {
    if (failure instanceof Error) return failure;
    if (typeof failure === 'string') return new Error(failure);
    // error： 1 can ， not to JSON （total undefined ）
    if (failure && typeof failure === 'object') {
      const message = failure.message ?? failure.error ?? failure.msg;
      if (typeof message === 'string' && message) return new Error(message);
      try {
        return new Error(JSON.stringify(failure));
      } catch {
        return new Error('操作失败（错误信息无法序列化）');
      }
    }
    return new Error(String(failure ?? '操作失败'));
  };

  /**
   * use and failure 1 Error —— **all this **（ invoke
   * not is use ， will add when ）。
   */
  const invoke = (command, args) =>
    invokeRaw(command, args).catch(failure => {
      throw asError(failure);
    });

  /** 1 use API：return after data，failure when （ by has try/catch ） */
  const call = (method, path, body) =>
    invoke('api_request', { request: { method, path, body: body === undefined ? null : body } });

  /** */
  const toQuery = query => {
    if (!query) return '';
    if (typeof query === 'string') return query.startsWith('?') || query === '' ? query : '?' + query;
    // URLSearchParams is available ， —— to down 
    // Object.entries will to empty（ not is has can ），
    // 「 no has parameter」： use as 、 is request
    //（ empty：emptyempty）。 this ，。
    if (query instanceof URLSearchParams) {
      const text = query.toString();
      return text ? '?' + text : '';
    }
    const params = new URLSearchParams();
    for (const [key, value] of Object.entries(query)) {
      if (value === undefined || value === null || value === '') continue;
      params.append(key, String(value));
    }
    const text = params.toString();
    return text ? '?' + text : '';
  };

  /**
   * after 。Tauri listen is ， this return，
   * and preload use （return unsubscribe） 1 。
   *
   * target as Any —— after `emit` use （login:state etc ） for 
   * target 。****（tauri://resize etc ） not 1 ：
   * by label to （Tauri emit_to_window Window /
   * WebviewWindow target， for Any 1 not ），must
   * current target —— onWindowResize。
   */
  const on = (event, callback, target) => {
    let unlisten = null;
    let disposed = false;
    invoke('plugin:event|listen', {
      event,
      target: target || { kind: 'Any' },
      handler: window.__TAURI_INTERNALS__.transformCallback(evt => callback(evt && evt.payload)),
    })
      .then(fn => {
        if (disposed && typeof fn === 'function') fn();
        else unlisten = fn;
      })
      .catch(error => console.warn('订阅事件失败:', event, error));
    return () => {
      disposed = true;
      if (typeof unlisten === 'function') unlisten();
    };
  };

  window.workbuddyDesktop = {
    // ── ──
    // （'macos' / 'windows' / 'linux'）。 use 
    // unavailable can （file「」 1 ）。
    platform: '__AGENT2API_PLATFORM__',

    // ── ──
    // use （、 1 ， app_title）。
    // "Dev" —— and document.title by ， 
    // when when up 1 can 。
    title: '__AGENT2API_TITLE__',

    // ── ────────────────────────────────
    // and HTTP ， not can cookie；return will 。
    // use by Key， will not can 。
    directCredentials: () => invoke('direct_credentials'),
    saveAssistantImage: id => invoke('save_assistant_image', { id: String(id || '') }),

    // ── will ──
    getState: () => call('GET', '/api/session'),
    startLogin: (edition, mode, provider, socialRestore, options) =>
      invoke('start_login', {
        edition: edition || 'cn',
        mode: mode || 'embedded',
        // provider to Rust workbuddy：（ or will use ）
        // not when must that ， this not 
        provider: provider || null,
        // is ； options can ， not can 。
        socialRestore: socialRestore === true,
        ...(options === undefined ? {} : { options }),
      }),
    // URL / code to ；accepted / pending not is ，startLogin
    // etc /wait done returnsuccess。
    submitLoginCallback: (state, callbackUrl) =>
      call('POST', '/api/session/login/callback', {
        state: String(state ?? ''),
        callbackUrl: String(callbackUrl ?? ''),
      }),
    getLoginState: () => invoke('login_state'),
    cancelLogin: () => invoke('cancel_login'),
    // ── AutoClaw OAuth（** by **）──────────
    // and startLogin ： that is 「 after need 」， this is 「 already 
    // to ， and etc 」。Reason is this before has 1 **must in 
    // ** （ SDK）， is that 
    // —— （ commands.rs start_autoclaw_oauth_login）。
    //
    // state / authUrl `POST /api/session/login/oauth/start`，
    // to this ； not can as empty（ will ）。
    startAutoclawOauthLogin: (state, authUrl, mode) =>
      invoke('start_autoclaw_oauth_login', {
        state: String(state || ''),
        authUrl: String(authUrl || ''),
        // ： and default 1 （ is optional）
        mode: mode === 'external' ? 'external' : 'embedded',
      }),
    // AutoClaw OAuth before （， not ）：
    // captchaConfig config（ before SDK）；
    // oauthStart parameter。
    // in this not is path： is 「 → 」 1 ，
    // （ invoke('api_request')） will asError error 1 
    // —— that when rejection is ，`error.message` is undefined，
    // will 「failure：undefined」（ sendSmsCode ）。
    getAutoclawOauthCaptchaConfig: provider =>
      call('POST', '/api/session/login/oauth/captcha-config', {
        ...(provider ? { provider: String(provider) } : {}),
      }),
    startAutoclawOauth: (provider, vendor, captchaVerifyParam) =>
      call('POST', '/api/session/login/oauth/start', {
        ...(provider ? { provider: String(provider) } : {}),
        vendor: String(vendor || ''),
        captchaVerifyParam: String(captchaVerifyParam || ''),
      }),
    // ── ZCode when （ ，path）──────
    // and up AutoClaw that 1 ，**must for in **：file is 
    // ，`server/src/web_shim.rs` is headless ——
    // 1 when ， 1 down will 「currentunsupported（）」
    // （zcode-claim.js `api?.zcodeClaimPreview` ）。
    //
    // （ `api/zcode_claim.rs` module header）：
    // · captchaConfig config（ before use SDK）；
    // return `{enabled:false}` up not need —— before ** not this **；
    // · preview ，return `{plans:[...], deployed}`；
    // `deployed:false` = （ before ， not is error）；
    // · claim ；`captchaVerifyParam` ** can as empty**（ up not need 
    // when not that ， `claim::claim` ）。**failure 200**，
    // by `ok:false` + `failure` （ before ）；return 
    // `claimedAt` is when ， before by 「 already 」。
    zcodeClaimCaptchaConfig: accountId =>
      call('POST', `/api/accounts/${encodeURIComponent(String(accountId || ''))}/zcode-claim/captcha-config`),
    zcodeClaimPreview: accountId =>
      call('POST', `/api/accounts/${encodeURIComponent(String(accountId || ''))}/zcode-claim/preview`),
    zcodeClaim: (accountId, planId, captchaVerifyParam, captchaRegion) =>
      call('POST', `/api/accounts/${encodeURIComponent(String(accountId || ''))}/zcode-claim`, {
        planId: planId ? String(planId) : '',
        captchaVerifyParam: String(captchaVerifyParam || ''),
        captchaRegion: captchaRegion ? String(captchaRegion) : '',
      }),
    // ── ZCode （ core/providers/zcode/captcha.rs）──
    // and up ** not is 1 **： that 「」（ use ，）；
    // this **** —— request need 1 ，
    // in after ， can by after （ 1 use is 
    // ui/zcode-captcha-pool.js）。response `api::zcode_captcha`。
    zcodeClaimVerification: payload => call('POST', '/api/zcode/claim-verification', payload),
    zcodeCaptchaStats: () => call('GET', '/api/zcode/captcha'),
    pushZcodeCaptchaTokens: tokens =>
      call('POST', '/api/zcode/captcha', { tokens: Array.isArray(tokens) ? tokens : [] }),
    onLoginState: callback => on('login:state', callback),
    // and /wait pending.hint； not or use 。
    onLoginProgress: callback => on('login:progress', callback),
    // KukuAI （， `start_kuku_sms`）：
    // {status: 'opening'|'filling'|'sending'|'sent'|'captcha'|'timeout'}。
    // when ， use not will 「 no 」。
    onKukuSmsStatus: callback => on('login:kuku-status', callback),
    refreshSession: async () => {
      await call('POST', '/api/session/refresh', {});
      return call('GET', '/api/session');
    },
    logout: async () => {
      await call('POST', '/api/session/logout', {});
      return call('GET', '/api/session');
    },

    // ── config ──
    getConfig: () => call('GET', '/api/config'),
    saveConfig: payload => call('POST', '/api/config', payload),

    // ── ──
    // new （「get」）：supportsdirectory 、，
    // return `{results, refreshed, skipped, failed, models}` —— `models` is 
    // **session ** （ /api/session that ）， to 「
    // /api/session 」 use use ； that manage （、
    // 「」） not in its in —— by use in new after 
    // （ models-fetch-modal.js onRefreshed）。
    //
    // optional ， is add optional（ use not = by default + ）：
    // · `{accounts: {providerId: accountId}}`：「get」 
    // 「」 down （ use this directory）；
    // `accountId` 。
    // · `{providers: [providerId, ...]}`： new ****， 
    // not not （ by 「 has ∪ has enabled 」
    // ， models-fetch-modal.js scopeProviders）。
    refreshModels: payload => call('POST', '/api/models/refresh', payload || {}),
    // （ / ）：return new {models, mappings, reasoningLevels}
    // OmniProxy ： for by （ and up id ）， 1 for 
    // can in not 1 （）；provider as empty = old 
    //
    // 4 parameter is ** etc **（ OmniProxy ，
    // down ）： = not already has etc （ old use as ），
    // '' / null = 「 not 」， its = this etc 。
    // 5 parameter is ****（chip up ）： = not ，bool = / 。
    // after by 「request has no has this 」， optionalparameter ——
    // `reasoning: reasoning || ''` will 「 not 」「empty」，
    // that is 1 。
    getModelManage: () => call('GET', '/api/models/manage'),
    getClientIdentification: () => call('GET', '/api/client-identification'),
    saveClientIdentification: payload => call('PUT', '/api/client-identification', payload),
    previewClientIdentification: payload => call('POST', '/api/client-identification/preview', payload),
    getCompactRoutes: () => call('GET', '/api/models/compact-routes'),
    saveCompactRoutes: payload => call('PUT', '/api/models/compact-routes', payload),
    setModelState: payload => call('POST', '/api/models/state', payload),
    addModelMapping: (alias, target, provider, reasoning, enabled, create) => call('POST', '/api/models/mappings', {
      alias, target, provider,
      ...(reasoning === undefined ? {} : { reasoning }),
      ...(enabled === undefined ? {} : { enabled }),
      ...(create === undefined ? {} : { create }),
    }),
    removeModelMapping: (alias, target, provider) => call('POST', '/api/models/mappings/remove', { alias, target, provider }),
    // （ up directory no has ）。「」 not is 「」——
    // after `api::model_manage::remove_custom` 。
    addCustomModel: (provider, id) => call('POST', '/api/models/custom', { provider, id }),
    removeCustomModel: (provider, id) => call('POST', '/api/models/custom/remove', { provider, id }),
    // can （； /api/custom-providers/models 
    // ， after handler ）。`capabilities` ： not to = not 、
    // null = 、 to = 。
    setModelCapabilities: (provider, id, capabilities) => call('POST', '/api/models/capabilities', { provider, id, capabilities }),
    // ： this 1 up 1 ****request（、
    // will ）， to 「 this 1 × 」， after 
    // `api::model_test` module header。
    //
    // payload optional（ is after request ）：`account_id` =
    // 1 ；`prompt` / `system_prompt` / `reasoning` /
    // `stream` = request；`test_id` by before —— when can use has 
    // `terminateStatsRequest(id)` （ invoke no has abort，
    // this is before 1 can in in ， after that ）。
    // failurereturn 2xx： up error in return status / error 
    testModel: (payload, apiKey) => invoke('api_request', { request: {
      method: 'POST', path: '/api/models/test', body: payload || {}, apiKey: String(apiKey || ''),
    } }),
    getModelApiKeys: () => call('GET', '/api/model-access/keys'),

    // ── API Key（ ）──
    getKeys: () => call('GET', '/api/keys'),
    createKey: payload => call('POST', '/api/keys', payload || {}),
    updateKey: (id, patch) => call('PATCH', '/api/keys/' + encodeURIComponent(id), patch),
    deleteKey: id => call('DELETE', '/api/keys/' + encodeURIComponent(id)),

    // ── ──
    getAccounts: () => call('GET', '/api/accounts'),
    getAccountSorting: () => call('GET', '/api/accounts/sorting'),
    saveAccountSorting: payload => call('PUT', '/api/accounts/sorting', payload || {}),
    // switchAccount only to 1 ， not enabled
    switchAccount: id => call('POST', '/api/accounts/current', { id }),
    refreshAccountToken: id => call('POST', '/api/accounts/refresh', id ? { id } : {}),
    removeAccount: id => call('DELETE', '/api/accounts/' + encodeURIComponent(id)),
    updateAccount: (id, patch) => call('PATCH', '/api/accounts/' + encodeURIComponent(id), patch),
    // （）： / after ， ** new ** up 
    // （ from to id ）。 after by priority 0、1、2…，
    // 「 + 」 1 ， not in PATCH in 。
    reorderAccounts: queue =>
      call('POST', '/api/accounts/reorder', {
        queue: Array.isArray(queue) ? queue.map(item => String(item)) : [],
      }),
    moveAccount: (id, direction) =>
      call('POST', '/api/accounts/' + encodeURIComponent(id) + '/move', {
        direction: direction === 'down' ? 'down' : 'up',
      }),
    // （「」）：model = this 
    clearRateLimits: (id, model) =>
      call('POST', '/api/accounts/' + encodeURIComponent(id) + '/rate-limits/clear',
        model ? { model } : {}),
    // ── by （ / / / / + can ）──
    getRoutingConfig: () => call('GET', '/api/routing'),
    getRoutingWorkspace: model => call('GET', '/api/routing/workspace' + toQuery({ model })),
    saveRoutingProfile: payload => call('PUT', '/api/routing/profile', payload || {}),
    getRoutingGroups: () => call('GET', '/api/routing/groups'),
    saveRoutingGroup: payload => call('PUT', '/api/routing/groups', payload),
    previewRouting: payload => call('POST', '/api/routing/preview', payload),
    getRoutingSticky: model => call('GET', '/api/routing/sticky' + toQuery({ model })),
    clearRoutingSticky: payload => call('POST', '/api/routing/sticky/clear', payload || {}),
    getRoutingReplays: query => call('GET', '/api/routing/replays' + toQuery(query)),
    getRoutingFlow: query => call('GET', '/api/routing/flow' + toQuery(query)),
    getRoutingTrace: query => call('GET', '/api/routing/trace' + toQuery(query)),
    getRoutingTraceRequest: query => call('GET', '/api/routing/trace/request' + toQuery(query)),
    setRoutingConfig: patch => call('PUT', '/api/routing', patch || {}),
    // can （「 by 」 ）：model = not 
    getRoutingQueue: model =>
      call('GET', '/api/routing/queue' + (model ? '?model=' + encodeURIComponent(model) : '')),
    batchAccounts: payload =>
      call('POST', '/api/accounts/batch', {
        action: String((payload && payload.action) || ''),
        ids: Array.isArray(payload && payload.ids) ? payload.ids : [],
        ...(payload && Object.prototype.hasOwnProperty.call(payload, 'proxy')
          ? { proxy: payload.proxy }
          : {}),
        ...(payload && Object.prototype.hasOwnProperty.call(payload, 'maxConcurrent')
          ? { maxConcurrent: payload.maxConcurrent }
          : {}),
        ...(payload && Object.prototype.hasOwnProperty.call(payload, 'usageQuery')
          ? { usageQuery: payload.usageQuery }
          : {}),
        ...(payload && Object.prototype.hasOwnProperty.call(payload, 'limiterGroups')
          ? { limiterGroups: payload.limiterGroups }
          : {}),
        ...(payload && Object.prototype.hasOwnProperty.call(payload, 'limiters')
          ? { limiters: payload.limiters }
          : {}),
        ...(payload && Object.prototype.hasOwnProperty.call(payload, 'limiterMode')
          ? { limiterMode: payload.limiterMode }
          : {}),
      }),

    // ── ──
    getProxies: () => call('GET', '/api/proxies'),
    testProxy: payload => {
      const body = {};
      if (payload && typeof payload === 'object' && !Array.isArray(payload)) {
        if (typeof payload.id === 'string' && payload.id) body.id = payload.id;
        // proxy null（）， by in 
        else if (Object.prototype.hasOwnProperty.call(payload, 'proxy')) body.proxy = payload.proxy;
      }
      return call('POST', '/api/proxies/test', body);
    },
    // ── Agent（ AI client config）──
    // and `server/src/web_shim.rs` for in （ up ）。
    // `api::agents` module header：
    // · getAgents client + use config + + 
    // · saveAgentProfiles 1 client config（ not clientfile）
    // · applyAgentProfile clientfile（config / config
    // enabled）， before ；response `applied`
    // · previewAgentProfile （ not ），response `changed` 
    // · restoreAgentClient by 1 
    // · importAgentClient from client has file 1 config
    // return `{clients}`（， not 1 ）。
    getAgents: () => call('GET', '/api/agents'),
    saveAgentProfiles: payload => call('POST', '/api/agents/profiles', payload || {}),
    applyAgentProfile: payload => call('POST', '/api/agents/apply', payload || {}),
    previewAgentProfile: payload => call('POST', '/api/agents/preview', payload || {}),
    restoreAgentClient: payload => call('POST', '/api/agents/restore', payload || {}),
    importAgentClient: payload => call('POST', '/api/agents/import', payload || {}),

    // ── （「」）──
    // and `server/src/web_shim.rs` for in （ that file is headless
    // ）： 1 when ， 1 down will 「」——
    // that 1 has ， zcode-claim that 。
    // return new ；failure success:false 。
    getProxyPool: () => call('GET', '/api/proxies/pool'),
    createProxyPoolItem: payload => call('POST', '/api/proxies/pool', payload || {}),
    updateProxyPoolItem: payload => call('POST', '/api/proxies/pool/update', payload || {}),
    setProxyPoolItemEnabled: payload => call('POST', '/api/proxies/pool/enable', payload || {}),
    removeProxyPoolItem: id => call('POST', '/api/proxies/pool/remove', { id: String(id || '') }),
    testProxyPoolItem: id => call('POST', '/api/proxies/pool/test', { id: String(id || '') }),
    syncClashToProxyPool: () => call('POST', '/api/proxies/pool/sync-clash', {}),
    // ── set（「」 set：）──
    // and web_shim.rs for in （ up ）。 `api::proxies`
    // module header settings ：GET `defaults`（ default），
    // POST is 、 400 + 1 can in 。
    getProxySettings: () => call('GET', '/api/proxies/settings'),
    saveProxySettings: payload => call('POST', '/api/proxies/settings', payload || {}),

    // file：parameter and 1 （camelCase ← Rust snake_case）
    pickFolder: defaultDir => invoke('pick_folder', { defaultDir: defaultDir == null ? null : String(defaultDir) }),

    // ── / to ──
    getUsage: () => call('GET', '/api/usage'),
    getCheckinStatus: () => call('GET', '/api/checkin/status'),
    claimCheckin: () => call('POST', '/api/checkin', {}),
    // `id` to = that 1 （ 「」 by this ）。
    // ** not enabled** —— use is 「 not and 」， and can none；
    // by enabled will 1 「return」。 core::usage_query。
    getAllBalances: id =>
      call('GET', '/api/accounts/usage' + (id ? '?id=' + encodeURIComponent(id) : '')),
    // 1 「 when 」 （ up ， 1 at when ）。
    // ， is use not by can to new 。
    getBalancesSnapshot: () => call('GET', '/api/accounts/usage/snapshot'),
    // `{counts: {id: n}}`（， 0）。
    // 「」 2 seconds —— after is ， this request。
    getAccountConnections: () => call('GET', '/api/accounts/connections'),
    checkinAllAccounts: id => call('POST', '/api/accounts/checkin', id ? { id } : {}),
    // ── Loomy new （ / 1 ）──
    // to after ： this ， has and 
    // （ ui-islands accounts-dialog-onboarding）。 and `server/src/web_shim.rs`
    // for （headless 1 ， 1 will in that 1 down ）。
    //
    // `refresh`：「」 when true → after `?refresh=1` up 。
    // not （ / to after ） after **** —— 1 
    // not up （ server api::onboarding ）。
    getOnboardingTasks: (id, refresh) =>
      call('GET', '/api/accounts/' + encodeURIComponent(String(id || '')) + '/onboarding'
        + (refresh ? '?refresh=1' : '')),
    claimOnboardingTasks: id =>
      call('POST', '/api/accounts/' + encodeURIComponent(String(id || '')) + '/onboarding/claim', {}),
    // ── WorkBuddy （： / / / ）──
    // from workbuddy2api task_runner.py and school_open_day_2026.py ，
    // server api::growth module header。 and `server/src/web_shim.rs` 
    // **must for **（headless 1 ， 1 will in that 1 down ）。
    //
    // `getGrowthStatus` is （ / can / / ）；
    // ****（runGrowthTask）—— 1s+ will API 
    // 60 seconds when ， 「 1 」 in use and 。
    getGrowthStatus: id =>
      call('GET', '/api/accounts/' + encodeURIComponent(String(id || '')) + '/growth'),
    runGrowthTask: (id, code, mode) =>
      call('POST', '/api/accounts/' + encodeURIComponent(String(id || '')) + '/growth/task',
        { code: code, mode: mode || 'full' }),
    runGrowthSchool: (id, lottery) =>
      call('POST', '/api/accounts/' + encodeURIComponent(String(id || '')) + '/growth/school',
        { lottery: lottery === true }),
    runGrowthTravel: id =>
      call('POST', '/api/accounts/' + encodeURIComponent(String(id || '')) + '/growth/travel', {}),

    // ── （AutoClaw / Loomy / Tabbit ）──
    // and that （、 etc ） not ： up no has ， is 「 →
    // use token」 use ， API not is start_login。
    //
    // and `server/src/web_shim.rs` **must for **（ that is headless
    // ， its ZCode that ）： this ，
    // by —— 1 when 1 will ****，issue #93 is 
    // web_shim that 1 （ loomy AutoClaw ）。
    //
    // must call not is invoke('api_request')：call will invoke/asError
    // `Err(String)` 1 Error —— when rejection is ****，
    // `error.message` will to undefined，「failure：undefined」。
    // `sendSmsCode` return deviceId need to verify（ up in when 
    // that up ， api::session::login_sms_send ）。
    // ── `sendSmsCode` （）──────────────────────
    // ****（）， new `{phone, provider}`。
    // need ： and is ， not when old not can 。
    //
    // `provider` is ** need that 1 **： in AutoClaw is 「 1 
    // 」（`autoclaw` / `autoclaw-intl` ）—— is 1 
    // path、 ， up ， will in after （ 
    // already ， api::session::login_sms_send）；Loomy then not ，
    // by down that 。（ and as 1 ）。
    sendSmsCode: input => {
      const isObject = input && typeof input === 'object';
      const phone = String((isObject ? input.phone : input) || '');
      const provider = isObject && input.provider ? String(input.provider) : '';
      // Loomy / Tabbit use ， in msgid / uuid；
      // AutoClaw use has ，`provider` up by after 。
      //
      // KukuAI is ****： no has can API（ need up 
      // ， `login::kuku_sms_script`）， by ** use 
      // **—— `login_kuku_sms_send`（return {state}），verify when 
      // 。 invoke not is call： this is WebView 。
      if (provider === 'kuku') {
        return invoke('login_kuku_sms_send', { phone });
      }
      const path = provider === 'loomy'
        ? '/api/session/login/loomy/sms/send'
        : provider === 'tabbit'
          ? '/api/session/login/tabbit/sms/send'
          : '/api/session/login/sms/send';
      return call('POST', path, {
        phone,
        ...(provider ? { provider } : {}),
        // Tabbit can can in before need 。 this 1 after ， will use 
        // 200 + `{status:"captcha", captcha:{verifyUrl}}` this （
        // ）， not is 409 error； not old to is 1 can 。
        // uuid in 「」 when need use 1 （ up in up ）。
        ...(isObject && input.supportsCaptcha ? { supportsCaptcha: true } : {}),
        ...(isObject && input.uuid ? { uuid: String(input.uuid) } : {}),
      });
    },
    // up to （ before has Tabbit will in before need ）。
    // use `open_release_page`： http(s)， and Windows ShellExecuteW
    // —— URL `&`（verifyUrl ） not will 。
    openVerifyPage: url => invoke('open_release_page', { url: String(url || '') }),
    verifySmsLogin: payload => {
      const provider = (payload && payload.provider) ? String(payload.provider) : '';
      // KukuAI： by （`login_kuku_sms_verify`），
      // after return {account, list}（ and response，
      // `login::submit_kuku_sms_code`）。
      if (provider === 'kuku') {
        return invoke('login_kuku_sms_verify', { code: String((payload && payload.code) || '') });
      }
      const path = provider === 'loomy'
        ? '/api/session/login/loomy/sms/verify'
        : provider === 'tabbit'
          ? '/api/session/login/tabbit/sms/verify'
          : '/api/session/login/sms/verify';
      return call('POST', path, {
        phone: String((payload && payload.phone) || ''),
        code: String((payload && payload.code) || ''),
        // deviceId（AutoClaw）/ msgid（Loomy）/ uuid（Tabbit）/ name optional：
        // by 「 has 」；return in must， not can 。
        ...((payload && payload.deviceId) ? { deviceId: String(payload.deviceId) } : {}),
        ...((payload && payload.msgid) ? { msgid: String(payload.msgid) } : {}),
        ...((payload && payload.uuid) ? { uuid: String(payload.uuid) } : {}),
        ...((payload && payload.name) ? { name: String(payload.name) } : {}),
        ...(provider ? { provider } : {}),
      });
    },

    // ── when to ──
    getAutoCheckin: () => call('GET', '/api/auto-checkin'),
    saveAutoCheckin: patch => call('POST', '/api/auto-checkin', patch),
    runAutoCheckinNow: () => call('POST', '/api/auto-checkin/run', {}),
    // when to ****（ to in 「 when to 」）： / 1 / delete / 
    getCheckinSchedules: () => call('GET', '/api/checkin-schedules'),
    saveCheckinSchedule: patch => call('POST', '/api/checkin-schedules', patch),
    deleteCheckinSchedule: id =>
      call('POST', '/api/checkin-schedules/' + encodeURIComponent(String(id || '')) + '/delete', {}),
    runCheckinSchedule: id =>
      call('POST', '/api/checkin-schedules/' + encodeURIComponent(String(id || '')) + '/run', {}),
    // WorkBuddy （ / ； and web_shim.rs for ）
    getCheckinKeepalive: () => call('GET', '/api/checkin-keepalive'),
    saveCheckinKeepalive: (models, apiKeyId) => call('POST', '/api/checkin-keepalive', { models, apiKeyId }),
    // when （）： / and when / 
    // （ and web_shim.rs for ； in core::growth_scheduler）
    getTravelSchedule: () => call('GET', '/api/travel-schedule'),
    saveTravelSchedule: patch => call('POST', '/api/travel-schedule', patch),
    runTravelScheduleNow: () => call('POST', '/api/travel-schedule/run', {}),
    // WorkBuddy （mode: full | claim | keepalive）
    runCheckinActivity: (id, mode) => call('POST', '/api/checkin-activity', { id, mode }),
    // to in （daily to / to set / to / 1 ），
    // api::checkin_center module header —— this 1 
    getCheckinCenter: () => call('GET', '/api/checkin-center'),
    // Qoder UMID （） and （ and web_shim.rs for ）。
    // is tauri:// 、 fetch to not after ， can this 
    //（call will X-API-Key 1 and up ， gateway::request_builder）。
    getQoderUmid: () => call('GET', '/api/qoder-umid'),
    installQoderUmid: () => call('POST', '/api/qoder-umid/install', {}),
    // to （ to ）：（limit/offset/outcome/provider/keyword/start/end）
    // and empty（must all=1， api::checkin_records module header）
    getCheckinRecords: params => {
      const query = new URLSearchParams();
      for (const [key, value] of Object.entries(params || {})) {
        if (value !== undefined && value !== null && value !== '') query.set(key, String(value));
      }
      const text = query.toString();
      return call('GET', '/api/checkin-records' + (text ? '?' + text : ''));
    },
    clearCheckinRecords: params => {
      const query = new URLSearchParams();
      for (const [key, value] of Object.entries(params || {})) {
        if (value !== undefined && value !== null && value !== '') query.set(key, String(value));
      }
      const text = query.toString();
      return call('DELETE', '/api/checkin-records' + (text ? '?' + text : ''));
    },

    // ── when （ / when / new / before new ）──
    // 1 use PATCH（ after when POST ：CORS no has PATCH，
    // when will down ； use can use ， this by use PATCH）。
    getScheduledTasks: () => call('GET', '/api/scheduled-tasks'),
    saveScheduledTask: (id, patch) =>
      call('PATCH', '/api/scheduled-tasks/' + encodeURIComponent(String(id || '')), patch),
    runScheduledTask: id =>
      call('POST', '/api/scheduled-tasks/' + encodeURIComponent(String(id || '')) + '/run', {}),

    // ── update ──
    // checkUpdate ：current has （ after is ），
    // by up to after 
    checkUpdate: () => invoke('check_update'),
    // set「」： 1 （ / / path / / ）。
    // by ** has ** ： when package_info 
    // （ after in 「 is 」 this up not is ）、WebView2 
    // （ / when is ）、 is dev 。
    // and `web_shim.rs` must for in —— 1 when 1 will 
    // 「」。
    getSystemInfo: options =>
      invoke('system_info', { hideUsername: options?.hideUsername === true }),
    // 1 「checkupdate」 （ when ；app.js ）
    getUpdateStatus: () => call('GET', '/api/update/status'),
    // 「update」 ：null = 。update「checkupdate」 down 
    // （check and down ， after 1 ）
    getUpdateProxy: () => call('GET', '/api/update/proxy'),
    setUpdateProxy: payload => call('POST', '/api/update/proxy', payload),
    getGithubAccess: () => call('GET', '/api/update/github'),
    setGithubAccess: payload => call('POST', '/api/update/github', payload),
    // 「update」 GitHub ： {filled, origin}（ after not ），
    // {token: '…' | null}（null = ）。 in after 
    getUpdateToken: () => call('GET', '/api/update/token'),
    setUpdateToken: payload => call('POST', '/api/update/token', payload),
    // 「 beta 」： {enabled}， {enabled: true|false}。
    // after updatecheck GitHub 1 and （ after down 1 check）；
    // response saved（false = already 、 will to ）
    getUpdateBeta: () => call('GET', '/api/update/beta'),
    setUpdateBeta: payload => call('POST', '/api/update/beta', payload),
    getUpdateChannels: () => call('GET', '/api/update/channels'),
    setUpdateChannels: payload => call('POST', '/api/update/channels', payload),
    downloadUpdate: payload => invoke('download_update', {
      url: String((payload && payload.url) || ''),
      name: (payload && payload.name) ? String(payload.name) : null,
    }),
    updateProgress: () => invoke('update_progress'),
    cancelUpdate: () => invoke('cancel_update'),
    runInstaller: (path, restart) =>
      invoke('run_installer', { path: String(path || ''), restart: restart !== false }),
    openReleasePage: url => invoke('open_release_page', { url: String(url || '') }),

    // ── ──
    // update； in old 。
    getSanitize: () => call('GET', '/api/sanitize'),
    saveSanitize: patch => call('PUT', '/api/sanitize',
      typeof patch === 'boolean' ? { sanitizeBlacklistFingerprints: patch } : patch),

    // ── ──
    // and getSanitize / saveSanitize ：GET 、PUT ，response is new 。
    // and web_shim.rs （ 1 down getCors that ）。
    getDeveloperRoleCompat: () => call('GET', '/api/developer-role-compat'),
    saveDeveloperRoleCompat: enabled =>
      call('PUT', '/api/developer-role-compat', { developerRoleCompat: enabled === true }),

    // ── Cline ──
    // and getSanitize / saveSanitize ：GET （overrides + effective），
    // PUT （empty = not this ， = default）。
    getClineHeaders: () => call('GET', '/api/cline/headers'),
    saveClineHeaders: overrides => call('PUT', '/api/cline/headers', { overrides }),

    // ── ZCode client ──
    // up by down not down ， after when is **empty**（ not is ），
    // to use 1 can ； api::zcode_version module header。
    // and web_shim.rs （ 1 = set that 1 not to ）。
    getZcodeVersion: () => call('GET', '/api/zcode/version'),
    saveZcodeVersion: version => call('PUT', '/api/zcode/version', { version }),

    // ── 网关面跨域访问（/v1/*，default）──
    // and web_shim.rs getCors / saveCors ： Tauri IPC to 
    // /api/cors， fetch， and must 1 （
    // in ）。 up 1 this when web_shim that 1 ， this ，
    // is set not to ， in 「 can set」。
    getCors: () => call('GET', '/api/cors'),
    saveCors: enabled => call('PUT', '/api/cors', { corsEnabled: enabled === true }),

    // ── （ 1 ） and （set「」）──
    // and web_shim.rs 。 before this in ：
    // not to `getCaptchaSetting`，set that 1 is （ this is after 
    // from 「」 to 「」 Reason 1 ）。
    getCaptchaSetting: () => call('GET', '/api/captcha'),
    // patch is `{captchaDesktop?, captchaWeb?}`（update）； old when 
    // as 「 1 」， no new （ old islands/ui.js）。
    saveCaptchaSetting: patch =>
      call(
        'PUT',
        '/api/captcha',
        typeof patch === 'boolean' ? { captchaEnabled: patch } : (patch || {}),
      ),
    getPanelAccount: () => call('GET', '/api/panel/account'),
    changePanelPassword: (oldPassword, newPassword) =>
      call('POST', '/api/panel/change-password', {
        oldPassword: String(oldPassword || ''),
        newPassword: String(newPassword || ''),
      }),

    // ── models.dev （ can and ）──
    // and web_shim.rs （ Tauri IPC，
    // fetch， and must 1 —— 1 up getCors that ）。
    // GET + ；refresh ，return and new after 。
    getModelsDev: () => call('GET', '/api/models/metadata'),
    saveModelsDev: enabled =>
      call('PUT', '/api/models/metadata', { modelsDevEnabled: enabled === true }),
    refreshModelsDev: () => call('POST', '/api/models/metadata/refresh'),

    // ── and ──
    // and getRetry / saveRetry ：GET 、PUT （），response is 
    // after （ is ）。`clearDegrade` is 1 up 1 
    // （ can etc to ， use ）。
    getPrompt: () => call('GET', '/api/prompt'),
    savePrompt: payload =>
      call('PUT', '/api/prompt', {
        promptMode: payload && payload.promptMode != null ? String(payload.promptMode) : null,
        promptFile: payload && payload.promptFile != null ? String(payload.promptFile) : null,
        // （empty = this 1 、file / default；
        // null = this 1 not ）。， not or 1 。
        promptText: payload && payload.promptText != null ? String(payload.promptText) : null,
        // by ：（ is for ， or null = this 1 ）。
        // this ** not ** 1 —— that （update / null delete） by 
        // after ， 1 will 1 can can and after 。
        promptProviders: payload && payload.promptProviders != null ? payload.promptProviders : null,
        // （ 1 ， is or null）—— 
        promptGateway: payload && payload.promptGateway != null ? payload.promptGateway : null,
        // （`{"<id>": {identity?, stable?, dynamic?} | null}`）：
        // is ， and update by after 
        promptGatewayText: payload && payload.promptGatewayText != null ? payload.promptGatewayText : null,
        clearDegrade: !!(payload && payload.clearDegrade),
      }),

    // ── ──
    getLogs: query => call('GET', '/api/logs' + toQuery(query)),
    getLogStats: () => call('GET', '/api/logs/stats'),
    // ── ──
    // need by （ ui/app.js adoptPluginPages）。
    // 、 need `packs[].ui[]` and `enabled` ，：
    // 1 「 to 」 will 「」 and 「」 ，
    // / after can can for not up 。
    getPlugins: () => call('GET', '/api/plugins'),
    // ── （ / / ）──
    // in （`kv` `sidebarLayout` ）， 
    // localStorage is —— and is origin，localStorage
    // not ， has that 1 can 「 in WebUI ，client not 」。
    // `{layout}`，`layout: null` = no （default）。
    // （null = default）。（ / ） in before ， after not ，
    // by api::sidebar_layout module header。
    getSidebarLayout: () => call('GET', '/api/sidebar-layout'),
    saveSidebarLayout: layout =>
      call('PUT', '/api/sidebar-layout', { layout: layout === undefined ? null : layout }),
    // emptysupports（optional for ， and getLogs 1 ）： = in ，
    // not = empty（ after not ， logs_api::clear_logs）
    clearLogs: query => call('DELETE', '/api/logs' + toQuery(query)),
    exportLogs: query => invoke('export_logs', { query: typeof query === 'string' ? query : toQuery(query).replace(/^\?/, '') }),

    // ── requeststatistics / ──
    // range today / yesterday / 7 / 30 / month / all / custom， by after return 400
    // （ not in this ： this to ，）。
    // dates（`{start, end}`，`YYYY-MM-DD`）** in range=custom when ** to ：
    // after for custom will need this parameter， its up is 
    getStatsSummary: (range, dates, options) => call('GET', '/api/stats/summary' + toQuery({
      range,
      startDate: dates && dates.start,
      endDate: dates && dates.end,
      apiKey: options && options.apiKey,
    })),
    // is optional for ， use up toQuery 「empty」：
    // empty not this `?model=` this not in 
    getStatsRequests: query => call('GET', '/api/stats/requests' + toQuery(query)),
    // down （ / ）。 not parameter、 not ——
    // in when 1 （ by api::stats_api::stats_request_filters）。
    // not to when before 「」， not （unavailable，）
    getStatsRequestFilters: () => call('GET', '/api/stats/requests/filters'),
    // emptysupports（ = in ， not = empty）
    clearStatsRequests: query => call('DELETE', '/api/stats/requests' + toQuery(query)),
    // by id request `{id, requestBody, responseBody, truncated}`
    // （「 for 」 ； not ，）。
    // not to to 404， before 「 no has 」。
    getStatsRequestRaw: id => call('GET', '/api/stats/requests/raw' + toQuery({ id })),
    // 1 ** in **request（ 「request」 by ）：
    // `{success:true,terminated:true}`； not in in to 404（ already /
    // up in ）， before new 
    terminateStatsRequest: id => call('POST', '/api/stats/requests/terminate' + toQuery({ id })),
    // statistics `{all, raw, dbBytes, vacuumRunning, lastVacuum}`：
    // and DELETE use 1 （ after filter_from_params）， N 、
    // is N —— and must，otherwise is new 「empty」
    getStatsClearPreview: query => call('GET', '/api/stats/requests/clear-preview' + toQuery(query)),
    // after （checkpoint + VACUUM）： already {started:true}； 409，
    // getStatsClearPreview vacuumRunning / lastVacuum 
    compactStatsDb: () => call('POST', '/api/stats/requests/compact'),
    getRetention: () => call('GET', '/api/retention'),
    // PUT is after already （ + ）。
    // gateway.rs request_builder supports GET/POST/PUT/PATCH/DELETE，
    // and for PUT none body when will 1 empty for ， this can 。
    saveRetention: patch => call('PUT', '/api/retention', patch),

    // ── （set「」）──
    // ： 1 in configdirectory agent2api.db ， not supportsdirectory
    // （ before relocateStorage / storageProgress already delete，
    // server/api/storage_api.rs module header）。
    getStorage: () => call('GET', '/api/storage'),

    // ── （ old JSON/JSONL → 1 SQLite ）──
    // when 、 not ； to pending after ， use 「」。
    // old not will delete（ as *.migrated）， server/api/upgrade_api.rs。
    getUpgrade: () => call('GET', '/api/upgrade'),
    runUpgrade: () => call('POST', '/api/upgrade/run', {}),

    // ── request（set「 use → request」）──
    // / ， after config.json（/api/retry）。
    // saveRetention：PUT ，return after 。
    getRetry: () => call('GET', '/api/retry'),
    saveRetry: patch => call('PUT', '/api/retry', patch),

    // ── up request when （set「 use → request when 」）──
    // seconds（ / etc response / empty / response），config（/api/timeouts）。
    // saveRetry：PUT ，return after 。
    getTimeouts: () => call('GET', '/api/timeouts'),
    saveTimeouts: patch => call('PUT', '/api/timeouts', patch),
    // ── etc （set「 use → etc 」）──
    // / seconds，config（/api/queue）。 saveTimeouts。
    getQueue: () => call('GET', '/api/queue'),
    saveQueue: patch => call('PUT', '/api/queue', patch),

    // ── will （set「 → 」）──
    // + / / ，config（/api/image-compress）。
    // saveQueue：PUT ，return after 。 use in for 
    // （ / ， not ）。 and web_shim.rs 
    // （ 1 = set that 1 not to ）。
    getImageCompress: () => call('GET', '/api/image-compress'),
    saveImageCompress: patch => call('PUT', '/api/image-compress', patch),

    // ── （set「 → 」）──
    // ZCode （ / / limit），
    // config（/api/zcode/captcha-pool）。 saveTimeouts。
    getCaptchaPool: () => call('GET', '/api/zcode/captcha-pool'),
    saveCaptchaPool: patch => call('PUT', '/api/zcode/captcha-pool', patch),

    // ── （set「 use → 」）──
    // config（debugMode）： after up （ already ）
    // to 1 debug_traffic ，request 「」。
    getDebug: () => call('GET', '/api/debug'),
    saveDebug: enabled => call('PUT', '/api/debug', { debugMode: enabled }),
    // by id 1 request （ not return， after debug_api module header）。
    // not to when after to 404， before 「 this request no has 」。
    getDebugTraffic: id => call('GET', '/api/debug/traffic?id=' + encodeURIComponent(id)),

    // ── ──
    onStateChanged: callback => on('accounts:state-changed', callback),
    onAutoMaintained: callback => on('accounts:auto-maintained', callback),
    // failure（）。payload is StartupFailure：
    // { message, conflict, canEndOccupant }。 is 「 1 」 ，
    // getBackendStatus as —— can can in before 。
    onBackendError: callback => on('backend:error', callback),
    onBackendReady: callback => on('backend:ready', callback),

    // ── has ： after and （optional use ） ──
    // after return { ready, phase, port, portFromEnv, failure }：failure as null 
    // ， null when { message, conflict, canEndOccupant }，
    // to not to 「 use / 」 。
    getBackendStatus: () => invoke('backend_status'),
    waitForBackend: () => invoke('wait_for_backend'),
    // / use （ to that PID， not none）
    getPortOccupant: () => invoke('port_occupant'),
    endPortOccupant: () => invoke('end_port_occupant'),
    // ：checkPort not （ when ），
    // changePort will setfile and after ，changePort already 1 
    checkPort: port => invoke('check_port', { port: Number(port) }),
    changePort: port => invoke('change_port', { port: Number(port) }),
    restartApp: () => invoke('restart_app'),

    // ── has ：（issue #48）──
    // set「」 use 。（ before 、
    // Key、） commands.rs ； shim not 
    // this 1 （ in can HTTP up ， by platform==='web'
    // not this 1 ）， and zcode-claim that not —— that need use ，
    // this 。
    panelAdminStatus: () => invoke('panel_admin_status'),
    panelRegister: (username, password) =>
      invoke('panel_register', {
        username: String(username || '').trim(),
        password: String(password || ''),
      }),
    // ── will ──────────────────────────────────────────
    // to old ； not 。
    panelSessionStatus: () => invoke('panel_session_status'),
    panelSessionStore: (accessToken, refreshToken, expiresIn) =>
      invoke('panel_session_store', {
        accessToken: String(accessToken || ''),
        refreshToken: String(refreshToken || ''),
        expiresIn: Number.isFinite(Number(expiresIn)) ? Number(expiresIn) : 0,
      }),
    panelSessionClear: () => invoke('panel_session_clear'),
    changeLanAccess: (enabled, panel) =>
      invoke('change_lan_access', { enabled: enabled === true, panel: panel === true }),
    localIp: () => invoke('local_ip'),

    // ── has ： ──
    // ：'dark' / 'light' ，null （ for Rust None）。
    // when must null， not can has ： when ，
    // WebView2 prefers-color-scheme will not is ，
    // matchMedia to ，「」 will in up 。
    // its （undefined etc ） no has ， by ， null。
    setWindowTheme: theme => invoke('set_window_theme', {
      theme: theme === 'dark' || theme === 'light' ? theme : null,
    }),

    // ── has ：（set「 → 」）──
    // is ****（0.8 ~ 1.3）， and set_zoom 1 ； up 
    // 100 。return （ will and ），
    // use 。 shim no has this （，
    // by platform==='web' this 1 use ， not will to this ）。
    setZoom: scale => invoke('set_zoom', { scale }),

    // ── has ： ──
    // （lib.rs decorations(false)）， /
    // / by （titlebar.js）。 this is for 
    // ， and its 1 1 invoke（error 1 Error）。
    // shim for to / empty（ no has use ）——
    // in not （titlebar.js platform ）， this is 。
    windowMinimize: () => invoke('window_minimize'),
    // / 。「current is 」 is windowIsMaximized
    // ： data-tauri-drag-region as ，
    // not this ， onWindowResize 
    windowToggleMaximize: () => invoke('window_toggle_maximize'),
    // = CloseRequested， and by ：「 to 」
    // when （），otherwise
    windowClose: () => invoke('window_close'),
    windowIsMaximized: () => invoke('window_is_maximized'),
    // （ / / ）。Tauri 
    // `tauri://resize` will ， after windowIsMaximized
    // can （ in ， not Tauri ）。
    // Note target not can use Any： by label to 
    // （ on ）， this use Tauri metadata current label ——
    // and invoke 1 is use when ， not when 
    onWindowResize: callback =>
      on('tauri://resize', callback, {
        kind: 'WebviewWindow',
        label: window.__TAURI_INTERNALS__.metadata.currentWebview.label,
      }),

    // ── has ： use set and ──
    // this not api_request：set in （ and after none），
    // need need use file for ， has can 。
    getAppSettings: () => invoke('get_app_settings'),
    // parameter patch must and Rust save_app_settings 1 
    saveAppSettings: patch => invoke('save_app_settings', { patch }),
    exportAccounts: () => invoke('export_accounts'),
    exportAccountFiles: selection => invoke('export_account_files', { selection }),
    importAccounts: () => invoke('import_accounts'),
    // ── / （set「 / 」）──
    // file by after by path（ api/backup_api.rs），
    // for ；parameter includeLogs must and Rust export_database 1 
    exportDatabase: (includeLogs, password) => invoke('export_database', { includeLogs, password }),
    importDatabase: password => invoke('import_database', { password }),
    // （set not can 500ms 1 ）。 use call not is 
    // invoke： this is 、noneparameter， after already has GET /api/backup/progress，
    // not as 1 。
    getBackupProgress: () => call('GET', '/api/backup/progress'),

    // ── （set「 → 」）──
    // up / down need MB， use call 60 seconds when not ， this 
    // （`cloud_sync_upload` / `cloud_sync_download`， to 300 seconds）；
    // its is request， use call can 。
    // empty = use after already that （ api::cloud_sync module header）。
    getCloudSync: () => call('GET', '/api/cloud-sync'),
    saveCloudSync: payload => call('PUT', '/api/cloud-sync', payload || {}),
    testCloudSync: payload => call('POST', '/api/cloud-sync/test', payload || {}),
    getCloudSyncRemote: () => call('GET', '/api/cloud-sync/remote'),
    cloudSyncUpload: () => invoke('cloud_sync_upload'),
    cloudSyncDownload: () => invoke('cloud_sync_download'),
    importCloudSyncKey: password => invoke('cloud_sync_import_key', { password }),
  };
})();
"#;
