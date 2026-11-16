# feat/net 当前状态与接续入口

更新于 2026-11-16T11:07:11.899929+00:00。用户最新“做完吧”已明确继续完整目标，goal active；[GOAL](GOAL.md)、[PLAN](PLAN.md) N0–N5、[REQUIREMENTS](REQUIREMENTS.md)尚未整体完成。root串行、不派子智能体。本机source-only；重型工作只在Turing/macserver/授权khor-build。未获main合并、现装App替换、发行/TestFlight或生产Relay采购部署授权。

## 当前运行与下一步

E=`/Volumes/disk0/Khor/evidence/net`。准确产品源 **5f51e3409e0e642c26b5b41c37cc069213c1d6a5** / tree **395f91ae9e82c3b3819d7fe96be8c3882f6313bd** / snapshot **b846df2162863114a2493a3fe868a2587faf4574**，描述 `E/net-product-v5-fixture-source-v2.json`。本页随后文档提交不冒充该源新接受。

- **当前Turing唯一在途**为DNS普通Release重建V2 **turing-20261116T111352Z-0b98ee96**，PID1075951（查原execution，不能重派），control **bbeb57ffb42ef6f7f541188b9e4c6f09779f0a5d**，源3bd998e8。V1 `110848Z-29898357`原1/51.986s，native compile101/E0277同Noq路径类型不一致，0业务；原件与identity/cleanup已闭。外部fixture与workspace共享target的un-hashed iroh cdylib是缓存疑因，V2只在准确source重新构建iroh依赖（mtime，不改内容/flags），必须fresh=False后原provenance全核；不把编译错误算DNS行为失败。
- **worker数量screen已拒绝** `105411Z-07c2f1ff`原0/775.382s，同源同ELF固定ABBAAB六场/18首读全成功。4 worker CPU8.464% vs默认8.914%，线程27.67 vs275.33、首读有限p95 29.360 vs38.643ms。原CPU70%门未过，因此不采用生产策略；线程50%与延迟门绿不抵销。`E/idle-cost-worker-screen-result-v1.json`；无替代试验、硬件唤醒/手机仍缺。

- **完整成本矩阵已闭** `095503Z-3cb48ff8` 原0/2140.078s，18/18 case、54 resident interval、156 preflight与48 post-idle读均成功；原2秒与全部owner/接口/CPU准入不变。1/3/5每resident CPU中位Direct约6.08/9.27/12.92%、Relay约5.90/9.01/12.60%，详[COST](COST.md)。这是量化，开销尚未接受，不能全归因网络；无硬件wakeups/手机能耗。
- **NAT64真实RED已闭** `104129Z-da0f8846` 原1/23.857s。无IPv4 client、真实SIIT nonce双向与系统合成AAAA均过，产品冷配对原10秒超时，0业务读被执行。`E/RUN/nat64-cold-red-verification-v1.json`。最小DNS候选 `khor-net-system-dns-v1` / **3bd998e8**只删除18行固定公共DNS覆盖，使用iroh系统DNS/现成fallback；`E/system-dns-plan-v1.json`。Mac原标准GH**43854682670**已闭319pass/0fail/ignored1，source/control/全部测试名及artifact SHA/CRC核验，`E/system-dns-mac-verification-v1.json`；Turing普通Release重建V2在途，同fixtureGREEN尚未派，不宣称因果已证。
- **cold hint caller消融Mac已闭** GH**43848132339**：源832c268a/treeeabab136、原12组**319pass/0fail/ignored1**，artifact SHA/CRC、全部控制与Linux四个原cfg排除核验，32 separate＋14 contact均过。`E/cold-hint-exit-mac-verification-v1.json`。Linux/物理与长期成本退出未齐，canonical尚未采用，不删除vendor API。

- **成本probe已闭** `094918Z-b4a35bc1` 原0/132.304s，3个真实resident、完整有向目录、30s settle/60s quiet、原2s后读全过。每进程CPU约单核8.57–8.81%、RSS约97–105MiB、275–276线程；静默eth0每方向约30–43KiB，后读23.606/26.774/28.170ms。进程PID/start与owner、接口单调、HZ、全部源码/ELF/环境/清理核验，`E/RUN/idle-cost-runtime-verification-v1.json`。开销有待解释；不能全归网络或由这一个cohort声称成本好。
- **正式Release恢复/控制已闭**：`092109Z-edcabaac` 原0/198.662s，5个原idle恢复全绿，phase0三次18.133/17.942/18.135s、phase5 13.046s、phase14 3.877s，原20s不改；`092523Z-d3726f6d` 原0，两个相反快慢路径、typing恢复1.392s与真实黑洞260回显/0fail/最大574.1ms全绿。准确源5f51，`E/RUN/net-product-v5-screen-verification-v1.json`。后续同源原preflight与共存固定6块仍需执行。
- **成本夹具编译已闭** `093646Z-60fe4209` 原0/576.331s，fixture ELF **04a83431022fd071310a26efefbab4ddce46359a985965163b51993cb8b33cfe**，原普通Release配置、原root全部8个patch绑定、原lock依赖子集与格式0全核；产品CLI仍64af80ac。`E/RUN/idle-cost-build-verification-v4.json`。前三轮控制器身份/PATH与offline解析拒绝保留：179241b7、f03d408b、b147ca33，分别0编译/0编译/0 compiler-artifact，均0运行，不是产品功能失败。源在 `/Volumes/disk0/Khor/test-fixtures/net/idle-cost-v1`，已复制canonical `tests/net-idle-cost-v1` 供最终版本化（本页后续doc/source提交不是已测5f源）。
- **同ELF最终shared/separate协议已冻结未派** `E/net-product-v5-coexistence-plan-v1.json`、control **9d1a905297805cde37f55442a472503f4b639373**，保留原SCCSSC/12case/8640 population、原相对/绝对门、原preflight分层准入和失败。不得将它写成已执行；N3未完成。
- **普通Release producer已闭** `090926Z-6289823d` 原0/**577.254s**，源5f51，control70e3a994。原opt3/thin-LTO/codegen1/strip、iroh1.4.1/Noq1.4.0、fresh CLI全核，无运行测试、46不变synthetic验证源仅按hash复用。CLI SHA **64af80ac8f5e1d7d1352d7beb7e6137e466836de4e0dd0a3ce11be5fcebb50cc**；netlab **32e35ac5ddd4238c5f914a81b7a0425fc5d39f333ec56111ebec8a23e70e8d6a**。`E/RUN/net-product-v5-release-compile-verification-v2.json`。V1 prepared未启动，已明确superseded，不可启动。
- **完整global separate回归已闭** `084734Z-e8016ef4` 原0/**546.320s**，准确源5f51，原12组 **323pass/0fail/原ignored1**，所有测试名/源/控制/owner及629个first-party Rust格式输入已核；511原排除不变，0格式差异。`E/RUN/product-v5-separate-node-verification-v1.json`。global env不等于各显式negative override均ON，不能声称这种覆盖。
- fixture修复只在正向窗口场景接受额外文件连接，专属协商/负控仍独占Incoming。23a第一次fix原4窗口红转绿却让4专属负控红（362fed6f原101/295pass/4fail/7组未执行）；5f只改mock构造/stop setup，全部断言/数字期限/生产代码不改，后续323全绿。原两轮红保持。计划 `E/net-product-v5-window-fixture-plan-v2.json`，旧08:40状态归档 [HISTORY](HISTORY-20261116-v5-fixture-through-0840.md)。
- **I5 shared标准已接受** `075454Z-f11ccfd0` 原0/512.203s，源9821d929，原12组323/0/ignored1。I5与5f生产逐字相同，后者只测试helper/格式分类/文档变化；不把旧tree当新tree全Quality绿。
- **Quality GH43842976362已结束12/13**，准确5f51/tree395f；原scripts gate已绿，唯一node-native job133218492213在signed SSH scratch测试普通member密码计数expected0/actual1失败。原日志`E/quality-43842976362/node-native-job.raw.log`保留。普通member也可能因公网不可达改为SSH-carried（按合同保留凭据），实际route缺诊断，尚不宣称根因。隔离`khor-net-ssh-local-relay-v1` **0955ac3a**已加入owned local Relay配置与脱敏失败信息，原kept0/期限/其他断言保持；原suite=node Quality **GH43855339936**已派一次，先查原run，尚未接受/采用。
- **Quality43827443762已结束12/13**，准确源I5 **9821d9298dbab876434041bc436a7a97c1d4c1d7** / tree4b7013df / snapshotb93d6b7a。唯一scripts job133164520860红是3个独立transport fixture Rust输入未分类；23a加入完整formatter（非排除），Turing原gate已绿。另12组全部绿，包括node-native/signed SSH。原红保留；最终源码冻结后需新同源Quality，尚未派。
- **Mac Lab V2 GH43829392360已4/4成功**，源仍I59821/snapshotb93。`E/public-native-lab-net-product-v5-v2` 已获取native/前端及checks ZIP **48,333,004B**；**104浏览器case无错误**，独立核验已闭：98条tracked hash、18条generated lineage、内ZIP SHA/CRC/arm64与真实backend一致，`E/public-native-lab-net-product-v5-v2/lab-acquisition-verification-v1.json`。不是手机/能耗接受。独立控制 **44b954b74fee82ad448909b6e19a9296e69cd30e** (`net-product-v5-lab-workbench-v1`) 只加入当前main同一Storybook构建命令，原门/断言不改。V1 frontend缺workbench原红及native/checks未执行保留；不动builder main。
- canonical `feat/net` 已ff集成I5＋5f；I5集成固定main93f6cb23与初始Direct近似质量次选、H/S改善；main现由其他任务继续推进，不自动追每个GUI提交。本任务不改main、LIVE23961/其他用户目录。I5独立树仍9821，供准确源Quality/Lab查验。
- 初始无current时，near-quality复用原5ms gain floor次选Direct；明显更快Relay/Carried仍赢，健康current原保持逻辑不变。Mac准确RED1/72与GREEN73/0/ignored1、同74测试名已核。`E/initial-direct-green-v2-mac-verification-v1.json`。
- N3原统计门口径是每profile/phase/op固定3次**pooled360**，并非每次120单独过。H pooled directory p95 **448002μs**/p99 **836037μs**；X **353557/751934μs**，均过原592518.1单条参考，各2160业务/3完整file/6post-health全成功。逐场613/614ms保留。无同期配对不作整体N3接受；X priority无必要性未采用。`E/current-high-pooled-verification-v1.json`。
- **仍未整体完成**：N1/N2完整合同、N3全部负载/同期main对照/1-3-5成本及separate默认采用决策、N4 NAT64/冷hint/双方和middle移动/Relay故障域、N5的60项custom审查已交付，行为/成本退出、设计运维、最终同源Quality/手机能耗仍待。不是仅差手机，不能把旧阶段绿代替完整目标。

## 历史组件候选 H（已集成I5；本节旧在途描述不作调度依据）

Worktree `/Volumes/disk0/Khor/worktrees/current/khor-net-selection-view-v1`，branch `experiment/net-selection-view-v1`，HEAD **9acc6097a7bc8fe01edd3f1fe6ef2d998d3d50c3** / tree **06b0c2cda62da87c2c1bb9ec214d0b999cdaeff5** / snapshot **9c54ad636ccbf1b42f1895f8c98a1efa8dd8458f**，clean。源描述 `E/selection-view-events-source-v1.json`。此处为H阶段历史，当时canonical尚未集成；现已集成I5，main现93f6cb23由其他任务推进，本任务未改，builder checkout5d3691未动，LIVE23961未动。

H在前代S之上只改Node状态传播：复用已有roads::follow的iroh Selected/start/Lagged当前快照，以Weak<Wires>、同peer/stable conn ID、仍存活/当前实际选路检查，及时发布连接词；Opened不算选择、Unknown不造事实、相同词不刷新年龄，旧/闭wire不得污染新wire或另一协议。原tick与两样本fallback/旧合同保留，无新增timer/probe/常驻轮询。见[SELECTION-VIEW](SELECTION-VIEW.md)。

- 窄原生 **044243Z-c6fc1ac1** 原0/116.250秒/23报告：Wires旧16＋新2=18通过、roads3通过，原fmt/源/产物/列表/全部group核验；真实QUIC验证无tick/无journal消费者、Weak所有权和旧/闭/peer/proto/年龄边界。仅21组件合同，不代整N1。
- 普通Release producer **060326Z-8d23414f** 原0/598.316秒/21报告，严格原native_provenance/opt3/thin-LTO/codegen1及新CLI行保持；CLI **0d3e520f7ebdcee184634d067dd202178e66cac41d35f857a838e9b17a94ab5e**，netlab **d70ac65e2f692efc90f6e6f965fa8648a5ab61821eb3a2f774d105a80d294c5b**。源和所选5产物在Turing闭归档，后续只恢复经hash核验的自有文件；编译与计时同host串行。
- 原idle恢复 **061753Z-6aa64695** 原0/231.532秒/17报告，原p7i phase0固定3次17.548313504/17.945023214/18.163609805s，phase5为13.046626094s、phase14为3.880704890s，**5/5过原20s**；0编译/原全部人口/CPU与owner/完整footer核验，proof `E/RUN/selection-view-screen-verification-v1.json`。原S/N六场及初始红仍保留。
- Mac **GH43814577842** 原12组 **316pass/0fail/原ignored1**，同H source/control68cadb57，11控制blobs、每个测试名、2新Wires、32明确separate、14contact和原4 Linux-only cfg排除均核。artifact14202416459/38830B/SHA256 **58df91c01191f71fb1de8987eb1361277c54a7f47fc837a4e440612193eab58e** 与CRC保持；`E/selection-view-mac-verification-v1.json`。原生unit，不是MacLab/App安装或发行。

## 本次修复的原失败与前代 S

S=`khor-net-stable-gain-v1` / `experiment/net-stable-gain-v1`，最终7d9f1c07/treecb045c42（产品fix5eea07cd），iroh1.4.1/Noq1.4.0。正常优化仅比较现有同代表副本/决定后ACK窗口：候选上界低于当前下界；复用8观测/2s，无新阈值/窗口/timer/probe/API，fault leave/far/current>2s例外与原2/4/30s预算不改。见[NORMAL-GAIN](NORMAL-GAIN.md)。

- c09851c5 RED034715Z-c940ffb7原101：重叠窗口误判Go，1红/4恢复控制绿。5eea07cd GREEN035134Z-5342f6de原0/90.102秒：完整iroh208通过/0失败/原ignored1，五合同逐字不变。
- 035501Z-5c11c26d原0/64.030秒，5实际运输场景通过；UDP静默client/server同连接1250.410/602.541ms内经Relay恢复，独立Relay仍通，原2s/真丢包/同锁和非根依赖闭包与owner核。不是统计尾概率。
- S Release producer040019Z-6ccd73cf原0/602.617秒，CLI63f14b10、netlab3638a513。准备v1 035737Z-84fdcd54原1/27.955秒：已有归档目录拒绝，0编译/0业务；v2只核验复用归档，原拒绝保持。
- S普通控制 **041241Z-fb745707** 原1/128.772秒，3绿/1红：两个p6和typing过，idle实际20.199002777s超原20s。v1解析器误把native负例作analysis_invalid，v2只修数据分类，原业务失败不改。
- 固定原idle同Release **SNNSSN六场**已闭，`E/normal-gain-idle-pair-verification-v1.json`：旧W CLI7454的S为18.139/20.174/20.188s，1绿/2红；新S候选CLI63f14的N为20.189/20.182/20.190s，0绿/3红。全部原夹具/预算/模式/双ELF/source恢复、CPU和cleanup保持。不能归因新gain独有；Wires1s×两样本的额外发布等待在H用原事件消除。原5红与初始红保留，不被H5绿覆盖。原S screen后5 preflight/3high从未执行。

## 仍阻止完整接受的范围

1. **N3未过**：[PULL-COEXISTENCE](PULL-COEXISTENCE.md)旧同ELF SCCSSC六块/12case/8640样本、24post-health和12完整文件全留。分连接low目录p95为84790μs（共享244009），high614406μs（共享1292588），仍超过原592518.1μs；文件中位low7.033/7.071、high25.572/25.342Mbit/s。共享第五块high原1终端timeout＋63blocked保持完整分母，不报成功子集p95。H新high三场2160交互全成功，原360 pooled目录p95为448002μs已过该参考；逐场超参考仍保留，见本页统计口径校正。原pool公平性/容量修复与资源合同/旧Linux317＋压力1、Mac314及原布局均只对其准确源成立；H Mac316已新验证。1/3/5成本、SSH/peer relay/同步共存、全负载和最终同期main对照仍缺。
2. **N1/N2**：完整置信/多middle/共享测量/quiet成本，以及总期限/取消/迟到Endpoint与policy队列变化、全部写family/包级0RTT、NET-16完整正负向继续按REQUIREMENTS。已有14联系、92hop、pool32等证据不等于完整接受。
3. **N4**：冷hint/新peer发布、双方和peer-relay移动/睡眠/Wi-Fi蜂窝、Direct UDP突断、Relay整机/半断/出口/登记/容量和NAT64产品RED待修仍有缺口。旧IPv6-only15场630读、双端dual/v4等闭证保留，不能替NAT64/手机。
4. **N5**：canonical已集成I5；当前Quality12/13、I5同生产Mac Lab已闭。后续实现选择冻结后仍需最终同源Quality、native MacLab/真实backend、手机换网与能耗、设计/运维和全报告。不是“只差手机”。未获main合并/发布授权，不因此停实现/验证。

## 上游与定制审计

2026-11-16官方release/crates.io再次核验稳定 **iroh1.4.1、Noq/noq-proto1.4.0**；当前隔离候选已跟上，canonical已集成I5。Noq GitHub latest alias旧1.1.0不作降级依据。`E/upstream-stable-check-20261116-v1.json`保存checksum/metadata。

原版A/当前C十运输合同及关闭Khor policy但保留vendor的B五合同均已闭，见[DIRECT-FALLBACK](DIRECT-FALLBACK.md)、[UPSTREAM-BOUNDARY](UPSTREAM-BOUNDARY.md)、[UPSTREAM-CONSUMERS](UPSTREAM-CONSUMERS.md)。A静默UDP原15292/15296ms、2s失败但最终同连接Relay通；C约602/605ms；B约603/605ms。B不是原版且硬tier违背U1，不能推广。上游已承担Direct/Relay/多路径，Khor已用公开path_selector/custom transport；不能称全部自造或全部vendor必要。

账本现为 **E/iroh-upgrade-patch-ledger-v11.json**，**60项历史均有逐项原因/当前上游合同对照/覆盖/保留退出记录**。1项上游替换、10项专属删除实验反例保留、3项Noq观测接口、1项BBR通知时序静态集成、45项iroh静态合同集成审查。**审查交付不等于所有custom必要或行为/成本/手机已接受**；后45项的行为/成本退出仍暂留，buffer64、第二Direct备用、数字home策略和诊断钩子明确无必要性证明。详细 [PATCH-REVIEW](PATCH-REVIEW.md)，原删除反例保持。

Noq1发布版已有ACK app-limited支持，当前额外的是无下一ACK前的starvation通知和loss前inflight扣除；无直接时序等价，暂留，仅原相同2edce601包467集成证据，无新的性能/专属删除证明。`E/noq-bbr1-exit-audit-v1.json`。iroh包0ec34a5d与S接受208库测试完全相同，不因审计重跑。

**Iroh45唯一生产caller删除隔离实验在跑**：`khor-net-cold-hint-exit-v1`，branch `experiment/net-cold-hint-exit-v1`，源 **832c268ae3ffcacd5ea9a96cc5e92f3a0e940cbc** / tree **eabab136afb36163679faf094f7c16d452de59da** / snapshot **dc51492ad0b52cf840294c10549283fc53855cc7**。仅删ask_early清旧提示六行，全部原测试/身份/期限/并发与vendorAPI保留；问48/49是否已使该caller多余。**Mac GH43848132339** 原control68cadb57、net-node全12组319人口，独立 `E/cold-hint-exit-mac-progress-v1.json`，先查此run，不重派。不是已采纳，也尚未删除vendorAPI；后者需说明旧提示长期成本及测试合同迁移。`E/cold-hint-exit-plan-v1.json`。

**NAT64 cold编译已准备、未派**：control **7488c8318832239106cf691091a75459065dc95e**，计划 `E/nat64-cold-plan-v1.json`、driver `E/ops/nat64-cold-compile-network-v1.sh`，来源为已格式化且版本化的独立fixture，对准确5f51产品CLI，先网络/DNSoracle再产品断言。等Turing当前计时完全闭证后才编译；不能同host并发。当前无产品RED/修复结论。见 [DNS-NAT64](DNS-NAT64.md)。

- 032958Z-36318d40原101/66.112秒，Noq7/9/11三隔离×4原合同=8绿/4红；原Noq2ed树与02807 baseline467全库相同，单点变换/原测试和期限/fresh产物/cleanup全核。
- 043413Z-c30c817d原101/92.095秒/122报告，Noq4/6/12/13四隔离×10原合同=36绿/4红，四专属恰各1红，普通ping/握手/真实0RTT等负控绿，原测试/门不变。证明这些行为当前不能直接删除，不证明唯一实现或整个vendor必要。
- Noq3/8/10是当前消费者所需的PathStats观测字段，上游1.4公开API无直接等价；有具体字段/消费者/467＋当前iroh库证据，是接口依赖审查而非物理删除红。退出条件已记，不宣称手机/成本接受。

## 调度与环境边界

E=`/Volumes/disk0/Khor/evidence/net`。实际调度以`E/source-refs.json`、receipt、远端execution为准；mac独立progress只有核验后追加common完成历史。stage/launch各一次；观测超时查已有handle，不能重复派。collect保留原退出码，identity＋真实合同＋全部owned groups核验后闭ledger。已闭证组无新理由不重跑。

Turing UID1110，自有`/tmp/khor-netlab-feat-net-1110`、原owner/flock/build.lock；Rust1.95/jobs16/nice10，start50/stop40GiB。计时CPU idle≥.8/PSI≤5/同UID无Cargo，同host编译和计时串行；Mac不同host可并行。只恢复自有精确归档文件，不碰其他用户/任务。原nft1.1.3隔离工具保持，不改宿主全局工具。

MacLab与Turing同LAN，macserver另一网络；macserver SSH本轮再次banner超时，授权公开colehank/khor-build通道正常。iPhone localhost仍旧安装版，先前前台就绪窗口已过，具体候选可测时再安排，不自动采样。无部署/采购/发行。

本入口整理前逐字档案[HISTORY-20261116-through-selection-view-recovery](HISTORY-20261116-through-selection-view-recovery.md)，SHA256 967f7c3fffff9a8037503fa5f4d4ac6c279ce36e85945065fdbb072394a2c3c6；其“在途/下一步”均是历史，不可据此重派。更早全部原失败/未执行及证据见该档案与其链接、RUNS/REQUIREMENTS，未被本页删除或重新归绿。
