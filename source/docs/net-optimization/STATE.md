# feat/net 当前状态与恢复入口

更新：2026-11-10 18:08 UTC（新加坡 11 月 11 日凌晨）。本页只记录当前状态；全部此前段落逐字保存在 [历史](HISTORY-20261111-before-current-state-compaction.md)，历史内部还有更早链。不得删除历史失败、用新绿覆盖旧红，或将历史中的运行句柄误当当前。

## 授权、范围与工作位置

完整 **N0–N5 尚未完成，用户已要求持续实施**；不能因分项绿、compact、状态问句或旧 goal 服务 paused 停工。root 串行，无子智能体。速度和实际体验优先，个人可信设备网；peer relay 与 IPv4/IPv6 随时变址是硬范围。没有 main 合并、发布、采购、生产部署或改已装 App/config/store/resident 的授权。

- 产品：`/Volumes/disk0/Khor/worktrees/current/khor-net`，`feat/net`。先核实际 HEAD/status/diff，不按文档覆盖 WIP。
- 控制：`/Volumes/disk0/Khor/worktrees/current/khor-build-net`，`feat/net`，当前固定 `68cadb57a0ed72d40a4ca9a843e86b9c6797bf34`；两处既有 untracked `__pycache__` 保留。
- N1 隔离：`/Volumes/disk0/Khor/worktrees/current/khor-net-recovery-deadlines`，detached `2ef6bee5a5aaf841af066f4edb5c894769d4539d`，clean，未接受。Iroh42–44 在该树预留，未合主。
- 证据根：`/Volumes/disk0/Khor/evidence/net`，ops 在其 `ops/`。`source-refs.json` 单写者，实际 receipt/ref/tree/control 优先于段落。
- 规划及验收：[PLAN](PLAN.md)、[REQUIREMENTS](REQUIREMENTS.md)、[ACCEPTANCE](ACCEPTANCE.md)、[NEXT-DESIGN](NEXT-DESIGN.md)、[REPORT](REPORT.md)、[VENDOR-AUDIT](VENDOR-AUDIT.md)、[本批审阅](REVIEW-20261111.md)。

## 最新已验证分项

**`153263c2df33d73f652a4d42691fba0307f873d4`**，tree `ee002acdb61286aec395dae2a54cef2418e18566`，snapshot `f4c5f39034e9493cba57bc942f134fc7415819f5`。

- Linux `turing-20261110T175147Z-8adb26d3`：全12组，hop44/wires15/budget30/resume1；256.669秒、21files、group空、identity与实际summaries核。
- Mac `42869844599`：同canonical，全12组，hop44/wires15/budget26/resume1；exact artifact `13790105312`、20files、11control blobs核。证据目录 `mac-42869844599-cold-ip-hint-recovery-v1`。
- 同canonical vendor Linux `turing-20261110T174726Z-1e1264ef`：Noq439/path50/home11绿；43.738秒、14files、group空、identity核。
- 实测范围：原地址指向错误身份，首次真实 client Refused 之后 trusted middle 才许可；peer relay 在原2秒 caller 内读 Sessions 一次。NO_ROUTE 时无业务请求；真实已有 authenticated wire 拒绝 hint 撤回且继续收发。较新 IP 来源保留，Relay/Custom不清。
- 实现：Iroh45 actor执行时只有无live连接才撤回所有来源时间均<=首次失败边界的IP hint，必要时清旧selected IP；Node只有未可能发early业务数据、无既有carrier且首次Refused时，在原绝对dial window内等现有hop并补至多一次普通握手。不放松身份验证/写重试/Asleep/总期限。
- `early-fallback-fixture-freeze.json` 与 `cold-ip-hint-fixture-freeze.json` 仅该scope true、field false。**不等于逻辑地址/network epoch闭合、手机问题解决、整包或完整计划接受。** 主vendor总54（Iroh1–41/45、Noq1–12）；45已有该分项验证，42–44隔离未接受。

此前已闭合的窄范围仍保留：sole-peer Recovery准入、慢middle不挡快普通拨号、有限pending lease、cancelled watcher successor、live middle先于两条旧低RTT缓存、cold/resumed读取并行；以及取消/响应owner/STOP/peer contact/actor conditional carried close。具体红绿源、双OS回执与限制见历史及对应freeze，不能升级成全部入口/所有代际已安全。

## 当前候选与正在跑的任务

最新覆盖（18:11 UTC）：74d8633c已终态采核，48项47/1，只有normal新pending消费者被旧释放误拆；其无消费者控制、initial Drop候选两项及原44全过，162.286秒/11files/group空/identity核。两处baseline freeze true；候选仍不接受。当前normal产品WIP：同pending admission锁内再核当前road与所有live连接comparison，只有与两LOOK结论一致才撤自己serial；有pending/实际carried使用/事实变化均保留，held重新从0开始沿原5秒周期观察。无新生产task/timer，未改LOOK/测量收益/信任/总预算。

下列56eb为已采核旧normal源，74d8633c不再运行；新candidate回执待登记，恢复先查ledger。

最新提交（核实际git）：`56eb822a40f1caca20163ddd9659357d4792c591`，tree `76d617b48297eac5f0ff131adc6d70956865da42`，snapshot `3b992170a63a2fdce192b4c41ed5ffc3c1d7338a`。

1. **initial PendingHop::drop 候选未接受**：复用 pending admission 锁，在同一决策内检查新pending/live使用并撤自己exact serial；有消费者交既有watcher，无新生产task/timer。旧行为已真实红：`e4eb1357` / Linux `turing-20261110T175953Z-9bb7adf7` 46项45/1，新pending+真实未登记carried TLS/echo进入后carrier被旧Drop拆；无successor与原44全过，142.192秒/11files/group空/identity核。格式首 `a98e1b6d` 保留。
2. **normal publication watcher仍旧产品行为**：新增真实两次原LOOK_EVERY=5秒后、新pending+未登记incoming carried-only TLS/echo进入的合同；独立IP/outer仍应可用，无消费者仍应释放。cfg(test)暂停点不改变正式版时序。
3. 已采核的 Turing baseline **`turing-20261110T180659Z-74d8633c`**，PID2698867，18:07:21 UTC启动，task net-node，source为上列56eb。回执 `evidence/net/<run>/receipt.json`；本轮兼测Drop候选/normal旧行为，预期即使normal红也不能称全组绿。不得重复提交。前 `60cfd819` 仅格式43.722秒/10files/group空，已采核并应用远端patch；没有行为结论。
4. `pending-drop-fixture-freeze.json` baseline true/candidate false；`normal-release-fixture-freeze.json` baseline/candidate均false。原published whole保留为prefix，新合同原时间/字节/清理不削弱。

## 下一步（连续推进）

- 74d8633c已采核47/1；现在验证两个释放候选同canonical双OS，不重复收/重提交旧run。
- 若normal实际正向红而无消费者控制及原46绿，再最小修normal释放：pending admission保护撤回，同时复核当前所有连接/比较，不能拿旧comparison误拆新owner，也不绕过原两LOOK测量规则。保持相同fixture和原预算；随后同canonical双OS全12与必要vendor，才接受两个新scope。
- 如果是fixture/格式/编译失败，保留该attempt并分类，不当行为红/候选绿，不盲重跑。
- 之后继续：incoming-only首腿、NoRelay/ToNoRelay缺失提示；MISSED/LOST冷却和IPv4/IPv6/本机与peer/middle网络代际、旧完成事件不能污染新网；accepted写job/晚到loser/unused carrier成本。不能仅清缓存就宣称完整epoch。

## 完整范围的未完成项

| 包 | 当前仍欠的完成条件 |
| --- | --- |
| N0 | 最终体验SLO/容差/采样量冻结，前瞻paired校准与足量p99；失败/超时不得从分布删除 |
| N1 | 隔离42–44物理恢复/quiet成本及原矩阵仍红，不能拿主Node绿覆盖。2ef bf6105为124/124但5bad，X5505.782/Y3496.330ms；quality确认窗重启及cold start仍待解决 |
| N2 | 上述初始Drop、正常release、epochs、各入口与ownership全合同，当前只接受各自已实测scope |
| N3 | Custom1024frames/256KiB公平性，SSH/hop混合、shared/separate 1/3/5的交互/CPU/字节/唤醒与手机能耗 |
| N4 | 实际两腿Relay、唯一middle/断腿/多middle失效；独立IPv4/IPv6变址及Wi-Fi↔蜂窝、peer/middle移动；真实手机自动恢复和拓扑证据。macserver↔Turing已有cohort不独立证明NAT穿透 |
| N5 | 原53+新增45及隔离42–44逐项审计/上游退出证据、Noq1/5缺口、最终Quality/native/proxy/多设备和手机能耗 |

原始独立失败保留：Linux pair setup core transfer Busy `854aa6ce`、此前原resume32 timeout、N1物理时限与cold-start红；后来的同scope绿不自动解释或关闭这些失败。全部详情/回执在历史及ledger。

## 真机事实与当前限制

- MacLab本地电脑与Turing同LAN；构建macserver是另一台，异网来自用户报告，尚非独立NAT验真。
- iPhone localhost 已安装且配对；旧现场经历Wi-Fi→蜂窝→Wi-Fi自动恢复失败，反复关开App后某次经Turing恢复。未独立证明OS force-quit，更不是自动收敛。
- IMG_5608显示蜂窝5GA、hinton Direct29ms而多设备不通。单可达peer在第二腿可达/策略允许时应有界尝试转发，但截图不证第二腿、当时源码或精确故障持续时间。
- **旧手机采样都已结束，候选没有安装。** 现在不启动phone RPC/sampler、不要求用户保持亮屏/切网。旧79样本47ok/22busy/9paused/1error不是新候选SLO。

## 执行与证据规则

本机source-only，不跑Cargo/Rust/Node/Xcode构建、测试、formatter或下载执行。Git/Python标准库与只读元数据核可用。Turing只经既有UID1110/flock/隔离netns/build admission，Rust1.95/jobs16/nice10、50GiB起40停/90min；无sudo/host网络变更。Mac用原单cache/jobs2/50–40GiB/100GiB guard。无重叠重型Turing profile；普通unit可Mac+Linux并行，计时物理netlab不得与Mac重型重叠。SSH不转发agent/credentials。

图谱先行，只作导航；build落后、flows0、tests_for0均不是安全/覆盖证明。避免宽读 `crates/net/src/endpoint.rs` 顶部编译relay bearer token。无需CUA。没有子智能体。

`ops/stage-launch-turing.py` 用已存canonical source JSON、固定control和前一terminal receipt；exec session必须poll完成再依赖其结果。collector只收终态/group空，已有report不重复覆盖；verify只核identity，实际行为必须读原日志。Mac登记source ref后按**精确artifact ID**采集并核11 control blobs。ledger单写者，stage期间不并发写。

最新全source-ref只读审计 `source-refs-audit-20261110T165639Z.json`：429 refs/tree、64 control labels、45ops SHA、335显式receipts存在；94 legacy row缺submitted_receipt原样保留，短9135b6d唯一解析不改旧label。不是runtime/artifact全审。

临时baseline worktrees已clean非force清理：early serial 9b680476保留pin `refs/khor/net-baselines/early-serial-calibrated-v2`；queued817与peer543同样已清理。不要按历史重建/重复删除。所有失败/原fixture/hash/refs保留。

## 对用户的估时口径

16:20 UTC已告知：下一份可评审候选约2–4工程小时（原锚点18:20–20:20 UTC），完整N0–N5含真机/换网/能耗仍3–5工程日量级。原13:00 UTC的4–6小时只指下一批，已向用户解释此前口径不清及新增缺陷造成延后。不得每次compact或状态询问无声重置起点；这些是工作量估计，不是完成保证。用户现在询问进度，已简答并持续执行。
