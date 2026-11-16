# feat/net 当前状态与接续入口

更新于 2026-11-16T08:40:13.899603+00:00。用户最新“做完吧”已明确继续完整目标，goal active；[GOAL](GOAL.md)、[PLAN](PLAN.md) N0–N5、[REQUIREMENTS](REQUIREMENTS.md)尚未整体完成。root串行、不派子智能体。本机source-only：不跑Cargo/Rustfmt/Node/Bun/Xcode/产品；重型工作只在Turing/macserver/授权khor-build。用户已授权实现与验证，未授权main合并、现装App替换、发行/TestFlight或生产Relay采购部署。

## 当前运行与下一步

- **当前Turing唯一在途** `turing-20261116T083237Z-362fed6f`、PID703767、task net-node、revision `net_product_v5_window_fixture_standard_v1`、control **41d2e4f6c35a6e734f693e4a9e48ed524dee8891**。源 **23a68888e34154b87a2e45596adf9ac6743e2382** / tree **2cd7316e44efdd67c62184f51f4a4dee1002c8e6** / snapshot **0758175304f0f3e488717dd00004959aadfb1790**，描述 `E/net-product-v5-fixture-source-v1.json`。本源仅改受控测试 responder 支持额外文件连接＋Rust格式分类，**生产代码与I5 9821d929相同**；完整原测试体、处理器assert/token与期限逐字（忽略格式）保持，见 `E/net-product-v5-window-fixture-plan-v1.json`。先查已有receipt/execution，不重派；结束collect→identity→`E/ops/verify-product-v5-window-fixture-standard-v1.py RUN separate`，该成功verifier仅在全部323通过时用。若失败，保留实际Native和未执行人口再诊断。
- 此run先按原first-party Rust checker检查全仓：只把 `tests/net-connection-boundary-v1/{lib,silent,upstream_util}.rs` 纳入原Rust2024 stdin formatter，不排除检查；随后原12组在global separate下运行。受控responder原只回答cached Main，新连接从未accept导致4个窗口/取消控制不到条件；现同handler/断言服务cached＋新文件连接，nested JoinSets全有owner。需等真结果，不能先称修复。三独立Rust文件本已格式正确，远端格式只改helper缩进。
- **旧I5完整标准已接受** `075454Z-f11ccfd0` 原0/512.203s，**323pass/0fail/原ignored1**，12组每个测试名/源/控制/owner已核。`E/RUN/product-v5-standard-node-verification-v1.json`。
- **旧I5 global separate红已闭** `080702Z-38aa0920` 原101/502.035s：已执行5组 **295pass/4fail/原ignored1**，余7 IPC/Content/Resume/STOP组未执行；四红仅原pull_window_wire取消/JobStop/slowHead/StopRPC，都只见issued0/1。完整原红/日志与owner闭证 `E/RUN/product-v5-separate-red-verification-v1.json`。global env不等于每个显式负控都ON，不能作这种覆盖声明。
- **Quality 43827443762**，准确源I5 **9821d9298dbab876434041bc436a7a97c1d4c1d7** / tree **4b7013df0e89837ddbe1e5d22c010bbf8ecf3cb8** / snapshot **b93d6b7acbc2195d7aefa6b75aba0c1f37f8553c**，分支 `verify/net-product-v5`。已知scripts job133164520860原1：first-party formatter拒绝未分类的独立transport fixture；新23a已分类，未放松原门。其他12组当前已知11成功，node-native133166355726仍在signed SSH安装/认证步骤，先读 `E/quality-43827443762/jobs-current.json` 或原run，不重派原job。需要新准确源Quality，等本轮失败/修复收齐后冻结；原scripts红保留。GitHub connector不支持该运行集合查询，gh只读回执为fallback。
- **Mac Lab V2 43829392360**，源仍I5 9821d929 / snapshotb93d6，acquirer session **17871**，独立目录 `E/public-native-lab-net-product-v5-v2`，源码仅在远端执行；当前frontend133170773653与native133173046669已success，checks133173046701在完整Chromium/WebKit测试。控制独立branch `net-product-v5-lab-workbench-v1` / **44b954b74fee82ad448909b6e19a9296e69cd30e**，只在源有Storybook配置时运行当前main同一workbench构建命令，再走未改package完整性门；无builder main修改、原生门/源码门/断言改动。`E/net-product-v5-lab-workbench-control-v1.json`。结果先从已有handle收齐，不重派。
- **Mac Lab V1 43827627887** 原frontend包不完整红：旧builder main9b671缺当前main新增Storybook构建步骤，native/checks未执行，非SSH故障。原acquirer错误/日志/请求在 `E/public-native-lab-net-product-v5`，source与V2逐字相同；只改独立control后重跑V2，未绕过完整性检查。
- canonical worktree仍 `/Volumes/disk0/Khor/worktrees/current/khor-net` branch feat/net，当前产品/test提交23a68888；独立集成树 `khor-net-product-integration-v5` / experiment/net-product-integration-v5 仍固定9821d929供本轮Quality/Mac Lab。main93f6cb23未被本任务改动、LIVE23961/其他用户目录未动。准确Source收据优先于文档HEAD，STATE后续doc-only更新不冒充新源接受。
- 本轮整合历史：J4b9ef2a6＋main93f6 merge76448e64，plan lineage e0d7 merge94534148，canonical以ff到9821；GUI实现与main逐字相同，仅attachment测试日志保留同channel与版本记录。`E/net-product-integration-v5-resolution-v1.json`。无main合并/安装/发行/生产Relay部署授权。
- 原N3统计门已正确核：H360目录bulk pooled p95 **448002μs**、p99 **836037μs**，X **353557/751934μs**，各2160业务、3完整192MiB、6post-health全成功，均过原592518.1μs此单条参考；逐场614ms仍留但不是独立cohort门。`E/current-high-pooled-verification-v1.json`。X优先级无必要性证明已从产品删除。正常同源Release最终配对、1/3/5成本、全部负载、NAT64/冷peer/移动/手机和custom退出审查继续，不缩成当前几个run。

## 当前准确产品候选 H

Worktree `/Volumes/disk0/Khor/worktrees/current/khor-net-selection-view-v1`，branch `experiment/net-selection-view-v1`，HEAD **9acc6097a7bc8fe01edd3f1fe6ef2d998d3d50c3** / tree **06b0c2cda62da87c2c1bb9ec214d0b999cdaeff5** / snapshot **9c54ad636ccbf1b42f1895f8c98a1efa8dd8458f**，clean。源描述 `E/selection-view-events-source-v1.json`。canonical feat/net当前仍保存计划与历史实现，尚未最终集成最新依赖/完整H；main现93f6cb23由其他任务推进，本任务未改，builder checkout5d3691未动，LIVE23961未动。

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
3. **N4**：冷hint/新peer发布、双方和peer-relay移动/睡眠/Wi-Fi蜂窝、Direct UDP突断、Relay整机/半断/出口/登记/容量和NAT64仍有缺口。旧IPv6-only15场630读、双端dual/v4等闭证保留，不能替NAT64/手机。
4. **N5**：最新H集成canonical feat/net和同期main后，仍需最终同源Quality、native MacLab/真实backend、手机换网与能耗、设计/运维和全报告。不是“只差手机”。未获main合并/发布授权，不因此停实现/验证。

## 上游与定制审计

2026-11-16官方release/crates.io再次核验稳定 **iroh1.4.1、Noq/noq-proto1.4.0**；当前隔离候选已跟上，canonical未最终集成。Noq GitHub latest alias旧1.1.0不作降级依据。`E/upstream-stable-check-20261116-v1.json`保存checksum/metadata。

原版A/当前C十运输合同及关闭Khor policy但保留vendor的B五合同均已闭，见[DIRECT-FALLBACK](DIRECT-FALLBACK.md)、[UPSTREAM-BOUNDARY](UPSTREAM-BOUNDARY.md)、[UPSTREAM-CONSUMERS](UPSTREAM-CONSUMERS.md)。A静默UDP原15292/15296ms、2s失败但最终同连接Relay通；C约602/605ms；B约603/605ms。B不是原版且硬tier违背U1，不能推广。上游已承担Direct/Relay/多路径，Khor已用公开path_selector/custom transport；不能称全部自造或全部vendor必要。

账本 **E/iroh-upgrade-patch-ledger-v8.json** 共60历史项：**46待审、1上游替换、10专属删除实验支持保留、3观测接口集成保留**。Noq只1 BBR app-limited尚待；iroh仍45项待审。见[NOQ-RECOVERY-AUDIT](NOQ-RECOVERY-AUDIT.md)。

- 032958Z-36318d40原101/66.112秒，Noq7/9/11三隔离×4原合同=8绿/4红；原Noq2ed树与02807 baseline467全库相同，单点变换/原测试和期限/fresh产物/cleanup全核。
- 043413Z-c30c817d原101/92.095秒/122报告，Noq4/6/12/13四隔离×10原合同=36绿/4红，四专属恰各1红，普通ping/握手/真实0RTT等负控绿，原测试/门不变。证明这些行为当前不能直接删除，不证明唯一实现或整个vendor必要。
- Noq3/8/10是当前消费者所需的PathStats观测字段，上游1.4公开API无直接等价；有具体字段/消费者/467＋当前iroh库证据，是接口依赖审查而非物理删除红。退出条件已记，不宣称手机/成本接受。

## 调度与环境边界

E=`/Volumes/disk0/Khor/evidence/net`。实际调度以`E/source-refs.json`、receipt、远端execution为准；mac独立progress只有核验后追加common完成历史。stage/launch各一次；观测超时查已有handle，不能重复派。collect保留原退出码，identity＋真实合同＋全部owned groups核验后闭ledger。已闭证组无新理由不重跑。

Turing UID1110，自有`/tmp/khor-netlab-feat-net-1110`、原owner/flock/build.lock；Rust1.95/jobs16/nice10，start50/stop40GiB。计时CPU idle≥.8/PSI≤5/同UID无Cargo，同host编译和计时串行；Mac不同host可并行。只恢复自有精确归档文件，不碰其他用户/任务。原nft1.1.3隔离工具保持，不改宿主全局工具。

MacLab与Turing同LAN，macserver另一网络；macserver SSH本轮banner超时，授权公开colehank/khor-build通道正常。iPhone localhost仍旧安装版，先前前台就绪窗口已过，具体候选可测时再安排，不自动采样。无部署/采购/发行。

本入口整理前逐字档案[HISTORY-20261116-through-selection-view-recovery](HISTORY-20261116-through-selection-view-recovery.md)，SHA256 967f7c3fffff9a8037503fa5f4d4ac6c279ce36e85945065fdbb072394a2c3c6；其“在途/下一步”均是历史，不可据此重派。更早全部原失败/未执行及证据见该档案与其链接、RUNS/REQUIREMENTS，未被本页删除或重新归绿。
