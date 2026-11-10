# feat/net 当前状态与恢复入口

更新：2026-11-10 01:36 UTC。root串行，无子智能体。完整N0–N5尚未完成，用户已授权继续；不因goal服务paused或阶段通过停止。速度/体验优先，可信个人设备网；无main合并、发布、中继采购/生产部署授权。

重写前逐字状态在[HISTORY-20261110-before-ipc-lease-integration](HISTORY-20261110-before-ipc-lease-integration.md)及其历史链；旧失败与时间参照勘误均保留。计划/硬要求/验收以[PLAN](PLAN.md)、[REQUIREMENTS](REQUIREMENTS.md)、[ACCEPTANCE](ACCEPTANCE.md)为准。

## 当前句柄与接续

- 先核Git/receipt/GH/真实PID再动作，禁止盲重发。产品HEAD343d1481：Iroh39已实现但物理未测；IPC v2仅private helpers/测试，未发布能力/接入真实Node请求。控制HEAD4bf7e6eb（已推）：net-node含ipc::全部helpers及32回复实际用例，WAN v2每方向30目录+30正文。原50 vendor＋新Iroh39候选共51项。
- **Turing唯一重型任务为第二轮matched candidate turing-20261110T012832Z-5e8674ab，PID1443515，01:29:44UTC已启动**，准确a42b5b64/tree48b907da/control4eeeea9，只有Noq12，不能夹Turing编译/其他重负载。前驱baseline2 turing-20261110T010548Z-d234b36c已1195.123s/rc0/原group空；原p1s/p5绿，最慢635.3ms、零move、9punch/3618包/1471863B，22CPU markers逐线程亲和核。首对baseline539.2ms/零move/13punch/4035包 vs candidate约1s/一次move/16punch/4788包已保留；matched repeats仍未收齐，不能性能验收。
- **IPC caller期限真红绿已闭合此分项**：42729827780实际IPC5绿1红（40ms caller被旧Op上限覆盖、250ms observer超时）；42733228660准确3ad3851a/tree9d7de9ef/controlce02实际fmt/IPC6/net67/hop12/wires11/journal30/budget20绿。实际红/绿整个测试块原始SHA10838747相同；先前a40fbc31抽取法未复现，原hash保留并明确勘误，不伪造其验证。只证明client，不证明server或remote取消。
- **IPC helpers与真实32回复red准备中**：dc1aa1d8引入lease framing/within但未接入真实Node；ace46921只应用42735177894远端formatter。6bee9944原32用例的42735757244仅格式失败；随后静态发现self-home content_page绕过IPC，343d1481明确改用已有content_on_resident(Window)保持32/原barrier/5秒门，兼补helpers Op安全cap与更早parent。新42736367611/request8ddfd3dc5c4144e59808ac0a01bbbbc5/source46560a8c/treebd6c4fc6/control4bf7e6eb：attempt1在bootstrap下载Git blob JSON截断，尚未导入源码/启动driver，3份partial artifacts与trace保留在artifact-attempt-1-bootstrap。确认终态prebuild后已重跑同一run的attempt2；必须核run_attempt/原hash与日志后分类，不用执行器/格式失败冒充32资源红。未见helpers或32行为绿，不接真实新协议。
- **Iroh39 vendor/portable已真绿，物理仍待**：42734017374准确ba7bcefb/tree144025c2/controlce02 fmt/Noq439/paths42/home9（含6个新预算单元）；42734610668同完整source/control9992254 portable67/12/11/30/20/IPC6全绿。source/control/11blobs/所有artifact hash已核。下一仍须原peer物理至少3轮/冻结4秒门/实际新增probe计数、原stalls与黑洞成本，不能把单元绿当四秒达标。
- **恢复缺新往返已实证**：turing-20261110T002954Z-cabd9725准确3013038a/tree4e9a79a3/snapshot8f483819/controlef0b，124/124、终态rc0/原group空，70条实际conn/path快照均PTO0/unacked None、received>judged而sample.sent<=judged。outer Go+1187/+1253ms，inner考虑+5302/+5313ms、Go+7552/+7562ms。不是SILENT_HOLD，不能弱化sent>judged。Mac275(42726616050)同source fmt/Noq439/iroh36/home9绿。候选最多3个PING申请的策略与限制已在ACCEPTANCE实施前修订；四秒checker/原物理fixture不动。
- **四秒门已有真实红**：267/272原数据，warm deadline后51次分别42/40选对、9/11仍Relay。scripts/netlab-peer-selection.py与whole physical hash冻结在peer-recovery-four-seconds-red-green.json；至少3独立physical重复。旧可达/rehold10秒绿不能替代此门。
- **Noq12单元/匹配fixture已真红绿**：266新floor单元438绿1红；268新floor绿但旧shortstall准备几何不一致红，原失败保留。270旧质量+真正20ms RTT fixture（snapshot9c8fa9d4）仅floor红；271新质量+同fixture（a42b5b64）fmt/439/36/9全绿；whole paths tests与whole multipath.rs分别SHA1d8da16d/fff96ca1相同，全部Rust diff仅selection样本9行。267/272 physical124/124但恢复仍慢。273同a42 portable67/12/11/30/20绿。Noq12不改transport SRTT/PTO/拥塞/25%收益门。
- **严格NAT门持续化已实际绿**：269 turing-20261109T235237Z-bb957183准确3afd69c1/tree3386599f/snapshot62dda02e/control4eeeea9，124/124，warm30/30实际新Direct、零mismatch、原group空；新report与冻结独立checker每request/problems一致。Iroh38的247红→249绿与whole rebind.rs SHAac977269不变。
- **N4正文只编译，尚未真实跨网**：277 Mac42727910806准确60ce10cf/tree0e8ef193/snapshotb4c490cb/control4eeeea9 build-only绿；新Mac worker SHA c377ce6482a71c358eb96c5324bb1104a39bc123b03e6769d548cf283b43edca、182600176B，read_content逐字核29B fixture。276仅格式失败。旧global wan-binary-manifests仍ff608 source，不能与已覆盖Mac cache混用；待Linux同b4c source受guard构建，再更新完整cohort/两端controller并真实正文验收。

## 已有真跨网与设备事实

A=macserver、B=Turing天然异网，P=iPhone，不再询问另找两电脑。用户“再试试”后的SSH确曾成功，随后多次banner timeout；最近01:03UTC重试仍timeout。这是iroh-ssh控制通道事实，不是Khor数据面故障。GH受guard通道可用。

- 263 Mac42721024415/coordinator e0b0b491f8336536/controlebf7f0d，同ff608e91/tree7d0c7467/snapshotb13e4398：实际两端fresh配对身份相符，双向目录30+30全部ok；Mac→Turing p50/p95/max61004/65659/84881us，反向59729/61452/71335us。真实公共Relay101.200.80.77:3340，非Direct/peer-relay/正文/冷建连/p99证明。两端joined/owned空/Linux原PGID空/锁释放，临时secret删除与fresh身份复制binary已清理。wan-first-paired-proof.json。
- Mac262与Linux69e7346各stop/EOF/kill三种真实Node lifecycle3/3，resident真正join、原group空、无forced cleanup，准确ff608源。Linux旧3套fresh homes在原flock下核owner/hash后已清除，不动cache/target。
- iPhone可操作/配对方式的可选async问题待答，服务器开发继续。未做iPhone换网/能耗，不冒称完成。

## N2进度边界

237blocked writer真实红→239Mac/240Linux同整fixture绿；189 stale owner红→193/194绿；178共享消费者红→179/180绿；F8/F9/F10已有分项真红绿。IPC caller短期限实际红绿见上；[IPC-LEASE-DESIGN](IPC-LEASE-DESIGN.md)已有private framing/within helpers及候选tests，真实server/EndpointFile/via_serve尚未接入或启用。正常legacy半关闭不能作取消，不因client结束宣称server/remote停止。

## 完整剩余范围，不得缩减

| 工作包 | 仍必须完成 |
| --- | --- |
| N0 | 最终目标/容差/样本量与p99充分性、全部outcomes、同版本baseline/candidate/固定好路和负向校准；不混旧量测版本 |
| N1 | 冷暖所有候选/多middle/双向、201/208/244/242慢方向因果与恢复后真实业务，新鲜/稀疏确认、探测成本、原hop/recovery/dynamic/stalls/full矩阵与162/182历史失败 |
| N2 | IPC剩余预算/取消贯通、长多阶段/两middle合计3秒、输者与晚候选、近期地址/重启/jitter合并/quiet epoch/close-vs-offer；写不重放/正常half-close不取消 |
| N3 | 真实Node32席位/content32、Custom1024帧/256KiB外层队列、SSH/hop混合公平/共享vs分连接、1/3/5设备CPU/字节/唤醒/手机能耗；Noq FIFO_v2低层704请求不能替代 |
| N4 | U12完整移动矩阵、Mac↔Turing↔iPhone真跨网、冷暖/双端不对称、登记/出口/整机故障域、布局容量运维；不是采购部署 |
| N5 | 原48＋新增Iroh38/39和Noq12，共51 vendor补丁（Iroh39候选）逐项覆盖/上游等价/退出、Noq1/5缺口、准确最终源可信Quality、Mac原生App/代理/多机/手机/能耗；证据前不升级/删补丁 |

## 执行与证据规矩

- 本机source-only，禁Cargo/Rust/Node/Xcode产品编译/测试/formatter/下载可执行物。Git/Python stdlib、数据分析与纯stub控制合同允许。不改installed policy/ensure-budget/已装App/生产store/其他worktree。
- Turing owned root `/tmp/khor-netlab-feat-net-1110`、uid1110，单flock重型槽，Rust1.95/jobs16/nice10，50GiB准入/40GiB停/90min。仅fresh user/netns、不sudo/改宿主网络；计时不并跑编译/其他重负载。
- Mac沿macserver/khor-build既有guard单槽/jobs2。installed run-build→build_guard.py持有cache/build.lock，50/40GiB与100GiB target门，不改/绕过。SSH开发也须走该guard，先核实际任务占用；不得让自有resident脱离guard生命周期。
- SSH BatchMode=yes/ForwardAgent=no/StrictHostKeyChecking=yes，不转个人key/token/agent/签名/模型凭据。源/控制/输入先锚refs/hash，变更后新快照；观察timeout≠停止，核/proc cmdline、终态原PGID空、collection，不盲重发。
- ops/submit-turing.py stage显式task/revision/完整control/role，当前HEAD或--source准确元数据；launch要求前驱完整终态采证、group空/flock/inputs正确。collect-network-ssh返回101/2可能是采到产品/fixture失败，先读collection不覆盖重采。
- ops/verify-mac-record.py只核GH/Git/artifact身份，分类须读log。源/transport/9control blobs/hash须核；别把fmt/编译/fixture/前置/行为/执行器失败混算。
- 旧baseline worktrees（detached6485与CPU snapshot）未动，现代fixture matched baseline仍待做；不能checkout/reset覆盖其WIP。Mac snapshot/bundle临时分支尚未清理，只有终态、所有权/哈希留证后才可清；不擅删复用者分支。
- graph first；本worktree无CodeGraph不自建，code-review-graph可能滞后HEAD，先图谱再定向读缺口。无子智能体授权，root串行。
- 183 recorder v4真实2ok/error/timeout/cancel全部outcomes、195 held_precondition_v2三轮原1500ms/30s门、176/181固定核stalls成本增加等旧证据详见历史；不因新校准抹去局限。


继续完整N0–N5。最终源可信Quality、Mac App/多机/iPhone/能耗以及上表缺口仍须闭合。

证据新增：source-refs-audit-20261110-287.json逐核287源refs/tree/control与25个ops hash；后续逐条pin，待终态再全量更新。register-mac-run.py仅锚GH实际控制HEAD/源码/refs，不分类行为。当前CPU重复比较在ack-floor-controlled-stalls-comparison.json。继续N0–N5，不在阶段绿后停工。
