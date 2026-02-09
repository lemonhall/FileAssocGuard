# FileAssocGuard PRD（OpenSpec）v1

> Source PRD: `init_prd.md`（PRD v0.3, 2026-02-09）  
> 本文目标：把“草稿 PRD”转成**可追溯**的需求清单（Req IDs），供 `docs/plan/vN-*` 引用与验收。

## Vision（愿景）

在 Windows 11 上，长期防止第三方软件后台反复篡改媒体文件（如 `.mp4`/`.mkv`/`.mp3`）的默认打开方式：**一旦被篡改，能被可靠检测并自动恢复**，并向用户提供清晰可验证的证据（日志/事件/输出）。

## Non-Goals（本阶段不做）

- 不在 MVP 中逆向实现 Win11 `UserChoiceLatest` 新 Hash（只做检测与引导）。见 `F30`。
- 不在 MVP 中做“篡改来源进程识别/溯源”（ETW/审计）。见 `F31`。
- 不追求跨机器配置漫游/同步。见 `F32`。

## Terms（术语）

- **Ext**：扩展名（带点），例如 `.mp4`
- **ProgId**：Windows 文件关联的程序标识符（例如 `PotPlayer.mp4`）
- **UserChoice**：`HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\FileExts\.<ext>\UserChoice`
- **HashVersion**：`HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\SystemProtectedUserData\<SID>\AnyoneRead\AppDefaults\HashVersion`

## Constraints（关键约束）

- **必须**在 Rust 中实现旧版 `UserChoice` Hash；不接受“调用外部 exe/商业工具”作为替代。
- 写入 `UserChoice` 的 `ProgId/Hash` 必须使用该键的**最后写入时间**（截断到分钟）计算 Hash；流程必须在同一分钟内完成（跨分钟需重试）。
- 初期目标聚焦媒体文件扩展名；不以 `.pdf`/`http(s)` 作为必须支持对象（它们受 `UCPD.sys` 保护机制影响）。

## Requirements（带追溯的需求）

### Phase 1（Rust CLI MVP）

**REQ-001（平台）**  
在 Windows 11 x86_64 上可运行；默认目标为用户级文件关联（`HKCU ... FileExts`）。

- Acceptance:
  - 提供可执行文件（Release）可在 Win11 运行。
  - 所有核心注册表操作仅针对用户级路径；系统级路径（`HKLM/HKCR`）不作为默认修改目标。

**REQ-010（读取关联）**  
读取指定扩展名的当前 `UserChoice`：输出 `ProgId`、`Hash`、`LastWriteTime`（FILETIME/可读时间）。

- PRD Trace: `F01`
- Acceptance:
  - `fag.exe read --ext .mp4` 输出包含 `ProgId` 与 `Hash`（若不存在要有可判定的“无关联/无 UserChoice”输出）。

**REQ-011（快照）**  
可对一组扩展名批量读取，并输出到配置文件（JSON）作为守护规则的初始来源。

- PRD Trace: `F02`
- Acceptance:
  - `fag.exe snapshot --extensions .mp4,.mkv` 生成/更新配置文件，包含每个 ext 的当前 `ProgId` 与（可选）程序路径字段。

**REQ-012（规则 CRUD）**  
支持对守护规则的 `list/add/remove`（CLI）管理，并持久化到 JSON。

- PRD Trace: `F09` `F10`
- Acceptance:
  - `fag.exe add --ext .mp4 --progid PotPlayer.mp4 --program-path "C:\\...\\PotPlayerMini64.exe"` 后，`fag.exe list` 中可见该规则。
  - `fag.exe remove --ext .mp4` 后规则消失；再次 `remove` 应返回明确的未找到提示（非 silent）。

**REQ-013（检测篡改）**  
对比“当前关联”与“守护规则”，报告差异（哪些 ext 被篡改、从什么变成什么）。

- PRD Trace: `F05`
- Acceptance:
  - `fag.exe check` 对每条规则输出状态：`OK` 或 `TAMPERED(old→new)`，且 exit code 可用于脚本化（例如 `0=无篡改`、`2=存在篡改`）。

**REQ-014（恢复）**  
当检测到篡改时，可将关联恢复到守护规则的 `ProgId`，并写入正确的 `Hash`，系统认可（双击验证）。

- PRD Trace: `F03` `F04`
- Acceptance:
  - `fag.exe restore` 对被篡改的 ext 写回后：`UserChoice` 的 `ProgId/Hash` 与系统体验一致（可通过“设置默认应用/双击打开”验证）。
  - 发生跨分钟导致 Hash 无效时，必须自动重试并给出可读日志（不是“偶尔失败”）。

**REQ-015（守护模式）**  
提供前台常驻 `watch`：按固定轮询间隔检测并自动恢复。

- PRD Trace: `F06`
- Acceptance:
  - `fag.exe watch --interval 5` 持续运行，且对篡改事件能在一个轮询周期内完成恢复并记录事件。

**REQ-016（系统通知）**  
恢复成功后可弹出 Win11 Toast 通知（可配置开关）。

- PRD Trace: `F07`
- Acceptance:
  - 在启用通知时，发生恢复会触发 Toast（至少包含 ext 与恢复到的 ProgId）。

**REQ-017（系统状态/自检）**  
提供 `sysinfo`，展示用户 SID、`HashVersion`、是否启用 `UserChoiceLatest`、`UCPD.sys` 状态，并给出明确可执行的建议。

- PRD Trace: `F08`（+ 风险章节）
- Acceptance:
  - `fag.exe sysinfo` 输出包含：SID、HashVersion、UserChoiceLatest 是否启用、UCPD 是否启用（以及“对媒体文件是否有影响”的文字结论）。

**REQ-018（UserChoiceLatest 应对）**  
若检测到启用新机制：输出明确的 ViveTool 禁用指令（仅提示，不自动执行）。

- PRD Trace: `L2/L3`（2.3 应对策略）
- Acceptance:
  - 当 `HashVersion=1`（或可判定为新机制启用）时，输出包含 `vivetool /disable /id:43229420` 与 `vivetool /disable /id:27623730` 的指引，并解释风险（Hash 计算不可用）。

**REQ-019（日志）**  
篡改与恢复事件必须记录到日志文件（包含时间、ext、old/new、动作、结果）。

- PRD Trace: `F11`
- Acceptance:
  - `watch/check/restore` 的关键事件写入 `logs/guard.log`（或由配置指定路径），且日志行可被脚本稳定解析（例如 JSON lines 或固定前缀格式，二选一）。

### Phase 2（Godot GUI，后续版本计划覆盖）

**REQ-100（系统托盘常驻）**：托盘图标+菜单，常驻守护体验。PRD Trace: `F20`  
**REQ-101（主界面规则列表）**：可视化展示/编辑规则。PRD Trace: `F21`  
**REQ-102（添加规则弹窗）**：选择扩展名+选择程序。PRD Trace: `F22`  
**REQ-103（事件面板）**：展示最近篡改/恢复事件。PRD Trace: `F23`  
**REQ-104（设置面板）**：间隔/自启/通知开关。PRD Trace: `F24`  
**REQ-105（状态检测 UI）**：展示 UCPD/UserChoiceLatest 状态并引导。PRD Trace: `F25`  
**REQ-106（深色模式）**：Win11 深色主题。PRD Trace: `F26`

## Open Questions（待澄清）

1) 配置文件路径与优先级：默认 `./config.json` 还是 `%APPDATA%\\FileAssocGuard\\config.json`？（涉及“免管理员 vs 需管理员”体验）
2) 日志格式：更倾向 `JSONL`（便于 GUI/脚本）还是“人类可读行”？（可同时输出，但要定义 DoD）
3) 是否需要支持“通过 program path 推导 ProgId”（从用户选 exe 到 ProgId 的映射）？（Phase 1 可先要求用户显式提供 ProgId）
4) 仓库/产品命名对齐：`init_prd.md` 中示例路径为 `E:\\development\\file-assoc-guard`，而当前工作目录为 `E:\\development\\FileAssocGuard`；需确定最终仓库名/可执行文件名（`fag.exe` vs `FileAssocGuard.exe`）的对外口径。
