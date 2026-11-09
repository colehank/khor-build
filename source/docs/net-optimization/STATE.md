# feat/net 当前状态与恢复入口

更新：2026-11-09 21:59 UTC（本地2026-11-10）。root串行，无子智能体。

完整目标仍是完成[PLAN](PLAN.md)的N0–N5，尚未完成。用户已授权实施/远端验证并多次要求继续；U14明确不能因goal服务paused或阶段通过自行停止。后台调度状态不是本轮停止/授权门，不另建目标，不伪造complete/blocked。速度和体验优先，个人可信设备网络；无合main、发布、中继采购/生产部署授权。

本页重写前逐字内容在[HISTORY-20261109-before-wan-access.md](HISTORY-20261109-before-wan-access.md)，再沿其历史链。所有旧真实红/失败保留；新绿不能回填旧因果。完整硬要求/协议见[REQUIREMENTS](REQUIREMENTS.md)、[ACCEPTANCE](ACCEPTANCE.md)，设计边界见[NEXT-DESIGN](NEXT-DESIGN.md)。

## 先核当前句柄，禁止盲重发

- 产品worktree `/Volumes/disk0/Khor/worktrees/current/khor-net`，feat/net；最新功能2654cbdf，后续增加诊断与文档，实际HEAD/diff以Git为准。control `/Volumes/disk0/Khor/worktrees/current/khor-build-net`，feat/net，已push **fc204ac4764fab1bcb71842b0e95ce9b49e4e7a5**。
- **当前唯一Turing重型任务246**：`turing-20261109T215631Z-7b99a145`，task netlab-nat-rebind-lifecycle、revision node_nat_port_rebind_v1，PID1182454/child1182681；last observed实际cmdline仍在，r0物理校准已1/1绿，r1业务在途。control fc204ac；source base831c13839a3836c0788d8e18877d82b39a96abdf/treeed32a17e52fc874c273b10a985b6e64557667cd7/snapshot4a7fb25017e9ae7593bf09cc3da2c17485c7eaf5。禁止并跑Turing编译/真实跨网计时。
- **245 Mac42706942409终态全绿**，request5479e3b5ea81417bb56fc6f167653dc2；与246完整source/control一致。fmt/net67+1ignored/hop12/wires11/journal30/budget20（49.90s），source/transport/9control blobs/artifact hashes核。证明新增request_stream_opened编译与便携合同，不替代Linux/真跨网验收。
- 244及此前所有任务已终态采证。`evidence/net/source-refs.json`、`turing-next-matrix.json`、每run receipt/collection/report hashes是真实句柄入口。最新批量refs审计source-refs-audit-20261109-owner-green-243.json逐核243 refs，后续逐条先pin；应在246终态后刷新全审计。

## 最新实证与未闭合问题

**246已终态**：132.16s/rc0/原group空，r0+r1可达2/2、124/124ok；after AB p5022166us、BA123616us，首BA660726us。nat-rebind-calibration.json与request-wire-correlation.json核141条非重复lifecycle；实际after-change每方向31个request streams，B业务wire已有新:50002但所有路径selected=false，另wire仍selected旧:50001。当前无Turing在途，下一只增加Noq真实tuple vs irohod缓存的gated mismatch诊断，保持旧行为查因，不盲改策略。


### Peer relay与最优连接（N1/U11）

- **冷候选排序223真实红→224 Linux/225 Mac同整fixture绿**：功能b05b5e81把已符合资格的候选按完整两腿成本排序，Direct只同成本tie-break。原Dial慢Direct120ms/未知65536ms曾压过快Relay+SSH5ms；现在5/120/65536。whole fixture SHA a0a106ebe2fac9795aa49e4ff1cab49c5b7bbd43bc822d14b3d68a295252bfa2，proof cold-hop-cost-red-green.json。资格限制/预算/收益门仍待完整设计，不冒称全部cold竞速完成。
- **Unused误退避232真实红→233 Linux/234 Mac同整hop tests模块绿**：功能58cb6475只在Outperformed且road=Relay时lost；Unused仍回收但不惩罚middle。所有Main/Tunnel/served live wire单快照/去重，unknown保守，旧owner guard保留。whole module SHA5f2cecfd427d3397c5a7147f1946248f99068e9317a263736b9510a96ff07ccc；proof unused-hop-release-red-green.json。原25%收益/两look/LOST十分钟未放宽。
- **228/235真实middle IPv4换网均124/124可达绿、最好路未恢复**。235仅含Unused修复，310.96s，before p50约108ms/after约155ms，60秒carried_observed_again=false；Y在换网后约5.6/10.6秒作Outperformed判定（Carried质量约263ms、age约4秒、samples0；Relay约131ms）并释放。228另为所有Custom消失后的Unused旧误退避。不能用同一解释覆盖两run。proof各peer-mobility-calibration.json、235/measured-release-trace.json，汇总peer-mobility-progression.json。
- **244新outer-wire关联：124/124可达绿、reported carried再次hold10秒**，turing-20261109T215036Z-46419d95，204.45s/rc0/原group空；与242/243同完整source d1a7a401/tree0bb908a7/snapshot6833a423/controle4c58dc。replug273ms、首AB636590us/BA617343us；before p50 110260/108046us，after155669/154371us，reported恢复hold从+21633至+32111ms。**测量reads在+21602ms已全结束，没有恢复后业务样本，不能称体验已达标**。当前serial2的实际outer绑定已记；原serial1在移动前约56秒已测量回收，不能把它算移动故障。outer-wire-correlation.json保留事实与限制。不同源包含owner停止修复和诊断，不能宣称单补丁因果。
- 新诊断仅在开关内记录实际initiator/receiver attachment→Proto::Tunnel wire、middle incoming/onward wire/splice起止；不额外持有连接或probe。新增request_stream_opened记录真实open_bi的wire/stream、reading布尔、当时path facts，不记正文/token/个人路径。它是使用连接与开流快照，不是逐包最终承载。
- **继续项**：先完成246实际业务wire关联，定位新路已打开却慢方向仍滞留的原因；244要核实际两腿wire，再补恢复后真实业务/多middle/稀疏新鲜确认与原f4/完整矩阵。201慢轮、208不足收益回收、162/182旧失败因果仍未关闭。

### 移动地址（U12）

- **215裸IPv6物理替换**1/1＋7正1负ping；**217真实Node IPv6替换124/124**；**218原双向与remote-first两例248/248**。真实同prefix旧址删/新址加、IPv4不变、两端eth0全部外部IPv4被nft阻断，不能借IPv4/唯一HTTP Relay过关。恢复可早于Node接口通知，不把业务恢复归功于210修复。不是静默后台/同wire/长流/手机证明。
- **226三病例3/3、372/372ok**：IPv4 Home→Strict NAT、双栈IPv4/NAT+IPv6 prefix同时变、IPv4-only→IPv6-only。真实kernel/12router rules/独立固定local UDP映射及Khor远端path记录均核；remote-first首分别675704/874294/827742us。旧nft0.9.3 fullcone map会在stdout前置warning，raw JSON/错误/成功text规则均保留，不冒称空规则；node-roam-calibration.json。
- **238 NAT-only r0实际物理1/1绿，但缺标准netlab-result导致driver拒绝，r1未执行**；只补成功assert后的结果行，shell guard不放宽。nat-rebind-calibration.json明确business_executed=false。
- **242 NAT-only r0+r1可达2/2绿，最优不绿**：turing-20261109T214614Z-f70be96f，138.23s，原group空。固定同local tuple/同WAN:50001→:50002；两族kernel相同、定向conntrack删旧/新QUIC映射实际留证。r1全部124/124ok，mapping82ms、remote-first紧接+0ms，首BA616119us/其后AB121397us；after AB p50 18186us、BA121073us（before18573/18048us）。B新Direct :50002在+610ms确实opened，另wire+1501ms仍selected旧:50001，new_quic_path_selected=false。不得写成完全没发现新端口，不能把可用绿当最优绿。完整nat-rebind-calibration.json。
- **246独立lifecycle模式**保留242全部物理/业务/hold/预算，只补实际request wire日志。控制fc204ac有新独立task，普通profile保留，8项Python cargo/rustc stub/小Git控制合同绿；没有本机产品执行。
- 仍需仅外部IP变化/更长映射缺口、双端/中继IPv6/前缀移动、连续变更旧结果晚到、休眠/长流/真实手机；部分可达绿不代表U12完成。

### 期限、取消、资源归属（N2）

- **237 Mac blocked writer实际双分支红→239 Mac/240 Linux同整fixture绿**：功能2654cbdf每个Link持有独有oneshot，detach/replacement drop后停止该旧read/write，即使flush阻塞；serial guard不变。真实一字节duplex证明writer在阻塞，远端不关闭前old Attached必须结束，旧源false/false，新源通过；替换新owner仍在。whole carry tests SHA f6a0e6324d3c2b27763cb80b8e1cb90533727f444dcc521c6742259e20534365，blocked-carrier-owner-red-green.json。双平台fmt/net67+1ignored/hop12/wires11/journal30、Linux预算24/Mac20全绿。不是N1慢路根因证明。
- 189真实stale-owner红→193 Linux/194 Mac同fixture绿：detach_attached同锁核serial，旧watcher不得撤新路/记lost。
- 178真实共享消费者红→179 Linux/180 Mac同fixture绿：所有live Main/Tunnel/served、stable ID去重；iroh selected FourTuple按peer共享，但各conn open paths不同，不能按每conn独立selector解释。
- F8/F9/F10期限、Accepted后取消已有真实红绿；245Mac20/240Linux24重核。跨进程IPC预算/取消尚未贯通；请求正常write shutdown后read回复，服务端EOF不能误当取消。写不重放、读≤1重试、0RTT信任边界/健康共享wire保持。
- 仍需输者/晚候选回收、两middle合计3秒长多阶段、jitter/事件合并、quiet epoch、旧地址/重启与close-vs-offer竞态；不能以本项资源绿缩小N2。

## 设备与执行器（用户硬要求）

- **A=macserver（Mac），B=Turing（Linux），两台天然异网，P=iPhone**。用户已提供设备组合，不再等待别找电脑。
- 用户2026-11-09约21:54UTC答“再试试 现在可能通了”；随后实际SSH已成功，host z13deMac-mini.local/Darwin arm64/uid501。Mac owned builder根/现有guard只读核，free约65.3GB；SSH别名ProxyCommand是/usr/local/bin/iroh-ssh，**不是Khor**。此前banner timeout只是当时通道观测，现已恢复，不能继续当阻塞。真Mac↔Turing数据面尚未执行。
- N4真跨网下一按NEXT-DESIGN：fresh私有owned root/identity/store/fixture、两端准确binary/source；Node::serve/open_as/invite/pair/devices/ls_of与同进程stop_serving已审计，具体lease EOF/绝对期限/身份核/真正join/清理合同及harness尚未实现。临时ticket仅可信controller私传，不进源码/普通日志/通用artifact，不传个人凭据/生产store。不能设KHOR_NETLAB；保持KHOR_NO_MDNS与独立home/socket。首次配对后读不能称冷网络建连。独立stage准备可以推进，计时不与246争用Turing。
- Turing128CPU/~504GiB用于缓存/独立Mac并行验证，不默认派子智能体。稳定路径缓存206/207同快照17.05→0.41s构建只是一组，不承诺通用倍数；fmt失败补丁在一次性副本生成，原cached tree干净/mtime保护已远端实核。

## 完整剩余范围，不得缩减

| 工作包 | 仍必须完成 |
| --- | --- |
| N0 | 最终目标/容差/样本量与p99充分性、全部outcomes、同版本baseline/candidate/固定好路和负向校准；不混旧量测版本 |
| N1 | 冷暖所有候选/多middle/双向、201/208/244/242慢方向因果与恢复后真实业务，新鲜/稀疏确认、探测成本、原hop/recovery/dynamic/stalls/full矩阵与162/182历史失败 |
| N2 | IPC剩余预算/取消贯通、长多阶段/两middle合计3秒、输者与晚候选、近期地址/重启/jitter合并/quiet epoch/close-vs-offer；写不重放/正常half-close不取消 |
| N3 | 真实Node32席位/content32、Custom1024帧/256KiB外层队列、SSH/hop混合公平/共享vs分连接、1/3/5设备CPU/字节/唤醒/手机能耗；Noq FIFO_v2低层704请求不能替代 |
| N4 | U12完整移动矩阵、Mac↔Turing↔iPhone真跨网、冷暖/双端不对称、登记/出口/整机故障域、布局容量运维；不是采购部署 |
| N5 | 48 vendor补丁（Iroh37+Noq11）逐项覆盖/上游等价/退出、Noq1/5缺口、准确最终源可信Quality、Mac原生App/代理/多机/手机/能耗；证据前不升级/删补丁 |

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

最近给用户完整计划粗估3–5开发日，受新回归/真机/能耗影响，非完成承诺。下一步继续246实际wire因果与新快路/业务验证，同时把恢复的Mac SSH接入后续N4；不要在阶段绿后停工。
