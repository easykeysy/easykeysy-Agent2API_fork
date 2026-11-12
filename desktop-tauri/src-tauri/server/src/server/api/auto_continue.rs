//! 「」 and limit：`GET/PUT /api/auto-continue`。
//!
//! ── this ──────────────────────────────────────────
//! up turn will ** in **： use （`finish_reason: length` 1 ）、 
//! 、or in 、 1 。client to is ，
//! none「 this 」 and 「 」。 after ** already that **
//! assistant to up 、 1 「」 1 ， new in 
//! ** 1 **clientresponse after 。、（limit + ） and 
//! in [`crate::server::core::upstream::continuation`]， this ：
//! not （`autoContinue`）、（`autoContinueMaxRounds`，default 3、 1~8）。
//!
//! ── Why 1 ， not is keepalive that 1 1 ──
//! as this **must 1 has **： ，limit is that ****
//! （ 1 is ， is 「 is not is up 1 」， is ），
//! 1 request up by this 。 then will 
//! 「、limit no 」 in —— that is 1 set ， 1 
//! PUT（ and `/api/routing` that 「 1 + has 」）。
//!
//! ── Whydefault、Why need can in ───────────────────────
//! is ： 1 before ， 1 ** not **
//! before 1 is and ， in 「 this 1 」 that 1 request up 。 +
//! 「client to 」 this can as ，default（
//! `config::KEY_AUTO_CONTINUE` ）。 in ， is as ** can in up 
//! and when **： 1 down 、request、 not for ， not need need new 。
//! request， down 1 request， not 。
//!
//! ── Why can （partial update）─────────────────────
//! use is ： use default 3 1 ， after to 1 ——
//! that 1 PUT not this already has （ 1 and 1 ），
//! not this need use GET can PUT。** 1 1 **，
//! ； when （ and `put_retry` / `put_retention`
//! 1 ），「、limit no 」 set。
//!
//! ── and /api/keepalive-strip、/api/sanitize ────────────
//! 1 （GET / PUT / response is new ）， in ：
//! that 1 ， is 1 + 1 has （ is ）。
//! is 「client to / as 」 and default， use 1 
//! and 1 （`[Config]`， to ）。

use axum::body::Bytes;
use axum::extract::State;
use axum::response::Response;
use serde_json::{json, Map, Value};

use crate::server::config::{
    self, AutoContinuePatch, KEY_AUTO_CONTINUE, KEY_AUTO_CONTINUE_MAX_ROUNDS,
    AUTO_CONTINUE_MAX_ROUNDS, AUTO_CONTINUE_MIN_ROUNDS,
};
use crate::server::config::parse::int_in_range;
use crate::server::errors;
use crate::server::http::{ok_json, parse_body};
use crate::server::logging;
use crate::server::ServerState;

/// GET /api/auto-continue —— current and limit
pub async fn get_auto_continue(State(_state): State<ServerState>) -> Response {
    ok_json(state_json())
}

/// PUT /api/auto-continue —— body `{autoContinue?: true|false, autoContinueMaxRounds?: 1~8}`
///
/// （ not ），。config after down 1 request
/// （request， not ）。 and 1 ： that 1 ，
/// 1 clientrequest can can for up 、 1 ， that 1 ——
/// 「 this request up 」 when ，must to this 。
pub async fn put_auto_continue(State(_state): State<ServerState>, body: Bytes) -> Response {
    let payload = match parse_body(&body) {
        Ok(value) => value,
        Err(error) => return errors::management_error(400, error.message),
    };
    let Some(object) = payload.as_object() else {
        return errors::management_error(400, "请求体必须是 JSON 对象");
    };
    let patch = match patch_from(object) {
        Ok(patch) => patch,
        Err(message) => return errors::management_error(400, message),
    };
    if patch.enabled.is_none() && patch.max_rounds.is_none() {
        // no to ： not 、 not （ 1 「set already update」 down no has ，
        // will as 1 ）， current to use 
        return ok_json(state_json());
    }
    if !config::set_auto_continue_patch(patch) {
        logging::log("[Config]", "⚠️  自动续写设置写入配置失败，本次运行内仍生效");
    }
    logging::log("[Config]", &change_note(&patch));
    ok_json(state_json())
}

/// request → update（`Err` is 400 ）。
///
/// ── Why ────────────────────────────────────────
/// need can ：`ServerState::bootstrap` will 、 `~/.agent2api`，
/// use not handler ， this is 「 to 1 request」
/// that 1 （ and `api::cline_headers` `validated_overrides` 1 ）。
///
/// ── Why not `null` ───────────────────────────────────────
/// and [`KEY_AUTO_CONTINUE`]、[`KEY_AUTO_CONTINUE_MAX_ROUNDS`] ** 1 **：
/// = use current（ not ），must is 。`null` in this can is 
/// —— this PUT no has 「 this 1 」 （setter insert， delete
/// ）， `null` not ： not is 「」（ that is `false`）， not is 
/// 「 to 」（ that need ）。 `put_retry` that 「 this 1 not 」，
/// is use to 200 no 。
fn patch_from(object: &Map<String, Value>) -> Result<AutoContinuePatch, String> {
    let mut patch = AutoContinuePatch::default();
    if let Some(value) = object.get(KEY_AUTO_CONTINUE) {
        let Some(enabled) = value.as_bool() else {
            return Err(format!("{KEY_AUTO_CONTINUE} 必须是 true 或 false"));
        };
        patch.enabled = Some(enabled);
    }
    if let Some(value) = object.get(KEY_AUTO_CONTINUE_MAX_ROUNDS) {
        // and use `int_in_range`（「 3.0 this 」 this 1 ）：
        // and 「 1 」 1 ，
        let Some(rounds) = int_in_range(
            value,
            AUTO_CONTINUE_MIN_ROUNDS,
            AUTO_CONTINUE_MAX_ROUNDS,
        ) else {
            return Err(format!(
                "{KEY_AUTO_CONTINUE_MAX_ROUNDS} 必须是 {AUTO_CONTINUE_MIN_ROUNDS}~{AUTO_CONTINUE_MAX_ROUNDS} 的整数"
            ));
        };
        patch.max_rounds = Some(rounds);
    }
    Ok(patch)
}

/// this 1 （）。
///
/// ： / limit / 1 。 need can ** this PUT
/// 1 **，otherwise「 limit from 3 to 8 」 this can request。
fn change_note(patch: &AutoContinuePatch) -> String {
    let mut parts: Vec<String> = Vec::new();
    if let Some(enabled) = patch.enabled {
        parts.push(if enabled {
            "自动续写已开启（一次被腰斩的回答会由网关自己问完）".to_string()
        } else {
            "自动续写已关闭：半截回答原样下发（与接入前逐字节一致）".to_string()
        });
    }
    if let Some(rounds) = patch.max_rounds {
        parts.push(format!("续写轮数上限已设为 {rounds}"));
    }
    parts.join(" / ")
}

/// and limit（GET and PUT use ， before use response new ）
fn state_json() -> Value {
    json!({
        KEY_AUTO_CONTINUE: config::auto_continue(),
        KEY_AUTO_CONTINUE_MAX_ROUNDS: config::auto_continue_max_rounds(),
    })
}

#[cfg(test)]
mod tests {
    //! PUT ： and 。
    //!
    //! ── Why in this not is response up ────────────────────
    //! handler need need `ServerState` can by `ServerState::bootstrap` ， will 
    //! 、 `~/.agent2api` —— use not 。 is [`patch_from`]
    //! （return `Err()`），handler that has 1 
    //! `Err(message) => errors::management_error(400, message)`： not ，
    //! 400 is that ， not is can 。 and `api::cline_headers`、
    //! `api::codearts_welfare` 1 。
    //!
    //! ── use is 「」───────────────────────────
    //! this for 1 **** ， not is ：
    //! `null`、 use `as_i64`（ is 3.0 ）、 when default。

    use super::*;

    /// 1 to request。
    fn body(pairs: &[(&str, Value)]) -> Map<String, Value> {
        pairs.iter().map(|(key, value)| (key.to_string(), value.clone())).collect()
    }

    /// `true` / `false`；`null` not 。
    ///
    /// ： `null` 「 no to 」（`put_retry` for null is this ，
    /// that —— will has ； this no has 「empty」），
    /// is use to 200 1 no 。
    #[test]
    fn the_switch_accepts_only_a_literal_boolean() {
        let only_false = patch_from(&body(&[(KEY_AUTO_CONTINUE, json!(false))]))
            .expect("字面 false 是合法值");
        assert_eq!(only_false.enabled, Some(false), "关掉必须能被表达出来");

        for bad in [json!("true"), json!(1), json!(null), json!({})] {
            let error = match patch_from(&body(&[(KEY_AUTO_CONTINUE, bad.clone())])) {
                Ok(patch) => panic!("{bad} 不该被接受，却拿到了 {patch:?}"),
                Err(error) => error,
            };
            assert!(
                error.contains("必须是 true 或 false"),
                "文案要与 /api/keepalive-strip 同形，拿到的是: {error}"
            );
        }
    }

    /// limit ：1 and 8 ，0 / 9 / ；`3.0` this **must**。
    ///
    /// ： use `value.as_i64()`（`routing_api` `int_field` is 
    /// this ）—— that 3.0、 400， 1 config
    /// （`parse::int_in_range` in by is 1 ）。
    #[test]
    fn the_round_cap_is_validated_against_the_same_range_as_the_read_side() {
        for (accepted, expected) in [(json!(1), 1), (json!(8), 8), (json!(3.0), 3)] {
            let patch = match patch_from(&body(&[(KEY_AUTO_CONTINUE_MAX_ROUNDS, accepted.clone())]))
            {
                Ok(patch) => patch,
                Err(error) => panic!("{accepted} 应当被接受，却报错: {error}"),
            };
            assert_eq!(
                patch.max_rounds,
                Some(expected),
                "整值浮点要按它表示的那个整数落盘，不是回落默认值"
            );
        }
        for rejected in [json!(0), json!(9), json!(-1), json!("3"), json!(null), json!(2.5)] {
            let error = match patch_from(&body(&[(
                KEY_AUTO_CONTINUE_MAX_ROUNDS,
                rejected.clone(),
            )])) {
                Ok(patch) => panic!("{rejected} 越界或类型不符，不该被接受，却拿到了 {patch:?}"),
                Err(error) => error,
            };
            assert_eq!(
                error,
                format!(
                    "{KEY_AUTO_CONTINUE_MAX_ROUNDS} 必须是 {AUTO_CONTINUE_MIN_ROUNDS}~{AUTO_CONTINUE_MAX_ROUNDS} 的整数"
                ),
                "文案是接口契约（面板按它提示），且必须带上真正的边界",
            );
        }
    }

    /// ： 1 when 1 is `None`（= not ）， not when is `None`。
    ///
    /// ：default —— that will 「 to 1 」 this 1 PUT
    /// （ use can down 1 ），
    /// and `set_auto_continue_patch` that not 「 no to 」 and 「 to is default」。
    #[test]
    fn an_absent_key_stays_absent_rather_than_becoming_the_default() {
        let only_switch = patch_from(&body(&[(KEY_AUTO_CONTINUE, json!(true))])).expect("合法");
        assert_eq!((only_switch.enabled, only_switch.max_rounds), (Some(true), None));

        let only_rounds =
            patch_from(&body(&[(KEY_AUTO_CONTINUE_MAX_ROUNDS, json!(1))])).expect("合法");
        assert_eq!((only_rounds.enabled, only_rounds.max_rounds), (None, Some(1)));

        let both = patch_from(&body(&[
            (KEY_AUTO_CONTINUE, json!(true)),
            (KEY_AUTO_CONTINUE_MAX_ROUNDS, json!(8)),
        ]))
        .expect("合法");
        assert_eq!((both.enabled, both.max_rounds), (Some(true), Some(8)));

        // empty for = no to ： not （ 1 PUT 1 is ， to is ），
        // must is None，otherwise handler will 1 no has 
        let nothing = patch_from(&body(&[])).expect("空对象不该报错");
        assert_eq!(nothing.enabled, None);
        assert_eq!(nothing.max_rounds, None);
    }
}
