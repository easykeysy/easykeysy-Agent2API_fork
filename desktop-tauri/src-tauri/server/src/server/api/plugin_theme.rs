//! ：、、。
//!
//! ```text
//! GET /plugins/runtime/theme.css and parameter
//! GET /plugins/runtime/islands/ui.js + React（）
//! GET /plugins/runtime/islands/ui.css Tailwind （）
//! GET /plugins/runtime/providers/<id>.png 
//! ```
//!
//! ── this ────────────────────────────────────
//! （ to in this ） in iframe ， is ****： not to 
//! `ui/css/tokens.css`、 not to already React / ， not to 
//! `ui/assets/providers/` 。 is can 1 、 1 、
//! use —— will and ，「 
//! not 1 」。
//!
//! this from to iframe， can use ** and that 1 **：
//! · `theme.css` → / / / ， 1 not ；
//! · `islands/ui.js` + `islands/ui.css` → 1 and 
//! （`window.wbUiKit` React and ， ui-islands/src/uikit.tsx）；
//!   · `providers/*.png` → ， and / use is 1 。
//!
//! ── Why not is ──────────────────────────
//! ** not ** `ui/` directory in （ by Tauri from ，
//! `ServerState::ui_dir` has headless set， `http.rs` ）。
//! path will in down 404。 this file use `include_str!` /
//! `include_bytes!` ， as 1 ， not 「file 」
//! this failure。：`ui.js` 1.3 MB（），exe 。
//!
//! ── not protected ──────────────────────────────────────────
//! and `/plugins/{pack}/ui/*` 1 by ：iframe request not up ， this 
//! file not —— is `ui/` directory that （`ui/index.html`
//! not ）。 request `/api/*`（ protected）。
//!
//! ── ──────────────────────────────────────────────────
//! release is `panic=abort`：file unwrap/expect/panic。

use axum::body::Body;
use axum::extract::Path;
use axum::http::{header, StatusCode};
use axum::response::Response;

/// parameter（ by ISLANDS_CSS ）。
const TOKENS: &str = include_str!("../../../../../ui/css/tokens.css");

/// + React iife （`ui-islands` when ）。
const ISLANDS_JS: &str = include_str!("../../../../../ui/islands/ui.js");

/// Tailwind （）。
const ISLANDS_CSS: &str = include_str!("../../../../../ui/islands/ui.css");

/// path； already in ISLANDS_CSS in 。
const LAYOUT_CSS: &str = include_str!("../../../../../ui/css/layout.css");

/// and old path， by 。
const COMPONENTS_CSS: &str = include_str!("../../../../../ui/css/components.css");

/// ：`(provider id, PNG )`。
///
/// ── Why use 1 not is directory ──────────────────────────
/// `include_bytes!` must in path， no directory。 this is 
/// 「file」 —— 1 new when need in this 1 。
/// is not is （ not is ）， can 。
///
/// **** use to that （ to in has to can ）。 use 
/// its not in this ： `ui/assets/providers/` path。
const PROVIDER_ICONS: &[(&str, &[u8])] = &[
    ("workbuddy", include_bytes!("../../../../../ui/assets/providers/workbuddy.png")),
    ("raccoon", include_bytes!("../../../../../ui/assets/providers/raccoon.png")),
    ("catpaw", include_bytes!("../../../../../ui/assets/providers/catpaw.png")),
    ("autoclaw", include_bytes!("../../../../../ui/assets/providers/autoclaw.png")),
    ("qoder", include_bytes!("../../../../../ui/assets/providers/qoder.png")),
    ("cline", include_bytes!("../../../../../ui/assets/providers/cline.png")),
    ("accio", include_bytes!("../../../../../ui/assets/providers/accio.png")),
    ("zcode", include_bytes!("../../../../../ui/assets/providers/zcode.png")),
    ("codearts", include_bytes!("../../../../../ui/assets/providers/codearts.png")),
    ("trae", include_bytes!("../../../../../ui/assets/providers/trae.png")),
    ("loomy", include_bytes!("../../../../../ui/assets/providers/loomy.png")),
    ("kuku", include_bytes!("../../../../../ui/assets/providers/kuku.png")),
    ("monkeycode", include_bytes!("../../../../../ui/assets/providers/monkeycode.png")),
    ("commandcode", include_bytes!("../../../../../ui/assets/providers/commandcode.png")),
    ("antigravity", include_bytes!("../../../../../ui/assets/providers/antigravity.png")),
    ("officeace", include_bytes!("../../../../../ui/assets/providers/officeace.png")),
    ("orcaterm", include_bytes!("../../../../../ui/assets/providers/orcaterm.png")),
    ("chatglm", include_bytes!("../../../../../ui/assets/providers/chatglm.png")),
    ("modelscope", include_bytes!("../../../../../ui/assets/providers/modelscope.png")),
    ("atomcode", include_bytes!("../../../../../ui/assets/providers/atomcode.png")),
    ("lobsterai", include_bytes!("../../../../../ui/assets/providers/lobsterai.png")),
];

const MINIMAX_CODE_SVG: &[u8] = include_bytes!("../../../../../ui/assets/providers/minimax-code.svg");

/// response ： builder 。
fn reply(status: StatusCode, mime: &'static str, body: Vec<u8>) -> Response {
    Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, mime)
        // and its 1 ： after to new ，
        // not will 「 use new 、js is old 」 this in 
        .header(header::CACHE_CONTROL, "no-store")
        .body(Body::from(body))
        .unwrap_or_else(|_| {
            Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::empty())
                .unwrap_or_else(|_| Response::new(Body::empty()))
        })
}

/// GET /plugins/runtime/{*rest}
///
/// 1 ：path is （`match`）， not before ——
/// after is 「request not in file to 」， 404 。
pub async fn serve(Path(rest): Path<String>) -> Response {
    // 1 ：empty and `.`，`..` 1 （ this has file，
    // not need need 「 to up 」 ）
    let normalized: Vec<&str> = rest
        .split('/')
        .filter(|segment| !segment.is_empty() && *segment != ".")
        .collect();
    if normalized.iter().any(|segment| *segment == ".." || segment.contains('\\')) {
        return reply(StatusCode::NOT_FOUND, "text/plain; charset=utf-8", b"not found".to_vec());
    }
    let path = normalized.join("/");
    match path.as_str() {
        "theme.css" => reply(
            StatusCode::OK,
            "text/css; charset=utf-8",
            format!("{ISLANDS_CSS}\n{TOKENS}").into_bytes(),
        ),
        "islands/ui.js" => reply(
            StatusCode::OK,
            "text/javascript; charset=utf-8",
            ISLANDS_JS.as_bytes().to_vec(),
        ),
        "islands/ui.css" => reply(
            StatusCode::OK,
            "text/css; charset=utf-8",
            ISLANDS_CSS.as_bytes().to_vec(),
        ),
        "layout.css" => reply(
            StatusCode::OK,
            "text/css; charset=utf-8",
            LAYOUT_CSS.as_bytes().to_vec(),
        ),
        "components.css" => reply(
            StatusCode::OK,
            "text/css; charset=utf-8",
            COMPONENTS_CSS.as_bytes().to_vec(),
        ),
        "tooltip.css" => reply(
            StatusCode::OK,
            "text/css; charset=utf-8",
            ISLANDS_CSS.as_bytes().to_vec(),
        ),
        "providers/minimax-code.svg" | "providers/minimax-code-global.svg" => reply(
            StatusCode::OK,
            "image/svg+xml",
            MINIMAX_CODE_SVG.to_vec(),
        ),
        other => {
            // ：`providers/<id>.png`
            let Some(name) = other.strip_prefix("providers/") else {
                return reply(StatusCode::NOT_FOUND, "text/plain; charset=utf-8", b"not found".to_vec());
            };
            let Some(id) = name.strip_suffix(".png") else {
                return reply(StatusCode::NOT_FOUND, "text/plain; charset=utf-8", b"not found".to_vec());
            };
            match PROVIDER_ICONS.iter().find(|(key, _)| *key == id) {
                Some((_, bytes)) => reply(StatusCode::OK, "image/png", bytes.to_vec()),
                None => reply(StatusCode::NOT_FOUND, "text/plain; charset=utf-8", b"not found".to_vec()),
            }
        }
    }
}
