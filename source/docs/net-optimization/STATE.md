# feat/net 当前状态与接续入口

更新于2026-11-11 14:55 UTC。完整目标“完成计划”仍 **active、未完成、未阻塞**，范围为 [PLAN](PLAN.md) 的 N0–N5，root串行，不派发子智能体。本页替代旧页中所有“当前/在途/下一步”。上一份STATE逐字保存于 [完整历史](HISTORY-20261111-through-kernel-calibration.md)，SHA `8172caf6900ae2196273ecdf86a4b8d17572537299392bfa1b5b1322eefbe540`；它继续链接此前所有历史与失败原件。历史段落只代表当时，实际worktree、收据、日志优先，不按摘要覆盖未提交内容。

## 当前执行入口：观察器六轮已核，最新生产长稳在途

**turing-20261111T145336Z-bc4dbca9** 已389.740秒0、25files/identity/原group空，固定六轮全绿，共4320请求全ok、六份16MiB完全一致、实际TC/探针/全部人口已按原始行独立复核。`observer-comparison-v1.json`、helper audit、ledger均终态，不重发。两模式bulk目录p95均109–113ms、终端81–83ms；记录方式没有稳定改善。probe多数p95 1–2ms；目录数百ms计划迟到主要是前一请求未完成形成的串行积压，残余dispatch p95约1.2–2.1ms。完整数表/分解与限制在WORKLOAD；不当完整N3、CPU/I/O、p99或手机接受。

**唯一新Turing：turing-20261111T150352Z-de04bcc1**，PID **67067**，15:04:14 UTC launch；task `netlab-stability` / revision `effective-route-stability-v3` / candidate / 原control68cadb57。stage exec85417已终态。准确源是已核Mac/Linux/四peer/动态27/恢复6的生产3de冻结source-v3，下文SHA不变；执行原p1/p1j/p1s/p5单场景串行矩阵，包含指定stall/busy，不与其他Turing构建或计时混跑。当前尚未终态，不重发。root同时可source-only增加现有opt-in诊断字段、保留实际业务wire/stream和server各阶段事实；不会据此直接改队列策略或放宽UX门。

冻结源 **0b9454472767d16bc4ff0e44b14bcf3a10256e3c** / tree **725b442271fd07d85a8aeaaac84598cd3f5028e5** / snapshot **2a969d8cb0b75d48cba18792d40bffbd04df024c**，clean/0excluded，证据根 `candidate-workload-observer-source-v3.json`。控制 **7e439456cd5941607270add201d33589ebb371ba**。本页随后docs-only提交不改变冻结实验。Rust fixture与已在Linux编译通过的a9ef逐字相同；新增最后提交仅严格parser/18子例，不能把编译或kernel运行当w3/w1/w2运行绿。

先真实执行 `workload::observer::w3_` 校准拥有者/取消记录；若失败，不执行六轮，原失败及parser结果保留。通过后依预先固定顺序 **Streaming、Buffered、Buffered、Streaming、Streaming、Buffered** 串行执行六个完整场景；一轮失败仍保留后续人口，首个非零退出码不覆盖。source-owned `scripts/netlab-observer.py` 校验原始日志、720人口/轮、真实20ms探针、容量/溢出、实际TC JSON/文本/接口读回和独立guard，不以总job绿单独接受行为。

协议 **resident_mixed_workload_v4** 详见 [WORKLOAD](WORKLOAD.md)。真实目录/PTY/16MiB，原三阶段各两类120请求/100ms计划/每类一个在途、调用2秒/bulk120秒/程序就绪与预热10秒总界均保持。两模式相同内存与父future定时探针；仅逐条打印或阶段末打印不同，flush单独计时。Attempt guard保留ok/error/timeout/blocked/cancelled/aborted，drop拥有者abort真实任务；无后台游离测量任务。缺样本/缺summary/探针上限/溢出不能通过。三次/模式、奇数配对只部分平衡，**不是产品加速、p99、完整N3或手机能耗接受**。close仍只是API成功，独立host/child退出待补。

control纯Python/shell stub **13项9.707秒绿**（含固定顺序、guard失败阻止后续、case失败不中断全人口/保留首错误），source metadata **1项18子例0.698秒绿**，bash语法/diff过，零本地Rust/产品执行。精确SHA/检查见证据根 `control-workload-observer-tests-v1.json`；OPS仅加入固定task allowlist，原UID/锁/namespace/资源/期限/launch/ledger门不变，helper审计 `control-workload-observer-helper-audit-v1.json`。

## 最新已核终态

证据根 `/Volumes/disk0/Khor/evidence/net`，以下均已收原件、匹配source/control、核identity/原group为空并写source-refs ledger终态；不重发。

| 验证 | 精确句柄 | 结果和边界 |
| --- | --- | --- |
| 最新生产动态三轮 | **turing-20261111T141304Z-f2f17750** | 1309.406秒0，15files；p2/p3/p7及相位矩阵 **27绿**，每个实际RPC started/finished匹配且ok，p3全在原1秒门。`dynamic-result-v1.json`。空闲回好路仍16–17秒（原20秒门内），活跃约2.2–3.15秒；绿不等于极致体验、严重stall或长稳关闭。 |
| 观察器Linux编译+独立kernel三轮 | **turing-20261111T144048Z-f970f1d5** | 104.751秒0，17files；源a9ef/tree699cf94ccd2dd7f8bc97487019aa032181b27201/snapshota5bb7fe0754f296cd80c3f03713f0bb60dadda9c，controlea44ab27af3011821d188e0a500ad91b20ed17b1。完整fixture编译，三w0 kernel绿；**未执行w3/w1/w2**。实际TC单位/两方向数据另核 `kernel-profile-verification-v1.json`，full_N3_accepted=false。它是下一Turing任务的准确predecessor。 |
| Mac观察器格式 | **43039285003** | request42116d54606f427d8931f52920d208fc，exact artifact13864796317，ZIP SHAd542cdc8b352f44494f944f83953427b25d89b30b6906da6cca0d00a3bd735e6；9files/11controls/identity，**仅格式失败、0行为样本**。`format-only-verification-v1.json`。准确patch已check/apply提交5b57160c13cf764fd0ff0a111ef7838c1e846335，随后a9ef编译通过。旧失败未改绿。 |
| 独立kernel原三轮 | **turing-20261111T140913Z-141b0ea2** | 94.220秒0，17files，df5b3f6a/controlea44；三w0全字节一致、实际12份TC JSON/文本、minRTT40.2ms和双向约7.52/7.55Mbit/s。JSON rate bytes/s、delay秒、lat微秒相互核对；真实qdisc drops18/21每轮保留，零随机loss不等于零drop。仅校准kernel模型。 |
| 生产恢复三轮 | **turing-20261111T134417Z-ce3b1222** | 568.791秒0，15files，p2/p3全6绿；p3黑洞后228/225/227真实目录RPC、worst644.5/664.6/590.5ms、0失败/0retired。`recovery-result-v1.json`，不是PTY或全部移动。 |
| 旧v3真实混合负载 | **turing-20261111T133630Z-2aff8ae2** | 216.845秒0，17files，3×720=2160全ok、三16MiB逐字节一致、每类每轮120实际bulk重叠。`workload-functional-verification-v1.json`/`workload-latency-decomposition-v1.json`。目录服务p95约107–112ms（空载62–66），终端81–84ms（43–44）；计划→完成目录289–320ms含观察器积压，第三轮空载自身85–93ms迟到，原因未证，不归因CPU/I/O或产品队列。旧cohort未做实际TC读回，不能追填为已校准或当v4对照。 |

## 已验证生产候选与实现边界

生产候选 **3de372a026488a64a794935600e47db29f2f6faf** / tree **ae76cd646749ca624d5e3e98af01f96099ebd0e0** / snapshot **a9f46eaa909b8cf45663395912798effb31b7261**，`candidate-effective-route-source-v3.json`。后续observer/kernel/parser改动均为测试/工具/文档，相对此生产文件和Cargo锁无改变。完整原件索引与SHA见 `source-refs.json` 及历史。

- Mac完整Node **43022962905**、Linux完整Node **turing-20261111T131248Z-5c2a0339** 同源全12绿；含六initialCustom、两incoming实际执行。短8 **678b0ebd** 不替代全12。
- Mac vendor筛选 **43023967463**：443Noq/58path/11home绿，不是全部59补丁专项或完整vendor。
- 四peer **turing-20261111T125643Z-6970ad46**：699.061秒0、37files、三轮全6绿。新f4r首内容1.263/1.253/1.223秒，每轮30持续样本约15.3–15.5秒，held/continuous/selected_every/carrier_both true。原始两端31响应/31匹配stream/每端单一business wire/一个outer commit，无detach/I/O-ended、首次在M恢复前。`physical-result-v1.json`。原f4控制业务最长仍3.6–3.7秒，不当所有长尾关闭。
- 已有carrier时原generic和一个显式Custom普通握手有界竞争；early保留原fast0RTT与一个借用Custom普通握手。原总caller/deadline、最多2活动、写/0RTT/认证resting边界保持；缓存失败不提前取消carrier。无新常驻map/task/timer/强Node，额外握手CPU/字节/能耗尚未测。
- Iroh49只在共享preferred不在本conn且唯一established/open路径时报告实际Noq fallback；0/多条仍Unknown，shared selection/发送/PathStatus不改。但rtt_of/Patience因此取实际RTT，质量/成本/长稳仍要验证。主vendor **59=Iroh1–41/45–49+Noq1–13**，42–44隔离不动，上游等价未新核。
- incoming所有权合同保留真实无marker carried-only前置和另一个真实同身份Direct/TLS端点。四installed-carrier原合同/1秒门和快速Direct负控保持。carrier在调用开始后到达/own prep已结束的唤醒仍是待覆盖假说，不凭入站业务活推定出站Link活，不提前加轮询。

## 保留失败与完整剩余范围

完整原始失败沿本页历史链与ledger保留，不删除、不放宽门或覆盖为绿。最近包括：Mac观察器/校准初始仅格式红；bd7e5f6c三项Rust绿但manifest被harness前缀导致总parser红，原件不改，修唯一marker后新141b三轮真实复验；混合负载最初E0599、三轮PTY预热超时、v2仍超时各有独立原件。后者定位CLI host_main startup=None、原Node host_args读环境变量未走CLI；当前fixture沿既有term_echo的显式OpenOnce argv入口，没有顺带改产品命令引擎。观察器格式patch前自己的probe草稿保存 `ops/observer-probe-window-before-format-v1.patch`，SHA32670fa1a6858125b20d947bdb6f862144554165e50e49c245d5ead70b050c37，再以a9ef重新实施，无用户draft覆盖。

旧573最终false/None/0、166 true/Unknown/1仍原门红，eb72前置红、412清理panic、旧物理f4r held/selected红、原f4 first-leg2059/2325ms与最长5.2/4.9秒均保留。严重6aa stall/quality_steps0/RTT不对称/carrier约21秒终止、约8秒尾巴、Busy854、resume32timeout、quiet/f4s/coldstart、Noq13物理7.3/5.4秒、截断8f3、Iroh47四项10秒/7.499秒仍未闭环。旧源长稳4绿、Noq704/15bulk不当当前完整接受，3样本不称p99。

完整N0–N5保持：
- N0最终UX门/足量p99/相同条件完整前后对照报告。
- N1恢复长尾、严重stall/质量/跨连接测量、quiet和比较成本；新同源动态27/恢复6绿已补，但最新长稳和严重场景未闭。
- N2真实同conn独立/同时IPv4/IPv6迁移、LOST身份、完整取消/owner/迟到/撤销、carrier在开始后到达。
- N3观察器对照后核真实wire/stream/队列/许可/发送缓冲/背压，比较共享/分连接与公平方案，再覆盖1/3/5设备CPU/字节/唤醒/能耗、独立host/child退出。
- N4完整双腿Direct/Relay/SSH、唯一/多middle、prepared/混合版本、独立IPv4/IPv6移动、整机/出口/登记故障域及运维设计。
- N5全部59vendor上游/退出、Noq1/5专项、最终Quality/native/proxy/多机/iPhone与一致报告。

MacLab/Turing同LAN；macserver是另一构建机，用户报告异网。localhost iPhone历史安装0.2.0，曾反复关开App才恢复，蜂窝截图仅hinton直连29ms。**候选未安装、无phone sampler、旧测试窗口已过期**；可信GitHub/TestFlight具体构建就绪后再交接真机，旧手机观察不是候选验收。

2026-11-11进度口径：上一份完整交付3–5天估计始于11-10，当前仍日级，暂估剩余2–4天、低置信，受严重故障复现、长稳和真机结果影响；不是从每次compact重新计3–5天。当前六轮预计10–15分钟出该批原件，不把单批耗时当完整交付。此前小时级预估过乐观。未宣称整体百分比或已完成。

## 执行约束与下一步

产品 `/Volumes/disk0/Khor/worktrees/current/khor-net` branch feat/net；控制 `/Volumes/disk0/Khor/worktrees/current/khor-build-net` branch feat/net本地HEAD7e439456，旧两个untracked pycache保留。新control仅用于observer等指定SSH任务；原Mac/full/physical/生产dynamic继续 **68cadb57a0ed72d40a4ca9a843e86b9c6797bf34**。控制未push，stage用准确本地Git对象；Mac admission/GH dispatch原门未改。

本机source-only，无Cargo/Rust/Node/Xcode/fmt/产品执行；Git/Python源码元数据和control stub允许。Turing严格串行UID1110/flock/user-netns `/tmp/khor-netlab-feat-net-1110`，Rust1.95/jobs16/nice10/50GiB起40停/90min原门，无sudo/host网改动；Mac独立jobs2/shared cache/原门可并行。SSH BatchMode/ForwardAgent=no/StrictHostKeyChecking。实现与上述验证已授权，无main合并/发布/采购/生产部署、已装App/config/store/resident修改授权；fixture仅namespace scratch。隔离Iroh42–44树detached2ef6bee5不动。

图谱先行，本树无.codegraph不擅建，CRG部分/滞后，0flow不证明无影响；explore/debug/review技能已使用。source-refs.json仅root单写，stage/register未终态前不并行ledger写；timeout/EOF不是终态，查同receipt/PID/Actions不重发。当前stage已终态。

下一步查de04bcc1原矩阵/PID/execution，终态收原件/identity并独立核原四项门；root source-only补充opt-in真实wire/stream/阶段事实与固定诊断任务，之后基于真实许可/队列定位瓶颈。继续完整N0–N5，目标active，无外部阻塞。
