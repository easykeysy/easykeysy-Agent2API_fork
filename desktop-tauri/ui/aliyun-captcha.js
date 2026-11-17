/* 阿里云验证码：V2 业务回调与 V3 验证令牌分开，所有调用共享一个可取消的流程。 */
(() => {
  const SDK_URL = 'https://o.alicdn.com/captcha-frontend/aliyunCaptcha/AliyunCaptcha.js';
  const SCRIPT_ID = 'aliyun-captcha-sdk';
  const SCRIPT_TIMEOUT_MS = 40000;
  const INIT_TIMEOUT_MS = 40000;
  const VERIFY_TIMEOUT_MS = 120000;
  const REQUEST_TIMEOUT_MS = 120000;
  const WARMUP_MS = 2100;
  const t = (key, params) => window.wbI18n?.t?.(key, params) || key;

  class CaptchaError extends Error {
    constructor(message, details = {}) {
      super(message);
      this.name = 'CaptchaError';
      this.code = details.code || '';
      this.requestId = details.requestId || '';
      this.upstreamMessage = details.upstreamMessage || '';
      this.retryAt = details.retryAt || 0;
    }
  }

  class CaptchaCancelledError extends CaptchaError {
    constructor() {
      super(t('Verification canceled'));
      this.name = 'CaptchaCancelledError';
    }
  }

  let scriptPromise = null;
  let sdkKey = '';
  let active = null;
  let sequence = 0;
  let retryAt = 0;
  let cooldownError = null;

  function language() {
    const value = String(document.documentElement.lang || 'zh-CN').replace(/_/g, '-').toLowerCase();
    if (value.startsWith('zh-hant') || value === 'zh-tw') return 'tw';
    if (value.startsWith('zh')) return 'cn';
    const code = value.split('-')[0];
    if (code === 'id') return 'in';
    return ['ar', 'de', 'es', 'fr', 'in', 'it', 'ja', 'ko', 'pt', 'ru', 'th', 'tr', 'vi'].includes(code) ? code : 'en';
  }

  function errorOf(value, fallback = t('Code verification failed; try again')) {
    if (value instanceof CaptchaError) return value;
    const source = value && typeof value === 'object' ? value : {};
    const result = source.Result || source.result || {};
    const code = String(source.VerifyCode || source.verifyCode || result.VerifyCode || result.verifyCode || source.code || source.Code || (typeof value === 'string' && /^F\d+$/.test(value) ? value : ''));
    const requestId = String(source.RequestId || source.requestId || '');
    const message = String(source.message || source.msg || source.Message || (typeof value === 'string' && value !== code ? value : '') || fallback);
    let reason = message;
    if (code === 'F010' || code === 'F011') reason = t('Verification requests are too frequent. Please try again later.');
    if (code === 'F008') reason = t('This verification token has already been used. Verify again.');
    if (['F001', 'F009', 'F024', 'F025'].includes(code)) reason = t('The upstream security service rejected verification. Please try again later.');
    const error = new CaptchaError([reason, code].filter(Boolean).join(' · '), { code, requestId, upstreamMessage: message });
    if (['F001', 'F009', 'F010', 'F011', 'F024', 'F025'].includes(code)) {
      retryAt = Date.now() + 5 * 60000;
      error.retryAt = retryAt;
      cooldownError = error;
    }
    const log = { code, requestId, message: reason };
    console.warn('[AliyunCaptcha]', log);
    window.wbApp?.debug?.(`[AliyunCaptcha] ${JSON.stringify(log)}`);
    return error;
  }

  function destroy(instance) {
    try { instance?.hide?.(); } catch {}
    try {
      if (typeof instance?.destroyCaptcha === 'function') instance.destroyCaptcha();
      else instance?.destroy?.();
    } catch {}
  }

  function current(flow) {
    return active === flow && !flow.settled;
  }

  function requireCurrent(flow) {
    if (!current(flow) || flow.signal?.aborted) throw new CaptchaCancelledError();
  }

  function settle(flow, value, error) {
    if (!current(flow)) return false;
    flow.settled = true;
    active = null;
    window.clearTimeout(flow.timer);
    flow.signal?.removeEventListener('abort', flow.onAbort);
    flow.cancelInit?.();
    flow.cancelInit = null;
    destroy(flow.instance);
    document.getElementById(flow.elementId)?.remove();
    document.getElementById(flow.buttonId)?.remove();
    if (error) flow.reject(error);
    else flow.resolve(value);
    return true;
  }

  function onPhase(flow, phase) {
    try { flow.onPhase?.(phase); } catch {}
  }

  function armTimeout(flow, delay, message) {
    window.clearTimeout(flow.timer);
    flow.timer = window.setTimeout(() => settle(flow, null, new CaptchaError(message)), delay);
  }

  function loadSdk(config) {
    const key = `${config.region}:${config.prefix}`;
    window.AliyunCaptchaConfig = { region: config.region, prefix: config.prefix };
    if (sdkKey && sdkKey !== key) {
      document.getElementById(SCRIPT_ID)?.remove();
      window.initAliyunCaptcha = undefined;
      scriptPromise = null;
    }
    sdkKey = key;
    if (typeof window.initAliyunCaptcha === 'function') return Promise.resolve();
    if (scriptPromise) return scriptPromise;
    const promise = new Promise((resolve, reject) => {
      document.getElementById(SCRIPT_ID)?.remove();
      const script = document.createElement('script');
      script.id = SCRIPT_ID;
      script.src = SDK_URL;
      script.async = true;
      const timer = window.setTimeout(() => finish(new CaptchaError(t('Verification component timed out while loading; check your network and try again'))), SCRIPT_TIMEOUT_MS);
      const finish = error => {
        window.clearTimeout(timer);
        script.onload = null;
        script.onerror = null;
        if (error) { script.remove(); reject(error); }
        else resolve();
      };
      script.onload = () => finish(typeof window.initAliyunCaptcha === 'function' ? null : new CaptchaError(t('Verification component encountered a loading error; try again')));
      script.onerror = () => finish(new CaptchaError(t('Verification component failed to load; check your network and try again')));
      document.head.appendChild(script);
    });
    scriptPromise = promise;
    promise.catch(() => { if (scriptPromise === promise) scriptPromise = null; });
    return promise;
  }

  function elements(flow) {
    const element = document.createElement('div');
    element.id = flow.elementId;
    element.style.position = 'relative';
    element.style.zIndex = '2147483000';
    const button = document.createElement('button');
    button.id = flow.buttonId;
    button.type = 'button';
    button.tabIndex = -1;
    button.setAttribute('aria-hidden', 'true');
    Object.assign(button.style, { position: 'fixed', width: '1px', height: '1px', opacity: '0', pointerEvents: 'none' });
    document.body.append(element, button);
  }

  async function submit(flow, param) {
    if (!current(flow) || flow.submitting) return null;
    if (typeof param !== 'string' || !param.trim()) {
      settle(flow, null, new CaptchaError(t('The verification component returned an incomplete token. Please retry.')));
      return null;
    }
    flow.submitting = true;
    onPhase(flow, 'submitting');
    armTimeout(flow, REQUEST_TIMEOUT_MS, t('The verified operation timed out. Refresh its status before retrying.'));
    try {
      const result = await flow.request(param);
      if (current(flow)) {
        // V2 SDK 必须先拿到业务回执，再销毁实例。
        if (flow.protocol === 'v2') window.setTimeout(() => settle(flow, result), 0);
        else settle(flow, result);
      }
      return result;
    } catch (error) {
      settle(flow, null, errorOf(error));
      return null;
    }
  }

  async function initialize(flow, config) {
    await loadSdk(config);
    requireCurrent(flow);
    elements(flow);
    await new Promise((resolve, reject) => {
      let done = false;
      const timer = window.setTimeout(() => finish(new CaptchaError(t('Verification component initialization timed out; try again'))), INIT_TIMEOUT_MS);
      const finish = error => {
        if (done) return;
        done = true;
        flow.cancelInit = null;
        window.clearTimeout(timer);
        if (error) reject(error);
        else resolve();
      };
      flow.cancelInit = () => finish(new CaptchaCancelledError());
      const fail = value => {
        if (!current(flow) || flow.submitting) return;
        const error = errorOf(value);
        finish(error);
        settle(flow, null, error);
      };
      const options = {
        SceneId: config.sceneId,
        mode: 'popup',
        element: `#${flow.elementId}`,
        button: `#${flow.buttonId}`,
        language: language(),
        showErrorTip: true,
        zIndex: 2147483000,
        getInstance: instance => {
          if (!current(flow) || done) { destroy(instance); finish(new CaptchaCancelledError()); return; }
          flow.instance = instance;
          flow.initializedAt = Date.now();
          finish();
        },
        fail,
        onError: fail,
        onClose: reason => {
          if (reason === 'userDismiss' && !flow.submitting) settle(flow, null, new CaptchaCancelledError());
        },
      };
      if (flow.protocol === 'v2') {
        options.captchaVerifyCallback = async param => {
          const result = await submit(flow, param);
          return { captchaResult: Boolean(result?.captchaResult), bizResult: Boolean(result?.bizResult) };
        };
        options.onBizResultCallback = () => {};
      } else {
        options.success = value => {
          const param = typeof value === 'string' ? value : value?.captchaVerifyParam || value?.verifyParam;
          void submit(flow, param);
        };
      }
      try { window.initAliyunCaptcha(options); } catch (error) { fail(error); }
    });
    requireCurrent(flow);
    const remaining = WARMUP_MS - (Date.now() - flow.initializedAt);
    if (remaining > 0) await new Promise(resolve => window.setTimeout(resolve, remaining));
    requireCurrent(flow);
    onPhase(flow, 'verifying');
    armTimeout(flow, VERIFY_TIMEOUT_MS, t('Code verification timed out; try again'));
    // 手动操作或已启用的定时领取可请求当次验证；模型令牌池不在后台主动验证。
    if (flow.protocol === 'v3' && typeof flow.instance?.startTracelessVerification === 'function') flow.instance.startTracelessVerification();
    else if (typeof flow.instance?.show === 'function') flow.instance.show();
    else throw new CaptchaError(t('Verification component unavailable; try again'));
  }

  function solve(config, request, signal, options = {}) {
    if (signal?.aborted) return Promise.reject(new CaptchaCancelledError());
    if (active) return Promise.reject(new CaptchaError(t('A verification is already in progress. Complete or cancel it first.')));
    if (Date.now() < retryAt && cooldownError) return Promise.reject(cooldownError);
    if (!config?.prefix || !config?.sceneId || typeof request !== 'function') {
      return Promise.reject(new CaptchaError(t('Risk-control verification config is incomplete; try again later')));
    }
    const id = ++sequence;
    return new Promise((resolve, reject) => {
      const flow = {
        protocol: options.protocol === 'v3' ? 'v3' : 'v2',
        elementId: `aliyun-captcha-element-${id}`,
        buttonId: `aliyun-captcha-trigger-${id}`,
        request, signal, resolve, reject, timer: 0, instance: null,
        initializedAt: 0, settled: false, submitting: false, onPhase: options.onPhase,
        onAbort: null,
      };
      flow.onAbort = () => { if (!flow.submitting) settle(flow, null, new CaptchaCancelledError()); };
      active = flow;
      signal?.addEventListener('abort', flow.onAbort, { once: true });
      onPhase(flow, 'preparing');
      armTimeout(flow, SCRIPT_TIMEOUT_MS + INIT_TIMEOUT_MS + WARMUP_MS, t('Verification component initialization timed out; try again'));
      void initialize(flow, { ...config, region: config.region || 'cn' }).catch(error => {
        settle(flow, null, errorOf(error));
      });
    });
  }

  function cancel() {
    return active && !active.submitting ? settle(active, null, new CaptchaCancelledError()) : false;
  }

  const requestToken = (config, signal, options = {}) => solve(config, async param => ({ param }), signal, { ...options, protocol: 'v3' });

  window.addEventListener('pagehide', () => {
    if (active) settle(active, null, new CaptchaCancelledError());
  });
  window.wbAliyunCaptcha = { solve, requestToken, cancel, isBusy: () => active !== null, CaptchaError, CaptchaCancelledError };
})();
