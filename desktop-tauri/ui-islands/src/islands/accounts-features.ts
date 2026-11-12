/**
 * **parameter**：、、 and 1 。
 *
 * ── Why 1 file ──────────────────────────────────────
 * this parameter in `accounts-domain.ts` （ that file already 、、
 * ）， is 1 ：** this 1 **。 need is 
 * —— before is 1 before ， in is in when （ 
 * `AccountFeatures`）。 after this 「 + 」 this 1 。
 *
 * ── ────────────────────────────────────────────────
 * 1. ****（`/api/session` `accountFeatures`）—— 
 * that this ；
 * 2. **`PROVIDER_FEATURES`** —— old not when ， in for ；
 * 3. **`GENERIC_FEATURES`** —— no when 。
 *
 * etc all， 2 can （ before 1 id）。
 */

import { shared, type LimiterRule } from './accounts-shared'

export type ProviderFeatures = {
  /** this 1 has no has / */
  usage: boolean
  /** has no has / （ is after 、valid ） */
  edition: boolean
  /** in （ by 1 has ：uid / userId / account） */
  identifier: readonly string[]
  /** valid in （expiresAt / tokenExpiresAt / tokenExpireAt） */
  expiry: readonly string[]
  /**
   * **valid**（ no has to when ）：valid「valid」。
   *
   * and `expiry: []` is **** not is ：empty「 this 1 no has 
   * can 」，「」；`longLived: true` is 
   * 「 not will to ， not valid」。 `tokenExpiryOf` return 0。
   *
   * up ， can ：
   * 1. not 、 access token by —— Antigravity 
   * Google refresh token、 refresh token；
   * 2. no has valid —— ModelScope `ms-` 、Command Code 
   *      `user_` key。
   * ** 「none、 can 」 **（Loomy 14 / MonkeyCode 30 /
   * CatPaw will Cookie）： has ， is after not 。
   */
  longLived?: boolean
  /** 「 this 」（Qoder / AutoClaw / Accio）。：
   * displayNameOf（**** ， as ） and 
   * models-fetch-modal accountLabel（ down ） */
  emailAsName?: boolean
  /**
   * has no has 「」 this （ has ZCode ）。
   *
   * ** not this 1 **： that 「」 by already ， 1 in to in 
   * 「」（ is after `plans` ， `api::checkin_center`）。
   * is （`AccountFeatures.claim`）， when can this 1 
   * is can 。
   */
  claim?: boolean
  /**
   * has no has 「 use （ up ）」 this set（ has ZCode ）。
   *
   * is 「 this request up 」：（ ，
   * OpenAI ） is （ when ，`zcode.z.ai`
   * Anthropic ）。 is ****，「 already to 」 this 
   * by its in 1 to —— is set not is （ after 
   * `providers::zcode::plan` module header）。
   */
  planChannel?: boolean
  /**
   * need not need use 「 + 」 （ has CodeArts）。
   *
   * total， and in ； by can this 。
   */
  usageDetail?: boolean
  /**
   * need not need 「」（： / already use / remaining +
   * → to ）—— already has WorkBuddy 、Trae、OfficeAce、AutoClaw 、
   * ZCode 、Qoder 、Loomy、。
   *
   * and `usageDetail` is ** not **： that will this 「 + 
   * by 」 ； this （ 「available N」、 +
   * + x / y） already use that ， is ** 1 ** not is 
   * 1 —— that will 。
   *
   * by can not is ： after `credits[]` is this （ and after 
   * this ） **** —— 1 has ， not is 1 empty。
   */
  creditDetail?: boolean
  /**
   * `credits[]` ** to when **（`expireAt`）， can by 「 to 」。
   *
   * and `creditDetail` is as is ： has ≠ has to 。
   * WorkBuddy / Trae / OfficeAce `expireAt`； to 1 、
   * ZCode total no has 、Qoder no has when 、Loomy /
   * ModelScope is 。
   */
  creditExpiry?: boolean
  /**
   * this 1 「 and limit」 default（>0 = ** no has **「 not 」 this 1 ）。
   * CodeArts 3 is up （ HTTP 400， and that is 、 not ），
   * that 1 `0` 「 by default 3」 not is 「 not and 」； after 1 in 
   * （ no 3、 by 3 ）， up for up 。
   */
  concurrencyDefault?: number
  /**
   * up no has `limiters` no has `lowBalance` when limit then 。
   *
   * by （`AccountFeatures::default_limiters`）， in when 
   * use ； use then 。`undefined` = no → down 
   * `PROVIDER_FEATURES` 。
   */
  defaultLimiters?: readonly LimiterRule[]
}

/**
 * provider can —— ** use **，（ `providerFeatures`）。
 *
 * ── Why in is ──────────────────────────────────────
 * this is 1 ， is 「 1 」must before 。 this parameter
 * is up （ in 、valid in 、 has no has ），
 * ， already to by when （`AccountFeatures`）。
 *
 * is as ****： old not ，must can for 。
 * is 「 → → GENERIC_FEATURES」。 etc all，
 * can （ before `accountFeatures` down 1 id）。
 *
 * Why is 「 by provider 」 not is in if： （ by 、
 * after 、） need 1 ——「 this has no has this 」。
 * in if ， 1 need 1 file， 1 not 、 1 by 。
 *
 * （ to ** not in ** this ： already none to by ， to in and can in 
 * after —— `core::auto_checkin` CHECKIN_PROVIDERS and `billing::checkin::
 * supports_checkin` is 1 ， to in that module header。）
 *
 * usage is true（ already to ）， by ； before 
 * 「 this 1 has no has this 」。CatPaw need 1 will （token2），
 * no config when after return can 「not configured」、 in —— by ，
 * use has 「config」 。
 */
const PROVIDER_FEATURES: Record<string, ProviderFeatures> = {
  // `creditDetail: true` —— 「」： after `query_usage` in has 
  // `credits[]`（ / already use / remaining / / to ，
  // providers::workbuddy::adapter query_usage）。 can 1 ，
  // need （ 1 that 1 GENERIC_FEATURES）。
  // enterprise no has 、 after not `credits`， by empty。
  workbuddy: { usage: true, creditDetail: true, creditExpiry: true, edition: true, identifier: ['uid'], expiry: ['expiresAt'] },
  // WorkBuddy （ after ， providers::workbuddy::region）：
  // `edition: false` is as 「WorkBuddy 」， 1 will 
  // to 「WorkBuddy 」（`editionSuffix` has 「 already not 」
  // ， this false ， and `zcode-intl` ）。
  'workbuddy-intl': { usage: true, creditDetail: true, creditExpiry: true, edition: false, identifier: ['uid'], expiry: ['expiresAt'] },
  // `creditDetail: true` —— （daily / will / /
  // ） 1 ： up to 1 ， no has / already use / to （ not ），
  // is 「 in 」。
  raccoon: { usage: true, creditDetail: true, edition: false, identifier: ['userId'], expiry: ['tokenExpiresAt'] },
  catpaw: { usage: true, edition: false, identifier: ['uid'], expiry: ['tokenExpiresAt'] },
  // AutoClaw can 1 ， in ；must —— 1 ，
  // that 1 will GENERIC_FEATURES（： by 、empty）
  // `creditDetail: true` —— 1 ， 1 「 will 」（ up to is 
  // for when ，， not when ）。 need up 。
  autoclaw: { usage: true, creditDetail: true, edition: false, identifier: ['userId'], expiry: ['tokenExpiresAt'] },
  'autoclaw-intl': { usage: true, creditDetail: true, edition: false, identifier: ['userId'], expiry: ['tokenExpiresAt'], emailAsName: true },
  // Qoder can 1 （ after is provider， by id ，
  // 1 will GENERIC_FEATURES）；`edition` in —— is 
  // 1 ， edition 
  // `creditDetail: true` —— 1 （ / / ），
  // 、 no has when ； need up 。
  qoder: { usage: true, creditDetail: true, edition: true, identifier: ['userId'], expiry: ['expiresAt'], emailAsName: true },
  'qoder-intl': { usage: true, creditDetail: true, edition: true, identifier: ['userId'], expiry: ['expiresAt'], emailAsName: true },
  // ： and Qoder 1 （ / / /
  // ）。 already 1 ， this is old 。
  qwenwork: { usage: true, creditDetail: true, edition: true, identifier: ['userId'], expiry: ['expiresAt'], emailAsName: true },
  'qwenwork-intl': { usage: true, creditDetail: true, edition: true, identifier: ['userId'], expiry: ['expiresAt'], emailAsName: true },
  // Cline ： 1 up by provider， 1 （
  // providers::cline::models）。 by id ， 1 will 1 
  'cline-free': { usage: true, edition: false, identifier: ['account'], expiry: ['expiresAt'] },
  'cline-pass': { usage: true, edition: false, identifier: ['account'], expiry: ['expiresAt'] },
  // Accio ： can （ up to usage）、 has 
  accio: { usage: true, edition: true, identifier: ['userId'], expiry: ['expiresAt'], emailAsName: true },
  'accio-cn': { usage: true, edition: true, identifier: ['userId'], expiry: ['expiresAt'], emailAsName: true },
  // ZCode ： is 「 when 」（claim ）。
  // `usage: true` for providers::zcode::balance —— is 1 ****：
  // has JWT when billing `/zcode-plan/billing/balance`（ + to ），
  // otherwise（ or JWT when ） use accessToken 
  // （ N when / remaining， providers::zcode::monitor）。
  // no has when after can 「not configured」， in not is 1 ，
  // this by 。
  // `claim: true` is can 。** already not ** use by （ that 「」
  // already ， in to in 「」）； this old ， and 1 。
  // ZCode is 1 new （plan_id ）， after that 1 
  // 「 already N 」、。
  // expiry expiresAt is to add_zcode_account （ when need expiration time）
  // `creditDetail: true` —— 1 （total / already use / remaining， when ）；
  // （ that ） has `remainingPercent`、 ，
  // （ zcode::balance credits_of）—— that 1 and in 1 not 。 need 。
  zcode: { usage: true, creditDetail: true, claim: true, planChannel: true, edition: true, identifier: ['userId'], expiry: ['expiresAt'] },
  'zcode-intl': { usage: true, creditDetail: true, claim: true, planChannel: true, edition: true, identifier: ['userId'], expiry: ['expiresAt'] },
  // CodeArts（ as AI ）。 has ，：
  // `usage: true` —— statisticsreturntotal and ， not to when not 。
  // `usageDetail: true` —— （ + ）：
  // 1 not down ，「available N 」 is total， not not 。
  // 「」 not in can ： already to 「 to in 」（api::checkin_center by 
  // provider ，ui/codearts-welfare.js use ）， not has 
  // this by —— after 1 ， not 。
  // `edition: false` —— no has /：region in cn-north-4 and must and token
  // 1 ， not is use optional；`login_type`（WEB/IDE） not is ， this 1 。
  // `expiry: ['expiresAt']` —— when 1 when to ， this 1 for ** is need **。
  codearts: {
    usage: true, usageDetail: true, edition: false,
    identifier: ['userId'], expiry: ['expiresAt'],
    concurrencyDefault: 3,
  },
  // Trae（ has SOLO that 1 ）：`edition: false` is —— SOLO and is ****
  // not is 1 ， will when provider id， not 。
  // `usage: true` for providers::trae::usage（ up ：ide_user_ent_usage /
  // + ide_user_pay_status request and SOLO and ）。
  // `creditDetail: true` —— 1 ent_usage to **** 
  // `start_time` / `expire_time`（ → to ）， after `credits[]`
  // （ providers::trae::usage credit_rows）。 to will by ，
  // 1 has this up 。
  trae: { usage: true, creditDetail: true, creditExpiry: true, edition: false, identifier: ['uid'], expiry: ['expiresAt'] },
  // Loomy（）。 has ：
  // `usage: true` for providers::loomy::balance —— is ****（ +
  // daily， `GET /api/v2/points/records` need ）， up 。
  // `edition: false` —— 1 （）， no has /。
  // `expiry: ['']`（** not longLived**）—— up response has 
  // `{session, userid, phone}`，** no has to when **。 before by 「 when + 14 」
  // 1 `expiresAt` valid， use 
  // 「expired」， up （ to after can to 、 can ）。
  // in not 、 not ： up `020002` / `100002`， that when to 
  // `needsRelogin`，「 need need new 」。 not `longLived` ——
  // session will ， is when not can 。
  // `creditDetail: true` —— 1 （ / daily）， after `daily`
  // → 「daily」 not is to （ to 0 not 「 already use 」，）。
  loomy: { usage: true, creditDetail: true, edition: false, identifier: ['userId'], expiry: [''] },
  // KukuAI（ AI）。 has ：
  // `usage: true` —— /（GET /bizapi/gfpro/getgfvipremain 
  // totalPoint， after providers::kuku::balance）。
  // `edition: false` —— 1 （ Cookie / / ），
  // no has 。
  // `identifier: ['uid']` —— uid（userreport uk）。
  // `expiry: ['']`（** not longLived**）—— BDUSS is Cookie， no has can 
  // to when ；** not can **「valid」： up when can can （ 
  // 「BDUSS invalid」 is 1 ）， and no has ，
  // can 。 this 1 for is 「 no has can 」， not is 「 not will 」。
  kuku: { usage: true, edition: false, identifier: ['uid'], expiry: [''] },
  // OfficeAce（ as / OfficeClaw）。 has ：
  // `usage: true` —— / need ****（V11-HMAC-SHA256 `/v1/subscription`），
  // that already （`officeace/balance.rs` + `query_usage`）， by and 
  // 。⚠️ Basic that 1 for 、 no has not to ，
  // server 400 `usage_not_configured`， by 「not configured」（ not is ）。
  // `edition: false` —— 1 （ OAuth / Basic ），
  // no has 。
  // `expiry: ['expiresAt']` —— when 2 when to ； Basic
  // when no has this ， not 「valid」（ that 1 not ）。
  // `identifier: ['id']` —— is ** id**（`officeace-<hash>`，
  // by + app key ）， not is ： up `modelAppKey`
  // will key in up （`identifierOf` is ****， not is ）。
  // `creditDetail: true` —— quota + 1 （expired 
  // —— and 「 already use 」 1 ）， 1 （「」，
  // and 「」 not ）。
  officeace: { usage: true, creditDetail: true, creditExpiry: true, edition: false, identifier: ['id'], expiry: ['expiresAt'] },
  // Antigravity（Google AI IDE）。 has ：
  // `usage: true` —— is ** × **（5 when / ） remaining，
  // after providers::antigravity::balance（retrieveUserQuotaSummary）；
  // up to 、 no has for ，（`remainingPercent` +
  // `balanceView`）。** this **（ `windows[]`， and OpenCode Go
  // 1 ； 1 ）—— Gemini + Claude/GPT ， after is 
  // issue #288 need that （ before 1 ， etc that ）。
  // and when ，（Pro / Ultra / Free）。
  // `edition: false` —— 1 （Google OAuth / ）， no has 。
  // `identifier: ['userId']` —— is email（ account_store::antigravity_accounts module header）。
  // `expiry: ['']` + `longLived: true` —— refresh token not （Google 
  // refresh token no has ， in / 6 not use / up 1 
  // client 100 limit when ），access token 1 when 、 by before 
  // 15 minutes（ providers::antigravity::credentials module header）。
  antigravity: { usage: true, edition: false, identifier: ['userId'], expiry: [''], longLived: true },
  // （chatglm.cn）。 has ：
  // `usage: true` —— is ** will **（`member-api` `current_score`，
  // `payment-api` will ）， after `providers::chatglm::balance`。
  // ⚠️ ** no has **： that for 1 401， after 400
  // `usage_not_configured`（「 no has 」）， by 「not configured」
  // not is —— and OfficeAce Basic that 1 1 。
  // `creditDetail: true` —— （ will + will ），。
  // `edition: false` —— 1 （ is 1 Z.AI/ZCode， already by provider
  // ）， no has /。
  // `identifier: ['uid']` —— uid（JWT `uid` ； has ）。
  // `expiry: ['']` + `longLived: true` —— refresh token by 
  // （`user/refresh` available）， then by use ** 1 ** new 
  // `guest/access` ，「 when to 」 for use no has 。
  // ⚠️ has 7 ， after is by up （
  // not is can ）； this is 「 not need need use valid」， not is 
  // 「 not 」。
  chatglm: { usage: true, creditDetail: true, edition: false, identifier: ['uid'], expiry: [''], longLived: true },
  // ModelScope（） （`modelscope` / `modelscope-intl`）： 1 
  // —— 、、 to 1 ， in （ that is and 
  // ， this not ）。
  // `usage: true` —— is （`available_balance`），「 already use use 」。
  // `creditDetail: true` ：available / / already use is snapshot ，
  // not as 、 to or remaining。
  // `edition: false` —— by provider id （ provider、 ），
  // not need need 1 。
  // `identifier: ['userId']` —— is up HavanaId
  // （ account_store::modelscope_accounts module header）。
  // `expiry: ['']` + `longLived: true` —— `ms-` is valid API key
  // （「session is api key …… not 」， no has 
  // refresh / renew ）， no has to when can 。
  modelscope: { usage: true, creditDetail: true, edition: false, identifier: ['userId'], expiry: [''], longLived: true },
  'modelscope-intl': { usage: true, creditDetail: true, edition: false, identifier: ['userId'], expiry: [''], longLived: true },
  // AtomCode（AtomGit / GitCode）。 has ：
  // `usage: true` —— is `status-v2` ** use **（ 200 /
  // 5 when ）， already `query_usage`（`providers/atomcode/usage.rs`），
  // by and 。
  // `edition: false` —— 1 （ no has ）。
  // `identifier: ['userId']` —— AtomGit use id（ when in `userId` ）。
  // `expiry: ['expiresAt']` —— ** not is JWT**， when by `createdAt + expiresIn`
  // after （ `providers/atomcode/credentials.rs` module header）。
  atomcode: { usage: true, edition: false, identifier: ['userId'], expiry: ['expiresAt'] },
  // Command Code（`api.commandcode.ai`）。 before ****， is 
  // GENERIC_FEATURES： by not （ after `supports_usage` is false，
  // that 1 is for up ）、 by `userId` empty（ no has 
  // userId， id is key ），valid will 1 not in 
  // `tokenExpiresAt`。
  // `usage: false` —— 本家**没有余额查询**：`/alpha/billing/*` in when 
  // use （ providers/commandcode/adapter.rs module header）， this 
  // （ and after `supports_usage` default false 1 ）。
  // `identifier: ['id']` —— no has can use ： id by key SHA-256 before 
  // 12 （`commandcode-<hash>`， account_store::commandcode_accounts
  // module header）， this id ， and officeace 1 。
  // `expiry: ['']` + `longLived: true` —— API Key， up no has refresh
  // 、key no has to when （ credentials.rs module header）。
  commandcode: { usage: false, edition: false, identifier: ['id'], expiry: [''], longLived: true },
  // LobsterAI（ has 「 has 」）。 has ：
  // `usage: true` —— is ****（`GET /api/user/profile-summary`， after 
  // `providers::lobsterai::endpoints` `PROFILE_SUMMARY_PATH`）， by 
  // and 。
  // `creditDetail: true` —— 1 profile-summary ****（ 
  // `creditItems[]` → / already use / remaining / to ， Go `credits.go` 
  // `creditPackage`）， is `credits[]` that 。 and ZCode not ，
  // this has for ， can 。⚠️ this 1 is ****： after 
  // `query_usage` not `credits` when not （accounts-panels 
  // `creditRows.length` ）， in is ； after if not ，
  // this 1 can ，。
  // `edition: false` —— 1 （`lobsterai.youdao.com`）， no has / ，
  // not after （ and orcaterm / atomcode 1 ）。
  // `identifier: ['userId']` —— `userId`： after when is 
  // `userId → uid → `， id by （`lobsterai-<identity>`，
  // `account_store::lobsterai_accounts`）， up and id after for up 。
  // `expiry: ['expiresAt']` —— is ms `expiresAt`（ > createdAt +
  // expiresIn > JWT `exp`）， and atomcode 1 。
  lobsterai: { usage: true, creditDetail: true, edition: false, identifier: ['userId'], expiry: ['expiresAt'] },
  // MonkeyCode（） 。 before ****（ after `PROVIDERS` has 、 no has ），
  // is GENERIC_FEATURES： 1 「—」（`supportsUsage` false），
  // by `userId` empty，valid 1 not in `tokenExpiresAt`。
  // `usage: false` —— ** no has **： `supports_usage` default false
  // （ `providers/monkeycode/adapter.rs` module header）， this is 「 up no has this 」，
  // not is 「 no 」。 this ， and after 1 。
  // `identifier: ['userId']` —— has `userId`（
  //   `account_store::monkeycode_accounts::to_monkeycode_public_account`）。
  // `expiry: ['expiresAt']` —— to is `expiresAt`（session cookie 30 limit，
  // `monkeycode/credentials.rs`），** not is ** `tokenExpiresAt`。
  // `edition: false` —— already by provider id （、 ）， 1 after 
  // will to 「MonkeyCode 」。 need ， 1 。
  monkeycode: { usage: false, edition: false, identifier: ['userId'], expiry: ['expiresAt'] },
  'monkeycode-intl': { usage: false, edition: false, identifier: ['userId'], expiry: ['expiresAt'] },
  // OrcaTerm（）。 before **** —— after `supports_usage` and 
  // `query_usage` （`DescribeUserBillingStatus` AI quota，
  // `providers/orcaterm/balance.rs`），README supports ✓， before 
  // not to ： by 、、 1 。
  // `usage: true` —— has （`available` + `unit: ""` 1 ）。
  // `identifier: ['uid']` —— to is `uid`（`orcaterm_accounts` ）。
  // `expiry: ['tokenExpireAt']` —— ⚠ and not （ not is `tokenExpiresAt`）：
  // `tokenExpireAt`， will valid 1 「—」。
  // `edition: false` —— 1 （ / ）， no has 。
  // no has `creditDetail`：`query_usage` to `available` and subscription，
  // not `credits[]`， will 1 empty。
  orcaterm: { usage: true, edition: false, identifier: ['uid'], expiry: ['tokenExpireAt'] },
}

/**
 * provider can ： not / —— this is provider has 
 * ，unknown not this has 。 userId， not to when 1 。
 */
const GENERIC_FEATURES: ProviderFeatures = {
  usage: false, edition: false, identifier: ['userId'], expiry: ['tokenExpiresAt'],
  emailAsName: false,
}

/**
 * parameter（`provider id → `）， by `/api/session` 。
 *
 * ── Why is 1 not is React ─────────────────
 * in （`identifierOf` / `tokenExpiryOf` /
 * `supportsUsage` this app.js、report.js、models-fetch-modal 
 * use ）， no has can state。 `/api/session` new 1 ——
 * 1 new Map ， not 。
 *
 * ── Whyempty is ────────────────────────────────────
 * （`/api/session` no ）、 old after （ not this ）、 no ——
 * is empty， when `PROVIDER_FEATURES` ， as and before 1 。
 */
let declaredFeatures: ReadonlyMap<string, ProviderFeatures> = new Map()
/** up 1 1 when use for （ use ：`/api/session` 1 1 ） */
let declaredSource: unknown = null

/**
 * down after （`/api/session` `accountFeatures`）。
 *
 * is `AccountFeatures` serde （snake_case）。 this 1 
 * 1 ： by `GENERIC_FEATURES` ， （ old 
 * after / config）， this not to 。
 */
export function adoptDeclaredFeatures(raw: unknown): void {
  if (!raw || typeof raw !== 'object' || Array.isArray(raw)) return
  const next = new Map<string, ProviderFeatures>()
  for (const [id, value] of Object.entries(raw as Record<string, unknown>)) {
    if (!id || !value || typeof value !== 'object' || Array.isArray(value)) continue
    next.set(id, normalizeDeclared(value as Record<string, unknown>))
  }
  declaredFeatures = next
}

/**
 * current，** `/api/session` update**。
 *
 * ── Why in this not is app.js ─────────────────
 * in 、、， not in React for up ；
 * 「 1 」 is not can can （ 1 is ）。
 * this by use ：`/api/session` new 1 1 new for ，「 use 
 * 1 」 new ， not will in path up Map。
 */
function declaredTable(): ReadonlyMap<string, ProviderFeatures> {
  const raw = shared().wbApp?.getState?.()?.accountFeatures
  if (raw !== declaredSource) {
    declaredSource = raw
    adoptDeclaredFeatures(raw)
  }
  return declaredFeatures
}

/** current down （ / use ； `providerFeatures`） */
export function declaredAccountFeatures(): ReadonlyMap<string, ProviderFeatures> {
  return declaredTable()
}

/** 1 （ by GENERIC ）。 */
function normalizeDeclared(raw: Record<string, unknown>): ProviderFeatures {
  return {
    usage: raw.usage === true,
    edition: raw.edition === true,
    identifier: keyList(raw.identifier, GENERIC_FEATURES.identifier),
    expiry: keyList(raw.expiry, GENERIC_FEATURES.expiry),
    longLived: raw.long_lived === true,
    emailAsName: raw.email_as_name === true,
    claim: raw.claim === true,
    planChannel: raw.plan_channel === true,
    usageDetail: raw.usage_detail === true,
    creditDetail: raw.credit_detail === true,
    creditExpiry: raw.credit_expiry === true,
    concurrencyDefault: Number(raw.concurrency_default) > 0 ? Number(raw.concurrency_default) : undefined,
    defaultLimiters: declaredLimiters(raw.default_limiters),
  }
}

/** ：， 1 ， its 。 */
function keyList(value: unknown, fallback: readonly string[]): readonly string[] {
  if (typeof value === 'string') return value ? [value] : []
  if (!Array.isArray(value)) return fallback
  const keys = value.filter((item): item is string => typeof item === 'string' && item.length > 0)
  return keys.length ? keys : []
}

/** limit then （ not for ）。 */
function declaredLimiters(value: unknown): readonly LimiterRule[] | undefined {
  if (!Array.isArray(value)) return undefined
  const rules: LimiterRule[] = []
  for (const item of value) {
    if (!item || typeof item !== 'object' || Array.isArray(item)) continue
    const rule = item as Record<string, unknown>
    const threshold = Number(rule.threshold)
    if (!Number.isFinite(threshold) || threshold <= 0) continue
    const type = rule.kind === 'token' ? 'token' : rule.kind === 'balance' ? 'balance' : null
    if (!type) continue
    const action = rule.action === 'disable' ? 'disable' : 'skip'
    const period = Number(rule.period_seconds)
    const fixed = type === 'token' && Number.isFinite(period) && period > 0
    rules.push({
      type,
      action,
      threshold,
      enabled: true,
      ...(fixed ? { period, reset: 'fixed' as const } : {}),
      ...(type === 'token' && !fixed ? { reset: 'daily' as const } : {}),
    })
  }
  return rules
}

/**
 * provider id → parameter。：** → → use **。
 *
 * ， is as that this ； in 
 * no （ old ） when ， in can for 。
 */
export function providerFeatures(providerId: string | undefined): ProviderFeatures {
  if (!providerId) return GENERIC_FEATURES
  const declared = declaredTable().get(providerId)
  if (declared) return declared
  return PROVIDER_FEATURES[providerId] || GENERIC_FEATURES
}

