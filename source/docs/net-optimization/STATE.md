# feat/net 当前状态与接续入口

更新于2026-11-11 10:29 UTC。完整N0–N5目标“完成计划”active，上一目标轮取得同源hint双端结果并归档，本轮取得迟到成功真实红→绿及新四节点fixture源码，属于进展。未完成、未阻塞，不派发子智能体。此前STATE逐字归档于[上一状态](HISTORY-20261111-through-hint-verification.md)，SHA `617b316640985eb22785a7d5ccf3d558355d3a1da24ed6479f1ff8224b7c54f1`，继续链接全部旧失败与历史；原始日志/准确收据优先，不凭摘要覆盖原改动。

## 位置与授权

- 产品 `/Volumes/disk0/Khor/worktrees/current/khor-net`，branch `feat/net`，本次新增实现453399→远端格式67929，随后CLI fixture811d53；另核实际HEAD/diff。最新**已验证生产实现**源67929（下节），新增CLI fixture尚未编译/接受，不能混淆两者。
- 控制树 `/Volumes/disk0/Khor/worktrees/current/khor-build-net` 固定68cadb57a0ed72d40a4ca9a843e86b9c6797bf34，保留原两个untracked pycache；Iroh42–44独立树 `/Volumes/disk0/Khor/worktrees/current/khor-net-recovery-deadlines` detached2ef6bee5a5aaf841af066f4edb5c894769d4539d，未动。
- 证据根 `/Volumes/disk0/Khor/evidence/net`，工具ops同根；source-refs.json仅root单写，stage/register进程未退出不得启动下一ledger写入。timeout/EOF不是终态，只查同receipt/Actions/PID，不重复提交。
- 用户授权完整实现与Turing、macserver/私库khor-build验证。本机source-only，不执行Cargo/Rust/Node/Xcode/fmt/产品；Git、Python元数据与源码编辑允许。无main合并/发布、采购、生产部署或已装App/config/store/resident修改授权。本轮fixture配置只在远端namespace scratch内。
- Turing严格串行，UID1110原flock/user-netns、`/tmp/khor-netlab-feat-net-1110`、Rust1.95/jobs16/nice10/50GiB起40停/90min原门，无sudo/host网改动。Mac独立jobs2/shared-cache/原门可并行；SSH BatchMode、ForwardAgent=no、StrictHostKeyChecking。

## 最新完成：迟到成功不能再次发布

**源67929a62b68324c35c2035f1ad16ef0ba2d971a4**，tree **0127518fd54080480e507a84ed5ee07f41618e70**，snapshot **05a05569dc058fada5f2dbb9f3dd8324c2355015**，candidate-late-success-candidate-source-v2.json，clean/0excluded。

- Mac **43001941088** request063b72dcf7d945d7893262f45b4f6965，exact artifact13846562644/ZIP SHA **4f5bf6b2e4bfdbce85a453326e25bdfb65a78123b56d7a575c225d08d0c0df86**，20files/11controls/identity；全部12组，hop89、net68+1ignored、wires16、budget26、resume1。
- Turing **turing-20261111T101309Z-699c9a6b** 已347.181秒终态0、21files/identity/原进程组空；全部12组，hop89、net68+1ignored、wires16、budget30、resume1。
- 两端各node-regression-verification-v1.json逐组保留日志SHA、准确源；ledger已completed_verified。无需再poll/重跑这两job。

真实旧行为证据：test-only格式源 **976d4a2b7c2d70672e9179b7264edcc35d95d407**，tree2187fff18400ad0a7e16df0377f35ee7ec9d9662/snapshot2310f4b2b56d752e49d425ed6b6fa9ba0d4ab56d，在 **100321Z-5541ec0c** 224.712秒、11files/identity/空group，hop **88pass/1FAIL**。失败在最终assert“an OK from an expired ask context must not commit a carrier”：实际旧Ask先挂起、新middle已完成真实carried TLS/echo并正常释放，旧OK在原预算内返回却再次挂载。same local network token/活first leg/真实完成及清理前置均通过；活消费者case当次已由原ID tie保护，不称两个case都旧红。late-success-old-behavior-v1.json与candidate-late-success-fixture-v1.json保留准确边界。

修复：原network-generation→MISSED→Links锁内同步核对并提交，无await/文档I/O；当前candidate、网络、原期限及SSH/双端竞争规则通过才挂载，实际接受后才清失败/替换token。Links::attach_if在同一注册表锁内查当前carrier并插入，拒绝流不改变serial、活I/O、last_carrier，不写成功journal；原3秒/更早caller、TRIES2、信任与写重放门不变，不增map/task/timer/常驻强Node。新增真实pipe控制证明SSH拒绝后原流可用、可正常detach，空位仍可接受新hop。目标端首帧后的策略检查仍有独立边界，完整并发/撤销和移动继续跟踪；单位绿不证明物理或手机。

首轮77db test格式：Turing100112Z-33047f91/Mac43000267871只format、0行为组，均收齐/identity/分类；Mac exact13845094798/ZIP SHA98eece91b28c13b24e1642f2a55f970875376ca09d94e864173bfd01caaebbcf，9files。候选453399首轮Turing101106Z-ef5bfe66/Mac43001790814也仅format（net test一处闭包换行），无行为；Mac exact13846975721/ZIP SHA5efe11862a5d7aae47209ff7ca8df8015f04f16b9f71a269a4faca841497359a，9files/11controls/identity。精确返回patch已应用，旧失败不算行为红，不重发。

## 下一项：四真实resident的prepared竞争

新源码 **811d53c2b9bb10aed713a35ad9044a181c68be9f**，tree40a27556a7c05750ec7d35a7f88019fd94ed82fb/snapshot9f70f5c778b0f41d977115fa9c979f254a90a489，candidate-prepared-physical-source-v1.json。新增 `hop/prepared_physical.rs`，父模块包装函数 `hop::f4_a_prepared_second_middle_serves_before_the_first_middle_resumes`，**必须在父模块以匹配trusted hop::f4_过滤器**，原三case正文未改。

四peer X/Y严格NAT、M/N可直达；先在配对前关闭所有peer-hop防预发布，实际IPC PipeOn暖resident-owned Tunnel两腿（不得调用tunnel_to另bind同key），验证Direct测量/目标capability/排序M<N。X取较低身份，禁X公共Relay、释放XY旧wire，冻结M和短暂冻结XY后启用peer-hop；从激活计时要求N在M仍OS-stopped时<1.5秒返回真实目录内容，X成功Hop journal须viaN且tried2；恢复M后>=10秒/>=30实际只读样本和两端同一carried wire/无退休替换。缺wire/attempt证据会失败，不凭最终Forwarded替代。

边界：M的迟到OK和输家取消仍明确unobserved，CPU/bytes为unknown；恢复M本身不能证明迟到请求/回复发生。初次使用既有 **netlab-hop-ownership** lifecycle控制：原f4+新f4r两case×三轮是新6项诊断校准，不冒充普通延迟SLO；完整netlab-hop将成为新12项集合，与旧9项对照隔离。仍需完整真实第二腿Relay、唯一middle、混合旧版本与地址移动。

**格式检查3fa819c3已终态1并核identity/10files/空group**：net-unit任务只跑format，0行为组；精确返回的两个CLI测试文件format.patch已应用、待提交/冻结v2。下一步先核真实diff/HEAD，提交格式和状态，冻结新source，再一次提交netlab-hop-ownership并登记receipt。不能重新提交旧3fa或把fixture源码当通过。prepared-physical-fixture-v1.json保留门与缺项。

## 已有其他证据，保留归因

- hint源 **0d1c0330036d42a3529e4e7f073c330fe9abda9e** /treea650622220a3d62a59ad4a13049085d1789250f0/snapshot84b75533a9a4781cce1c08437aa6be16666ce4af，在Mac42997902740/Turing094808Z-fb91c212双端全部12/hop87。旧Mac42994929836实际83pass/4FAIL，四种目标/middle独立v4/v6有效hint变化被旧退避挡住；同合同候选绿。顺序/重复/无效输入不解除冷却；middle adapter与独立TLS probe不是物理第二腿移动。完整产物/SHA见上一状态，均终态不重跑。
- prepared源c2b615fedc928dfc87c1cc088ca2c06dfb3cb550在Mac42989990177/Turing084053Z-8d9819ab双端全12/hop79；最多2借用父future的准备、显式hop-ready协议、有限同预算legacyfallback、目标完整首帧才提交。半预算/3秒receiver lease仍候选参数，未以loopback定最优。
- 比较源a73b9cae0f9d147cb4ec445d32190e395ca97244在Mac42992730535/Turing091507Z-2f94f130双端全12/hop81；每middle工作绑定实际网络/wire事实，保留工作记录，未试第三个不继承前两者间隔。替代全局门后总工作上限不同，须测1/3/5设备CPU/字节/quiet，不能只因沿用30秒声称成本未变。
- 原物理c2b **084720Z-2463e1e5** 三轮9green，f4i重新回peer28.2/17.8/34.6秒；a73 **092344Z-a59e4d3f** 同条件三轮9green，8.1/8.6/8.1秒。f4最慢echo基线3.7/3.5/3.5→候选3.4/3.5/3.4；f4s9.1/9.3/9.3→8.7/8.5/**9.6**，第三更慢。候选第二f4s有Idle@5.277→Forwarded@8.412、x Relay6.1→Carried13.1瞬态。不能称全面改善/无卡顿。f4i是公共Relay已可用、middle恢复后的回peer收敛，不是首响应/全断连；只有3样本，非p99。该原3peer只有1middle，不能证明双middle准备竞争。准确rawSHA、两source和限制在comparison-context-original-hop-result-v1.json，各original-hop-verification-v1.json已核，不poll/重跑。

## 全范围未完成项与现场

N0最终UX门/足够样本p99/一致完整前后证据；N1约8秒尾巴、严重单向stall、f4s瞬态与quiet/新比较成本；N2同connection实际v4/v6迁移、LOST本机身份/上下文、完整取消/迟到成功/并发策略；N3真Node交互与批量、共享/分连接、背压公平、1/3/5设备CPU/流量/唤醒/能耗；N4完整双腿Direct/Relay/SSH、唯一/多middle、实际prepared竞争、混合版本、独立/同时v4-v6及Wi-Fi↔蜂窝、拓扑故障域；N5 vendor57（Iroh1–41/45–47+Noq1–13，42–44隔离）完整上游/退出与Noq1/5专项、Quality/native/proxy/多机/手机/最终一致报告，全部继续。

历史严重6aa无Go/quality_steps0、RTT不对称/缺失、carrier约21秒结束；Busy854、resume32timeout、CPUquiet/f4s、coldstart；Noq13物理7.3/5.4秒与截断8f3；Iroh47固定4项两C过、B初始10秒超时与7.499秒红，不追加第五、不称p99；8cd动态27green/长稳4green属于旧源，不能用新单位绿抹掉这些失败。准确版本/原件/成本边界均沿历史与VENDOR-AUDIT保留。congestion704原始样本/15bulk行与独立FIFO仅底层Noq，非真Node/手机。

MacLab和Turing同LAN；macserver是另一构建机，用户报告异网。iPhone localhost旧0.2.0现场靠反复关开App某次恢复，截图5GA仅hinton Direct29ms；候选未安装，无phone sampler，旧窗口已过期。最终走可信GitHub/TestFlight真机流程。最新向用户估计完整仍3–5天且低置信，承认此前按小时估计混淆局部与整体验收；不在compact后重置或把阶段绿当整体完成。

图谱优先，本树无.codegraph不擅建；CRG部分/滞后，0flow不是无影响；debug/explore/review技能已使用。执行继续，goal保持active。


## 10:28 UTC：四peer物理任务已单次启动

当前准确源 **1f0a8be64bd4a692475b901e34fb8d7996dda034**，tree **50a9b88e52f6e2c4bad8ce27b5f37116623625d7**，snapshot **84e7eec83981530c44604ea6ed23ac94e85fb220**，candidate-prepared-physical-source-v2.json，clean/0excluded；生产carry/hop/stream_trace/wires逐字核对与双OS已过67929相同，新CLI fixture独立待编译/行为。远端精确格式补丁已提交，上一节旧“待提交/冻结”由此替代。

Turing **turing-20261111T102641Z-ebc3630e**，PID **3828986**，10:27:03 UTC launch，netlab-hop-ownership、原control68cad、role diagnostic；stage exec97445已成功退出，仅提交一次。前驱3fa格式-only已收齐、identity/空group。prepared-physical-run-plan-v1.json记录新2case×3的6项诊断校准；先查同receipt/execution/PID与task.log，不能用原9项checker，也不能在活物理计时期间启动另一个Turing构建/测试。无其他Turing计时、无Mac在途、无手机采样。下一步核编译和真实前置/业务结果；若失败保留原轮次，不盲重发或把返回码当覆盖证明。完整N0–N5保持active。


## 10:29 UTC：前一fixture编译失败已修，当前只查874eb4c4

前一物理ebc3630e已58.500秒终态1、13files/identity/空group；三次driver均因同一E0631编译错误未进入任何网络case：Option<Link>::is_some_and传入了借用Link的谓词。保存全部原编译日志，分类fixture_compile_E0631_no_network_scenarios，不是产品网络红。源码两处改为as_ref后提交 **42014586604f54a7541e0873d219332a3596743c**；candidate-prepared-physical-source-v3.json tree **5312d9cc34176feddb6e3adc3853e0b9df73abbd** /snapshot **033023d20020d0bfb4849499dff933422b38cd88**，clean/0excluded。

当前唯一Turing任务 **turing-20261111T102838Z-874eb4c4**，PID **3831005**，10:28:59 UTC launch，netlab-hop-ownership/同control68cad/role diagnostic；stage exec57262已成功退出，已按同PID实际ps核在运行。prepared-physical-run-plan-v2.json保存新2case×3合同和前驱编译失败。先核同receipt/execution/PID/task.log；不要再poll/重发已终态ebc、3fa或67929双OS单位。编译/前置/业务/缺观察分开分类。任何新计时未终态前不穿插Turing构建。无Mac在途、无phone采样。下一步必须核新fixture实际编译与两middle前置，随后合同/失败归因；完整N0–N5不缩小，goal active。
