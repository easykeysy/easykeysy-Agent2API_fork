#!/usr/bin/env node
//! parity ：** vs **， 1 1 。
//!
//! ── this ────────────────────────────────────
//! 1 provider from 「」 to 「」 when ， not is 
//! not ， is ** as **： 、 default、 
//! 。 this can ， can 「」， to request。
//!
//! this ****： 1 to ， for 。
//!
//! ── （ by 「」）──────────────────────────────
//! 1. `chat`：URL / request / request（** etc **，）
//! 2. `classify`： （：402→429、403 ）
//! 3. `models`： and 
//! 4. can and provider id
//!
//! ── ──────────────────────────────────────────────────
//! ```bash
//! # 1. 
//! cd agent2api-providers && cargo build
//! # 2. parity
//! node scripts/parity-check.mjs
//! ```
//! not in when （ not failure）。
//!
//! ── Why use Node not is Rust ──────────────────────
//! need need need `ServerState`（、config、…），
//! that is 。**** need need stdin/stdout —— Node 
//! 。 is this is ：** use Node **， and 「 from 
//! down 」 for 。 ， is 。

import { spawn } from "node:child_process";
import { existsSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const root = join(here, "..");

function pluginExe() {
  const candidates = [
    join(root, "agent2api-providers/target/debug/agent2api-providers.exe"),
    join(root, "agent2api-providers/target/debug/agent2api-providers"),
    join(root, "agent2api-providers/target/release/agent2api-providers.exe"),
    join(root, "agent2api-providers/target/release/agent2api-providers"),
  ];
  return candidates.find((path) => existsSync(path));
}

// ── 1 「」：、 ────────────────────────
class Gateway {
  constructor(exe) {
    this.child = spawn(exe, [], { stdio: ["pipe", "pipe", "inherit"] });
    this.nextId = 1;
    this.pending = new Map();
    this.buffer = "";
    this.child.stdout.on("data", (chunk) => {
      this.buffer += chunk.toString("utf8");
      let index;
      while ((index = this.buffer.indexOf("\n")) >= 0) {
        const line = this.buffer.slice(0, index);
        this.buffer = this.buffer.slice(index + 1);
        if (!line.trim()) continue;
        let frame;
        try {
          frame = JSON.parse(line);
        } catch {
          continue;
        }
        if (frame.id && this.pending.has(frame.id)) {
          const { resolve, reject } = this.pending.get(frame.id);
          this.pending.delete(frame.id);
          if (frame.error) reject(new Error(frame.error.message));
          else resolve(frame.result);
        }
      }
    });
  }

  call(method, params) {
    const id = this.nextId++;
    return new Promise((resolve, reject) => {
      this.pending.set(id, { resolve, reject });
      this.child.stdin.write(JSON.stringify({ id, method, params }) + "\n");
    });
  }

  async init() {
    return this.call("init", {
      env: {
        data_dir: "",
        host_version: "parity",
        target: "x86_64-pc-windows-msvc",
        proxy: "",
        debug_traffic: false,
      },
    });
  }

  close() {
    this.child.stdin.end();
    this.child.kill();
  }
}

// ── （failure need can 1 1 、 in ）─────────────────
let failures = 0;
let checks = 0;

/**
 * ，**ignore for **。
 *
 * ── Why not can JSON.stringify ────────────────────────────
 * request by `serde_json` ， `Map` default is **BTreeMap**
 * （ by ）。 is 1 ， its 1 ——
 * 「」 will this in none up （e.g. to 1 
 * ）。JSON for no has ， can 。
 */
function deepEqual(a, b) {
  if (a === b) return true;
  if (a === null || b === null) return a === b;
  if (typeof a !== typeof b) return false;
  if (Array.isArray(a) !== Array.isArray(b)) return false;
  if (Array.isArray(a)) {
    if (a.length !== b.length) return false;
    return a.every((item, index) => deepEqual(item, b[index]));
  }
  if (typeof a === "object") {
    const ka = Object.keys(a).sort();
    const kb = Object.keys(b).sort();
    if (ka.length !== kb.length) return false;
    if (ka.some((key, index) => key !== kb[index])) return false;
    return ka.every((key) => deepEqual(a[key], b[key]));
  }
  return false;
}

function check(label, actual, expected) {
  checks += 1;
  if (deepEqual(actual, expected)) return;
  failures += 1;
  console.error(`\n✗ ${label}`);
  console.error(`  实际: ${JSON.stringify(actual)}`);
  console.error(`  期望: ${JSON.stringify(expected)}`);
}

function checkTruthy(label, value, note = "") {
  checks += 1;
  if (value) return;
  failures += 1;
  console.error(`\n✗ ${label}${note ? `（${note}）` : ""}`);
}

/**
 * `headers` is `Vec<(String, String)>`， after is ****（，
 * as in up that has ）。 this for by 。
 */
function headerMap(headers) {
  if (!headers) return {};
  if (Array.isArray(headers)) {
    return Object.fromEntries(headers.map(([key, value]) => [key, value]));
  }
  return headers;
}

/** （ not —— `deepEqual` ） */
function checkKeySet(label, actual, expected) {
  checks += 1;
  const a = Object.keys(actual).sort();
  const e = [...expected].sort();
  if (JSON.stringify(a) === JSON.stringify(e)) return;
  failures += 1;
  console.error(`\n✗ ${label}`);
  console.error(`  实际: ${JSON.stringify(a)}`);
  console.error(`  期望: ${JSON.stringify(e)}`);
}

// ── ： store.get to secret ──────────────
function accountSecret(provider) {
  if (provider === "commandcode") return { apiKey: "user_parity_test_key_123456" };
  return { accessToken: "ms-parity-token-abcdefgh", userId: "2220378210908" };
}

/**
 * 1 「」 ： `store.get` 。
 *
 * ── Whymust ──────────────────────────────────────
 * `build_chat` / `ensure_token` will `store.get` 。
 * not that use will to when —— this need is 。
 */
class Host extends Gateway {
  constructor(exe, secret) {
    super(exe);
    this.secret = secret;
    this.child.stdout.removeAllListeners("data");
    this.buffer = "";
    this.child.stdout.on("data", (chunk) => {
      this.buffer += chunk.toString("utf8");
      let index;
      while ((index = this.buffer.indexOf("\n")) >= 0) {
        const line = this.buffer.slice(0, index);
        this.buffer = this.buffer.slice(index + 1);
        if (!line.trim()) continue;
        let frame;
        try {
          frame = JSON.parse(line);
        } catch {
          continue;
        }
        // （ has method、 no has result/error）
        if (frame.method === "store.get") {
          const provider = frame.params?.provider ?? "";
          const id = frame.params?.account || `${provider}-parity`;
          this.child.stdin.write(
            JSON.stringify({
              id: frame.id,
              result: { id, provider, secret: this.secret, public: {} },
            }) + "\n"
          );
          continue;
        }
        if (frame.id && this.pending.has(frame.id)) {
          const { resolve, reject } = this.pending.get(frame.id);
          this.pending.delete(frame.id);
          if (frame.error) reject(new Error(frame.error.message));
          else resolve(frame.result);
        }
      }
    });
  }
}

// ══════════════════════════════════════════════════════════════
// Command Code
// ══════════════════════════════════════════════════════════════

/**
 * （ from `providers/commandcode/plan.rs` ）。
 *
 * ── this Why in this ────────────────────────────────
 * is 「 in this down 」。 = ： 1 
 * ， not is 1 after 1 。 need this ，must when and 
 * Why。
 */
const CC_EXPECT = {
  url: "https://api.commandcode.ai/alpha/generate",
  headerNames: [
    "Content-Type",
    "User-Agent",
    "x-command-code-version",
    "x-cli-environment",
    "x-project-slug",
    "x-taste-learning",
    "x-session-id",
    "Authorization",
    "traceparent",
  ],
  userAgent: "cli",
  cliEnvironment: "production",
  envelopeKeys: [
    "config",
    "memory",
    "taste",
    "skills",
    "permissionMode",
    "mode",
    "params",
  ],
  permissionMode: "standard",
  mode: "agent",
  defaultModel: "deepseek/deepseek-v4-flash",
  defaultMaxTokens: 64000,
  maxTokensCap: 200000,
  placeholderSystem: [{ type: "text", text: " " }],
};

async function checkCommandCode(exe) {
  const host = new Host(exe, accountSecret("commandcode"));
  const infos = await host.init();
  const info = infos.find((item) => item.id === "commandcode");
  checkTruthy("Command Code 在握手里出现", Boolean(info));
  if (!info) return host.close();

  // can （ and for ）
  check("cc.capabilities.chat", info.capabilities.chat, true);
  check("cc.capabilities.stateful", info.capabilities.stateful, false);
  check("cc.capabilities.refresh", info.capabilities.refresh, false);
  check("cc.capabilities.model_refresh", info.capabilities.model_refresh, true);
  check("cc.capabilities.usage", info.capabilities.usage, false);
  check("cc.capabilities.checkin", info.capabilities.checkin, false);
  check("cc.capabilities.web_login", info.capabilities.web_login, false);
  check("cc.label", info.label, "Command Code");

  // ── chat：request ───────────────────────────────────────
  const plan = await host.call("chat", {
    provider: "commandcode",
    account: "cc-1",
    proxy: null,
    body: { messages: [{ role: "user", content: "hi" }] },
    headers: {},
    stream: true,
    wire_model: "deepseek/deepseek-v4-flash",
    reasoning: "",
  });
  check("cc.url", plan.url, CC_EXPECT.url);
  check("cc.upstream", plan.upstream, "ndjson");
  const headers = headerMap(plan.headers);
  checkKeySet("cc.headers 集合", headers, CC_EXPECT.headerNames);
  check("cc.headers.User-Agent", headers["User-Agent"], CC_EXPECT.userAgent);
  check("cc.headers.x-cli-environment", headers["x-cli-environment"], CC_EXPECT.cliEnvironment);
  check("cc.headers.Authorization", headers.Authorization, "Bearer user_parity_test_key_123456");
  check("cc.headers.x-taste-learning 是字符串 false", headers["x-taste-learning"], "false");
  checkTruthy(
    "cc.headers.traceparent 是 W3C 形态",
    /^00-[0-9a-f]{32}-[0-9a-f]{16}-01$/.test(headers.traceparent ?? ""),
    headers.traceparent
  );

  // ── ────────────────────────────────────────────────
  // has will id when will 1 threadId（ as UUID）
  const expectedKeys = [...CC_EXPECT.envelopeKeys, "threadId"];
  checkKeySet("cc.envelope 键集合", plan.body, expectedKeys);
  check("cc.envelope.memory", plan.body.memory, null);
  check("cc.envelope.taste", plan.body.taste, null);
  check("cc.envelope.skills", plan.body.skills, null);
  check("cc.envelope.permissionMode", plan.body.permissionMode, CC_EXPECT.permissionMode);
  check("cc.envelope.mode", plan.body.mode, CC_EXPECT.mode);
  checkTruthy(
    "cc.envelope.threadId 是合法 UUID",
    /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/.test(
      plan.body.threadId ?? ""
    ),
    plan.body.threadId
  );

  // ── params ──────────────────────────────────────────────
  const params = plan.body.params;
  check("cc.params.model 缺省值", params.model, CC_EXPECT.defaultModel);
  check("cc.params.max_tokens 缺省值", params.max_tokens, CC_EXPECT.defaultMaxTokens);
  check("cc.params.stream 恒 true", params.stream, true);
  check("cc.params.system 空格占位", params.system, CC_EXPECT.placeholderSystem);
  check("cc.params.tools 总是下发（空数组）", params.tools, []);
  checkTruthy("cc.params 没有 tool_choice 键", !("tool_choice" in params));
  check("cc.params.messages 是块数组", params.messages, [
    { role: "user", content: [{ type: "text", text: "hi" }] },
  ]);

  // ── config ：must is in ──────────────────
  const config = plan.body.config;
  check("cc.config.workingDir 是伪造值", config.workingDir, "C:\\Users\\dev\\projects\\app");
  check("cc.config.environment", config.environment, "win32");
  check("cc.config.structure", config.structure, []);
  check("cc.config.isGitRepo", config.isGitRepo, false);
  check("cc.config.currentBranch", config.currentBranch, "");
  check("cc.config.recentCommits", config.recentCommits, []);
  checkTruthy(
    "cc.config.date 是 YYYY-MM-DD",
    /^\d{4}-\d{2}-\d{2}$/.test(config.date ?? ""),
    config.date
  );

  // ── max_tokens limit ─────────────────────────────────────
  const capped = await host.call("chat", {
    provider: "commandcode",
    account: "cc-1",
    proxy: null,
    body: {
      messages: [{ role: "user", content: "x" }],
      max_tokens: 999999,
    },
    headers: {},
    stream: true,
    wire_model: "m",
    reasoning: "",
  });
  check("cc.params.max_tokens 被截到上限", capped.body.params.max_tokens, CC_EXPECT.maxTokensCap);

  // ── tool_choice = none → empty tools and not down tool_choice ─────
  const noTools = await host.call("chat", {
    provider: "commandcode",
    account: "cc-1",
    proxy: null,
    body: {
      messages: [{ role: "user", content: "x" }],
      tools: [{ type: "function", function: { name: "f", parameters: {} } }],
      tool_choice: "none",
    },
    headers: {},
    stream: true,
    wire_model: "m",
    reasoning: "",
  });
  check("cc.tool_choice=none → tools 清空", noTools.body.params.tools, []);
  checkTruthy("cc.tool_choice=none → 不下发 tool_choice", !("tool_choice" in noTools.body.params));

  // ── tool_choice = required → any ────────────────────────
  const anyTools = await host.call("chat", {
    provider: "commandcode",
    account: "cc-1",
    proxy: null,
    body: {
      messages: [{ role: "user", content: "x" }],
      tools: [{ type: "function", function: { name: "f", parameters: {} } }],
      tool_choice: "required",
    },
    headers: {},
    stream: true,
    wire_model: "m",
    reasoning: "",
  });
  check("cc.tool_choice=required → any", anyTools.body.params.tool_choice, { type: "any" });
  check("cc.tools 没有 type 字段", anyTools.body.params.tools[0].type, undefined);

  // ── error（ ）──────────────────────────────────
  const cases = [
    [400, { message: "bad request" }, "fatal", 400],
    [401, { message: "unauthorized" }, "token_expired", null],
    [403, { message: "forbidden" }, "token_expired", null],
    [402, { message: "payment required" }, "quota_limited", 429],
    [404, { message: "not found" }, "fatal", 404],
    [429, { message: "rate limited" }, "quota_limited", 429],
    [503, { message: "unavailable" }, "fatal", 503],
    [500, { message: "boom" }, "fatal", 502],
  ];
  for (const [status, body, kind, downstream] of cases) {
    const result = await host.call("classify", { provider: "commandcode", status, body });
    check(`cc.classify(${status}).kind`, result.kind, kind);
    if (downstream !== null) {
      check(`cc.classify(${status}).status`, result.status, downstream);
    }
  }
  // ：400 + → content_blocked
  const blocked = await host.call("classify", {
    provider: "commandcode",
    status: 400,
    body: { message: "Blocked by security policy" },
  });
  check("cc.classify(内容拦截).kind", blocked.kind, "content_blocked");
  checkTruthy(
    "cc.classify(内容拦截).message 带可执行提示",
    blocked.message.includes("系统提示词"),
    blocked.message
  );

  // ── （ 26 ）──────────────────────────────
  const modelList = await host.call("models.list", {
    provider: "commandcode",
    account: "",
  });
  check("cc.models 兜底条目数", modelList.models.length, 26);
  check("cc.models.meta.remote（没拉过远程）", modelList.meta.remote, false);
  const first = modelList.models[0];
  check("cc.models[0].id", first.id, "claude-sonnet-4-6");
  check("cc.models[0].name", first.name, "Claude Sonnet 4.6");
  check("cc.models[0].supportsToolCall", first.supportsToolCall, true);
  check("cc.models[0].supportsReasoning", first.supportsReasoning, true);
  check("cc.models[0].supportsImages", first.supportsImages, false);

  host.close();
  return host;
}

// ══════════════════════════════════════════════════════════════
// ModelScope
// ══════════════════════════════════════════════════════════════

const MS_EXPECT = {
  cn: {
    id: "modelscope",
    label: "ModelScope 国内版",
    inference: "https://api-inference.modelscope.cn/v1/chat/completions",
  },
  intl: {
    id: "modelscope-intl",
    label: "ModelScope 国际版",
    inference: "https://api-inference.modelscope.ai/v1/chat/completions",
  },
};

async function checkModelScope(exe, region, expect) {
  const host = new Host(exe, accountSecret(region.id));
  const infos = await host.init();
  const info = infos.find((item) => item.id === expect.id);
  checkTruthy(`${expect.id} 在握手里出现`, Boolean(info));
  if (!info) return host.close();

  check(`${expect.id}.label`, info.label, expect.label);
  check(`${expect.id}.capabilities.chat`, info.capabilities.chat, true);
  check(`${expect.id}.capabilities.stateful`, info.capabilities.stateful, false);
  check(`${expect.id}.capabilities.refresh`, info.capabilities.refresh, false);
  check(`${expect.id}.capabilities.model_refresh`, info.capabilities.model_refresh, true);
  check(`${expect.id}.capabilities.usage`, info.capabilities.usage, true);
  check(`${expect.id}.capabilities.checkin`, info.capabilities.checkin, true);
  check(`${expect.id}.capabilities.web_login`, info.capabilities.web_login, false);

  // ── chat：URL / / include_usage ─────────────────
  const plan = await host.call("chat", {
    provider: expect.id,
    account: `${expect.id}-1`,
    proxy: null,
    body: { messages: [{ role: "user", content: "hi" }], model: "Qwen/Qwen3.8-Flash-Next" },
    headers: {},
    stream: true,
    wire_model: "Qwen/Qwen3.8-Flash-Next",
    reasoning: "",
  });
  check(`${expect.id}.url`, plan.url, expect.inference);
  check(`${expect.id}.upstream`, plan.upstream, "chat");
  const headers = headerMap(plan.headers);
  check(
    `${expect.id}.Authorization`,
    headers.Authorization,
    "Bearer ms-parity-token-abcdefgh"
  );
  check(`${expect.id}.Accept`, headers.Accept, "text/event-stream");
  // ⚠️ this is 1 （ usage as 0）
  check(
    `${expect.id}.stream_options.include_usage`,
    plan.body.stream_options?.include_usage,
    true
  );
  check(
    `${expect.id}.model 原样透传`,
    plan.body.model,
    "Qwen/Qwen3.8-Flash-Next"
  );
  check(`${expect.id}.messages 原样透传`, plan.body.messages, [
    { role: "user", content: "hi" },
  ]);

  // client stream_options when ， include_usage、 not for 
  const merged = await host.call("chat", {
    provider: expect.id,
    account: `${expect.id}-1`,
    proxy: null,
    body: { messages: [], stream_options: { custom_flag: 1 } },
    headers: {},
    stream: true,
    wire_model: "m",
    reasoning: "",
  });
  check(`${expect.id}.stream_options 其余键保留`, merged.body.stream_options.custom_flag, 1);
  check(
    `${expect.id}.stream_options.include_usage 仍被置真`,
    merged.body.stream_options.include_usage,
    true
  );

  // ── error （ ）────────────────
  // ① ：429 + insufficient balance → QuotaLimited + reset_at empty
  const balance = await host.call("classify", {
    provider: expect.id,
    status: 429,
    body: { message: "insufficient balance" },
  });
  check(`${expect.id}.classify(魔粒耗尽).kind`, balance.kind, "quota_limited");
  checkTruthy(
    `${expect.id}.classify(魔粒耗尽).reset_at 必须给出来`,
    typeof balance.reset_at === "number" && balance.reset_at > Date.now(),
    balance.reset_at
  );

  // ② ：403 + → QuotaLimited（** not is ** TokenExpired）
  const unverified = await host.call("classify", {
    provider: expect.id,
    status: 403,
    body: {
      message:
        "For account security, please complete your profile via email verification. " +
        "ModelScope Magicubes is only available to verified accounts.",
    },
  });
  check(`${expect.id}.classify(未验证邮箱).kind`, unverified.kind, "quota_limited");
  checkTruthy(
    `${expect.id}.classify(未验证邮箱).reset_at`,
    typeof unverified.reset_at === "number",
    unverified.reset_at
  );

  // ③ 403 → TokenExpired
  const forbidden = await host.call("classify", {
    provider: expect.id,
    status: 403,
    body: { message: "access forbidden" },
  });
  check(`${expect.id}.classify(裸 403).kind`, forbidden.kind, "token_expired");

  // ④ 429（）→ QuotaLimited and ** not ** to reset_at
  const rateLimit = await host.call("classify", {
    provider: expect.id,
    status: 429,
    body: { message: "rate limit exceeded" },
  });
  check(`${expect.id}.classify(速率窗口).kind`, rateLimit.kind, "quota_limited");
  checkTruthy(
    `${expect.id}.classify(速率窗口).reset_at 应为 None`,
    rateLimit.reset_at === null || rateLimit.reset_at === undefined,
    rateLimit.reset_at
  );

  // ⑤ insufficient balance 429 when ** not ** that 
  const wrongStatus = await host.call("classify", {
    provider: expect.id,
    status: 400,
    body: { message: "insufficient balance" },
  });
  checkTruthy(
    `${expect.id}.classify(400+insufficient).kind 不是 quota_limited`,
    wrongStatus.kind === "fatal" || wrongStatus.kind === "content_blocked",
    wrongStatus.kind
  );

  // ── ： no has ， no is empty ────────────────
  const modelList = await host.call("models.list", {
    provider: expect.id,
    account: "",
  });
  check(`${expect.id}.models 初始为空`, modelList.models.length, 0);
  check(`${expect.id}.models.meta.remote`, modelList.meta.remote, false);

  host.close();
}

// ══════════════════════════════════════════════════════════════
// 
// ══════════════════════════════════════════════════════════════

async function main() {
  const exe = pluginExe();
  if (!exe) {
    console.log("跳过：插件二进制不存在（先在 agent2api-providers 里 cargo build）");
    return;
  }
  console.log(`插件: ${exe}`);

  await checkCommandCode(exe);
  await checkModelScope(exe, "cn", MS_EXPECT.cn);
  await checkModelScope(exe, "intl", MS_EXPECT.intl);

  console.log(`\n${checks} 项检查，${failures} 项失败`);
  if (failures > 0) process.exit(1);
  console.log("✓ parity 通过：插件实现与内置实现行为一致");
}

main().catch((error) => {
  console.error("脚本异常:", error);
  process.exit(1);
});
