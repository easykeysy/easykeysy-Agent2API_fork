use super::*;
use crate::server::core::upstream::usage::RequestTelemetry;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

async fn mock_upstream(
    status: u16,
    content_type: &'static str,
    body: &'static str,
) -> (
    TransportRequest,
    Arc<AtomicUsize>,
    tokio::task::JoinHandle<()>,
) {
    let hits = Arc::new(AtomicUsize::new(0));
    let count = hits.clone();
    let app = axum::Router::new().route(
        "/messages",
        axum::routing::post(move || {
            count.fetch_add(1, Ordering::SeqCst);
            async move {
                (
                    axum::http::StatusCode::from_u16(status).unwrap(),
                    [(axum::http::header::CONTENT_TYPE, content_type)],
                    body,
                )
            }
        }),
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}/messages", listener.local_addr().unwrap());
    let task = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    (
        TransportRequest {
            url,
            headers: vec![("Content-Type".into(), "application/json".into())],
            payload: r#"{"stream":true}"#.into(),
            proxy: None,
            // by default can （false = ）， and not this 1 
            system_proxy_when_unset: false,
        },
        hits,
        task,
    )
}

// ── ZCode response and error（ to ）────────────────────────
//
// this has use is ZCode ：
// · `is_error_response`：HTTP 200 + `application/json` is **** not is 
// SSE （`1005 exceed quota limit` 200 ），musterror；
// · `classify_error`：401 → TokenExpired、429 / 1005 → QuotaLimited。
//
// this ZCode to `agent2api-providers` ， use 1 
// —— in will not （`ZCODE_ADAPTER` already not in ）。 use 
// in `zcode::mod` `classify` 。
//
// **Why this **：`provider_loop` 「HTTP 200 can can is error」
// and 「401/429 」 this **** no has ， down use 
// （ will path ） in 。

// ── will path （`mark_conversation_limited`）──────────────────────
//
// this is 1 ： before this use `rotate::mark_account_limited`，
// `reset_at` `None` ⇒ when in parameter up ， to 
// 「 10 minutes」 （`account_store::store_admin::mark_rate_limited`）。
// not is ， is ** 1 already by new **：CodeArts
// need to has ， when 8 （ 0.2–0.3 seconds、
// not 、request 200 —— response not ）。
//
// in before up 1、4 ； 2、3 is 「 no has 」 for 。

static TEMP_SEQ: AtomicUsize = AtomicUsize::new(0);

/// when directory，Drop when delete。
///
/// not in before ：file pid， down 1 pid not to （ in 1 up 
/// ）。 use by `let (_dir, service, id) = temp_service();` —— 
/// ，directory。
struct TempDbDir(std::path::PathBuf);

impl Drop for TempDbDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// 1 when + in its up [`UpstreamService`] + 1 that id。
///
/// not request： and ， and use is 1 。
fn temp_service() -> (TempDbDir, UpstreamService, String) {
    use crate::server::core::account_store::AccountStore;
    use crate::server::core::auth::AuthService;
    use crate::server::db::Db;

    let seq = TEMP_SEQ.fetch_add(1, Ordering::SeqCst);
    let dir = std::env::temp_dir().join(format!("pl-quota-{}-{seq}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let store = AccountStore::with_db(Some(Db::open(&dir.join("agent2api.db")).expect("临时库应当能建起来")));
    let saved = store
        .add_account(
            &json!({"auth": {"accessToken": "at"}, "account": {"uid": "quota-1"}}),
            None,
            None,
        )
        .expect("临时账号应当能建起来");
    let id = saved["id"].as_str().expect("公开形态应带 id").to_string();
    let service = UpstreamService::new(store.clone(), AuthService::for_store(store));
    (TempDbDir(dir), service, id)
}

/// → （ this 1 ，directory ）。
fn single(name: &str) -> Vec<String> {
    vec![name.to_string()]
}

#[test]
fn stateful_quota_bookkeeping_keeps_the_adapter_reset_at() {
    let (_dir, service, id) = temp_service();
    let now = logging::now_ms();
    // can when ：3 when after 。 and 10 minutes not —— before this is 
    // now+600000， etc ； no has ， will 
    // （`errors::parse_quota_reset_at`） not this ， can 。
    let reset_at = now + 3 * 60 * 60 * 1000;
    let class = UpstreamErrorClass::QuotaLimited {
        reset_at: Some(reset_at),
        message: "上游返回 403: insufficient quota".to_string(),
        upstream_code: Some(4291200),
        status: 403,
    };
    // （ by ） and is 1 ， this to 
    let group = vec!["m-benefit-a".to_string(), "m-benefit-b".to_string()];
    mark_conversation_limited(&service, &id, &group, &class, "上游返回 403: insufficient quota");
    for name in &group {
        let stored = rotate::account_limit_reset_at(&service, &id, name);
        assert_eq!(stored, reset_at, "{name} 的 resetAt 应当是适配器给的恢复时刻");
        let minutes = (stored - now) / 60_000;
        assert!(
            (stored - (now + 10 * 60 * 1000)).abs() > 60_000,
            "{name} 落到了 10 分钟兜底冷却：{minutes} 分钟"
        );
    }
    // 1 （ None that ——
    // no has ， this is 1 1 can 1 「 1 」 ）
    let limits = {
        let snapshot = service.store.list_accounts();
        snapshot["accounts"].as_array().unwrap()[0]["rateLimits"].clone()
    };
    assert_eq!(limits["m-benefit-a"]["status"], json!(403));
    assert_eq!(limits["m-benefit-a"]["code"], json!(4291200));
}

#[test]
fn stateful_quota_bookkeeping_falls_back_when_adapter_gives_no_reset_at() {
    let (_dir, service, id) = temp_service();
    let now = logging::now_ms();
    // 429 is minutes，CodeArts classify that to None（ to will 1 is 
    // when 1 ）—— is as ， not 。
    // none：（`parse_quota_reset_at`） not when 。
    let message = "上游返回 429: too many requests";
    let class = UpstreamErrorClass::QuotaLimited {
        reset_at: None,
        message: message.to_string(),
        upstream_code: None,
        status: 429,
    };
    mark_conversation_limited(&service, &id, &single("m-429"), &class, message);
    let stored = rotate::account_limit_reset_at(&service, &id, "m-429");
    let seconds = (stored - now) / 1000;
    assert!(
        stored > now + 9 * 60 * 1000 && stored <= now + 11 * 60 * 1000,
        "无恢复时刻时应落 10 分钟兜底，实际 {seconds} 秒"
    );
}

#[test]
fn stateful_quota_bookkeeping_ignores_non_quota_classes() {
    let (_dir, service, id) = temp_service();
    // for ： use `if let` already down QuotaLimited， not this ——
    // / that 1 failure not this （ 1 its not can to ）
    let message = "上游返回 403: permission denied";
    let class = UpstreamErrorClass::Fatal {
        status: 403,
        message: message.to_string(),
        upstream_code: None,
    };
    mark_conversation_limited(&service, &id, &single("m-fatal"), &class, message);
    assert_eq!(rotate::account_limit_reset_at(&service, &id, "m-fatal"), 0, "非限额分类不该落冷却");
}

#[test]
fn codearts_daily_pool_envelope_cools_to_the_boundary_instead_of_ten_minutes() {
    use crate::server::errors::GatewayError;

    let (_dir, service, id) = temp_service();
    // that （HTTP 200 SSE 403， will ）
    let error = GatewayError::with_status(403, "上游报告 InferHub.4291.200：insufficient quota");
    let class = adapter_for(ProviderKind::CodeArts).classify_conversation_error(&error);
    let UpstreamErrorClass::QuotaLimited { reset_at: Some(reset_at), .. } = &class else {
        panic!("福利池耗尽的文案应分类为带恢复时刻的 QuotaLimited，实际 {class:?}");
    };
    mark_conversation_limited(&service, &id, &single("glm-5.3-flash"), &class, &error.message);

    let stored = rotate::account_limit_reset_at(&service, &id, "glm-5.3-flash");
    assert_eq!(stored, *reset_at, "落库的恢复时刻必须是适配器算出的日池重置点");
    // ── 「 is down 1 」， not is 「 in > 10 minutes」（2026-10-13 ）──
    //
    // is `minutes > 10`， is 「 not is 10 minutes」； has 
    // when ： is ** down 1 **， is 23:50–00:00 this minutes
    // 「」 10 minutes， use （ 2026-10-13 23:49 up ）。
    // that is ， not is —— and none failure 。
    //
    // ： when must in ** or ** up ，
    // and in not 24 when （ is 「 to 」）。 this when 
    // ， not 「 in has 」。
    let minutes = (stored - logging::now_ms()) / 60_000;
    assert!(
        (0..=24 * 60).contains(&minutes),
        "日池的恢复时刻应当落在 24 小时以内，实际 {minutes} 分钟"
    );
    // ： when in when must is 00:00:00 —— by 。
    // 10 minutes（now + 10min） in up can ignore， this 1 excluding。
    let at = crate::server::core::beijing::date_time_of_ms(stored).expect("合法时间戳");
    use chrono::Timelike;
    assert!(
        at.hour() == 0 && at.minute() == 0 && at.second() == 0,
        "恢复时刻必须是北京零点（日池按天重置），实际 {}",
        at.format("%Y-%m-%d %H:%M:%S")
    );
}

#[test]
fn that_one_argument_is_the_divider_between_a_day_and_ten_minutes() {
    let (_dir, service, id) = temp_service();
    // 1 ： 1 by 「 after 」， 1 by 「 before 」
    // （ 1 rotate::mark_account_limited， is when None）。
    // 1 、 1 —— has that 1 。
    let pre_fix = service
        .store
        .add_account(
            &json!({"auth": {"accessToken": "at2"}, "account": {"uid": "quota-2"}}),
            None,
            None,
        )
        .expect("对照账号应当能建起来");
    let pre_fix_id = pre_fix["id"].as_str().expect("公开形态应带 id").to_string();

    let now = logging::now_ms();
    let reset_at = now + 3 * 60 * 60 * 1000;
    let message = "上游返回 403: insufficient quota";
    let class = UpstreamErrorClass::QuotaLimited {
        reset_at: Some(reset_at),
        message: message.to_string(),
        upstream_code: Some(4291200),
        status: 403,
    };
    mark_conversation_limited(&service, &id, &single("m-after"), &class, message);
    // before that 1 ： old ， when None
    rotate::mark_account_limited(&service, &pre_fix_id, "m-before", 403, Some(4291200), None, message);

    let after = rotate::account_limit_reset_at(&service, &id, "m-after");
    let before = rotate::account_limit_reset_at(&service, &pre_fix_id, "m-before");
    assert_eq!(after, reset_at, "改后应透传适配器给的恢复时刻");
    let before_minutes = (before - now) / 60_000;
    assert!(
        before_minutes >= 9 && before_minutes <= 11,
        "改前落的是存储层兜底冷却（10 分钟），实际 {before_minutes} 分钟"
    );
    // ： 1 ， 2 when up —— is from this 
    assert!(
        after - before > 2 * 60 * 60 * 1000,
        "透传与写死 None 的差距应当是「一整天 vs 10 分钟」量级，实际差 {} 分钟",
        (after - before) / 60_000
    );
}

const APIG_TOO_LARGE_BODY: &str = r#"{"error_msg":"Request entity too large","error_code":"APIG.0201","request_id":"apig-test"}"#;

fn fatal(status: u16) -> UpstreamErrorClass {
    UpstreamErrorClass::Fatal {
        status,
        message: format!("上游返回 {status}"),
        upstream_code: None,
    }
}

/// 413 is 「 this 」， 1 no has will ⇒ not 。
/// before to that 1 ， ×5 seconds + 13 seconds = client 60 seconds
/// etc to ，「408 client in response before 」，（413） in 。
#[test]
fn a_payload_size_rejection_gets_no_in_place_retry() {
    assert!(
        PAYLOAD_SIZE_REJECTION_STATUSES.contains(&413),
        "413 应当在「按体积拒收」名单里"
    );
    assert!(
        fallback_retry_advice(&fatal(413), 3, 413).is_none(),
        "预算还剩 3 次也不该给 413 提原地重发"
    );
}

/// for ： its 4xx/unknownfailure old （ this "" that 1 ，
/// no has 400 this " by up " 1 and excluding）。404 already issue #255 
/// （ by / failure， `DETERMINISTIC_NOT_FOUND_STATUSES`）， not in 。
#[test]
fn other_fatal_statuses_still_get_the_fallback_resend() {
    for status in [400u16, 409] {
        assert!(
            fallback_retry_advice(&fatal(status), 3, status).is_some(),
            "HTTP {status} 的语义由上游决定，兜底原地重发的既有行为不该变"
        );
    }
    assert!(
        !TRANSIENT_RETRY_STATUSES.contains(&413),
        "413 不该被当成瞬时抖动（否则瞬时那一档会把它重发回去）"
    );
}

/// to ： to return 413 up ，** 1 **（ before is 1 + 3 ）。
#[tokio::test]
async fn a_real_413_response_is_sent_once_and_handed_back() {
    let (transport, hits, task) =
        mock_upstream(413, "application/json; charset=utf-8", APIG_TOO_LARGE_BODY).await;
    let mut budget = RetryBudget::new(3);
    let capture = crate::server::core::debug_traffic::TrafficCapture::begin("payload-too-large");
    // 「 up error」 this 1 ；413 in ****
    // （`transport::RetryBudget` + `fallback_retry_advice`）， 1 
    // can 。 this use ZCode ， to —— 
    // WorkBuddy， is 1 。
    let result = send_with_retry(
        adapter_for(ProviderKind::WorkBuddy),
        &transport,
        &mut budget,
        Some(&capture),
        &std::sync::Arc::new(RequestTelemetry::new()),
        false,
        None,
        None,
    )
    .await;
    task.abort();
    let failure = match result {
        Ok(_) => panic!("HTTP 413 不该被当成成功流"),
        Err(failure) => failure,
    };
    assert_eq!(
        1,
        hits.load(Ordering::SeqCst),
        "体积拒收的失败应当一次就交回，不该原地重发"
    );
    assert!(
        matches!(failure.class, UpstreamErrorClass::Fatal { status: 413, .. }),
        "状态码 413 要原样带回，不该被折成别的失败"
    );
}

#[test]
fn a_deterministic_404_skips_the_in_place_resend() {
    // issue #255 ：OfficeAce for directory has 、 no has 
    // `404 {"code":null,"message":"Invalid model."}`， old Fatal，
    // 3 × 5 seconds——client etc 15 seconds， is 1 404。
    let not_found = UpstreamErrorClass::Fatal {
        status: 404,
        upstream_code: None,
        message: "上游返回 404: {\"code\":null,\"message\":\"Invalid model.\"}".to_string(),
    };
    assert!(
        fallback_retry_advice(&not_found, 3, 404).is_none(),
        "名字类确定性失败不该再原地重发"
    );
    // for ： 400 （400 by up ， is default）
    let bad_request = UpstreamErrorClass::Fatal {
        status: 400,
        upstream_code: None,
        message: "上游返回 400: whatever".to_string(),
    };
    assert!(
        fallback_retry_advice(&bad_request, 3, 400).is_some(),
        "404 的豁免不许顺手把 400 也豁免了"
    );
}
