# feat/net 当前状态与接续入口

更新于 2026-11-15T17:29:14.455082+00:00。活动目标“好 完成完整目标”已由用户暂停（2026-11-16 Asia/Singapore），goal服务已确认paused；完整[GOAL](GOAL.md)、[PLAN](PLAN.md) N0–N5与[REQUIREMENTS](REQUIREMENTS.md)未整体完成。root串行、不派子智能体；本机source-only，重型工作在Turing/macserver/授权khor-build。用户已授权实施与验证，未授权main合并、现装App替换、发行/TestFlight或生产Relay采购部署。

## 当前执行

- **已暂停，无在途任务**。2026-11-15T17:27Z核对goal状态paused；本地串行控制器PID2009已退出，phase finished。原campaign于2026-11-14T13:00:52Z完成全部六个业务块（另有一个校准块），全部已collect/验identity/owner/记录proof；common ledger无pending。不要因下方历史执行说明而重新启动控制器或测试。
- **整体验收仍未通过**。固定C组high bulk/directory完整360样本中21个超过原592518.1μs绝对上限，超过nearest-rank p95允许的18个，原性能门失败；S组第五业务块turing-20261114T125212Z-1290b1d0有原生功能检查失败。全部原结果及完整分母保留，不能把campaign phase completed当作通过。用户暂停期间不继续实现、汇总、构建或验证；恢复后先核本状态、原progress/proofs和当前实际源码。
- **性能冻结协议**E/pull-coexistence-comparison-plan-v1.json，SHA2561d0e1ddf5a74f9938574e51fa0b77fac8ed31644f0019b4d704b63eecc5e857c；control a76425ab3228b1b69ad302f3164e403549f8763f，task net-node，准确source6f7bbf43/treeaddd2218/CLI7454caa6。先唯一同源5case校准，再SCCSSC六块各low/high共12业务case/8640样本/24post-health。源/ELF/原数值门/CPU≥80%idle/PSI≤5/同UID无Cargo/全部owner门保持。数据适配器SHA固定；不启lifecycle日志、不编译、不替换失败；每块collect→identity→行为→recorder后才派下一块，C硬业务或原绝对门固定360人口不可恢复时停止未来块。控制器观察失败只重读同handle，未知stage/launch结果停控制器并保留intent，不能再派。
- **同源校准123130Z-493d2d67已闭：原退出1、4原生case通过/1冷TCP参考失败，原分层model准入通过**。17files/identity/owner；两个真实ownership/取消守卫、low TCP、独立UDP容量通过；high TCP内容/68echo/双向payload/server/几何及270–420ms RTT通过，而冷TCP平均率未达原40–60Mbit门，原Rust/native失败保留，绝不写成5/5。复用早已版本化MODEL-DECISION原分层谓词；E/pull-coexistence-comparison-design-v2.json在新校准/计时前明确继承，无原数值或几何门变化。当前S/C只改文件连接模式，校准不重复分模式。
- **布局v2确认122458Z-df120c3c已闭：0/192.624163秒**，16files/identity/owner、720/720、192MiB完整、2真实health、全部1130客户端回复及对应server回复、1128原RPC精确配对；连续缓存前缀2块、唯一extra后缀766块，目录Main/独立Tunnel与最终双向健康均成立。E/run/pull-coexistence-layout-verification-v2.json；原v1布局预测拒绝保留。三场布局均仅诊断，不混入性能人口。
- **新夹具布局v1两场均已闭原业务成功，但separate布局预测被拒绝**。shared121017Z-78156ce0为720/720＋192MiB＋2真实health/1128原RPC及Main/Tunnel均过；separate121511Z-dbd5e48c为0/192.683567秒、16files/identity/owner/全部720/完整payload/2health/1128配对均过，但expected_layout_verified=false：实际offset0和262144在原Main，其余766块在唯一extra，原合同误预测只能1个缓存块。原proof/v1协议不改，不称原布局门通过。
- **源码已证是夹具预测过窄**：准确link.rs10357–10518在opening等待时继续window.next，已准入future各自捕获line，prepared后保留pending并扩大窗口，随后新请求才用extra。强迫首块后停住等握手会破坏cached进展。E/pull-coexistence-layout-design-v2.json记录source SHA及原预测/反例；仅改新诊断合同，不改产品、不编译、不改变任何数值性能门。当前新场用于确认一般的连续缓存前缀→extra后缀，原v1拒绝始终保留。
- **当前产品普通Release已闭113837Z-6286623e**：0/560.512640秒，sourceff3360/tree4f0e7db7，23files/identity/owner/原fmt/完整依赖闭包/fresh CLI。CLI SHA2567454caa6b7347669b322c64fafa16ac2ae7c4eb86e699f5b3496578552847845；netlab c4d3da80b62be6933fe163bc173cd4b6713232e873c6a3d1bbbe65e5072b092b。E/run/pull-pool-release-compile-verification-v1.json。
- **新夹具普通Release已闭115519Z-81fbb62f**：0/242.800272秒，source6f7bbf43/treeaddd2218，仅workload.rs一个测试文件变化；CLI逐字仍7454caa6，netlab6ec6753b108877bf516b69a6451295fd11a07136cb1a9e0b7a7f2fb200040eed。E/run/pull-coexistence-compile-verification-v1.json。原标准317＋压力1及Mac314对相同产品继续适用，不因此重跑原组。
- **同源Mac原生回归GH43590483138/job132433946148已闭：314pass/0fail/原ignored1**（[Actions](https://github.com/colehank/khor-build/actions/runs/43590483138)）。准确source/control与11个control blobs、原12组每个test名、32个明确separate合同、14contact、4个原Linux-only cfg排除均核对；artifact14112262250/38847B/SHA2566c1cc072e169bc19954af8a25055db37645347aec4ea1afe380d767d422f2c56及CRC已保留。E/pull-pool-mac-verification-v1.json；独立Mac pending已闭、common ledger只追加completed，Turing pending未改。Rust1.95/arm64/jobs2/原admission门，未安装或发行App。原iPhone就绪窗口已过，不自动采样。

当前新夹具worktree **/Volumes/disk0/Khor/worktrees/current/khor-net-pull-coexistence-v1** / **experiment/net-pull-coexistence-v1**，clean；HEAD **6f7bbf43ca2eafdb2f2f4a02d33872fc88469714** / tree **addd2218068ce891bc8098c194ba59163cc52fd3** / snapshot **8c62bdd17b27ab453c76e4c42952e0f658c1ecd1**；E/pull-coexistence-source-v1.json。产品基准仍ff3360cd6af3629c869e6e47028636e2ee0d46c3/tree4f0e7db7（压力合同源），产品修复29b05533/treee71011ed。新源只加2个独立命名的speed-first ignored workload cases：原几何/字节/周期/调用期限/720样本保持；新case起终允许健康Direct/Relay/SSH/Forwarded，终端关闭后双向实际目录验证完整文件，两端最终Link live/no-down。原三个fixed-Direct case入口/条件及旧screen失败保留。默认仍shared，separate opt-in；未并入canonical feat/net或main。

**120420Z-f2ed80e5是明确零执行的启动准备错误**：root误用了netlab-workload task，控制runner在line221拒绝，未创建report、未入build driver、0编译/0业务/0样本。已核input source/control/bundle、PID与原PGID全部消失，launcher原件保留，E/run/startup-refusal-verification-v1.json闭ledger；无原退出码收据，不捏造0或1。正确既有net-node入口使用完全相同source/control/protocol/revision启动当前任务；不是替换原业务失败或观测超时重派。此后新布局/比较都用net-node。

**新数据适配器**E/ops/netlab-speed-first-observer-v1.py：静态检查发现旧observer数据解析器仍把direct_after列为必需。于收集/读取任何新布局日志前修正新verifier，只接受speed_first_coexistence_v1明确marker；保留全部原样本/几何/期限/footer核对，用live_after＋post_health_ok验证新case，并独立重算双向Link/两个实际health结果。原parser与旧结果未改，记录E/pull-coexistence-layout-data-adapter-preparation-v1.json。布局RPC配对预先明确剥离且完整保存2个post-health客户端回复＋2个服务端回复，仅原1128个测量RPC作同序配对；不能把新增回复当原样本或丢未知回复。

E=`/Volumes/disk0/Khor/evidence/net`；原receipt、远端execution及E/source-refs.json为实际调度依据。上版STATE逐字归档[HISTORY-20261114-through-pull-pool-resource-baseline](HISTORY-20261114-through-pull-pool-resource-baseline.md)，SHA256 `ec8e653e1163cc65000dce1474a61fadc125f227d61b45958f6755da191df8df`。其中在途/下一步是历史，不得据此重派。更早的全部证据、原失败、未执行人口通过历史档案与REQUIREMENTS保持，不被本入口替代。

## 最新完成的证据

- **当前新夹具shared布局121017Z-78156ce0已闭：0/192.628945秒**。16files/identity/owner、720原交互全ok、192MiB完整、2个post-health真实读均成功（307760/308974μs）、两端最终Link live/no-down；1130个客户端回复及对应服务端回复全部保存，原1128 RPC逐项配对且只有1对Main，独立Tunnel成立，0未知/缺失/重复。原Direct-after本次true，仅作记录。E/run/pull-coexistence-layout-verification-v1.json；有日志不作性能晋升。数据适配器额外3个明确synthetic data-only守卫均过：允许健康Relay而无Direct标签；即使原720全ok，post-health失败仍归functional false；拒绝旧fixed-Direct marker。E/pull-coexistence-data-parser-guards-v1.json，0产品执行/0性能样本，不混入业务人口。
- **资源基线104608Z-b34b246d：1pass/3fail**（sourceb292）。4实际extra满额，remote确已关闭1而旧Lease Arcs保留，第5peer仍被capacity拒绝，这是准确产品RED。by-value Endpoint完整cached文件/实际正校准通过。另2 owner用例原join(waiter,accept)未分离结果，不归为产品故障。
- **同产品owner诊断105305Z-6ccfbea6：2/2**（sourced4f，0/98.165546秒）。不改500ms owner、30ms waiter pending、原Incoming故障及5秒观察；只分开实际waiter结果与accept，记录一次aborted握手后继续接受。取消与到期均证明未到期waiter实际成功、额外容量归还、cached Main身份与真实Sessions保持；原单次accept观察器不足，未因此改产品拨号语义。原失败保留。
- **关闭容量修复110024Z-d65102fc：31/31**（source29b，0/118.244874秒）。仅在extra满额时，快照weak peers、释放map锁，回收实际close_reason非空的Lease permit后立即再试；健康4连接仍按原cap后退。相同资源测试证明remote已关、旧Arc仍活时peer5实际握手成功，其他3健康，旧Arc Drop不关闭新连接。无后台timer/probe、无抬高cap/期限。E/run/pull-pool-resource-native-verification-v1.json。
- **原标准12组110608Z-de8ff73e：317pass/0fail/原ignored1**（同source29b，0/502.085646秒），21files、identity/owner/原fmt和逐测试名闭证；31个明确separate合同及14个contact均过，多数旧case仍shared。E/run/pull-pool-resource-standard-node-verification-v1.json。**不重复此标准组。**
- **五对端全资源压力112137Z-7b003b5a：1/1**（当前ff3360，0/92.307242秒）。一个Turing进程内5个真实认证QUIC endpoints，非5物理设备/WAN性能。前4 peer持有32bulk/28实际decoded-ready与4extra，第5peer通过cached持有8foreground；新caller到期/取消均无RPC，满额时Sessions仍回应。只释放1个head便使该完整文件及随后另一个13块文件完成，其余11head仍保持。abort/join后11个实际STOP、全部credit/extra返还，5条cached Main原stable-id/真实Sessions保持；无重复offset。E/run/pull-pool-pressure-native-verification-v1.json。当前源是在产品相同的标准317上新增此1case，不能称一次完整318运行。

[PULL-POOL](PULL-POOL.md)、[PULL-POOL-DESIGN](PULL-POOL-DESIGN.md)与[PULL-COEXISTENCE](PULL-COEXISTENCE.md)记录归属、head cursor、bulk32/peer8＋foreground8资源边界。原公平性RED→同测试GREEN、旧27及全部准备/运输/写不重放边界保留；资源满额通过不等于Release性能或手机体验通过。

## 接下来的顺序与完整范围

**以下为恢复后的工作队列；当前用户已暂停，不执行。** 上述campaign已terminal，原session69980/PID2009均结束；第1项旧“正在执行”为历史调度约定，只能恢复证据，不能重派。

1. 当前root串行campaign正在执行。先看控制器实际liveness及实时pending；session69980存活时不手动推进同campaign。已过shared/v2 separate布局、两轮普通Release、317标准＋压力1、Mac314及旧三臂screen不重做。控制器terminal后查campaign phase与全部proof；stopped保持未执行人口，未知结果恢复原receipt。
2. campaign terminal后只做stdlib数据审计E/ops/summarize-pull-coexistence-comparison-v1.py，输出E/pull-coexistence-comparison-verification-v1.json（尚未生成）。S/C每cell/arm/profile360人口、原相对p95×1.10＋20000μs/文件率×0.90与全部原绝对参考门保持；任何未齐/失败只报条件分布，不晋升。若失败按真实业务/调度/选路事实定位，在新版本修复，不改原门或重复已停cohort。若设计门过，继续1/3/5 CPU/RSS/字节/唤醒、小文件首内容、同步/预览/SSH/hop共存及手机成本。**本轮同ELF设计选择不替代最终集成源对公开main的同期回归。**
3. N1置信/多middle/切换和成本、N2完整期限/取消/迟到Endpoint与policy队列变化/写入0-RTT/NET-16负向、N4冷hint/新peer发布/Direct UDP突断/peer relay自身移动/休眠/Wi-Fi蜂窝/Relay故障域与容量/NAT64，以及N5全部60项custom退出审计与最终同源Quality/Mac/phone/报告继续按REQUIREMENTS完成。手机在具体候选可测时重新安排。不得把“Mac＋性能＋手机”缩成全部剩余范围。

## 不可误用的既有证据与边界

- 原S三臂screen已按预定hard-fail停在7C high direct_after=false：10080读/14文件成功，但组合门失败；8A/9B未运行，原门与失败保持。D fixture-only最终Link固定有/无日志各3诊断共4320读/6文件成功，0/6复现，不是修复，不再原样重复。4个p6速度优先普通Release合同已过，不重跑。
- 旧fe61 Linux294/Mac290、eacc标准306均仅对准确源成立；新pool不能沿用它们当Mac或Release通过。最新317及压力1也不能证明全部N3。原公平性同测试GREEN及其他组件证据不等于N0–N5完整接受。
- 上游最近官方核验（2026-11-14T05:37:22Z）目标iroh1.4.1/Noq1.4.0已在独立候选，canonical feat/net仍旧依赖；60历史补丁仅1上游替代、2Noq移除红与1iroh38移除红有专属结论，56待退出核查，不能称全部custom收口。
- MacLab与Turing同LAN，macserver另网络。iPhone localhost已配对但尚未接受本候选换网/能耗。khor-build公开已获明确授权，不等于App发行/main合并。LIVE23961未动。
- Turing UID1110、原flock/磁盘/工具门；计时不与同host编译混跑。root唯一ledger写者；stage一次、launch一次；collect原返回码后核identity/owner及真实合同才闭ledger。Mac另主机可并行。
- 旧实验worktree图谱0；新夹具commit hook自动局部35nodes/3files，但built_at仍ff3360与新HEAD不匹配，不能代表新源全图。缺口定向fallback，risk/testgap不作覆盖证据；不擅自重索引。本机不跑Cargo/Rustfmt/Node/Bun/Xcode/产品；main9a4219ffe5e3与builder checkout HEAD5d3691保持。
