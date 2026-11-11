# feat/net 当前状态与接续入口

更新于2026-11-11 12:08 UTC。完整N0–N5目标“完成计划”**active**，本轮有新真实物理证据、Mac同源12组验证与新四个已有carrier合同；未完成、未阻塞，root串行，不派发子智能体。上份STATE逐字归档到[业务连续性v2以前](HISTORY-20261111-through-business-continuity-v2.md)，SHA **909d6598e60cb5fa05fd66d4a998682f0dd79f443c3fd301cbdfbefe4d23ab0a**，它继续链接hint、comparison和全部更早失败。原始日志/准确收据优先，不能按摘要覆盖工作树。

## 旧行为573的两个任务（12:13 UTC已终态，收集在继续）

冻结 **573a5880b31368a29b51197266f623bbd6702fcb**，tree **7eb044bc4f13d08e0ee399b06ec52e0de969d5a9** /snapshot **9975d20990d786026b8f662fecb9474923315edd**，candidate-installed-carrier-contract-source-v3.json，clean/0excluded。它只修四个新真实合同的清理顺序，**不含产品修复草稿**。

- 唯一Turing **turing-20261111T120544Z-5df2c88e**，PID **4015240**，12:06:05 UTC launch，net-node/role baseline/control **68cadb57a0ed72d40a4ca9a843e86b9c6797bf34**，stage exec14830终态，只提交一次。按同receipt/execution/PID实际核；不要另起Turing编译/测试。
- Mac **Actions43017591561**，request **10e7b6bcae6f45808c3dd7cfd88b1fad**，同573源net-node/control68cad；submit exec36816终态，登记进程等实际句柄终态后才能再写ledger；不重发。

前置是同身份旧IP连接仍活、调用前后实际Sessions/SessionRows都成功；另一个只有Custom的同身份端点已安装carrier。分别只允许Custom或Direct接受新握手，走Node普通line/early ask_on，要求原1秒caller内正确路径/唯一实际响应/旧IP和既有carrier保持。两救通+两快Direct负控，不是物理两relay腿/完整移动或accepted0RTT复制覆盖。

此前test-only **412edaf46d1b8f9f57ecc06c1aface161ef8a79d** /treeec65656dde1a204bcb23106e4e345c32a1d8a593/snapshot7548fe49d3d77ec3c583a4d29b13b215895cb2ed，在Linux **1fda87ba** 已353.454秒终态101、14files/identity/空group，net68+1ignored/hop89/wires16/journal30绿，budget34pass/2FAIL；Mac **43016439707** exact13853451235/ZIP SHA992f292e48902149d7107b5e1e9a258e4d4691de8775b80c0d0767cfeec73540，13files/11controls/identity，budget30pass/2FAIL。**两端失败均先在待accept的测试服务端清理时Option None panic，再被cleanup assertion遮住，未到最终业务断言，不能称产品真实红。** 两快Direct负控绿、warm调用前后都已到达。fixture-cleanup-partial-v1.json与ledger保留；本次573先abort/join该owner再关endpoint，判据/1秒不变。

其首轮721源只格式：Linux02c8dbd2 48.090秒1/10files/identity/空group；Mac43015707721 exact13852756952/ZIP SHA f442a0a58c3cf9cc62398b5c21ed4d7a897de87ec62084b58584bff176e5ab78，9files/11controls/identity。精确format.patch两端逐字相同并已应用；0行为组，均已终态分类不再poll。

## 当前未提交草稿，必须保留

HEAD为573（或随后docs-only），另核实际git diff。root自己的两文件草稿已保存/校验并恢复：`/Volumes/disk0/Khor/evidence/net/ops/installed-carrier-draft-v1.patch`，SHA **1b1c8f683011ffd57fe064e82e43b3706e97e1d7d28d089672639954bd87b441**。

1. `link.rs`：已有carrier时，普通拨号把原第二个identity-only槽换为立即显式Custom普通握手，原generic路径继续竞争，最多2活动；early同原期限借用一个Custom普通握手future，原fast early仍可胜出。一个缓存路失败不能提前终止在途carrier，认证resting仍立即决定；原early task输掉时被drop/abort，写/0RTT准入不扩大，不增map/timer/常驻强Node。**草稿尚未编译/验证**，先核573旧行为真实最终断言，再决定提交/冻结候选；不得将root dirty snapshot冒充baseline。
2. 四peer `prepared_physical.rs`：仅额外记录失败样本两端hop/via并继续取完30样本，held累计false仍使原门失败；分开打印selected_every/carrier_both。所有原业务/时间/连续性断言保留。为了采证不再first-false早停，故含失败时观察更长，不能与旧v2当同一测量时长；产品实现与测量改动要分别归因。

## 最新完整四peer结果：仍三红

**b1a57db97943d07b33ca040aa69180c6bb237d87** /tree0651fe376493b9825405b8c0b449e538e4cadd27/snapshot75536ca3a37ba0883eace011523d28a7b5a198ec，candidate-business-continuity-source-v2.json。

- Mac **43014256036** exact13852421273/ZIP SHA **24e0d3b428a3b2ae1c4992456deb446db45f9a9d1df3cabcb455056f38356d32**，20files/11controls/identity，全12（hop89/budget28/net68+1ignored）绿，node-regression-verification-v1.json/ledger完成。
- Turing **2f2a53e1** 已668.578秒终态101、37files/identity/原group空。新f4r首内容 **1.220730562/1.251522755/1.241848942秒**，都M OS-stopped/two_asks/viaN=true；前两轮持续第一样本held=false，只有双方2响应/stream匹配/每端同wire；第三持续30样本、15.326926679秒、双方31响应/stream匹配/每端同wire、两端单一outer serial无detach/I/O-ended，但第一响应时唯一Custom path的selected=false，continuous仍false。**三项原红不追认绿**。原f4三控制green/最长业务3.6/3.7/3.7秒；不关闭旧源两红。
- 证据：该run的physical-result-v1.json完整6行与日志SHA；根 business-continuity-decomposition-v1.json 保存第三轮每条真实完成与outer事实。v2日志含诊断开销，6项新诊断，不是原9项/普通SLO。

更早源963的a653cfe2：三次首内容1.251580165/1.224817780/1.218185943秒、各30业务成功，但started-only wire helper为空，三原红保留。**原f4一/三轮红**：first-leg-heard后2059/2325ms超过原1.5秒门、最长业务5.2/4.9秒；第二绿3.5秒。old-f4-business-owner-audit-v1.json证明实际业务wire也从carried退Relay后恢复，不能全部归为日志错误。

## scoped初始路径已验证的准确边界

源 **9634737a01dea34a4dc7fb54d9a32a352a9f5018** /tree52c536154a3a51829c64e5c167ac7965b2792a93/snapshote48e12746c900206c4c8b59d0b9c010a234a7018，双OS Node全12：Mac43010224456（hop89/budget28）与Linux3fc797b0（369.804秒0、21files/identity/空group，hop89/budget32）；Mac vendor43012074442 exact13850403233/ZIP SHA5307a4609011bc80d83efd7f11680bbc05ca3a6c5755e871e5ad82f5de7679c9，13files/11controls/identity、443Noq/58path/11home。per-run证明和ledger均终态，不重poll。

Iroh48每次握手显式Custom初始目的地，不清共享selected、不改TLS目标id；Node给新准备carrier一个受原deadline约束的普通握手机会。实际PathId0/TLS/echo、错误目标身份明确拒绝和未显式提供Custom拒绝通过。新增48仍**候选**：主58=Iroh1–41/45–48+Noq1–13，42–44隔离。上述筛选绿不证明所有补丁专项/Noq1、5/完整vendor或移动/能耗；已有carrier分支仍缺，所以有本轮四合同。

之前只解除无活连接selected的d521在真实三轮依然红（live=true/registered1/reset=false），已撤回该行为；其Mac443/58/11绿不能当scoped新版验收。原420旧四peer三超时也保留。准确中间format/宏/错误身份listener联合等待失败均见历史与独立proof，不当同一种产品红。

## 已明确的观测与设计欠项

`PathStateReceiver::get`取actor快照；record_selected按peer共享fourtuple在各conn的open-path列表匹配，可能找不到。Noq paths.rs1085–1095明确无Available时Backup承担数据；现有Wires::carrier_in_use也接受非空且全carried列表。v2首次响应实际完成但selected=false正说明shared selection不能直接等同此conn实际业务路。**不能直接放宽v2为绿**：后续用真实每path帧/字节或实际Noq调度证据修正观测语义，或修复provider准确性；保留所有旧红并重新冻结/运行。

links_now只读outgoing Main/Tunnel的settled word，served carried业务不直接代表该端下一次outgoing操作；原两端Link held失败需新草稿快照定位，不能仅靠“元数据延迟”归因。已有carrier初始路竞争、每connection有效测量和N1旧恢复长尾继续分别审查。

## 先前完成与必须保留的失败

迟到OK原子提交源67929已同源双OS全12/hop89，通过实际旧Ask pending、新middle TLS/echo完成释放后旧OK不能再发布；旧976测试88pass/1真实最终FAIL。network-generation→MISSED→Links现有锁内验证/提交，无await/新常驻状态；目标端首帧策略的并发/撤销仍欠。

hint源0d1c双OS全12/hop87、准备c2b双OS全12/hop79、比较a73双OS全12/hop81。原3peer一middle矩阵c2b三轮9绿，f4i28.2/17.8/34.6秒；a73三轮9绿，8.1/8.6/8.1秒。f4s第三候选9.6秒比9.3更差，第二有Idle→Forwarded和Relay6.1→Carried13.1瞬态；三样本不是p99，f4i公共Relay可用下回peer收敛不是全网断开首业务。根comparison-context-original-hop-result-v1.json与各per-run checker保留，不重跑旧job。

历史严重6aa无Go/quality_steps0、RTT不对称/缺失、carrier约21秒结束仍未闭环；Busy854、resume32timeout、quiet/f4s/coldstart、Noq13物理7.3/5.4秒及截断8f3均保留。Iroh47固定4项两C近4秒绿、B初始10秒超时及7.499秒红，不追加第五/不称p99。8cd动态27绿/长稳4绿属于旧源。Noq拥塞704原样本/15bulk与独立FIFO只是底层，不是真Node/手机。

## 完整范围，继续N0–N5

N0最终UX门/足量p99/同条件完整对照；N1约8秒尾巴/严重stall/质量与跨连接测量/quiet比较成本；N2实际同conn独立/同时v4-v6迁移、LOST身份、完整取消与owner/迟到/撤销、已有carrier；N3真Node交互+批量、共享/分连接/背压公平、1/3/5设备CPU/流量/唤醒/能耗；N4双腿Direct/Relay/SSH、唯一/多middle、prepared竞争/混合版本/地址移动/故障域；N5全部vendor上游/退出与Noq1/5专项、最新Quality/native/proxy/多机/iPhone/一致报告。局部绿不代替完整计划。

MacLab与Turing同LAN；macserver为另一构建机，用户报告异网。localhost iPhone旧0.2.0、历史反复关开App才恢复，截图蜂窝5GA仅hinton Direct29ms；**候选未安装、无phone sampler，旧窗口已过期**。最终可信GitHub/TestFlight具体构建后再交接真机。向用户全验收暂估3–5天低置信；此前按小时过乐观，不因compact重置或把阶段测试耗时当整体ETA。

## 执行与资源边界

产品 `/Volumes/disk0/Khor/worktrees/current/khor-net`，branch feat/net；控制树同current下khor-build-net固定68cad、原2 untracked pycache保留。Iroh42–44独立recovery-deadlines树detached2ef6bee5不动。证据 `/Volumes/disk0/Khor/evidence/net`，ops同根，source-refs.json仅root单写；stage/register未终态前不并行ledger写入，timeout/EOF不是终态、只查同receipt/PID/Actions不重发。

本机source-only，无Cargo/Rust/Node/Xcode/fmt/产品执行；Git/Python源码元数据允许。Turing严格串行UID1110/flock/user-netns/tmp/khor-netlab-feat-net-1110/Rust1.95/jobs16/nice10/50GiB起40停/90min原门，无sudo/host网改动。Mac独立jobs2/shared-cache/原门可与Turing并行；SSH BatchMode/ForwardAgent=no/StrictHostKeyChecking。全实现/这些远端验证已授权；无main合并/发布/采购/生产部署或已装App/config/store/resident修改授权。fixture只动远端namespace scratch。

图谱优先，本树无.codegraph不擅建；CRG滞后/部分覆盖，0flow不是无影响；debug/explore/review技能已使用。继续先核实际diff与两任务，然后按本页下一步，完整目标保持active。


## 12:13 UTC：旧实现真实最终业务两红，候选进入远端验证

Turing5df2c88e已355.398秒终态101、14files/identity/原group空，hop89/net68+1ignored/wires16/journal30绿；budget **34pass/2真实最终FAIL**。两项救通都到第310行实际业务断言，early=true/false均answered=false、path=None、requests=0；快Direct两负控绿、旧活IP调用前后继续应答/既有carrier保持，清理无panic。该最终旧行为可以作为产品红，不再与前一v2清理错误混淆。Mac43017591561已failure，exact artifact/identity收取继续，尚未按Linux结果代认Mac失败内容。

现将原两文件草稿提交为候选：普通第二个已有slot与early一个父借用future为已安装carrier提供显式Custom普通握手，原genericDirect/early先快者仍可胜，缓存路失败不取消尚在途carrier，认证resting仍决定。最多2活动、原拨号/caller期限、writes/0RTT门保持；已有快速Direct不等待新carrier。性能成本未测，不能称零开销。新四合同逐字保持，必须同一最终断言转绿。四peer额外sample hop/via与组件输出仅采证，held累计false仍失败且1.5秒/10秒/30样本不变；其新增失败后继续采样不回写旧v2绿。未做provider selected修复、未改v2 selected_every门。

先冻结新source并单次提交Mac/Turing net-node，回收准确同源全部12/四新合同；Turing前驱5df已结束，允许下一串行任务。继而四peer验证、provider实际数据路径与共享selected观测语义、旧f4恢复长尾及N3/N4/N5，完整目标active。
