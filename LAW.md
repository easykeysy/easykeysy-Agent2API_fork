# LAW.md — Fork 宪法（只增不改的条文）

> 本文是 fork 的宪法层。它**引用** `AGENTS.md`（日常政策，可随工程节奏修订），并以强化条文锁死不可退让的底线。
> 冲突时：本文 > `AGENTS.md`。任何修改本文的动作，本身必须先走 `AGENTS.md`「两条红线」流程——先问人，不得自行推进。
>
> 条文格式：`LAW-N` = 宪法条文（不可删改）；`LAW-N.R` = 强化细则（解释与执行口径，随实现可修订，但修订须注明所引条文号）。

---

## LAW-1 署名与 AI 落款
*（引自 AGENTS.md「附加红线（署名与 AI 落款）」）*

1. 本 fork 的所有提交信息**不得**包含 AI 助手落款（如 `🤖 Generated with Codebuff`、`Co-Authored-By: ... <noreply@...>` 等任何 harness/agent 署名行）。
2. 提交者署名为 `easykeysy`（邮箱 `easykeysy@users.noreply.github.com`），正文里不出现 "Generated with"、"Co-Authored-By"、机器人 emoji。
3. 若 CI / Release 流程脚本要写 AI 落款，须先删除再推。

**LAW-1.R 自检**：每次推之前跑
`git log --format=%B <range> | grep -c "Generated with\|Co-Authored-By\|🤖"` 必须为 `0`。
（genesis 提交 `3ed3dd0` 已按此自检通过一次。）

## LAW-2 发版双确认
*（引自 AGENTS.md「两条红线」第 1 条）*

改版本号、推 `main` / `beta`、打 tag、挂 Release 之前，先把版本号与「这次要发什么」讲清楚，等用户确认；**未经确认不得推任何分支或 tag** —— tag 一旦推出去就会触发构建、产出公开的安装包与镜像，收不回来。

## LAW-3 更新日志双过目
*（引自 AGENTS.md「两条红线」第 2 条）*

写好的日志（= 发布提交信息 = Release 正文）必须先给用户逐条过目；按反馈改完**再确认一次**才能提交。

## LAW-4 单提交 Genesis（不可再拆）
*（引自 AGENTS.md「Genesis 分支」；SHA 以实际 git 状态为准，强化见 LAW-4.R）*

- `main-clean`（本地 `genesis`）是 fork 开发线的起点，由 **1 个**提交构成，直接以 `e8bed73`（upstream v3.4.2 tip）为父。
- 该提交：42 文件，+2698/−404，0 删除；含 4 个纯新增文件 + 38 个 upstream 文件加接线 + `AGENTS.md` 本身。
- 标签 `v3.4.2-fork.1` 固定钉在该提交上。旧 `main`（`85eba93`）永久冻结，仅当考古。

**LAW-4.R 当前事实（2026-11-15 核验）**：genesis tip = **`3ed3dd0`**（父 `e8bed73`），tag `v3.4.2-fork.1` 在其上，`main-clean` 已推至 origin。AGENTS.md 中写 `652879a` 的行是早期草稿，以 `git rev-list` 实测为准——**实测优先于文档**。

## LAW-5 功能批次：纯增量 / 零扣减 / 零重写
*（引自 AGENTS.md「功能批次规则」）*

每个 PR 到 `main-clean` 必须：
1. **纯增量**——只新增文件或在 upstream 文件里加接线行（一个 `mod`、一个 `pub use`、一个守卫 `if` 分支）。
2. **零扣减**——不许删任何 upstream 逻辑行。
3. **零重写**——不许在 upstream 逻辑里改写现有代码；fork 专有逻辑放新文件。
4. **版本号同步**——5 个版本文件一起 bump。
5. **CI 绿**——双产物（Windows NSIS + Linux AppImage）全过。

验证通过：`main-clean` 合并/快进 `main` → 删 `main-clean` → 在 `main` 上打 `v3.4.2-fork.N`（N 递增）。**一次合并 = 一个功能批次。**

## LAW-6 只 Spin-off，禁止 hybrid 文件编辑
*（引自 AGENTS.md「Fork 贡献政策」）*

**红线：永远不要在原文件里加入 fork 专有逻辑来改 upstream 文件。**
upstream 文件本体（除接线行 + 5 个版本文件外）必须与 upstream 保持字节级一致。fork 改动一律以新文件落盘。

经典反例：`44e982f`（注释翻写 733 文件）——本 fork 已为它付过一次完整 cherry-pick 重建的代价。**不许重演。**

拿不准时：问 fork 维护者，再碰那个 upstream 文件。热路径 3-way merge 的代价是几小时冲突；问一句的代价是一条消息。

## LAW-7 反代问题先搜后查
*（引自 AGENTS.md「反代提供商问题排查」）*

收到反代提供商问题反馈，**优先网络搜索同类项目的 Issue / PR / 修复记录**，先核对已有方案的适用性；未找到可靠方案才做针对性本地排查。不默认从零自查。

## LAW-8 文档六份同步
*（引自 AGENTS.md「文档落点」）*

安装/部署/升级/排错只在 `docs/install.md`（及 5 份译文）维护；根 `README.md` 为英文，简体中文在 `docs/README.zh-CN.md`。产物名、端口、环境变量、安装脚本参数有改动时，**六份一起改**。

---

## LAW-9 跨版本复用契约（今日新增，强化条文）

*（依据 2026-11-15 在 `pdebian-GA`（用户 `hb`）实地核验；机制引自 upstream `paths.rs` / `db/mod.rs` / `kv` 存储，均为未改动 upstream 代码）*

未来任何 fork 版本，只要遵守 LAW-1 至 LAW-8，在**同一台机器**上必须自动认知并复用既有配置，不丢失 GW 密钥与账号：

### LAW-9.R1 配置目录解析不变
- 机制：`default_config_dir()` 优先探测 `/mnt/ramdisk/Agent2API`（WSL 写探针成功才用）；探测失败或 ramdisk 不存在时，**回退 `$HOME/.agent2api`**。
- 实证（`pdebian-GA`）：`/mnt/ramdisk/Agent2API` 存在但属主为 `hb`、父目录 root-owned → 非 root 写探针失败 → **实际落在 `/home/hb/.agent2api/`**（持久）。
- 条文：**任何 fork 版本不得改动此解析顺序或回退逻辑。** 这是「机器上已有的配置会被新版本自动找到」的唯一结构性保证。

### LAW-9.R2 密钥与状态存储不变
- GW API keys、账号、配额等全部存于 SQLite `agent2api.db` 的 `kv` 表（key 如 `apiKeys`，357B 实证）与 `accounts` 表；加密密钥在 `account.key` 与 db 同目录。**不在 `config.json`**。
- 条文：不得将密钥改存 `config.json` 或任何非 `kv` 位置；不得改 `account.key` 的派生/存储方式。

### LAW-9.R3 Schema 只升不降（按机器分别判定）
- 条文：fork 版本的 `SCHEMA_VERSION` 必须 **≥ 目标机器上实际 db 的 `user_version`**。upstream `migrate()` 对「db 比代码新」做**拒绝打开**保护（downgrade refusal）——这是设计，不是 bug，不得绕过。
- 判定基准是**机器**而非构建：一台 db 升到 29 的机器上，schema 24 的版本**不能**打开它；同一条 build 在 24 的 db 上可以打开。两台机器可以并存不同 schema，只要各自满足本条。
- 实证：
  - `pdebian-GA`：db `user_version = 24` = genesis `SCHEMA_VERSION`，**直接可开**。
  - `Windows / Administrator`：db `user_version = 29`（20 张表，含 3.5.3 时代的 `tool_image_jobs` / `maintenance_runs` / `coding_submissions` / `model_usage_recency` 等新增表），**genesis（24）打不开**；任何未来版本要在此机自动复用配置，`SCHEMA_VERSION` 必须先升到 ≥ 29（走 LAW-5 的纯增量批次：bump `schema.rs` + 加对应 `revisions.rs` 行）。任何把 schema 降回 < 目标机 db 版本的版本在该机上将打不开库 → 表现为「丢配置」，属违反本条。

### LAW-9.R4 侧目录可弃，主配置不可弃
- `/mnt/ramdisk/Agent2API/{cache,logs,tmp}`（`side_dir()` 规则）：重启即弃，**不算**违反本条。
- 主配置 + db + `account.key`：必须落在持久盘。任何让主配置落到 ramdisk 的实现变更 = 违反本条。

### LAW-9.R5 启动自知（可观测性，强化项）
- 条文：版本启动时应**明确日志**：① 解析出的配置目录绝对路径；② 打开的 db `user_version` 与代码 `SCHEMA_VERSION` 是否匹配。
- 目的：把「认知配置在机器上」从隐式行为变为**用户可验证行为**——一眼可判「新版本是否找到了旧配置」，而非静默新建空白配置。
- 现状：genesis 尚无此日志（属后续功能批次，须走 LAW-5 的纯增量流程加，不得在 base-update 夹带）。

---

## 条文执行顺序（速查）

| 动作 | 先过哪条 |
|---|---|
| 写/推提交 | LAW-1（自检 grep 0 落款） |
| 改版本 / 推分支 / 打 tag | LAW-2（先问人） |
| 写发布日志 | LAW-3（两次过目） |
| 新增 fork 功能 | LAW-5 + LAW-6（纯增量、新文件、接线守卫） |
| 同步 upstream | LAW-6（只解决接线冲突，绝不丢 fork 文件） |
| 换机器 / 升级后「配置丢了」 | LAW-9（查 9.R1 解析 → 9.R2 存储 → 9.R3 schema 版本） |

## 修订纪律

- 本文条文**只增**：可新增 LAW-N，可加 .R 细则；**不得删除、不得弱化既有条文**。
- 修订须注明：所引 AGENTS.md 章节、修订人、日期。
- 与 `AGENTS.md` 冲突时以本文为准；修订后须同步更新 `AGENTS.md` 对应章节（走 LAW-3 流程）。

*（本文无 AI 落款。署名：easykeysy。）*
