//! ：（ WebView or ） and after to 。
//!
//! after ：start state+authUrl → use in up →
//! wait 。 and ， not 。
//!
//! ── provider （Why this need ）────────────────
//! - **workbuddy**： after to up need state/authUrl， by after up ，
//! ** no has need **。（external） can use ，
//! as use in —— after 。
//! - ****： in success after 
//! `office-raccoon://auth/callback?code=…&state=…`， that code must by 
//! （code is to server ， not is to use ）。** in can to that URL**：
//! Electron can in will `session.protocol.handle('office-raccoon', …)` 
//! ，Tauri no has this can （Tauri custom-protocol is to webview
//! before use ， not is URL scheme ），WebView2 
//! NavigationStarting in ** before ** 1 need not need 。 this ：
//! in URL → ** down ** → URL POST to after 
//! （`/api/session/login/callback`）， by 「」。
//! ，****： down that need 
//! `office-raccoon://` （ that is client ， has ），
//! to use 1 「 not to 」 will 。
//! - **Qoder**：（PKCE）—— to use that ，
//! by （`providers::qoder::oauth`），
//! no has 。 and optional（ before use new when directory，
//! can not ）。
//! - **CatPaw**： up `{token, state}` **POST to ** loopback
//! ， no has need （ `core::login::catpaw`）。
//! - **AutoClaw （OAuth）**： and CatPaw 1 （ in 
//! loopback ），** is ** —— by ** before ** to 
//! （ before has 1 must in ，
//! [`start_autoclaw_oauth`] ）， not `start()` that 
//! 「 after need authUrl」 ， is by `start_autoclaw_oauth_login` this 
//! `{state, authUrl}`。
//!
//! ── provider use 1 ────────────────────
//! has 「」 and 「success」， after 
//! `LoginTask`：「 URL → 2 seconds 1 `/wait` → by 」。
//! 1 is （ has need need ）， workbuddy and Qoder 1 
//! path； for 1 ** new when directory**（
//! `login_profile.rs`： this is 「 when not use up 1 」 ）。
//!
//! ── User-Agent Why** not need need **（ and Electron ）─────
//! Electron `Electron/xx` ， （ has will 
//! by UA client）。Tauri/WebView2 this no has this ： no has UA
//! （`WebviewWindowBuilder::user_agent` no use ，wry not ），
//! is use is **WebView2 Edge UA**
//! （`Mozilla/5.0 … Chrome/… Edg/…`）， is 1 。
//! has ： 1 is need 1 UA（ will ），
//! is 「UA and CH-UA not 1 」 in 「 1 Electron 」。

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use serde_json::{json, Value};
use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder};

use crate::gateway;
use crate::login_profile::LoginProfile;
use crate::state::ActiveLogin;

pub const LOGIN_TIMEOUT: Duration = Duration::from_secs(5 * 60);
const POLL_INTERVAL: Duration = Duration::from_secs(2);

/// WorkBuddy 「 OneID/」 id。
///
/// up before （`index-*.js`） in `enable_oneid_only_login` as true when ， this 
/// iframe head， `#kc-social-providers`（Google / GitHub / X
/// by ） and to tab。：
///   GET https://www.workbuddy.ai/v2/plugin/login/entry-policy
///   → {"data":{"enable_oneid_only_login":true}}
/// by and Keycloak broker is （ `broker/google/login`
/// can accounts.google.com）， this can 。
const SOCIAL_HIDDEN_STYLE_ID: &str = "oneid-only-login-style";

/// up that ， Google / GitHub / X 。
///
/// ── Why use `for_all_frames` ──────────────────────────
/// is ****： `www.workbuddy.ai/login` is ， 
/// Keycloak in iframe ， by that iframe。 need 
/// in iframe ，must use `initialization_script_for_all_frames`
/// （wry in Windows up `AddScriptToExecuteOnDocumentCreated`，all frame）。
///
/// ── Why is 「」 not is 「 1 」 ────────────────────────
/// not 1 ： iframe new （ in 、） will 
/// 1 ， and 「 has no has this id」——must**** and ，
/// not is 1 。 use 1 setInterval is and not DOM ：
/// can can in DOMContentLoaded before head， that 1 。
/// 200ms 1 getElementById， can ignore。
///
/// ── ──────────────────────────────────────────────────
/// 1. in `workbuddy.ai` up ： will to accounts.google.com /
/// github.com， that not this 。
/// 2. ** that 1 id** style， not ， not `switchTab` as ——
/// use default is up that tab， is not 。
fn social_restore_script() -> String {
    // id from ， this 1 （ 1 not 1 is 
    // 「 in no 」 this no has ）。
    format!(
        r#"
(function () {{
  var STYLE_ID = '{style_id}';
  if (!/(^|\.)workbuddy\.ai$/.test(location.hostname)) return;
  function strip() {{
    var el = document.getElementById(STYLE_ID);
    if (el && el.parentNode) el.parentNode.removeChild(el);
  }}
  strip();
  setInterval(strip, 200);
}})();
"#,
        style_id = SOCIAL_HIDDEN_STYLE_ID,
    )
}

/// KukuAI ：、、 etc 、。
///
/// ── not by ─────────────────────────────────────
/// passport success after `success.html`，`BDUSS` is **HttpOnly** Cookie，
/// `document.cookie` not to —— by in `run_embedded` etc 
/// `success.html` after WebView2 Cookie 
/// （`cookies()` HttpOnly， [`submit_kuku_login`]）。
/// 「 use 」： by `start_kuku_sms` ， by 
/// `submit_kuku_sms_code` `window.eval("window.__kukuSmsCode='…'")`
/// （ to ）。
///
/// ── ─────────────────────────────────────
/// passport for IP / can can need （ up 1 ）。
/// to can `document.title` up （ 
/// ）， use after ，。
///
/// ── React ─────────────────────────────────────────
/// passport is React ， `el.value = x` not will React ，
/// must value setter + `input` （ use is `TANGRAM__PSP_4`
/// before ， `loginMerge.html` and use passport 1 DOM，
/// asar for and 2026-10-07 ）。
fn kuku_sms_script(phone: &str) -> String {
    format!(
        r#"
(function () {{
  // ── ──────────────────────────────────────────────
  // will ** new **（initialization_script when in 
  // WebView2 when can can location ， has no 
  // ； eval ）。 1 document by this flag ；
  // after window ，flag 、 new 。
  if (window.__kukuInjected) return;
  window.__kukuInjected = true;
  var PHONE = '{phone}';
  // ── （ not TANGRAM__PSP_4 before ）──────────────────
  // （PSP_4 is loginMerge.html current）， has 
  // `__smsPhone` this ** after ** is 。（2026-10-07）
  // name is username、 name is password， and ——
  // ** not by name **， 1 id after 。
  function q(sel) {{ return document.querySelector(sel); }}
  function findAll(sel) {{ return Array.prototype.slice.call(document.querySelectorAll(sel)); }}
  function phoneInput() {{ return q('[id$="__smsPhone"]'); }}
  function codeInput() {{ return q('[id$="__smsVerifyCode"]'); }}
  function sendBtn() {{
    return findAll('button, a').find(function (n) {{
      return /发送动态密码|获取验证码/.test(n.textContent || '');
    }});
  }}
  function submitBtn() {{ return q('[id$="__smsSubmit"]'); }}
  // ── up （ document.title ， start_kuku_sms）──
  // eval not to （fire-and-forget），title is can 。
  // in （READY/FILLED/CLICKED） and before can no 。
  var T_READY = 'WB_KUKU_READY';          // 脚本已启动（页面加载完成）
  var T_FILLED = 'WB_KUKU_PHONE_FILLED';  // 手机号已填入
  var T_CLICKED = 'WB_KUKU_SEND_CLICKED'; // 已点「发送动态密码」
  var T_SENT = 'WB_KUKU_SMS_SENT';        // 短信已发出：保持隐藏等用户填码
  var T_CAPTCHA = 'WB_KUKU_NEED_CAPTCHA'; // 需要图形验证码：亮窗让用户填
  var T_LOGIN_OK = 'WB_KUKU_LOGIN_OK';    // 登录成功（同页「提交成功」形态）
  function report(title) {{
    try {{ document.title = title; }} catch (e) {{}}
  }}
  function loginDone() {{
    // loginMerge.html success** not **， in 「，success！」
    // （ 2026-10-07）。（BDUSS by Cookie ）。
    var body = (document.body && document.body.innerText) || '';
    return /提交成功|登录成功/.test(body);
  }}
  function sendDone() {{
    // success ： by when （「 new (48)」
    // or +s when ， and disabled）。Note** not can ** use 「」
    // —— after by ， use old then not （ bug）。
    var b = findAll('button').find(function (n) {{
      return n.disabled && /重新发送|\d+\s*s/.test(n.textContent || '');
    }});
    return Boolean(b);
  }}
  function captchaBoxVisible() {{
    // （passport verifyCode）。
    // use 1 verifyCode —— after 
    // if by already when （ already ）， that and none， down 。
    var boxes = findAll('input[id*="verifyCode"], input[id*="vcode"]');
    var v = boxes.find(function (n) {{
      var style = window.getComputedStyle(n);
      return style.display !== 'none' && style.visibility !== 'hidden';
    }});
    return Boolean(v);
  }}
  var timer = setInterval(function () {{
    try {{
      // no to （ can can in about:blank）： etc down 1 。
      // check in not is —— check will 「 location
      // 」 this 1 （： not ）。
      if (location.hostname.indexOf('baidu.com') === -1) return;
      // 0. up （， not will title）。 need ：
      // success（「success」） —— when by when 
      // can can in ， not can success； already （ when ）
      // （ when has can ** not is **， 2026-10-07）。
      if (loginDone()) {{
        report(T_LOGIN_OK);
      }} else if (sendDone() || window.__kukuSentOk) {{
        window.__kukuSentOk = true;
        report(T_SENT);
      }} else if (captchaBoxVisible()) {{
        report(T_CAPTCHA);
      }} else if (!window.__kukuReady) {{
        window.__kukuReady = true;
        report(T_READY);
      }}
      // 1. to 「」tab（ 1 can ；tab not in already is ）
      if (!window.__kukuTab) {{
        var tab = q('a.pass-sms-btn') || findAll('a, span, div').find(function (n) {{
          return /短信快捷登录|短信登录/.test(n.textContent || '');
        }});
        if (tab) {{ window.__kukuTab = true; tab.click(); }}
      }}
      // 2. + 「」
      var phone = phoneInput();
      if (phone && !window.__kukuSent) {{
        window.__kukuSent = true;
        report(T_FILLED);
        var setter = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, 'value').set;
        setter.call(phone, PHONE);
        phone.dispatchEvent(new Event('input', {{ bubbles: true }}));
        setTimeout(function () {{
          var b = sendBtn();
          if (b) {{
            report(T_CLICKED);
            b.click();
          }}
        }}, 400);
      }}
      // 3. + 「 and 」 in by 
      var code = codeInput();
      if (code && window.__kukuSmsCode) {{
        // 1 （ not empty not use ）； by ——
        // passport by can can disabled（ no ），
        // 1 flag will that 。
        if (!window.__kukuCodeFilled) {{
          window.__kukuCodeFilled = true;
          var setter2 = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, 'value').set;
          setter2.call(code, window.__kukuSmsCode);
          code.dispatchEvent(new Event('input', {{ bubbles: true }}));
        }}
        var sub = submitBtn();
        if (sub && !sub.disabled && !window.__kukuSubmitting) {{
          window.__kukuSubmitting = true;
          sub.click();
          // if invalid（ by after ），8 seconds after 1 
          setTimeout(function () {{ window.__kukuSubmitting = false; }}, 8000);
        }}
      }}
      // not limit：，（ to Cookie after Done）
      // or —— use / when need 。
    }} catch (e) {{}}
  }}, 500);
}})();
"#,
        phone = phone,
    )
}

/// KukuAI ：（） after **return**，
/// in after —— by [`submit_kuku_sms_code`] in use after 。
///
/// ── Why not in this etc ────────────────────────────────────
/// is 「 → → 」， after must etc use ；
/// 1 etc to ， before etc not to verify that 1 。 this 
/// **spawn after etc **（ use `run_embedded`： etc `/wait` Done / failure / when 
/// after ），return。
pub async fn start_kuku_sms(app: &AppHandle, phone: &str) -> Result<serde_json::Value, String> {
    if current_login(app).is_some() {
        return Err("已有登录流程在进行，请先完成当前登录或等待超时".to_string());
    }
    let started = gateway::call(
        "POST",
        "/api/session/login/start",
        Some(&json!({ "provider": "kuku" })),
    )
    .await?;
    let login_state = started
        .get("state")
        .and_then(serde_json::Value::as_str)
        .ok_or("后端未返回登录状态")?
        .to_string();
    let auth_url = started
        .get("authUrl")
        .and_then(serde_json::Value::as_str)
        .ok_or("后端未返回登录链接，请检查网络")?
        .to_string();

    {
        let state = app.state::<crate::state::AppState>();
        let mut guard = state.login.lock().map_err(|_| "登录状态锁不可用")?;
        *guard = Some(ActiveLogin {
            state: login_state.clone(),
            edition: String::new(),
            mode: "sms".into(),
            provider: "kuku".to_string(),
        });
    }
    emit_login_state(
        app,
        LoginState {
            active: true,
            mode: Some("sms".into()),
            edition: None,
            provider: Some("kuku".to_string()),
        },
    );

    // after and ：
    // 1. run_embedded —— ， / etc / ，
    // after （Done / failure / when ） and ；
    // 2. —— 「 need need when 」：
    // - up 「 need need 」（）→ ，
    // use after ，；
    // - 90 seconds no etc to （ / etc ）→ ，
    // use in that （ will BDUSS）。
    // failure： in after ，`submit_kuku_sms_code` 
    // `/wait` when will to and to before 。
    let app_for_task = app.clone();
    let login_state_for_task = login_state.clone();
    let phone_for_task = phone.to_string();
    let monitor_label = login_window_label(&login_state_for_task);
    tauri::async_runtime::spawn(async move {
        let monitor = async {
            // 1 seconds 1 （ document.title up ）。
            // ， not use etc ：
            // - 10 seconds no has → no / no ，
            // （ use can to ）；
            // - 20 seconds no success → can can ，；
            // - 30 seconds。
            // ： + `login:kuku-status` to before 
            // （ when ， use not 「 no 」）。
            let mut last_seen = String::new();
            let emit_status = |status: &str| {
                let _ = app_for_task.emit("login:kuku-status", json!({ "status": status }));
            };
            let mut saw_script = false;
            for tick in 0..30 {
                tokio::time::sleep(Duration::from_secs(1)).await;
                let Some(window) = app_for_task.get_webview_window(&monitor_label) else {
                    return; // 窗口已销毁（登录完成/取消），无事可监
                };
                let title = window.title().unwrap_or_default();
                if title != last_seen {
                    eprintln!("[login] KukuAI 短信窗口状态: {title}");
                    last_seen = title.clone();
                    match title.as_str() {
                        "WB_KUKU_SMS_SENT" => {
                            emit_status("sent");
                            return; // 短信已发出：保持隐藏，等用户填码
                        }
                        "WB_KUKU_NEED_CAPTCHA" => {
                            emit_status("captcha");
                            let _ = window.show();
                            return;
                        }
                        "WB_KUKU_READY" => {
                            saw_script = true;
                            emit_status("opening");
                        }
                        "WB_KUKU_PHONE_FILLED" => {
                            saw_script = true;
                            emit_status("filling");
                        }
                        "WB_KUKU_SEND_CLICKED" => {
                            saw_script = true;
                            emit_status("sending");
                        }
                        _ => {}
                    }
                }
                // 10 seconds no has ： no / no → 
                if tick == 10 && !saw_script {
                    eprintln!("[login] KukuAI 短信窗口 10 秒无脚本状态，亮窗让用户手动完成");
                    emit_status("timeout");
                    let _ = window.show();
                    return;
                }
            }
            // 30 seconds：（ use can ， by ）
            if let Some(window) = app_for_task.get_webview_window(&monitor_label) {
                eprintln!("[login] KukuAI 短信发码 30 秒未确认，亮窗让用户手动完成");
                emit_status("timeout");
                let _ = window.show();
            }
        };
        let run = run_embedded(
            &app_for_task,
            "kuku",
            &auth_url,
            "登录 KukuAI 账号",
            &login_state_for_task,
            false,
            Some(&phone_for_task),
            true, // start_hidden：窗口隐藏，全程脚本后台操作
        );
        let (result, ()) = tokio::join!(run, monitor);
        if result.as_ref().map(|value| value.get("ok").and_then(serde_json::Value::as_bool)) != Ok(Some(true)) {
            let _ = gateway::call(
                "POST",
                "/api/session/login/cancel",
                Some(&json!({ "state": login_state_for_task.clone() })),
            )
            .await;
        }
        clear_active_login(&app_for_task, &login_state_for_task);
        if let Err(error) = result {
            eprintln!("[login] KukuAI 短信登录后台流程结束: {error}");
        }
    });

    Ok(json!({ "state": login_state }))
}

/// KukuAI ： use to ， etc 。
///
/// ：`window.eval("window.__kukuSmsCode='…'")` —— 500ms
/// this ， to （ [`kuku_sms_script`] 3 ）。
///
/// after `/wait` to after Done（、 already ） or failure / when ；
/// after `/api/accounts` this return `{account, list}`（ and 
/// `POST /api/accounts` response， before to 「 already 」）。
pub async fn submit_kuku_sms_code(
    app: &AppHandle,
    code: &str,
) -> Result<serde_json::Value, String> {
    let active = current_login(app).ok_or("没有进行中的 KukuAI 短信登录，请先点「获取验证码」")?;
    if active.provider != "kuku" || active.mode != "sms" {
        return Err("没有进行中的 KukuAI 短信登录，请先点「获取验证码」".to_string());
    }
    let window_label = login_window_label(&active.state);
    let Some(window) = app.get_webview_window(&window_label) else {
        return Err("KukuAI 登录窗口已关闭，请重新发起短信登录".to_string());
    };
    let code = code.trim().to_string();
    if code.len() != 6 || !code.chars().all(|ch| ch.is_ascii_digit()) {
        return Err("请填写 6 位数字验证码".to_string());
    }
    // etc （ and run_embedded etc 1 ：/wait ）
    let deadline = tokio::time::Instant::now() + LOGIN_TIMEOUT;
    loop {
        if tokio::time::Instant::now() >= deadline {
            return Err("等待登录完成超时，请重试".to_string());
        }
        // （fire-and-forget）：passport can can in after 
        // new （ / ）， 1 will old 1 ——
        // 500ms this ， new to 。
        let _ = window.eval(&format!("window.__kukuSmsCode = '{}';", code));
        tokio::time::sleep(POLL_INTERVAL).await;
        match poll_once(&active.state).await {
            PollOutcome::Pending(_) => continue,
            PollOutcome::Done(_) => break,
            PollOutcome::Failed(error) => return Err(error),
            PollOutcome::Unreachable(error) => {
                if task_gone(&error) {
                    return Err("登录已被取消或过期，请重新发起".to_string());
                }
                eprintln!("[login] 读取登录进度失败（继续等待）: {error}");
                continue;
            }
        }
    }
    // ：， KukuAI return（ and response）
    let accounts = gateway::call("GET", "/api/accounts", None)
        .await
        .map_err(|error| format!("登录完成，但读取账号列表失败：{error}"))?;
    let list = accounts
        .get("list")
        .cloned()
        .unwrap_or_else(|| serde_json::json!([]));
    let account = list
        .as_array()
        .and_then(|items| {
            items
                .iter()
                .filter(|item| item.get("provider").and_then(serde_json::Value::as_str) == Some("kuku"))
                .max_by_key(|item| item.get("addedAt").and_then(serde_json::Value::as_i64).unwrap_or(0))
                .cloned()
        })
        .unwrap_or_else(|| serde_json::json!({ "provider": "kuku", "name": "KukuAI 账号" }));
    Ok(json!({ "account": account, "list": list }))
}

/// workbuddy ， cli/product.json 
/// internalDomain / externalDomain / iOADomain， need /QQ 。
///
/// **（Google / GitHub / X） not in this **： is optional，
/// has 「」 when ， [`SOCIAL_IDENTITY_HOSTS`]
/// and [`host_allowed`]。
const WORKBUDDY_ALLOWED_HOSTS: &[&str] = &[
    "copilot.tencent.com",
    "staging-copilot.tencent.com",
    "codebuddy.cn",
    "workbuddy.cn",
    "codebuddy.ai",
    "staging-codebuddy.tencent.com",
    "workbuddy.ai",
    "staging.workbuddy.ai",
    "tencent.com",
    "qq.com",
    "wechat.com",
    "weixin.qq.com",
    "tenpay.com",
];

/// 「 Google / GitHub 」 when 。
///
/// ── Whymust ────────────────────────────────────────
/// by after ** not in iframe **： `handleAuthLogin` is 
/// `window.parent.location.href = url`（，）。 is will 
/// Google / GitHub， after Keycloak broker workbuddy.ai。
/// this not in will `host_allowed` down ， is 「 Google 
/// 1 empty」—— is `allowed_hosts` that 。
///
/// ── ──────────────────────────────────────────
/// Google after is `accounts.google.com`（），GitHub 
/// `github.com`。 its is up and 
/// （Google gstatic ，GitHub githubusercontent etc ），
/// will 「 can to 、 not 」 。
const SOCIAL_IDENTITY_HOSTS: &[&str] = &[
    "accounts.google.com",
    "accounts.youtube.com",
    "google.com",
    "gstatic.com",
    "googleusercontent.com",
    "github.com",
    "githubusercontent.com",
    "githubassets.com",
    "twitter.com",
    "x.com",
    "twimg.com",
];

/// 。
///
/// ── Why not is 「 has xiaohuanxiong.com」──────────────────────
/// in `https://xiaohuanxiong.com/code/authorize`， / 
/// in （、、 has ）， is path。
/// this ** `LOGIN_ALLOWED_HOSTS`**（`desktop/main.cjs`）——
/// that is 1 up use ， not is ；「」 
/// 1 ， will 「 not 、 in 」 1 
/// has use can to new 。
///
/// `RACCOON_MAIN_SITE_URL` to （） when this ** not **：
/// 1 can config etc no has ， this and new 。
const RACCOON_ALLOWED_HOSTS: &[&str] = &[
    "xiaohuanxiong.com",
    "sensetime.com",
    "aliyun.com",
    "aliyuncs.com",
    "alicdn.com",
    "qq.com",
    "baidu.com",
];

/// （`office-raccoon://auth/callback`）， and after 
/// `raccoon::oauth` must 1 （ that ， this ）。
const RACCOON_CALLBACK_SCHEME: &str = "office-raccoon";
const RACCOON_CALLBACK_HOST: &str = "auth";
const RACCOON_CALLBACK_PATH: &str = "/callback";

/// Qoder and CatPaw ：**unlimited**。
///
/// ── Why this not ───────────────────────────────
/// will ， and not can ：
/// - Qoder by use to （Google / GitHub），
/// in `qoder.com`， SSO not ；
/// - CatPaw passport（`passport.meituan.com` → `settoken` →
/// `catpaw.meituan.com` `login-callback`）， will / 
/// this ， no has 。
///
/// `on_navigation` return false is ** down **： 1 as 「
/// 1 empty」 or 「 no 」 this —— CatPaw when 
/// is as this 1 、default to WorkBuddy 。
///
/// in this can ： not 「 」——
/// Qoder need in PKCE verifier can （
/// `providers::qoder::oauth`）；CatPaw token by up POST to 
/// loopback ， and need 1 state （ `core::login::catpaw`）。
/// and use is when directory（`login_profile`）， not 。
///
/// ── Why workbuddy / ─────────────────────
/// that is （ / + ），
/// can ； in this is 1 ， not 。
///
/// ── AutoClaw Why not （）───────────────────
/// and not in （Zai 、Google 
/// `accounts.google.com`），** need is in **： after will 
/// ** to `http://localhost:<>/auth/callback-zai|google`**
/// （z.ai client that ）。
/// that loopback 1 down ， is 「 use success、
/// etc not to 」—— and [`is_login_callback`] that ，
/// this not ： is 1 HTTP ，
/// no has 「」 can ， can in this 1 。
/// （ that by **** and down ， not and —— 
/// `autoclaw/callback_server.rs`。must to this 。）
fn allowed_hosts(provider: &str) -> Option<&'static [&'static str]> {
    match provider {
        "raccoon" => Some(RACCOON_ALLOWED_HOSTS),
        "doubaowork" => Some(&["doubao.com", "feishu.cn", "larksuite.com"]),
        // CatPaw / Qoder / Cline / AutoClaw not （ by up ）；** new 
        // provider when not need to default** —— that will use WorkBuddy 
        // ， is （AutoClaw that will 1 ）。
        //
        // Cline its need Note: in `authkit.cline.bot`， can can 
        // （WorkOS AuthKit will by config to Google / GitHub / SSO
        // etc not can ）—— and Qoder 1 ， not 。
        //
        // AutoClaw ****： before not to this （ no has 
        // ）， 1 up is has —— ， up 
        // OAuth will default to WorkBuddy ， that 。
        // Accio not ：（www.accio.com / www.accio-ai.com）
        // can can use to not can （Google / / ），
        // and Qoder / AutoClaw 1 。
        //
        // ZCode not ： in `zcode.z.ai`， by use 
        // （Z.ai / Google / or ） to 
        // ， not can 。**must** —— default will to WorkBuddy
        // ， is up that ： 1 empty， up not 。
        // ： use 1 ， 1 will in will use when to 。
        //
        // CodeArts not ： in `codearts.huaweicloud.com/portal/authorize`，
        // will by use as （`account.huaweicloud.com`）、
        // IAM 、 etc not can ； after need to ** ** loopback 
        // （portal to port，path by ，
        // `providers::codearts::oauth::authorize_url`）。 that loopback 1 
        // down ， is up that ： use in success，
        // etc not to 。
        //
        // Trae must in this ：default will to WorkBuddy ，
        // that ** no has ** `trae.cn`（）** no has ** `127.0.0.1`（），
        // is file up that —— 1 empty， not 。
        //
        // Antigravity not ： in `accounts.google.com`， Google 
        // can can not can （ / SSO etc ）； need is 
        // in ** ** loopback 
        // （`http://localhost:{}/oauth-callback`，
        // `providers::antigravity::oauth`）。 that loopback 1 down ，
        // is up that ： use success， etc not to 
        // —— is 1 HTTP ， no has can 。
        //
        // OfficeAce not ： in `auth.huaweicloud.com`， will by use 
        // as / etc not can （ and CodeArts 1 ）。
        // ** not use ** —— by ****（`/v1/claw/auth/code`，
        // `providers::officeace::oauth` module header）， this no has CodeArts that 
        // loopback need ；「 not 」 is must （default
        // will to WorkBuddy → ）。
//
        // OrcaTerm（） not ： in OAuth 
        // （`orcaterm.com/oauth/authorize`）， will 302 to `open.weixin.qq.com`
        // （`qrconnect`）， will `res.wx.qq.com` and 
        // `localhost.weixin.qq.com` —— not can 。****
        // （default will to WorkBuddy → ，
        // not ）。
        //
        // AtomCode not ： by to （`acs.atomgit.com/auth/login`
        // `login_url`， in AtomGit up ）， after will by use 
        // not can 。** not use ** —— by ****
        // （`auth/check` → `auth/token`， `providers::atomcode::oauth` 
        // module header）， this no has loopback need ；「 not 」 is must
        // 。
        //
        // LobsterAI not ， and this 1 **must is None**： after will 
        // **302 **（`http://127.0.0.1:<>/auth/callback`）， that 
        // 1 down ， is use success、 etc not to ，
        // not （ and CodeArts / Trae loopback 1 ）。
        // in `lobsterai.youdao.com`， can can has 。
        // in `qwenwork.cn` / `.ai`（ will 302 to 
        // `/biz/signin`， not can ）；2026-11-03 has 1 
        // ****（`http://127.0.0.1:<>/callback`，
        // `providers/qoder/authorize.rs`）—— that down is 
        // 「success not to 」。 and Qoder ，must is None。
        "catpaw" | "qoder" | "qoder-intl" | "qwenwork" | "qwenwork-intl" | "cline-free"
        | "cline-pass" | "autoclaw"
        | "autoclaw-intl" | "accio" | "accio-cn" | "zcode" | "zcode-intl" | "codearts" | "trae"
        | "kuku" | "antigravity" | "officeace" | "orcaterm" | "atomcode" | "lobsterai" => None,
        // WorkBuddy use this 1 （ `workbuddy.ai` that 1 is 
        // ）。**** not is down default： up that need 
        // 「 new provider to default」must is has ， is 
        // （ after `workbuddy-intl` is new id，default when none from ）。
        "workbuddy" | "workbuddy-intl" => Some(WORKBUDDY_ALLOWED_HOSTS),
        _ => Some(WORKBUDDY_ALLOWED_HOSTS),
    }
}

/// this URL is in 。
///
/// Note `office-raccoon:` this in this ** 1 return false**，
/// must —— will ， that 
/// and 「 down 」 in WebView2 is 1 ， not ，
/// as 「 use success 1 no to code」。
///
/// `social_restore` when for WorkBuddy [`SOCIAL_IDENTITY_HOSTS`]：
/// Google / GitHub by is ， not will （ by this ）。
fn host_allowed(url: &url::Url, provider: &str, social_restore: bool) -> bool {
    if provider == "deepseek-web" {
        return url.as_str() == "about:blank" || crate::deepseek_login::allowed(url);
    }
    if provider == "doubaowork" {
        return url.as_str() == "about:blank" || crate::doubaowork_login::allowed(url);
    }
    if let Some(region) = crate::server::core::providers::monkeycode::Region::from_provider_id(provider) {
        return url.as_str() == "about:blank" || crate::monkeycode_login::allowed(url, region);
    }
    match url.scheme() {
        "about" | "data" => true,
        "https" | "http" => {
            let Some(allowed) = allowed_hosts(provider) else {
                // unlimited（Qoder： not can ， by allowed_hosts）
                return true;
            };
            let Some(host) = url.host_str() else {
                return false;
            };
            let host = host.to_lowercase();
            let matched = |list: &[&str]| {
                list.iter()
                    .any(|item| host == *item || host.ends_with(&format!(".{item}")))
            };
            // `social_restore` to this already is ****（ use 「WorkBuddy
            // + + use 」 1 ，
            // `run_embedded` use ）， this not provider —— 
            // after is after （`workbuddy-intl`） ，Google / GitHub
            // by down 。
            matched(allowed)
                || (social_restore && matched(SOCIAL_IDENTITY_HOSTS))
        }
        // http(s) its 1 （ up about / data）：
        // in WebView2 will to ， not this by 
        _ => false,
    }
}

/// this URL is not is provider （ for scheme/host/path）。
///
/// host not ： in `url` crate opaque host ， not 
/// http(s) that ，`office-raccoon://AUTH/callback` will 。
/// this is ****（ after `raccoon::oauth::parse_callback_code` is ，
/// has 1 1 ）； 1 ， 1 URL in and after to not 。
///
/// ** has and OrcaTerm **：workbuddy by after up 
/// ，Qoder （ `providers::qoder::oauth`）， no has 
/// —— 「 no has 」「」。
///
/// ── OrcaTerm Why（）───────────────────────────
/// is that ， as 
/// `orcaterm://oauth_callback?session_id=…`（
/// `providers/orcaterm/login.rs` module header）。 that URL `session_id` is 
/// ** 1 **， after —— must down 
/// （ to = no client when is 1 error， not ）。
fn is_login_callback(url: &url::Url, provider: &str) -> bool {
    if provider == "orcaterm" {
        let host_matches = url
            .host_str()
            .map(str::to_ascii_lowercase)
            .is_some_and(|host| host == "oauth_callback");
        return url.scheme() == "orcaterm" && host_matches;
    }
    if provider != "raccoon" {
        return false;
    }
    let host_matches = url
        .host_str()
        .map(str::to_ascii_lowercase)
        .is_some_and(|host| host == RACCOON_CALLBACK_HOST);
    url.scheme() == RACCOON_CALLBACK_SCHEME && host_matches && url.path() == RACCOON_CALLBACK_PATH
}

/// 1 before provider id（ workbuddy， this ）。
///
/// ── Why Cline 「」 this 1 ─────────────────────
/// is this IPC（ 、 after ），
/// and Qoder ； and not this 
/// （ that `POST /api/accounts`）。
fn normalize_provider(provider: &str) -> Result<&'static str, String> {
    if let Some(region) = crate::server::core::providers::monkeycode::Region::from_provider_id(provider.trim()) {
        return Ok(region.provider_id());
    }
    match provider.trim() {
        "" | "workbuddy" => Ok("workbuddy"),
        // WorkBuddy （2026-10 after is provider）： that 
        // 「」 by in provider id， is 1 
        // ， in after by 1 。** this 1 not can ** ——
        // before will to 「unknown provider」 by failure。
        "workbuddy-intl" => Ok("workbuddy-intl"),
        "raccoon" => Ok("raccoon"),
        "doubaowork" => Ok("doubaowork"),
        // Qoder （2026-10 after is provider）：，
        // provider id， by after by id （ and ZCode ）。
        "qoder" => Ok("qoder"),
        "qoder-intl" => Ok("qoder-intl"),
        // ：** and Qoder 1 **（ by by 
        // 、， `providers::qoder::oauth`）， is 
        // provider id、 by by id 。** this 1 not can ** ——
        // 「」 down is 「unsupported ：qwenwork」。
        "qwenwork" => Ok("qwenwork"),
        "qwenwork-intl" => Ok("qwenwork-intl"),
        "catpaw" => Ok("catpaw"),
        "cline-free" => Ok("cline-free"),
        "cline-pass" => Ok("cline-pass"),
        "autoclaw" | "autoclaw-intl" => Ok("autoclaw-intl"),
        // Accio ： by after （PKCE）， and 
        // —— and workbuddy / Qoder 1 。 by **provider **
        // （ provider）， not 1 。
        "accio" => Ok("accio"),
        "accio-cn" => Ok("accio-cn"),
        // ZCode ： by ** after ** up （`/oauth/cli/init`），
        // and —— and Qoder / Cline 1 。
        //
        // this 1 Note: ** no has to **
        // （ `providers::zcode::oauth` module header）， by ZCode server
        // 。file etc after `/api/session/login/wait`
        // （ `poll_once` and `run_embedded` etc ）， not 
        // to —— this ： after in `zcode://`
        // up not ，** not **。
        "zcode" => Ok("zcode"),
        "zcode-intl" => Ok("zcode-intl"),
        // CodeArts（ as ）： by ** after **（PKCE + URL，
        // `providers::codearts::oauth::authorize_url`）， and 
        // —— and ZCode 1 。 by portal 
        // `http://127.0.0.1:<>/oauth/callback` ，
        // **must** to （ down allowed_hosts 1 ）。
        "codearts" => Ok("codearts"),
        // LobsterAI（ has 「 has 」）： by ** after **
        // （ hash by + `redirect_uri` to that loopback ，
        // `providers::lobsterai::oauth::build_authorize_url`），
        // and —— and CodeArts 1 。 in 
        // `http://127.0.0.1:<>/auth/callback`（**path**， by 
        // `http.rs` that by ），**must** to ，
        // down allowed_hosts 1 。
        "lobsterai" => Ok("lobsterai"),
        // Trae（SOLO CN）： by ** after ** up guidance （PKCE + URL），
        // and —— and ZCode 1 。， in 
        // `http://127.0.0.1:<>/authorize`（ that loopback，
        // up this by then ），**must** to ，
        // otherwise use 、request not （ down allowed_hosts 1 ）。
        "trae" => Ok("trae"),
        // Antigravity： by ** after **（Google OAuth ，
        // `providers::antigravity::oauth::build_authorize_url`），
        // and —— and CodeArts 1 。 in 
        // `http://localhost:{}/oauth-callback`（ to Google 
        // redirect_uri），**must** to （ down 
        // allowed_hosts 1 ），otherwise use success after etc not to 。
        "antigravity" => Ok("antigravity"),
        // KukuAI（）： by ** after **（client
        // `passApi/html/loginMerge.html`， `providers::kuku::login::build_login_url`），
        // （； already ）、 and in 
        // success `success.html` after Cookie 
        // （`POST /api/session/login/kuku/complete`， `run_embedded` 
        // etc and `submit_kuku_login`）。
        "kuku" => Ok("kuku"),
        // OfficeAce（ as / OfficeClaw）： by ** after **
        // （ to need state， `providers::officeace::oauth`），
        // and —— and ZCode / CodeArts 1 。 by ****
        // （ not ）， after in up not 。
        "officeace" => Ok("officeace"),
        // OrcaTerm（）： by ** after **return（ OAuth ，
        // will 302 to ）， and 
        // `orcaterm://oauth_callback?session_id=…` 
        // （ `start_orcaterm` and `providers/orcaterm/login.rs` module header）。
        "orcaterm" => Ok("orcaterm"),
        // AtomCode（AtomGit / GitCode）： by ** after ** to 
        // （`providers::atomcode::oauth`）， and —— and 
        // ZCode / OfficeAce 1 。 by in 
        // `acs.atomgit.com/callback` 、** not to **， by 
        // （`auth/check` → `auth/token`）， after in up 
        // not ， not use not use 。
        //
        // ** this 1 not can **： before （`add-account-configs.ts` ATOMCODE） 
        // 「」 by ， down is 
        // 「failure：unsupported ：atomcode」。
        "atomcode" => Ok("atomcode"),
        other => Err(format!("不支持网页登录的提供商：{other}")),
    }
}

/// AutoClaw OAuth ：** already by before **， this + etc 。
///
/// ── Why this and not 1 ───────────────────────
/// that need by after up （workbuddy / CatPaw / Qoder / Cline），
/// need （）， 1 「POST /start → authUrl → 」。
/// AutoClaw 1 **must in **（
/// SDK， `providers::autoclaw::oauth` module header）， is 
/// that —— is ：** before 、parameter
/// ， to **。
///
/// this is `state` + `authUrl`（ not is 
/// `/login/start`）—— that before already to ， 1 etc 1 
/// （ will ， its in 1 ）。
///
/// ── not need need ──────────────────────────────────────
/// by ** 302 to loopback **
/// （`navigate_uri` is ）， not use 、
/// not use —— and CatPaw that 1 （ `core::login::catpaw`）。
/// this `mode = "external"`（）**available**：
/// in ， and in none。
pub async fn start_autoclaw_oauth(
    app: &AppHandle,
    state: String,
    auth_url: String,
    mode: &str,
) -> Result<serde_json::Value, String> {
    if state.trim().is_empty() || auth_url.trim().is_empty() {
        return Err("缺少登录状态或授权地址，请重新发起".to_string());
    }
    let use_external = mode == "external";
    let mode_id = if use_external { "external" } else { "embedded" };
    {
        let state_guard = app.state::<crate::state::AppState>();
        let mut guard = state_guard.login.lock().map_err(|_| "登录状态锁不可用")?;
        *guard = Some(ActiveLogin {
            state: state.clone(),
            edition: "intl".into(),
            mode: mode_id.into(),
            provider: "autoclaw-intl".to_string(),
        });
    }
    emit_login_state(
        app,
        LoginState {
            active: true,
            mode: Some(mode_id.into()),
            edition: Some("intl".into()),
            provider: Some("autoclaw-intl".to_string()),
        },
    );

    let result = if use_external {
        open_external(app, &auth_url, "AutoClaw 国际版登录页", &state).await
    } else {
        // social_restore false： that can WorkBuddy 。
        // provider "autoclaw-intl"：（ this not ， allowed_hosts
        // AutoClaw —— in loopback， will 1 ）。
        run_embedded(
            app,
            "autoclaw-intl",
            &auth_url,
            "登录 AutoClaw 国际版账号",
            &state,
            false,
            None,
            false,
        )
        .await
    };
    if result
        .as_ref()
        .map(|value| value.get("ok").and_then(serde_json::Value::as_bool))
        != Ok(Some(true))
    {
        let _ = gateway::call(
            "POST",
            "/api/session/login/cancel",
            Some(&json!({ "state": state })),
        )
        .await;
    }
    clear_active_login(app, &state);
    result
}

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginState {
    pub active: bool,
    pub mode: Option<String>,
    pub edition: Option<String>,
    /// in provider（ before by ， applyLoginState use ）
    pub provider: Option<String>,
}

pub fn current_login(app: &AppHandle) -> Option<ActiveLogin> {
    app.state::<crate::state::AppState>()
        .login
        .lock()
        .ok()
        .and_then(|guard| guard.clone())
}

/// to （ by enabled/ use and ）
fn emit_login_state(app: &AppHandle, state: LoginState) {
    let _ = app.emit("login:state", state);
}

/// 1 `/wait` 。
///
/// ── Why need 「failure」 and 「 when not to 」 ────────
/// 「 1 、」， is ****failure need etc to 
/// 5 minutes when to use ， and is 「 etc when 」—— use not to Reason
/// （ that its ：state failure、 already will this when ）。
/// in ： already （done + error） etc and ；
/// has error（ when not can ） —— that error down 1 ，
/// before will 1 。
enum PollOutcome {
    /// `{pending:true,hint?}`； done response by etc 。
    Pending(Option<String>),
    /// `{done:true, …}` —— （`{account, warning…}`，
    /// when to before ，e.g. KukuAI 「 already up 」）。
    Done(serde_json::Value),
    /// `{done:true, error:…}` —— ， to use 
    Failed(String),
    /// not to ： error（failure / 2xx ）。
    ///
    /// ， as its in 1 ** not is **「 after will 」： use or when 
    /// after from ，`/wait` 404「 not in or expired」。
    /// that must etc （ `task_gone`），otherwise after this will 1 
    /// to 5 minutes when —— as 「 after need etc 5 minutes by 」。
    Unreachable(String),
}

/// 1 after 。
async fn poll_once(state: &str) -> PollOutcome {
    let query = {
        let mut query = url::form_urlencoded::Serializer::new(String::new());
        query.append_pair("state", state);
        query.finish()
    };
    let path = format!("/api/session/login/wait?{query}");
    let value = match gateway::call("GET", &path, None).await {
        Ok(value) => value,
        Err(error) => return PollOutcome::Unreachable(error),
    };
    if value.get("pending").and_then(serde_json::Value::as_bool) == Some(true)
        || value.get("pending").is_some_and(serde_json::Value::is_object)
        || value.get("done").and_then(serde_json::Value::as_bool) != Some(true)
    {
        let hint = value.get("hint").or_else(|| value.pointer("/pending/hint"))
            .and_then(serde_json::Value::as_str).map(str::to_string);
        return PollOutcome::Pending(hint);
    }
    if let Some(error) = value.get("error").and_then(serde_json::Value::as_str) {
        return PollOutcome::Failed(error.to_string());
    }
    PollOutcome::Done(value)
}

/// 1 etc error is not is 「 already after 」（ / ）。
/// this down after from ，`/wait` 404「 not in or expired」。
fn task_gone(message: &str) -> bool {
    message.contains("登录任务不存在") || message.contains("已过期")
}

/// after in ；used for use /
pub async fn cancel(app: &AppHandle) -> Result<(), String> {
    let active = current_login(app);
    if let Some(active) = active {
        if let Some(window) = app.get_webview_window(&login_window_label(&active.state)) {
            let _ = window.destroy();
        }
        clear_active_login(app, &active.state);
        let _ = gateway::call(
            "POST",
            "/api/session/login/cancel",
            Some(&json!({ "state": active.state })),
        )
        .await;
    }
    Ok(())
}

/// new options in methodIndex in when enabled； old Rust and use 。
pub async fn start_with_options(
    app: &AppHandle,
    edition: String,
    mode: String,
    provider: String,
    social_restore: bool,
    options: Option<Value>,
) -> Result<Value, String> {
    let dynamic = options.as_ref().is_some_and(|value| value.get("methodIndex").is_some());
    if provider.trim() == "deepseek-web" {
        if current_login(app).is_some() {
            return Err("已有登录流程在进行，请先完成当前登录或等待超时".into());
        }
        if mode == "external" {
            return Err("DeepSeek 自动网页登录需要桌面内嵌窗口；系统浏览器登录后请使用粘贴令牌。".into());
        }
        let mut options = options.unwrap_or_else(|| json!({}));
        if !dynamic { options["methodIndex"] = json!(1); }
        return crate::deepseek_login::start(app, &options).await;
    }
    if !dynamic {
        // old can ： up for 
        // （antigravity this up in none up has use can ）。
        // its （inputs / methodIndex） old not ， not 。
        let proxy = options.as_ref().and_then(|value| value.get("proxy")).cloned();
        return start(app, edition, mode, provider, social_restore, proxy).await;
    }
    let options = options.unwrap_or(Value::Null);
    if options.get("methodIndex").and_then(Value::as_u64)
        .filter(|index| *index <= 9_007_199_254_740_991).is_none()
    {
        return Err("登录方式索引必须是非负安全整数".into());
    }
    crate::startup::wait().await?;
    let id = if provider.trim().is_empty() { "workbuddy" } else { provider.trim() };
    // current already （ or runtime provider）， not old 
    // MissingPlugin or any id when directory。
    if !crate::server::core::providers::is_executable_provider(id) {
        return Err(format!("提供商未在运行时目录中登记：{id}"));
    }
    // id not old 1 ， not from kind ； after 。
    let provider = id;
    let mut nonce = [0u8; 16];
    getrandom::getrandom(&mut nonce).map_err(|_| "无法生成登录流程标识")?;
    let reservation = format!("starting-{}", nonce.iter().map(|byte| format!("{byte:02x}")).collect::<String>());
    {
        let state = app.state::<crate::state::AppState>();
        let mut guard = state.login.lock().map_err(|_| "登录状态锁不可用")?;
        if guard.is_some() {
            return Err("已有登录流程在进行，请先完成当前登录或等待超时".into());
        }
        *guard = Some(ActiveLogin {
            state: reservation.clone(), edition: edition.clone(), mode: mode.clone(),
            provider: provider.into(),
        });
    }
    emit_login_state(app, LoginState {
        active: true, provider: Some(provider.into()), edition: Some(edition.clone()), mode: Some(mode.clone()),
    });
    let mut request = json!({
        "provider": provider, "edition": edition, "mode": mode,
        "socialRestore": social_restore, "options": options,
    });
    // HTTP new options， when current API 。 not 
    // null（inputs is {}，proxy null then must as ）。
    for key in ["methodIndex", "inputs", "proxy"] {
        if let Some(value) = options.get(key) { request[key] = value.clone(); }
    }
    let response = gateway::call("POST", "/api/session/login/start", Some(&request)).await;
    let started = match response {
        Ok(value) => value,
        Err(error) => { clear_active_login(app, &reservation); return Err(error); }
    };
    let Some(flow) = started.get("state").or_else(|| started.get("flow"))
        .and_then(Value::as_str).filter(|value| !value.trim().is_empty())
    else {
        clear_active_login(app, &reservation);
        return Err("后端未返回登录状态".into());
    };
    let auth_url = match started.get("url").or_else(|| started.get("authUrl")) {
        None | Some(Value::Null) => "",
        Some(Value::String(value)) => value.as_str(),
        Some(_) => {
            let _ = gateway::call("POST", "/api/session/login/cancel", Some(&json!({ "state": flow }))).await;
            clear_active_login(app, &reservation);
            return Err("后端返回的登录链接格式无效".into());
        }
    };
    let only_external = started.get("externalBrowser").and_then(Value::as_bool) == Some(true);
    let mode_id = if auth_url.trim().is_empty() { "auto" }
        else if only_external || mode == "external" { "external" } else { "embedded" };
    let claimed = {
        let state = app.state::<crate::state::AppState>();
        let claimed = match state.login.lock() {
            Ok(mut guard) if guard.as_ref().is_some_and(|active| active.state == reservation) => {
                *guard = Some(ActiveLogin {
                    state: flow.into(), edition: edition.clone(), mode: mode_id.into(), provider: provider.into(),
                });
                true
            }
            _ => false,
        };
        claimed
    };
    if !claimed {
        let _ = gateway::call("POST", "/api/session/login/cancel", Some(&json!({ "state": flow }))).await;
        return Ok(json!({ "ok": false, "canceled": true }));
    }
    let mut instructions = started.get("instructions").and_then(Value::as_str).unwrap_or("").to_string();
    if only_external && mode == "embedded" && !auth_url.trim().is_empty() {
        if !instructions.is_empty() { instructions.push('\n'); }
        instructions.push_str("此登录方式只支持系统浏览器，已按提供商要求打开；不会伪装成内嵌窗口。");
    }
    emit_login_progress(app, flow, &started, &instructions, None);
    let result = run_dynamic_login(app, provider, flow, auth_url, mode_id, &started, &instructions).await;
    if result.as_ref().map(|value| value.get("ok").and_then(Value::as_bool)) != Ok(Some(true)) {
        let _ = gateway::call("POST", "/api/session/login/cancel", Some(&json!({ "state": flow }))).await;
    }
    clear_active_login(app, flow);
    result
}

/// ， not URL、options.inputs or /。
fn emit_login_progress(app: &AppHandle, flow: &str, started: &Value, instructions: &str, hint: Option<&str>) {
    let Some(active) = current_login(app).filter(|active| active.state == flow) else { return; };
    let manual_callback = started.get("manualCallback").and_then(Value::as_bool) == Some(true);
    let callback_input = if manual_callback && started.get("callbackInput").and_then(Value::as_str) == Some("code") {
        "code"
    } else { "url" };
    let progress = json!({
        "active": true, "provider": active.provider, "edition": active.edition, "mode": active.mode,
        "state": flow, "manualCallback": manual_callback, "callbackInput": callback_input,
        "instructions": instructions, "hint": hint,
    });
    let _ = app.emit("login:progress", &progress);
    // only onLoginState UI ，add not will old 。
    let _ = app.emit("login:state", &progress);
}

#[derive(Clone)]
struct DynamicNavigation {
    hosts: Vec<String>,
    callbacks: Vec<url::Url>,
    /// ****（`127.0.0.1:<port>` this ）。
    ///
    /// ── Why need need 1 ──────────────────────────────────
    /// 「HTTPS 」 and 「 」
    /// 。 has （AStudio） is HTTP （ need 
    /// ）， not is HTTPS not is ； not 「」
    /// 「 not is HTTPS or 」。
    /// ** **， and must by in `allowedHosts` 
    /// ， not 。
    loopback_pages: Vec<(String, u16)>,
}

/// `127.0.0.1:1234` / `localhost:1234` / `[::1]:1234` → (host, port)。
fn loopback_page(value: &str) -> Option<(String, u16)> {
    let text = value.trim().to_ascii_lowercase();
    for host in ["127.0.0.1", "localhost", "[::1]"] {
        if let Some(rest) = text.strip_prefix(host) {
            if let Some(port) = rest.strip_prefix(':') {
                if let Ok(port) = port.parse::<u16>() {
                    if port > 0 {
                        return Some((host.to_string(), port));
                    }
                }
            }
            return None;
        }
    }
    None
}

impl DynamicNavigation {
    fn new(auth: &url::Url, started: &Value) -> Result<Self, String> {
        let raw = match started.get("allowedHosts") {
            None | Some(Value::Null) => Vec::new(),
            Some(Value::Array(items)) => items.iter().map(|value| {
                value.as_str().ok_or("登录域名白名单格式无效".to_string())
                    .map(|text| text.trim().to_string())
            }).collect::<Result<Vec<_>, String>>()?,
            Some(_) => return Err("登录域名白名单格式无效".into()),
        };
        let mut hosts = Vec::new();
        let mut loopback_pages = Vec::new();
        for item in raw {
            if let Some(page) = loopback_page(&item) {
                loopback_pages.push(page);
                continue;
            }
            let host = item
                .trim_start_matches("*.")
                .trim_end_matches('.')
                .to_ascii_lowercase();
            if host.is_empty() { return Err("登录域名白名单含有空主机".to_string()); }
            hosts.push(host);
        }
        let mut callbacks = Vec::new();
        let mut capture = |key: &str, value: &str| {
            if matches!(key, "redirect_uri" | "redirect" | "redirect_url" | "callback_url"
                | "auth_callback_url" | "navigate_uri" | "return_url") {
                if let Ok(callback) = url::Url::parse(value) {
                    if callback.host_str().is_some() && callback.username().is_empty() && callback.password().is_none() {
                        callbacks.push(callback);
                    }
                }
            }
        };
        for (key, value) in auth.query_pairs() { capture(&key, &value); }
        // SPA will redirect_uri in hash by 。
        if let Some((_, query)) = auth.fragment().and_then(|fragment| fragment.split_once('?')) {
            for (key, value) in url::form_urlencoded::parse(query.as_bytes()) { capture(&key, &value); }
        }
        Ok(Self { hosts, callbacks, loopback_pages })
    }

    fn is_callback(&self, url: &url::Url) -> bool {
        if matches!(url.scheme(), "file" | "javascript" | "data" | "about" | "blob") { return false; }
        if matches!(url.scheme(), "https" | "http") && !self.allows(url) { return false; }
        self.matches_callback(url)
    }

    fn matches_loopback_page(&self, url: &url::Url) -> bool {
        let Some(host) = url.host_str() else { return false; };
        let Some(port) = url.port_or_known_default() else { return false; };
        self.loopback_pages.iter().any(|(page_host, page_port)| {
            host.eq_ignore_ascii_case(page_host) && port == *page_port
        })
    }

    fn matches_callback(&self, url: &url::Url) -> bool {
        url.username().is_empty() && url.password().is_none() && self.callbacks.iter().any(|target| {
            url.scheme() == target.scheme() && url.host_str() == target.host_str()
                && url.port_or_known_default() == target.port_or_known_default() && url.path() == target.path()
        })
    }

    fn allows(&self, url: &url::Url) -> bool {
        if !url.username().is_empty() || url.password().is_some() { return false; }
        let Some(host) = url.host_str() else { return false; };
        match url.scheme() {
            "https" => self.hosts.is_empty() || self.hosts.iter().any(|item| {
                host.eq_ignore_ascii_case(item) || host.to_ascii_lowercase().ends_with(&format!(".{item}"))
            }),
            // HTTP used for callback， or 
            // ；only localhost not 。
            "http" => matches!(host, "localhost" | "127.0.0.1" | "[::1]")
                && (self.matches_callback(url) || self.matches_loopback_page(url))
                && (self.hosts.is_empty() || self.hosts.iter().any(|item| {
                    host.eq_ignore_ascii_case(item) || host.to_ascii_lowercase().ends_with(&format!(".{item}"))
                })),
            _ => false,
        }
    }
}

/// old Cookie/；empty URL has ，。
async fn run_dynamic_login(
    app: &AppHandle, provider: &str, flow: &str, auth_url: &str, mode: &str,
    started: &Value, instructions: &str,
) -> Result<Value, String> {
    let mut profile = None;
    let mut window = None;
    if !auth_url.trim().is_empty() {
        if auth_url.chars().any(|ch| ch <= '\u{20}' || ch == '\u{7f}') {
            return Err("登录链接包含无效字符".into());
        }
        let auth = url::Url::parse(auth_url).map_err(|_| "登录链接无效")?;
        let policy = DynamicNavigation::new(&auth, started)?;
        if !policy.allows(&auth) { return Err("登录链接不是允许的 HTTPS 站点或合法本机回调地址".into()); }
        if mode == "external" {
            if std::env::var("WORKBUDDY_SKIP_OPEN_BROWSER").as_deref() != Ok("1") {
                open_in_browser(auth_url)?;
            }
        } else {
            profile = Some(LoginProfile::new()?);
            let nav_policy = policy.clone();
            let nav_state = flow.to_string();
            let popup_state = nav_state.clone();
            let seen = Arc::new(AtomicBool::new(false));
            let nav_seen = seen.clone();
            let built = WebviewWindowBuilder::new(app, login_window_label(flow), WebviewUrl::External(auth))
                .data_directory(profile.as_ref().ok_or("登录环境不可用")?.path().to_path_buf())
                .general_autofill_enabled(false)
                .title(format!("登录 {} 账号", crate::server::core::providers::label_of(provider)))
                .inner_size(1100.0, 820.0).min_inner_size(760.0, 560.0).center()
                .on_navigation(move |url| {
                    if nav_policy.is_callback(url) {
                        if !nav_seen.swap(true, Ordering::SeqCst) {
                            let state = nav_state.clone();
                            let callback = url.as_str().to_string();
                            tauri::async_runtime::spawn(async move { submit_callback(&state, &callback).await; });
                        }
                        return false;
                    }
                    nav_policy.allows(url)
                })
                .on_new_window(move |url, _| {
                    if policy.is_callback(&url) && !seen.swap(true, Ordering::SeqCst) {
                        let state = popup_state.clone();
                        let callback = url.as_str().to_string();
                        tauri::async_runtime::spawn(async move { submit_callback(&state, &callback).await; });
                    }
                    tauri::webview::NewWindowResponse::Deny
                }).build().map_err(|_| "打开登录窗口失败")?;
            let handle = app.clone();
            let state = flow.to_string();
            built.on_window_event(move |event| {
                if matches!(event, tauri::WindowEvent::CloseRequested { .. }) && login_is_current(&handle, &state) {
                    clear_active_login(&handle, &state);
                    let payload = json!({ "state": state });
                    tauri::async_runtime::spawn(async move {
                        let _ = gateway::call("POST", "/api/session/login/cancel", Some(&payload)).await;
                    });
                }
            });
            window = Some(built);
        }
    }
    // pollIntervalMs=0 ， not can /wait 。
    let interval = Duration::from_millis(started.get("pollIntervalMs").and_then(Value::as_u64)
        .filter(|value| *value > 0).unwrap_or(2_000).clamp(1_000, 30_000));
    let deadline = tokio::time::Instant::now() + LOGIN_TIMEOUT;
    let mut last_hint = None;
    let result = loop {
        if !login_is_current(app, flow) { break Ok(json!({ "ok": false, "canceled": true })); }
        if tokio::time::Instant::now() >= deadline { break Err("登录等待超时（5 分钟），请重试".into()); }
        match poll_once(flow).await {
            _ if !login_is_current(app, flow) => break Ok(json!({ "ok": false, "canceled": true })),
            PollOutcome::Pending(hint) => {
                if hint != last_hint {
                    emit_login_progress(app, flow, started, instructions, hint.as_deref());
                    last_hint = hint;
                }
            }
            PollOutcome::Done(payload) => break Ok(json!({ "ok": true, "payload": payload, "external": mode == "external" })),
            PollOutcome::Failed(error) => break Err(error),
            PollOutcome::Unreachable(error) if task_gone(&error) => break Ok(json!({ "ok": false, "canceled": true })),
            // not transport error ： can can URL。
            PollOutcome::Unreachable(_) => {}
        }
        tokio::time::sleep(interval).await;
    };
    if let Some(window) = window { let _ = window.destroy(); }
    drop(profile);
    result
}

/// 1 。 to 、failure、 or when 。
///
/// `provider` （empty） by workbuddy —— not will ，
/// that as must。
///
/// `social_restore`： is WorkBuddy Google / GitHub / X 
/// （ [`social_restore_script`]）。default not ， false。
pub async fn start(
    app: &AppHandle,
    edition: String,
    mode: String,
    provider: String,
    social_restore: bool,
    proxy: Option<Value>,
) -> Result<serde_json::Value, String> {
    if current_login(app).is_some() {
        return Err("已有登录流程在进行，请先完成当前登录或等待超时".to_string());
    }
    let provider = normalize_provider(&provider)?;
    if provider == "doubaowork" {
        return crate::doubaowork_login::start(app, &mode).await;
    }
    if let Some(region) = crate::server::core::providers::monkeycode::Region::from_provider_id(provider) {
        return crate::monkeycode_login::start(app, region, &mode).await;
    }

    // ： by ** after **（ + state），
    // request provider， not edition（ that is workbuddy ）。
    if provider == "raccoon" {
        return start_raccoon(app).await;
    }
    // KukuAI：（），success after by 
    // Cookie （ `submit_kuku_login`）。
    if provider == "kuku" {
        return start_kuku(app, &mode).await;
    }
    // OrcaTerm： OAuth （ to ，），
    // `orcaterm://oauth_callback?session_id=…` by down 
    // （ `start_orcaterm`）。
    if provider == "orcaterm" {
        return start_orcaterm(app, &mode).await;
    }
    // CatPaw： need 1 up `login-config`， and token by up **** to 
    // loopback up （ core::login::catpaw）。 no has edition 。
    if provider == "catpaw" {
        return start_catpaw(app, &mode).await;
    }

    // Qoder （ core::login::qoder），`cn` to after ——
    // by and 1 。 this not 。
    //
    // ── Cline not this （ after 1 ）────────────
    // `cline` use `edition` this （`pass` / `free`）， as that when 
    // is ** **、 by 1 。 in is ** provider**
    // （`cline-free` / `cline-pass`）， already in provider id ， no has ——
    // this 1 edition is 1 （ will and provider when not ）。
    let edition_id = match provider {
        // Qoder provider （ up is ， no has down ），
        // already in provider id —— and ZCode ， not edition 。
        "qoder" => "cn",
        "qoder-intl" => "intl",
        // （、 no has down ）
        "qwenwork" => "cn",
        "qwenwork-intl" => "intl",
        // ZCode by **provider **（ up is ，
        // no has down ， not will `edition`）。 not `edition` 
        // after is 「/」 ， already in 
        // provider id ， not this 1 as empty 。
        "zcode" => "cn",
        "zcode-intl" => "intl",
        // WorkBuddy （ after is provider）： already in provider id
        // ， this / use 。 will when `edition`
        // （ 1 ）， 1 when none， not 1 when ** provider as **
        // —— and after `login_start` （ api::session ）。
        "workbuddy-intl" => "intl",
        _ => {
            if edition == "intl" { "intl" } else { "cn" }
        }
    };
    let edition_label = if edition_id == "intl" { "国际版" } else { "国内版" };
    // for workbuddy / qoder / cline ： in after 
    // （ up auth/token、、）， in 。
    // ** has ** can （ need ，module header），
    // to this will 1 etc not to 。
    let use_external = mode == "external" && provider != "raccoon";

    let mut request = json!({ "edition": edition_id, "provider": provider });
    // ： `null`（ use ） need ， and 1 
    // （ up that `for key in ["methodIndex", "inputs", "proxy"]` ）。
    if let Some(proxy) = proxy {
        request["proxy"] = proxy;
    }
    let started = gateway::call(
        "POST",
        "/api/session/login/start",
        Some(&request),
    )
    .await?;
    let login_state = started
        .get("state")
        .and_then(serde_json::Value::as_str)
        .ok_or("后端未返回登录状态")?
        .to_string();
    let auth_url = started
        .get("authUrl")
        .and_then(serde_json::Value::as_str)
        .ok_or("后端未返回登录链接，请检查网络")?
        .to_string();

    {
        let state = app.state::<crate::state::AppState>();
        let mut guard = state.login.lock().map_err(|_| "登录状态锁不可用")?;
        *guard = Some(ActiveLogin {
            state: login_state.clone(),
            edition: edition_id.to_string(),
            mode: if use_external { "external".into() } else { "embedded".into() },
            provider: provider.to_string(),
        });
    }
    emit_login_state(
        app,
        LoginState {
            active: true,
            mode: Some(if use_external { "external".into() } else { "embedded".into() }),
            edition: Some(edition_id.to_string()),
            provider: Some(provider.to_string()),
        },
    );

    // 。 WorkBuddy is （client that 1 ），
    // its —— 1 will 「 WorkBuddy 」
    // is ， use 1 will as is by 。
    let provider_label = match provider {
        // Qoder （、none down ）， not after 
        "qoder" => "Qoder 中国版",
        "qoder-intl" => "Qoder 国际版",
        "qwenwork" => "千问办公 国内版",
        "qwenwork-intl" => "千问办公 国际版",
        "cline-free" => "Cline Free",
        "cline-pass" => "Cline Pass",
        "catpaw" => "CatPaw",
        "raccoon" => "小浣熊",
        // ZCode ， and ** already ** —— down when 
        // need edition after ，otherwise will to 「 ZCode 」
        "zcode" => "ZCode 国内版",
        "zcode-intl" => "ZCode 国际版",
        // CodeArts has 1 （region in cn-north-4 and must and token 
        // 1 ， not is use optional， `providers::codearts` module header），
        // not need need 
        "codearts" => "CodeArts",
        // Trae has 1 （ SOLO ； is 1 、 provider id），
        // not need need 
        "trae" => "Trae",
        // Antigravity（Google AI IDE） has 1 ， no has 
        // （ `providers::antigravity` module header： 1 ）
        "antigravity" => "Antigravity",
        // OfficeAce has 1 （ OAuth 1 ， no has /），
        // not need need 
        "officeace" => "OfficeAce 果办",
        "lobsterai" => "LobsterAI",
        // AtomCode has 1 （AtomGit / GitCode Agent， no has /），
        // not need need 
        "atomcode" => "AtomCode",
        _ => "WorkBuddy",
    };
    // ：Cline 、ZCode 、CodeArts / Trae / OfficeAce
    // 1 already in ， not edition after （otherwise will 「 Cline Free
    // 」 this not ）
    let title = if matches!(
        provider,
        "cline-free" | "cline-pass" | "zcode" | "zcode-intl" | "codearts" | "trae" | "qoder"
        | "qoder-intl" | "qwenwork" | "qwenwork-intl" | "antigravity" | "officeace" | "lobsterai"
        | "atomcode"
    ) {
        format!("登录 {provider_label} 账号")
    } else {
        format!("登录 {provider_label} {edition_label}账号")
    };
    // for ** WorkBuddy** has ， in this ，
    // to run_embedded is 「 this need not need 」 ：
    // ① has workbuddy has this （ / CatPaw / Qoder has ）；
    // ② has up —— is / /
    // / SSO， no has Google / GitHub by ， no has that （），
    // for google.com no has need 。
    // **** not is id： after is `workbuddy-intl`，
    // id will not （ is Google / GitHub by 
    // —— and no has that by is ）。
    let intl_workbuddy = {
        use agent2api_server::server::core::providers::workbuddy::Region;
        Region::from_provider_id(provider).is_some_and(|region| region != Region::Cn)
    };
    let social_restore = social_restore && intl_workbuddy;
    let result = if use_external {
        open_external(app, &auth_url, edition_label, &login_state).await
    } else {
        run_embedded(app, provider, &auth_url, &title, &login_state, social_restore, None, false).await
    };

    if result.as_ref().map(|value| value.get("ok").and_then(serde_json::Value::as_bool)) != Ok(Some(true)) {
        let _ = gateway::call("POST", "/api/session/login/cancel", Some(&json!({ "state": login_state }))).await;
    }
    clear_active_login(app, &login_state);
    result
}

/// ： + by （module header）。
///
/// to 1 ， by module header（ down 
/// need need `office-raccoon://`， that is client ， has ）。
async fn start_raccoon(app: &AppHandle) -> Result<serde_json::Value, String> {
    let started = gateway::call(
        "POST",
        "/api/session/login/start",
        Some(&json!({ "provider": "raccoon" })),
    )
    .await?;
    let login_state = started
        .get("state")
        .and_then(serde_json::Value::as_str)
        .ok_or("后端未返回登录状态")?
        .to_string();
    let auth_url = started
        .get("authUrl")
        .and_then(serde_json::Value::as_str)
        .ok_or("后端未返回登录链接，请检查网络")?
        .to_string();

    {
        let state = app.state::<crate::state::AppState>();
        let mut guard = state.login.lock().map_err(|_| "登录状态锁不可用")?;
        *guard = Some(ActiveLogin {
            state: login_state.clone(),
            edition: String::new(),
            mode: "embedded".into(),
            provider: "raccoon".to_string(),
        });
    }
    emit_login_state(
        app,
        LoginState {
            active: true,
            mode: Some("embedded".into()),
            edition: None,
            provider: Some("raccoon".to_string()),
        },
    );

    // social_restore false： for WorkBuddy 
    // （`run_embedded` for provider == "workbuddy" ），
    // no has this ， false 「 not and this can 」。
    let result = run_embedded(app, "raccoon", &auth_url, "登录小浣熊账号", &login_state, false, None, false).await;
    if result.as_ref().map(|value| value.get("ok").and_then(serde_json::Value::as_bool)) != Ok(Some(true)) {
        let _ = gateway::call("POST", "/api/session/login/cancel", Some(&json!({ "state": login_state }))).await;
    }
    clear_active_login(app, &login_state);
    result
}

/// KukuAI ：** + Cookie **。
///
/// and its （ `providers/kuku/login.rs` module header）：
/// passport success after not （ in `.baidu.com` Cookie ），
/// not 「 → URL」 that ， is by in 
/// `success.html` after WebView2 Cookie （`BDUSS` is HttpOnly，
/// not to ）POST `POST /api/session/login/kuku/complete`
/// （ `run_embedded` etc and `submit_kuku_login`）。
///
/// before already not （ 1 ），
/// as （`start` can can use ）。
async fn start_kuku(app: &AppHandle, mode: &str) -> Result<serde_json::Value, String> {
    // by Cookie （ `submit_kuku_login`）——
    // not to ， etc not to Cookie。 external 
    // ， not （ use external ， is 
    // 「 by and as for not up 」）。
    if mode == "external" {
        return Err(
            "KukuAI 的登录依赖壳侧读取登录窗口的 Cookie，只支持内嵌窗口；\
             请用「内嵌窗口」发起，或改用「粘贴 Cookie」添加账号"
                .to_string(),
        );
    }
    let started = gateway::call(
        "POST",
        "/api/session/login/start",
        Some(&json!({ "provider": "kuku" })),
    )
    .await?;
    let login_state = started
        .get("state")
        .and_then(serde_json::Value::as_str)
        .ok_or("后端未返回登录状态")?
        .to_string();
    let auth_url = started
        .get("authUrl")
        .and_then(serde_json::Value::as_str)
        .ok_or("后端未返回登录链接，请检查网络")?
        .to_string();

    {
        let state = app.state::<crate::state::AppState>();
        let mut guard = state.login.lock().map_err(|_| "登录状态锁不可用")?;
        *guard = Some(ActiveLogin {
            state: login_state.clone(),
            edition: String::new(),
            mode: mode.into(),
            provider: "kuku".to_string(),
        });
    }
    emit_login_state(
        app,
        LoginState {
            active: true,
            mode: Some(mode.into()),
            edition: None,
            provider: Some("kuku".to_string()),
        },
    );

    let result = run_embedded(app, "kuku", &auth_url, "登录 KukuAI 账号", &login_state, false, None, false).await;
    if result.as_ref().map(|value| value.get("ok").and_then(serde_json::Value::as_bool)) != Ok(Some(true)) {
        let _ = gateway::call("POST", "/api/session/login/cancel", Some(&json!({ "state": login_state }))).await;
    }
    clear_active_login(app, &login_state);
    result
}

/// OrcaTerm ：** OAuth + + after **。
///
/// is `orcaterm.com/oauth/authorize?...&session_id=<uuid>` —— this 
/// ** 302 to **（appid `wx56e5199ef872dc22`，OrcaTerm 
/// use ）， use up to to 
/// `orcaterm://oauth_callback?session_id=…`， that 1 down to after 
/// （ `providers/orcaterm/login.rs` module header， that 「Why not 
/// Web `ot_session`」）。
///
/// and raccoon 1 （ URL → after → ）。 is 
/// `orcaterm://oauth_callback`，must by ** ** down ，supports
/// ： that 1 will to （ OrcaTerm client
/// when ， no then is 1 error）， etc not to `session_id`。
async fn start_orcaterm(app: &AppHandle, mode: &str) -> Result<serde_json::Value, String> {
    if mode == "external" {
        return Err(
            "OrcaTerm 的回调是自定义协议（orcaterm://oauth_callback），只能由内嵌窗口接住；\
             请用「内嵌窗口」发起，或改用「粘贴 accessToken」/「导入桌面端登录态」添加账号"
                .to_string(),
        );
    }
    let started = gateway::call(
        "POST",
        "/api/session/login/start",
        Some(&json!({ "provider": "orcaterm" })),
    )
    .await?;
    let login_state = started
        .get("state")
        .and_then(serde_json::Value::as_str)
        .ok_or("后端未返回登录状态")?
        .to_string();
    let auth_url = started
        .get("authUrl")
        .and_then(serde_json::Value::as_str)
        .ok_or("后端未返回登录链接，请检查网络")?
        .to_string();

    {
        let state = app.state::<crate::state::AppState>();
        let mut guard = state.login.lock().map_err(|_| "登录状态锁不可用")?;
        *guard = Some(ActiveLogin {
            state: login_state.clone(),
            edition: String::new(),
            mode: mode.into(),
            provider: "orcaterm".to_string(),
        });
    }
    emit_login_state(
        app,
        LoginState {
            active: true,
            mode: Some(mode.into()),
            edition: None,
            provider: Some("orcaterm".to_string()),
        },
    );

    let result = run_embedded(
        app,
        "orcaterm",
        &auth_url,
        "登录 OrcaTerm 账号",
        &login_state,
        false,
        None,
        false,
    )
    .await;
    if result.as_ref().map(|value| value.get("ok").and_then(serde_json::Value::as_bool)) != Ok(Some(true)) {
        let _ = gateway::call("POST", "/api/session/login/cancel", Some(&json!({ "state": login_state }))).await;
    }
    clear_active_login(app, &login_state);
    result
}

/// CatPaw ：**passport + up to loopback **。
///
/// and （ `core::login::catpaw` module header）：token is ** up to 
/// ** —— passport `login-callback` `{token, state}` 
/// POST to URL to `redirect`， that is 
/// `127.0.0.1:<port>/api/session/login/catpaw-callback`。 this need need 
/// 「 → /wait」， not need need or 。
///
/// `mode`： and supports（， and in none），
/// and Qoder ； that 「 has can 」 in this not 。
async fn start_catpaw(
    app: &AppHandle,
    mode: &str,
) -> Result<serde_json::Value, String> {
    let started = gateway::call(
        "POST",
        "/api/session/login/start",
        Some(&json!({ "provider": "catpaw" })),
    )
    .await?;
    let login_state = started
        .get("state")
        .and_then(serde_json::Value::as_str)
        .ok_or("后端未返回登录状态")?
        .to_string();
    let auth_url = started
        .get("authUrl")
        .and_then(serde_json::Value::as_str)
        .ok_or("后端未返回登录链接，请检查网络")?
        .to_string();

    let use_external = mode == "external";
    let mode_id = if use_external { "external" } else { "embedded" };
    {
        let state = app.state::<crate::state::AppState>();
        let mut guard = state.login.lock().map_err(|_| "登录状态锁不可用")?;
        *guard = Some(ActiveLogin {
            state: login_state.clone(),
            edition: String::new(),
            mode: mode_id.into(),
            provider: "catpaw".to_string(),
        });
    }
    emit_login_state(
        app,
        LoginState {
            active: true,
            mode: Some(mode_id.into()),
            edition: None,
            provider: Some("catpaw".to_string()),
        },
    );

    let result = if use_external {
        // "CatPaw " is to when use ： this no has 「 / 」 this 1 ，
        // empty will to 「 already ， etc when 」 this 。
        open_external(app, &auth_url, "CatPaw 官方登录页", &login_state).await
    } else {
        // social_restore false： and ， this can WorkBuddy 。
        run_embedded(app, "catpaw", &auth_url, "登录 CatPaw 账号", &login_state, false, None, false).await
    };
    if result.as_ref().map(|value| value.get("ok").and_then(serde_json::Value::as_bool)) != Ok(Some(true)) {
        let _ = gateway::call("POST", "/api/session/login/cancel", Some(&json!({ "state": login_state }))).await;
    }
    clear_active_login(app, &login_state);
    result
}

/// ， old not can after 。
fn clear_active_login(app: &AppHandle, expected_state: &str) {
    let state = app.state::<crate::state::AppState>();
    if let Ok(mut guard) = state.login.lock() {
        if guard.as_ref().is_some_and(|active| active.state == expected_state) {
            *guard = None;
            emit_login_state(app, LoginState { active: false, mode: None, edition: None, provider: None });
        }
    };
}

fn login_is_current(app: &AppHandle, expected_state: &str) -> bool {
    current_login(app).is_some_and(|active| active.state == expected_state)
}

fn login_window_label(login_state: &str) -> String {
    use sha2::{Digest, Sha256};
    format!("login-{:x}", Sha256::digest(login_state.as_bytes()))
}

/// ： and ， after by after 。
/// use not etc （ can can already ）。
async fn open_external(
    app: &AppHandle,
    auth_url: &str,
    edition_label: &str,
    login_state: &str,
) -> Result<serde_json::Value, String> {
    if std::env::var("WORKBUDDY_SKIP_OPEN_BROWSER").as_deref() != Ok("1") {
        open_in_browser(auth_url)?;
    }
    let deadline = tokio::time::Instant::now() + LOGIN_TIMEOUT;
    while tokio::time::Instant::now() < deadline {
        tokio::time::sleep(POLL_INTERVAL).await;
        // already （/）：， not error
        if !login_is_current(app, login_state) {
            return Ok(json!({ "ok": false, "canceled": true }));
        }
        match poll_once(login_state).await {
            PollOutcome::Pending(_) => continue,
            PollOutcome::Done(payload) => {
                return Ok(json!({ "ok": true, "external": true, "payload": payload }))
            }
            PollOutcome::Failed(error) => {
                // already failure（ after error）：Reason，
                // not 「 etc when 」—— that up 「 already /
                // state failure」 is this 1 。
                return Err(error);
            }
            PollOutcome::Unreachable(error) => {
                // after （ already / expired，`/wait` 404） when ，
                // not etc 5 minutes； its error down 1 can can ， etc 。
                if task_gone(&error) {
                    return Ok(json!({ "ok": false, "canceled": true }));
                }
                eprintln!("[login] 读取登录进度失败（继续等待）: {error}");
                continue;
            }
        }
    }
    Err(format!("已打开系统浏览器完成{edition_label}登录，但等待超时（5 分钟），请重试"))
}

pub(crate) async fn run_cookie_login(
    app: &AppHandle,
    provider: &'static str,
    auth_url: &str,
    title: &str,
    flow: &str,
) -> Result<serde_json::Value, String> {
    run_cookie_login_with_proxy(app, provider, auth_url, title, flow, None).await
}

pub(crate) async fn run_cookie_login_with_proxy(
    app: &AppHandle,
    provider: &'static str,
    auth_url: &str,
    title: &str,
    flow: &str,
    proxy: Option<url::Url>,
) -> Result<serde_json::Value, String> {
    {
        let state = app.state::<crate::state::AppState>();
        let mut guard = state.login.lock().map_err(|_| "登录状态锁不可用")?;
        if guard.is_some() {
            return Err("已有登录流程在进行，请先完成当前登录或等待超时".into());
        }
        *guard = Some(ActiveLogin { state: flow.into(), edition: String::new(), mode: "embedded".into(), provider: provider.into() });
    }
    emit_login_state(app, LoginState { active: true, mode: Some("embedded".into()), edition: None, provider: Some(provider.into()) });
    let result = run_embedded_with_proxy(app, provider, auth_url, title, flow, false, None, false, proxy).await;
    clear_active_login(app, flow);
    result
}

/// WebView ：， when after 。
///
/// `provider` and （ need `office-raccoon://` ）；
/// `title` by use to （ not ）。
///
/// `social_restore` is ** already **（ use already by provider and ， `start`）：
/// as true when [`social_restore_script`] and —— must for ：
/// not ， Google will 。
///
/// `kuku_phone`：**KukuAI ** （`Some` when 
/// [`kuku_sms_script`]：、 etc 、 —— by 
/// `submit_kuku_sms_code` `window.eval` ； by in 
/// etc Cookie ， [`submit_kuku_login`]）。
/// `None` = （ use ， Cookie）。
///
/// `start_hidden`：**create**（KukuAI use ： 
/// by after ， use not this to —— has to etc need need 
/// by use ， `start_kuku_sms` ）。
async fn run_embedded(
    app: &AppHandle,
    provider: &'static str,
    auth_url: &str,
    title: &str,
    login_state: &str,
    social_restore: bool,
    kuku_phone: Option<&str>,
    start_hidden: bool,
) -> Result<serde_json::Value, String> {
    run_embedded_with_proxy(app, provider, auth_url, title, login_state, social_restore, kuku_phone, start_hidden, None).await
}

async fn run_embedded_with_proxy(
    app: &AppHandle,
    provider: &'static str,
    auth_url: &str,
    title: &str,
    login_state: &str,
    social_restore: bool,
    kuku_phone: Option<&str>,
    start_hidden: bool,
    proxy: Option<url::Url>,
) -> Result<serde_json::Value, String> {
    let window_label = login_window_label(login_state);
    // ── 1 （： will to up 1 ）──
    // use ** new ** when directory：WebView2 Cookie / localStorage / 
    // by directory， use defaultdirectory etc use —— when 
    // will up 1 ， use no has will 1 。
    // directory in after delete（ login_profile.rs）。
    //
    // ** not **（`incognito(true)`）： that will Google / GitHub 
    // as 「unsupported 」（ to no ），
    // directory already can need need —— 1 
    // Cookie ， and before 1 not 。
    let profile = LoginProfile::new()?;
    let url = url::Url::parse(auth_url).map_err(|error| format!("登录链接无效: {error}"))?;

    // ── 1 ────────────────────────────────────────
    // 1 URL can can from to （WebView2 for 「 to 」 in 
    // path up will NavigationStarting new request， after 
    // failure can can 1 ）。 is ** 1 ** ： will from up 
    // 200035「 already 」， 1 success 1 failure。 this and 
    // Electron `callbackSeen` 1 use 。
    let callback_seen = Arc::new(AtomicBool::new(false));
    let login_state_owned = login_state.to_string();
    // KukuAI 1 （ Cookie ，
    // success after ； POST complete etc ， no has need ）。
    let kuku_submitted = Arc::new(AtomicBool::new(false));
    // KukuAI Cookie ：
    // - ****（on_navigation） → down 1 cookie
    // （ use when ： passport、）；
    // - 4 （8 seconds） 1 （ use after when not not ）；
    // - BDUSS after etc 4 seconds： cookie
    // （BDUSS_BFESS etc ） can can new ， will old 
    // 「」（userreport errno=-6，2026-10-07 ）。
    let mut kuku_check_tick: u32 = 0;
    let kuku_nav_counter = Arc::new(std::sync::atomic::AtomicU64::new(0));
    let mut kuku_last_nav: u64 = 0;
    let nav_counter_for_check = kuku_nav_counter.clone();
    let nav_counter_for_nav = kuku_nav_counter.clone();
    let mut kuku_bduss_seen_at: Option<tokio::time::Instant> = None;
    // failure after （30 seconds not ）： after that failure as up 
    // when （ 1 minutes after ，2026-10-07 for ），
    // will 。
    let mut kuku_retry_block_until: Option<tokio::time::Instant> = None;
    let mut doubaowork_submitted = false;
    let monkeycode_region = crate::server::core::providers::monkeycode::Region::from_provider_id(provider);
    let mut monkeycode_submitted = false;
    let mut deepseek_submitted = false;
    let mut deepseek_capture = crate::deepseek_login::CredentialCapture::default();

    // and ** use 1 **： to can can is 1 
    // （ location、 WebView2 1 request）， 1 
    // will ， to is 「 already 」。
    let seen_for_nav = callback_seen.clone();
    let state_for_nav = login_state_owned.clone();
    let seen_for_window = callback_seen.clone();
    let state_for_window = login_state_owned.clone();

    let window = WebviewWindowBuilder::new(app, &window_label, WebviewUrl::External(url))
        .data_directory(profile.path().to_path_buf())
        // / ： this use 1 ， will 
        // 。 when up no has Settings4 ，wry will 。
        .general_autofill_enabled(false)
        .title(title.to_string())
        .inner_size(1100.0, 820.0)
        .min_inner_size(760.0, 560.0)
        .visible(!start_hidden)
        .center();
    let window = match proxy {
        Some(proxy) => window.proxy_url(proxy),
        None => window,
    };

    // ： by up Google / GitHub / X ，
    // that （ by and social_restore_script）。
    //
    // use for_all_frames： in ** iframe** head ， frame
    // not 。 by hostname 1 ， accounts.google.com
    // after in that empty。
    //
    // KukuAI （only need need ）：
    // `kuku_phone` empty when 「」（、 etc 、
    // ， `kuku_sms_script`）；`None`（） not need need ——
    // use ， by Cookie （ etc kuku
    // and `submit_kuku_login`）； after 「 will 」 in after 
    // （`providers/kuku/engine.rs`）。 and social_restore ， by provider 1 。
    let window = if provider == "kuku" {
        match kuku_phone {
            Some(phone) => window.initialization_script(kuku_sms_script(phone)),
            None => window,
        }
    } else if social_restore {
        window.initialization_script_for_all_frames(social_restore_script())
    } else {
        window
    };

    let window = window
        // in up ： will SSO in ，
        // if any， can can to 。
        //
        // ── Whymust ────────────────────────
        // `host_allowed` for 1 false， will 
        // 「」 —— and 「」 in WebView2 ，
        // is use success、 etc not to code。
        //
        // ── AutoClaw Why not in this （ after ）─────────────────
        // in z.ai that up ，** by and down **
        // （ `autoclaw/callback_server.rs`）。 not can ： after 
        // not to （ that not ）。 this 
        // —— for autoclaw not （ `allowed_hosts`）。
        .on_navigation(move |url| {
            if is_login_callback(url, provider) {
                // return false this （otherwise WebView2 will to ：
                // no office-raccoon:// when is 1 error）。
                // is ， is ， spawn 。
                if !seen_for_nav.swap(true, Ordering::SeqCst) {
                    let login_state = state_for_nav.clone();
                    let callback_url = url.as_str().to_string();
                    tauri::async_runtime::spawn(async move {
                        submit_callback(&login_state, &callback_url).await;
                    });
                }
                return false;
            }
            // KukuAI ：（ passport、 after ）
            // +1， etc 1 Cookie 。
            if provider == "kuku" {
                nav_counter_for_nav.fetch_add(1, Ordering::SeqCst);
            }
            host_allowed(url, provider, social_restore)
        })
        // （window.open） is can can ： has in to after will use 
        // `window.open('office-raccoon://…')` not is location —— that 
        // NewWindowRequested， not to ， use will to 「success
        // no 」。 this ** 1 **：
        // - is → down and （ use ， up ）；
        // - its 1 Deny —— this and not when default as ** 1 **
        // （wry in no has handler when for NewWindowRequested SetHandled(true)），
        // this not is 「」， is that 1 from 。
        //
        // AutoClaw by （ up ）， this not ： and 
        // window.open will to up 。
        //
        // KukuAI ： 「」 by can can passport 。
        // 1 Deny will use none —— this ****（ and 
        // 1 WebView2 directory、 Cookie ，success after 
        // etc can from Cookie to BDUSS）。
        .on_new_window(move |url, _features| {
            if let Some(region) = monkeycode_region {
                return if crate::monkeycode_login::allowed(&url, region) {
                    tauri::webview::NewWindowResponse::Allow
                } else {
                    tauri::webview::NewWindowResponse::Deny
                };
            }
            if provider == "deepseek-web" && crate::deepseek_login::allowed(&url) {
                return tauri::webview::NewWindowResponse::Allow;
            }
            if provider == "doubaowork" && crate::doubaowork_login::allowed(&url) {
                return tauri::webview::NewWindowResponse::Allow;
            }
            if provider == "kuku" {
                return tauri::webview::NewWindowResponse::Allow;
            }
            if is_login_callback(&url, provider) && !seen_for_window.swap(true, Ordering::SeqCst) {
                let login_state = state_for_window.clone();
                let callback_url = url.as_str().to_string();
                tauri::async_runtime::spawn(async move {
                    submit_callback(&login_state, &callback_url).await;
                });
            }
            tauri::webview::NewWindowResponse::Deny
        })
        .build()
        .map_err(|error| format!("打开登录窗口失败: {error}"))?;

    // as ： after in ， and etc 
    let handle = app.clone();
    let state_for_close = login_state.to_string();
    window.on_window_event(move |event| {
        if let tauri::WindowEvent::CloseRequested { .. } = event {
            if login_is_current(&handle, &state_for_close) {
                clear_active_login(&handle, &state_for_close);
                let payload = json!({ "state": state_for_close });
                tauri::async_runtime::spawn(async move {
                    let _ = gateway::call("POST", "/api/session/login/cancel", Some(&payload)).await;
                });
            }
        }
    });

    let deadline = tokio::time::Instant::now() + LOGIN_TIMEOUT;
    let outcome = loop {
        if tokio::time::Instant::now() >= deadline {
            break Err("网页登录等待超时（5 分钟），请重试".to_string());
        }
        tokio::time::sleep(POLL_INTERVAL).await;

        // （ or already ）： etc 
        if app.get_webview_window(&window_label).is_none() || !login_is_current(app, login_state) {
            break Ok(json!({ "ok": false, "canceled": true }));
        }
        // KukuAI（， `providers/kuku/login.rs` module header）：
        // use in `kuku.baidu.com` ， **Cookie 
        // BDUSS** as success（ cookie + when ，
        // `read_window_cookies`；Windows up cookies() will ）。
        // when ：（： passport、）
        // after 1 ； has 4 （8 seconds），「 after when 
        // not 」。 BDUSS after etc 4 seconds（ etc cookie
        // BDUSS_BFESS new ， up `kuku_bduss_seen_at` ）。
        //
        // after 「 will 」（ genflowpro use STOKEN） in after 
        // （`providers/kuku/engine.rs`）—— that 1 need client，
        // not ， not etc 。
        if provider == "kuku" {
            kuku_check_tick += 1;
            let nav_count = nav_counter_for_check.load(Ordering::SeqCst);
            let navigated = nav_count != kuku_last_nav;
            if navigated {
                kuku_last_nav = nav_count;
            }
            let quick_url_hit = window
                .url()
                .map(|current| current.to_string().contains("/passApi/html/success.html"))
                .unwrap_or(false);
            let mut should_submit = quick_url_hit;
            if !kuku_submitted.load(Ordering::SeqCst)
                && (navigated || kuku_bduss_seen_at.is_some() || kuku_check_tick % 4 == 0)
            {
                match read_window_cookies(&window).await {
                    Ok(cookies) => {
                        let (header, has_bduss) = kuku_cookie_header(&cookies);
                        eprintln!(
                            "[login] KukuAI Cookie 检测：共 {} 条，BDUSS {}（头长 {}）",
                            cookies.len(),
                            if has_bduss { "已出现" } else { "未出现" },
                            header.len()
                        );
                        if has_bduss {
                            let is_first = kuku_bduss_seen_at.is_none();
                            let first_seen = *kuku_bduss_seen_at
                                .get_or_insert_with(tokio::time::Instant::now);
                            if is_first {
                                kuku_log_bduss_candidates(&cookies);
                            }
                            let stable = first_seen.elapsed() >= Duration::from_secs(4);
                            if stable {
                                // use ** to new ** cookie （ etc 
                                // cookie can can already new ）
                                should_submit = true;
                            } else {
                                eprintln!(
                                    "[login] KukuAI 已发现 BDUSS，等 4 秒让会话稳定后再交回…"
                                );
                            }
                        }
                    }
                    Err(error) => {
                        eprintln!("[login] KukuAI 读取窗口 Cookie 失败（下轮重试）: {error}");
                    }
                }
            }
            if should_submit && !kuku_submitted.load(Ordering::SeqCst) {
                let blocked = kuku_retry_block_until
                    .map(|until| tokio::time::Instant::now() < until)
                    .unwrap_or(false);
                if blocked {
                    eprintln!("[login] KukuAI 交回退避中（30 秒），等下一轮再试");
                } else {
                    kuku_submitted.store(true, Ordering::SeqCst);
                    if let Err(error) = submit_kuku_login(&window, login_state).await {
                        eprintln!("[login] 交回 KukuAI 登录态失败（下轮重试）: {error}");
                        kuku_submitted.store(false, Ordering::SeqCst);
                        kuku_retry_block_until = Some(
                            tokio::time::Instant::now() + Duration::from_secs(30),
                        );
                    } else {
                        kuku_bduss_seen_at = None;
                    }
                }
            }
        }
        // OrcaTerm（ OAuth ， `providers/orcaterm/login.rs` 
        // module header）：** not need need this ** —— 
        // （`orcaterm://oauth_callback?session_id=…`） by `on_navigation` /
        // `on_new_window` down and to after ，success and 1 `/wait`。
        // that 「 cookie ot_session」 already delete： that Cookie in 
        // SPA when not will （module header for ）。
        if provider == "deepseek-web" && !deepseek_submitted {
            match crate::deepseek_login::credentials(&window, &mut deepseek_capture).await {
                Ok(Some(credentials)) if login_is_current(app, login_state) => {
                    deepseek_submitted = true;
                    submit_callback(login_state, &credentials).await;
                }
                Ok(_) => {}
                Err(error) => {
                    eprintln!("[login] {error}");
                }
            }
        }
        if let Some(region) = monkeycode_region.filter(|_| !monkeycode_submitted) {
            match crate::monkeycode_login::credentials(&window, region).await {
                Ok(Some(credentials)) if login_is_current(app, login_state) => {
                    monkeycode_submitted = true;
                    submit_callback(login_state, &credentials).await;
                }
                Ok(_) => {}
                Err(error) => eprintln!("[login] {error}"),
            }
        }
        if provider == "doubaowork" && !doubaowork_submitted {
            match crate::doubaowork_login::cookie(&window).await {
                Ok(Some(cookie)) => {
                    doubaowork_submitted = true;
                    submit_callback(login_state, &cookie).await;
                }
                Ok(None) => {}
                Err(error) => eprintln!("[login] 豆包工作 Cookie 读取失败：{error}"),
            }
        }
        match poll_once(login_state).await {
            PollOutcome::Pending(_) => continue,
            // （`{account, warning…}`）success： before 
            // in 「 already 」 after 1 （e.g. KukuAI 
            // up —— KukuAI when is this ）。
            PollOutcome::Done(payload) => break Ok(json!({ "ok": true, "payload": payload })),
            // already failure（state failure / already / failure）：
            // 。 will 5 minutes after 「 etc when 」，
            // use not to can Reason。
            PollOutcome::Failed(error) => break Err(error),
            PollOutcome::Unreachable(error) => {
                // already after （ use / ）→ and 1 
                if task_gone(&error) {
                    break Ok(json!({ "ok": false, "canceled": true }));
                }
                eprintln!("[login] 读取登录进度失败（继续等待）: {error}");
                continue;
            }
        }
    };

    let _ = window.destroy();
    drop(window);
    drop(profile);
    outcome
}

/// to URL to after 。
///
/// failure： error by after ， down 1 `/wait` return，
/// before and etc from that （ 1 ， not in this ）。
/// etc 2 seconds 1 `/wait`， is 1 HTTP use （ms），
/// not need need 。
async fn submit_callback(login_state: &str, callback_url: &str) {
    let payload = json!({ "state": login_state, "callbackUrl": callback_url });
    match gateway::call("POST", "/api/session/login/callback", Some(&payload)).await {
        Ok(_) => eprintln!("[login] 已捕获登录回调并提交给网关"),
        Err(_) => eprintln!("[login] 提交登录回调失败，请查看登录进度提示"),
    }
}

/// WebView2 Cookie （ HttpOnly）。
///
/// ── Whymust + when ────────────────────────────────
/// Windows up `cookies()` need etc WebView2 ： in /
/// will and etc ****（tauri Known issues，
/// is 「、 not 」）。 this to `spawn_blocking` ，
/// 1 3 seconds when —— not to failure， not use 。
async fn read_window_cookies(
    window: &tauri::WebviewWindow,
) -> Result<Vec<tauri::webview::Cookie<'static>>, String> {
    let window_owned = window.clone();
    // return：`cookies()` return 1 and not ，
    // this will （ not is in after match 
    // 1 error）。
    let handle = tokio::task::spawn_blocking(
        move || -> Result<Vec<tauri::webview::Cookie<'static>>, tauri::Error> {
            window_owned.cookies()
        },
    );
    let nested = tokio::time::timeout(Duration::from_secs(10), handle).await;
    // Result ：Elapsed（ when ）→ JoinError（failure）→
    // tauri::Error（cookies() ）， after to Vec<Cookie>。
    let inner = match nested {
        Ok(result) => result,
        Err(_) => return Err("读取登录窗口 Cookie 超时（WebView2 未响应）".to_string()),
    };
    let joined = match inner {
        Ok(result) => result,
        Err(error) => return Err(format!("读取登录窗口 Cookie 的后台任务失败: {error}")),
    };
    match joined {
        Ok(list) => Ok(list),
        Err(error) => Err(format!("读取登录窗口 Cookie 失败: {error}")),
    }
}

/// Cookie Cookie ，return `(, is BDUSS)`。
///
/// ** Cookie**（ not BDUSS/STOKEN）： up userreport is 
/// will ， BAIDUID / BAIDUID_BFESS will 「」
/// （2026-10-07 for ）。statistics（Hm_* / HMACCOUNT） —— for 
/// none use ， will Cookie 。
fn kuku_cookie_header(cookies: &[tauri::webview::Cookie<'static>]) -> (String, bool) {
    let mut parts: Vec<String> = Vec::new();
    let mut has_bduss = false;
    let mut seen: Vec<String> = Vec::new();
    for cookie in cookies {
        let name = cookie.name().to_string();
        let domain = cookie.domain().unwrap_or("").to_string();
        if !domain.ends_with("baidu.com") {
            continue;
        }
        if name == "gfprotpl"
            || name.starts_with("Hm_")
            || name == "HMACCOUNT"
            || seen.iter().any(|existing| existing == &name)
        {
            continue;
        }
        if name == "BDUSS" {
            has_bduss = true;
        }
        seen.push(name.clone());
        parts.push(format!("{name}={}", cookie.value()));
    }
    (parts.join("; "), has_bduss)
}

/// ： cookie all BDUSS （domain / path / / before ）。
///
/// used for「 is has BDUSS（ not or path）」「 when is 1 」——
/// userreport 「」 when ， 1 need excluding「」。
fn kuku_log_bduss_candidates(cookies: &[tauri::webview::Cookie<'static>]) {
    for cookie in cookies {
        if cookie.name() == "BDUSS" {
            let value = cookie.value();
            let prefix: String = value.chars().take(20).collect();
            eprintln!(
                "[login] KukuAI BDUSS 候选：domain={:?} path={:?} 长度={} 前缀={prefix}…",
                cookie.domain().unwrap_or(""),
                cookie.path().unwrap_or(""),
                value.len()
            );
        }
    }
}

/// KukuAI ： use in success after ， WebView2 Cookie
/// （ HttpOnly —— `document.cookie` not to BDUSS）， 
/// will Cookie POST （ `read_window_cookies` ）。
async fn submit_kuku_login(
    window: &tauri::WebviewWindow,
    login_state: &str,
) -> Result<(), String> {
    let cookies = read_window_cookies(window).await?;
    let (cookie_header, has_bduss) = kuku_cookie_header(&cookies);
    if !has_bduss {
        return Err("登录窗口没有 BDUSS（登录态尚未写入），请稍后重试".to_string());
    }
    let payload = json!({ "state": login_state, "cookie": cookie_header });
    gateway::call("POST", "/api/session/login/kuku/complete", Some(&payload))
        .await
        .map_err(|error| format!("网关收尾失败: {error}"))?;
    Ok(())
}


/// use default。
///
/// to commands.rs use （ Release ）： is 1 
/// `cmd /C start`， has down this `&` —— update
/// not ， no has by as not 1 。
///
/// ── Why not use `cmd /C start`（ ， ）──
/// URL `https://.../login?platform=workbuddy&state=<uuid>`，
/// `cmd` `&` ****：
///   ① `start "" https://.../login?platform=workbuddy`
/// ② `state=<uuid>`（ 1 not in ）
/// after has ， and from ：
/// 1. ** no has state** —— use can ， up none 
/// this and ， after `auth/token` return
/// 11217（ in ）， before 1 in 「 etc 」 to 5 minutes when ；
/// 2. ② that cmd ， 1 。
///
/// use `ShellExecuteW`： to shell URL， not ，
/// not `&`， not create。`SW_SHOWNORMAL` before 。
#[cfg(windows)]
pub fn open_in_browser(url: &str) -> Result<(), String> {
    use std::os::windows::ffi::OsStrExt;

    const SW_SHOWNORMAL: i32 = 1;
    #[link(name = "shell32")]
    extern "system" {
        fn ShellExecuteW(
            hwnd: *mut std::ffi::c_void,
            operation: *const u16,
            file: *const u16,
            parameters: *const u16,
            directory: *const u16,
            show_cmd: i32,
        ) -> *mut std::ffi::c_void;
    }

    let to_wide = |text: &str| -> Vec<u16> {
        std::ffi::OsStr::new(text).encode_wide().chain(std::iter::once(0)).collect()
    };
    let operation = to_wide("open");
    let file = to_wide(url);

    // ShellExecuteW return <= 32 failure（ this is Win32 ）
    let result = unsafe {
        ShellExecuteW(
            std::ptr::null_mut(),
            operation.as_ptr(),
            file.as_ptr(),
            std::ptr::null(),
            std::ptr::null(),
            SW_SHOWNORMAL,
        )
    };
    if result as isize <= 32 {
        return Err(format!("打开系统浏览器失败（ShellExecute 返回 {result:?}）"));
    }
    Ok(())
}

/// Windows etc （ has Windows，
/// is as `cargo check` in its can ）
#[cfg(not(windows))]
pub fn open_in_browser(url: &str) -> Result<(), String> {
    let opener = if cfg!(target_os = "macos") { "open" } else { "xdg-open" };
    // URL as argv ， not shell，`&` not will 
    std::process::Command::new(opener)
        .arg(url)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map(|_| ())
        .map_err(|error| format!("打开系统浏览器失败: {error}"))
}
