# feat/net 当前状态与接续入口

更新于2026-11-11 07:55 UTC。完整N0–N5未完成，用户“完成计划”目标active；root串行，不派发子智能体。此前逐字STATE保存在[上一状态](HISTORY-20261111-through-first-leg-facts.md)，SHA `0e424f63d4f2fcee7f40961de1ddb666f7dd90647032f4a0f4eec5e89aa5ee70`，继续链接此前原始历史及失败。收据、真实进程和实际worktree优先，不能把观察超时或调度paused当作停止/重发依据。

## 位置、授权与当前源

产品 `/Volumes/disk0/Khor/worktrees/current/khor-net`，branch `feat/net`。最新生产候选 **1bd9692aac088026436a0c1044296e54b43344d6**，tree **407ab1b825e3cc9c4db5bcb3e58fa53c0cf86e36**，snapshot **5bd853bbf58cd3a873b6b67207905aced483c448**；它是6c3b86e首腿地址事实修复原样应用远端format.patch，无行为调整。source文件 `candidate-address-facts-candidate-source-v2.json`。实际行为尚未验证。

最新已经准确同源双OS全12通过的是 **2874985e90c1523a6db1b4171a48de085718552d**，tree2780ffe007956ec655bb4475b1a2c57b1853bb7e，snapshot8cdc87e0040c5a813e40c70197ea6d40dcf237c7。不要外推给新候选。

控制树 `/Volumes/disk0/Khor/worktrees/current/khor-build-net`，固定68cadb57a0ed72d40a4ca9a843e86b9c6797bf34，原两个untracked pycache保留。Iroh42–44隔离树 `/Volumes/disk0/Khor/worktrees/current/khor-net-recovery-deadlines`，detached2ef6bee5a5aaf841af066f4edb5c894769d4539d，未动。证据根 `/Volumes/disk0/Khor/evidence/net`，ops同根/ops，source-refs.json由root单写。

用户已授权完整计划实现、Turing及macserver私库构建；main合并、发布、生产部署、采购、已安装App/config/store/resident修改不由此自动授权。本机source-only，不运行Cargo/Node/Xcode/fmt/产品。代码图谱先行，本树无.codegraph不建索引；CRG部分覆盖且滞后，0flow并非无影响。三个debug/explore/review技能此前已读并告知。

## 正在进行：先查同一收据，不重复提交

07:54已各提交一次新候选1bd9692/tree407ab/snapshot5bd853：
- Turing `stage-launch-turing.py net-node first-leg-facts-candidate-v2`，run **turing-20261111T075404Z-9850241b**，PID3592746，exec session16726已结束；前驱 `turing-20261111T074844Z-fdbdcb85/receipt.json`，role candidate。先恢复该session/实际ledger与receipt，确认run/PID，不重发。stage writer退出前禁止另一个ledger写入。
- Mac同源net-node，run **42983120010**，request42464aa10903435094cdabd2bcfdce13，已登记，exec session55033已结束。先查session及实际request收据；拿到Actions ID且Turing stage writer结束后用register-mac-run.py登记，不能把提交超时当失败。
- 前版Mac42981901522已failure，exact artifact13838176447、API zipSHA3dbe0c9955f3d6f08e880fcc0e3319e226f86898f85f37c3d0ba6d9400ca1591；collector和identity核已结束，9files/11controls，与Turing同一format.patch逐字相同，只有格式失败、零测试组。

前版Turing **turing-20261111T074844Z-fdbdcb85**准确6c3/tree4c0725bf6b055254b66379fb74cb4ca9d5e0586d/snapshot01669b09f542b978c45ab1038556083a3946ce3a，47.3056秒终态return1，10files/identity/空group核，只有format.patch，无测试组。已逐字应用远端hop.rs格式补丁并提交1bd9692。source-refs两个旧format-only项均已终态分类。

没有phone sampler；候选没有装到iPhone。

## 最新证据与实现

### 已验证的缓存与成功观察修复

287只改made_in_epoch：成功后count归零，保留最新observation/candidates，在原entry替换Arc token使旧完成失效；None纯helper仍删除。不新建map/task/timer，不强持Node/Connection。whole成功fixture SHA37aed9428b2bafa5097f651ce4c1ca6124af111477b2a6e8435bc6b3fa9b8161保持。

旧Mac42973738072真实hop67/2红，两新真实carried TLS/echo合同在使用中和close/detach后均多向旧middle发一次。候选 **Mac42975178687**全12/hop69/wires16/budget26/resume1绿，exact13835396432，zipSHA6b34b08ec3f76bfc92613e2018549c204547b68b17aef1718a7e51cef94ed5e0，20files/11controls/identity。首次GH只读EOF后重核同run成功，没有产品重跑。

准确同源 **Turing turing-20261111T071058Z-ea7d904d**已329.0583秒终态，21files/identity/空group，全部12组实际0failed，hop69/wires16/budget30/resume1。node-regression-verification-v1.json保存每组结果，ledger终态。此前6e5候选变化/旧登记双OS绿及所有旧红/编译/格式失败见上一状态和fixture JSON，不被本轮覆盖。

### 同ID真实换址：旧行为已红，新候选待验

`hop/candidate_address_tests.rs` whole SHA **4a552f9afd3f27f414324f36e8ef4b8432de1480f8b15bad0aae3c467f073ce7**，仅追加module，原69测试正文保留。两个合同在原NO_ROUTE后建立同middle key的新真实Tunnel，分别更换IPv4端口、IPv6-only地址，核稳定设备ID/新connection ID/实际路径和原本机network epoch，随后要求旧冷却窗口内完成真实carried TLS+echo。另有不变首腿仍冷却控制。双栈重绑发生在任何测量miss之前；不手改MISSED，资源实际关闭/await。

旧Mac **42978461882**（e704/tree028b22/snapshotcaf360）真实hop70pass/2FAIL，原69和不变首腿控制全过，两个真实新地址前置均过而恢复被挡。net67+1ignored绿，剩余组未跑。exact13836098921，zipSHA417ac8ad3f4c563cb6a559872578e3cfb7b7c2f6b960dee79c16bf4a2e8f40dd。此前42977479470仅格式失败保留。

新增cold自拨Tunnel控制 `candidate_dial_fact_tests.rs` SHA **661e9b3f4c090b89cd71f3d9811376008f3894a649f9750fee1925561b8eec9c**：真实Main已连而Tunnel缺失，第一次hop自建Tunnel并收到真实NO_ROUTE，后一次不得把自己建Tunnel当外部变化以取消退避；要求同wire、仅1次Ask，原network epoch不变。

旧 **Turing turing-20261111T074059Z-52b2aba1**，准确86dc5c60ec140bfaf012f924227fed3dc0c1eb44/treee027ca05df3fdef1217670d0f49c9cf781d716db/snapshot9b039f8d53f1fef69face0f13181cf80a91682f0，220.6269秒、11files/identity/空group，hop71pass/2FAIL：cold控制、不变首腿和原69都过，仅两个地址恢复失败。net67+1ignored过，其余组未跑。cold-control-old-behavior-v1.json实核；此前073749Z-1d2cea8c仅格式失败，精确返回patch已应用。

6c3→1bd修复：同一MISSED锁内取实际首腿connection ID、规范化IP/Relay地址集合、WeakConnectionHandle，比较时核旧weak仍可升级且open，防stable ID生命周期后重用；不含RTT/age/quality/排序/path ID。current Tunnel优先于Main，否则近期认证incoming。生成快照不await，锁顺序审计无registry→MISSED反向嵌套。真正tunnel Ask之前used_middle在同锁记录这次自建首腿事实，不推进observation或epoch，避免原NO_ROUTE被自身建线误解为新路。原3秒总预算、TRIES2、30秒至10分钟backoff、信任/暂停/单跳门均保留，不新增map/task/timer/强连接持有。candidate-address-facts-source-audit-v1.json记录依据，实际行为待两OS验证，不能称整个N2解决。

为先验证cold旧控制，自己未提交补丁曾保存到first-leg-facts-candidate-working-v1.patch，SHAfa87de83e363420400771732a6b92761ae3ff9ae31a2701b12abdffd064954e0，反向移除仅自身改动后提交测试baseline，再逐字重放。未覆盖他人WIP。

## 原netlab矩阵：绿色不等于最终SLO

6e5原三轮hop矩阵 **turing-20261111T064010Z-1501a97c**已终态1510.756秒，15files/identity/空group，原1线程3轮f4/f4s/f4i全部9green，非运行中。每轮3/0（479.50/491.36/483.85秒）。original-hop-verification-v1.json及原计划/ledger已终态。

f4i回peer relay收敛 **17.7/31.6/13.6秒**：先恢复公共Relay并visit等Relay可用，再恢复middle并启动Forwarded计时；不是首业务响应/总断连时长。原assert验journal started语义和120秒可达，非最终体验SLO。f4最慢echo3.4/3.5/3.5秒（人工1700ms relay延迟）。f4s最慢8.8/9.4/8.6秒（Relay每向2500ms）；有些round未观察到切换，never/0不能算瞬时恢复。

静态证据：hop.rs COMPARE_EVERY=30秒，进程全局LAST_COMPARE，compare_roads在查路径事实前先限速；唯一实际调用为link.rs resident beat的note_roads之后且!resting。13.6–31.6秒可能受轮询相位影响，是待实际关联/控制实验的假设。下一设计需有界事件触发比较和明确状态所有者，不能只缩间隔而忽略成本。

8cd原动态3d36cba8已27green，p2约0.94–0.99秒、p3最慢0.48–0.55秒。8cd原长稳088e00f9已4/0，p1/p1j/p1s各600秒零切换/退役，20×300ms CPU停顿最慢578.7ms；p5原150/300/600ms×4零Relay切换/零punch，最慢299.1/495.3/948.5ms。准确源与所有原日志在上一状态/verification JSON。不是手机或最终peer恢复接受。

## 仍须保留的失败与完整剩余范围

- Iroh47固定物理四项C-B/B-C已结束，不追加第五：C62030f1b和C817adf28均124业务/warm51绿，Go3569/3536及3820/3533ms；Bfe19c4ed初始业务10秒超时尚未replug；Ba43e01ff124业务但warm11/51失败、Go7499/3473ms。两C太少且接近原4秒门，不作可靠性/p99结论。全部4innercopies、3+3预算、8方向进展、原相邻成本界均保留；inner汇总流量非per-probe/整网/手机能耗。
- 严重6aa物理单向stall尚未闭环：两端无Go、quality_steps0、RTT缺失/不对称，carrier约21秒结束，outer40ms/open不足以证明byteflow；后续普通样本成功不是根因已修。旧Busy854、resume32timeout、CPUquiet/f4s、coldstart及Noq13固定四7.3/5.4秒/截断8f3均保留。
- NoRelay缺第二腿提示修复ff743旧3红→候选60绿，已随287双OS通过，但物理完整第二腿仍N4。Iroh47旧2红→候选vendor443/58/11；Noq13旧1红→443/55/11及双OS。主vendor57项=Iroh1–41/45–47+Noq1–13，42–44隔离。完整上游/退出审计及Noq1/5专门覆盖仍欠。
- N2本轮仅换新Tunnel地址；同一connection移动、目标有效地址提示/LOST本机身份namespace仍待。原TRIES2顺序尝试可能让首个挂起middle耗尽3秒，也可能让两个拒绝使未试第三候选一并退避；必须补真实合同并收敛每middle失败范围/有界竞争、取消输家所有权，不止通过当前较窄测试。
- N1速度优先已有实现/对照，但peer4秒稳定性、单向stall、慢路收敛、quiet/stalls成本及最终足够样本SLO未完成。
- N3交互与批量共享/分连接、背压公平、1/3/5设备和后台CPU/字节/唤醒/能耗待完整验收。
- N4实际完整两腿Relay、唯一/多middle、断腿、不对称、独立v4/v6及同时换址、Wi-Fi↔蜂窝、自动恢复、拓扑/故障域测量待闭环。
- N5全vendor审计、最终Quality/native/proxy/多设备/iPhone及完整前后报告待；仅部分回归绿不能完成全计划。

MacLab与Turing同LAN；macserver为另一台构建机，用户报告异网。iPhone localhost旧安装版现场反复关开App后某次经Turing恢复；5GA截图仅hinton Direct29ms。候选未安装，不能用旧现场冒充候选验证。手机最终沿可信GitHub/TestFlight边界，当前无真机采样。

## 下一步与时间口径

1. 先恢复16726/55033/87979及实际receipt，登记Mac ID、核format-only旧attempt，不重复投递；等待新1bd源两OS真实合同（预期hop73），若编译/前置/行为失败须分开判定，fixture和原门不削弱。
2. 收齐同源两OS后推进同wire移动/有效地址与LOST、未试第三和挂起第一middle真实合同，收敛失败作用域与有界竞争；并按证据处理30秒比较相位，不重造第二套恢复状态。
3. 按完整N0–N5继续，不因当前批次通过停止。保留所有历史红及未验证范围，最终phone窗口另行协调。

用户最新询问预计时间。**07:44 UTC/15:44新加坡时间**给出的当前地址修复批次估计为1–2小时（无新发现前提），不在compact后重置窗口；完整计划仍按数天计，结束时点不可靠。此前更乐观估计已失准，不能包装成接近最终验收。

## 执行/证据约束

Turing只UID1110 `/tmp/khor-netlab-feat-net-1110`，原flock+user/netns、Rust1.95/jobs16/nice10、50GiB起40停/90min；不sudo、不改host网。Turing计时期间不穿插其构建；独立Mac原jobs2/共享cache/资源门可同时进行。SSH BatchMode/ForwardAgent=no/StrictHostKeyChecking，不转发凭据。

stage-launch只一次并等exec结束再做依赖ledger写入；查同一PID和execution.json，timeout不等于失败。终态collect→identity→原合同；Mac核exactartifact/ZIP SHA/11control blobs和每组实际结果。依赖分析顺序执行，缺数据不补零，不覆盖原日志。source-refs单写。技能/AGENTS中早期“只存计划”不是永久暂停，以PLAN记录后续用户授权为准。
