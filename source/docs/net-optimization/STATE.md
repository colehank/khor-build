# feat/net 当前状态与恢复入口

更新：2026-11-09 22:32 UTC（本地2026-11-10）。root串行，无子智能体。

完整目标仍是完成[PLAN](PLAN.md)的N0–N5，尚未完成。用户已授权实施/远端验证并多次要求继续；U14明确不能因goal服务paused或阶段通过自行停止。后台调度状态不是本轮停止/授权门，不另建目标，不伪造complete/blocked。速度和体验优先，个人可信设备网络；无合main、发布、中继采购/生产部署授权。

本页重写前逐字内容在[HISTORY-20261109-before-wan-access.md](HISTORY-20261109-before-wan-access.md)，再沿其历史链。所有旧真实红/失败保留；新绿不能回填旧因果。完整硬要求/协议见[REQUIREMENTS](REQUIREMENTS.md)、[ACCEPTANCE](ACCEPTANCE.md)，设计边界见[NEXT-DESIGN](NEXT-DESIGN.md)。

## 先核当前句柄，禁止盲重发

- **256 Mac42713674809**，request3b53b13bb8f043f29014d9067666c360，baseff608e91/tree7d0c7467/snapshotb13e4398/controlbe11104。只验证WAN worker构建，未运行居民。Turing下一build-only将复用此完整source，在253终态后才启动，避免污染计时；以新receipt核实际状态。
- **下一N1实际outer各副本快照诊断**：只保留WeakConnectionHandle，不延长实际Connection生命；initiator用原5秒look，middle/receiver仅在诊断开启时每5秒读actual path facts，不加probe。默认splice/ended直接await，无新增timer；新增诊断醒来不能作普通性能对照。目的是在253这种“inner Custom消失”前核实际四个outer端，而非设备汇总/另一健康副本。尚待format/编译/真实诊断，不改选路/回收策略。
- **253样本诊断已终态**：turing-20261109T222815Z-b033159a，246.74s/rc0/原group空；准确原source、samples模式1，124/124可达但60秒未reported双向再hold10s。inner只见+约0.58s Custom→Relay故障Go，未见20秒内回Custom确认；X+21.5/+26.5s已无Custom→Unused回收，+56.4s重新attach serial3。不是单纯固定确认门7.5s可概括；实际outer carrier副本需要单独快照。不把详细日志当普通SLO对照。
- **255首次WAN仅编译失败**：42712837277/requestb8c661be41984a3a920027748ca7e76e，basecba63d7b/tree48a934a9/snapshotcd8f0d29/controlbe11104。root fmt通过、Rust example因DeviceInfo没有Serialize而失败，没有启动Node或产出成功manifest。只将devices控制回复显式映射id/name，避免序列化无关设备字段；继续build-only验。lease supervisor/真实清理合同尚未实现，不称跨网已通。
- **当前唯一Turing重型任务253**：turing-20261109T222815Z-b033159a，netlab-peer-mobility-samples，PID1217909/child以execution核；复用251/249完整source c01c72de/tree4d493502/snapshote3be1244，control ef0b1ab021802a17710a6748e373ea12b87bf52d。只是额外samples debug诊断，不混普通性能/放宽门。后续本地WAN代码不混入。
- **254 Mac42712203779仅WAN草稿format前置失败**，requesta04744c77223427b86f4d945de8448b0，base73cd2166/tree2d2b4be3/snapshot05532efd/controlef0b1ab。尚未编译/执行；远端format.patch原样应用。源码/transport/9control blobs/hash按verification核，不把格式失败记成资源/业务反例。
- **build-only控制be11104f0149637c918456bbfba5329578aea096已push**：net-wan-build在原guard内仅cargo build khor-node example、核cache二进制是本UID正规非symlink可执行文件并写source/hash/path/size manifest，明确executed=false。普通artifact不含二进制；cache路径可变，真正runtime copy前必须在guard下重核hash。12个Python stub/小Git控制合同绿，含普通/缺文件/伪symlink与bootstrap输入校验；本机不运行Rust。下一Mac只编译，不启动居民；lease supervisor与真实EOF/kill/join合同及双host实测仍未完成。
- **251/252已终态，当前无Turing重型在途**：251 turing-20261109T221609Z-bf323de3，168.30s/rc0/原group空，124/124ok、原hold10420/10406ms、replug298ms。实际outer Direct约+1.2s，inner selected carried约+7.4/+7.5s。after AB p50155031us、BA110693us，但两个方向顺序采样（AB较早），不能据差值称持续方向性故障；实际request wire在+7873ms也回Carried。3resident没有tuple mismatch/refresh，M移动不是247的被动NAT缓存反例；仍要收敛约6秒inner确认/恢复等待。252 Mac42710424112同249准确source/control，fmt/net67+1ignored/hop12/wires11/journal30/budget20（49.89s）全绿，完整source/transport/9control blobs/hash核。
- **下一独立样本诊断**：保留251整source/fixture，另netlab-peer-mobility-samples只把diagnostic=lifecycle改成1（samples debug），记录Moving/确认/质量时序；这不是无诊断性能比较，不能放宽原门。控制正在单独stub验证/提交，句柄后续按ledger核。
- **N4跨网进程源码初稿**：crates/node/examples/net_wan_probe.rs。Unix私有lease socket、fresh root/role/run-id/权限核、Node::serve+open_as、仅mint/pair/devices/links/fixture-read/stop，peer身份与machine对应，租约EOF/绝对期限时stop_serving并join；Windows仅明确unsupported入口。尚未远端format/编译或启动resident，supervisor与EOF/kill合同/双host实际验收未写完，不能当N4已通。首次mint/pair能力只过私有socket，不进stdout/artifact。
- **249严格NAT实物理红绿闭合、250 Mac vendor同源全绿**：249 turing-20261109T221123Z-cf202137，136.20s/rc0/原group空；原整个fixture SHA ac97726906429a5dd63dd91b5c9ed99624c273d9a253f7facb916ffcf47e8a9b和同一独立严格合同字节未变，30/30实际warm remote streams选中新Direct :50002、0 cached/actual mismatch、实际refresh1次。124/124全部ok，after AB/BA p50 21115/21297us；首BA748962us仍需优化。250 Mac42709847858准确同snapshot/control，fmt/Noq438/iroh路径36/选家9全绿；source/transport/9control blobs/hash核。nat-tuple-red-green.json、249/nat-selection-contract.json留证；只有单病例/单轮，不冒称p99/整个U12完成。
- **251已终态**：turing-20261109T221609Z-bf323de3，netlab-peer-mobility，沿249完整source/control，PID按receipt。原peer_relay_replug_v1整fixture保持，复验middle换网与实际承载。252 Mac另跑同249源net-node，句柄待提交收据核；不同主机，不污染Turing计时。
- **249已终态**：turing-20261109T221123Z-cf202137，PID1198443/child1198657，netlab-nat-rebind-lifecycle；basec01c72de/tree4d49350250ac97e03e0bac4b5974e1de2b97e949/snapshote3be1244d4741390d296205f1a5372c80f9d043c/controlfc204ac。候选功能4a0c359b。最后已实际编译、r0物理1/1绿，r1在途；原整个rebind.rs与独立严格合同SHA均核不变。
- **250 Mac42709847858已终态绿**，requesteb05eac32faa47e5bb25772d55ae925a，net-vendor，与249完整source/control一致，运行Noq整个lib、iroh remote_state与khor_home，非仅便携net-unit。终态按句柄核。
- 产品worktree `/Volumes/disk0/Khor/worktrees/current/khor-net`，feat/net；最新功能2654cbdf，后续增加诊断与文档，实际HEAD/diff以Git为准。control `/Volumes/disk0/Khor/worktrees/current/khor-build-net`，feat/net，已push **fc204ac4764fab1bcb71842b0e95ce9b49e4e7a5**。
- **247已终态采证，无Turing在途**：turing-20261109T220331Z-f5000d4f，netlab-nat-rebind-lifecycle，PID以receipt为准。base3852c0cc/treeb6c2174e08297d6c1110de612f73575bff6bc09b/snapshotbe5032f83a7cc9b986ad8f3ffde5badb9f0b35a7/controlfc204ac；只加tuple mismatch诊断，旧行为保持。246已完整终态。
- **248 Mac42708373196已终态全绿**，request0df1499e94ed49488b4a86e938c23ce0；与247同base/tree，但snapshot6cacbb659e00b3bc8550eca637f7a119262774d5（created_at不同，不称同完整snapshot）。便携net-node预检，结果按实际句柄核。
- **245 Mac42706942409终态全绿**，request5479e3b5ea81417bb56fc6f167653dc2；与246完整source/control一致。fmt/net67+1ignored/hop12/wires11/journal30/budget20（49.90s），source/transport/9control blobs/artifact hashes核。证明新增request_stream_opened编译与便携合同，不替代Linux/真跨网验收。
- 248及此前所有任务已终态采证。`evidence/net/source-refs.json`、`turing-next-matrix.json`、每run receipt/collection/report hashes是真实句柄入口。最新批量refs审计source-refs-audit-20261109-owner-green-243.json逐核243 refs，后续逐条先pin；应在246终态后刷新全审计。

## 最新实证与未闭合问题

**247已终态，NAT错路因果取得真实反例**：138.17s/rc0/原group空、124/124可达绿；B实际Noq conn/path1已迁50002、iroh仍缓存50001，69条逐次不一致。独立合同nat-selection-contract.json实际红（warm remote 30次0通过、新tuple未selected）；原整个rebind.rs SHA ac97726906429a5dd63dd91b5c9ed99624c273d9a253f7facb916ffcf47e8a9b保留不动。下一候选只在既有reselect前reconcile真实tuple、移除变更path旧Reading、重新登记/强制apply；不改策略/计时/并发/发包。尚待同整fixture真实绿与vendor合同，不称已完成。248 Mac同代码tree便携全绿（67/12/11/30/20），不同snapshot元数据明确保留。


**246已终态**：132.16s/rc0/原group空，r0+r1可达2/2、124/124ok；after AB p5022166us、BA123616us，首BA660726us。nat-rebind-calibration.json与request-wire-correlation.json核141条非重复lifecycle；实际after-change每方向31个request streams，B业务wire已有新:50002但所有路径selected=false，另wire仍selected旧:50001。247只增加Noq真实tuple vs iroh缓存的gated mismatch诊断，保持旧行为查因，不盲改策略。


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
| N5 | 原48＋新增38候选，共49 vendor补丁（Iroh38+Noq11）逐项覆盖/上游等价/退出、Noq1/5缺口、准确最终源可信Quality、Mac原生App/代理/多机/手机/能耗；证据前不升级/删补丁 |

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
