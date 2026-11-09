# feat/net 当前状态与恢复入口

更新：2026-11-09 19:09 UTC（本地2026-11-10）。root串行，无子智能体。完整目标仍是“完成计划”N0–N5，未完成；用户明确“继续”并补充U11–U13，当前工作已恢复。goal服务19:06前最近读仍paused，自动跨轮续跑由用户面板控制；不得伪造complete/blocked或另建目标。速度/体验优先、个人可信设备网；没有合main、发布、中继采购/生产部署授权。

## 当前源码与已收齐证据

- **213物理IPv6前置校准的Mac格式预检仅format红，已终态采证**：Mac42681901453/net-unit，base34679e65/treea4bf836c/snapshot3aec9348/control ea97dd4a1d7d10b92b8201c4b49adba518ad9d66。新ignored mobility.rs排版而非编译/物理校准失败；完整source/transport/9 control blobs/hash核，原format.patch已应用。两Router另显式DualStack以满足冻结几何。尚无Linux运行，不预记物理成功；接下来新准确source先Mac format/net-unit，再同源Turing netlab-mobility-calibrate。新profile只fresh namespace内地址加/DAD/删，三阶段kernel JSON+8packet检查（7正1负），不当Node/手机迁移验收。control8 Python stub/Git数据检查绿，jobs/原profiles/产品策略不改。响应U12先补N0注入前置，完整N1不缩。

- **最新功能10ae1ca0：稳定接口通知不被无关候选更新推迟**。Interfaces::saw在filtered set等于seen时直接返回，仅真正新集合重置原1秒settle。生产文件只增加四行guard，210整个fixture逐字不变；空网/同址恢复、回到已通知集合、Local↔QAD来源翻转原合同保留。
- **210真实红→211 Linux/212 Mac同完整source绿**：210 `turing-20261109T185254Z-88fdeca5`，base16a0bafc/treee6aebe0c/snapshotcbbbac6f，fmt/net66+1ignored/hop10/wires11过，journal29过1红、118.09s、原group空；first IPv6变化/IPv4保持iteration失败，reverse未到。211 `turing-20261109T190201Z-d70459fa` 与212 Mac42679667230完整source相同：base10ae1ca0b0935f8b72e78aab89a65c1932a6333b/tree00ca0b6f31322d551402610b7b38d5dd8a7e48da/snapshotaa259f3d1376aa9a2e96a0cb257d336eb20a57e6，control9135b6d4311ce74ee3a90dac188f4b094f780e35。
- 211 root fmt/net66+1ignored/hop10/wires11/**journal30/Linux24**（91.86s）全过，206.47s、原group空；212同源root fmt/net66+1ignored/hop10/wires11/**journal30/Mac20**（50.06s）全过，完整source/transport/9 control blobs/artifact hashes核。整fixture SHA `d66be0e62f349d2298d666c8470e09403f4bee6b53832d9e2fbf9f839315a822`；整生产文件除四行guard外逐字不变，proof `stable-interface-red210-green211-212.json`。green两族iteration均到；不是物理重编号、真跨网或手机能耗证明。
- **211/212均已收齐**（213新任务见首条）：211/212终态完整采证，queue空；`source-refs-audit-20261109-interface-green-212.json`已逐run核全部212 snapshot/tree refs。任何后续恢复先查实际句柄/branch/diff，不按摘要重发。

## Peer relay与速度仍待收口

- **201 hop_reference_v2校准9/9**：`turing-20261109T175508Z-912bcc0c`，base9c2e7348/treedcad2092/snapshotd862e1ae/control7598180，1228.41s、原group空。576/576全ok（稳态540、bootstrap/setup36），三case每轮双向n30，raw与summary的p50/p95/max一致；实际两端nft relay input/output drop、peer-hop=off、60秒准备和最终10秒hold核。第一/三轮automatic约147/149ms、fixed-carried约145/146ms；第二轮automatic189.106/173.863ms，同轮fixed-carried145.221/145.701、fixed-relay190.427/193.093ms。可用性绿不代表最快路稳定胜出，v1/v2不混。
- **205/208 lifecycle各3/3**，每run192/192全ok、9诊断文件samples off、原group空；同完整source d90bf666/tree9a7bd3d0/snapshot1a501da8。205 `…183013Z-fdf5424c`/control32fa23c，416.55s，无release，未复现201；208 `…184729Z-59b8eed7`/control9135b6d，435.58s，有双向不对称起点。208第三轮当前serial1（非旧owner）经两次shared_comparison释放并close，测量前31秒，约16秒后serial2重建。原值Carried110.692/109.433、Relay132.754/130.884ms，samples1/0，age约2/4秒；`measured-release-trace.json`保留。符合现行quarter margin的“不足收益”逻辑，不等于替代路明显更快；不能盲降gain或称201因果已定。
- `middles_with_cost`的Measured按两腿成本排序，但冷Legs::Dial等仍有direct首腿优先和候选限制；单middle稳态不能代替全候选冷/暖设计。新候选有界验证、足够新鲜确认、无消费者/试用到期回收应分清，LOST十分钟与“尚未证实收益”不能未经合同混为一类。下一步反例/正控与速度、字节/探测/能耗一起收敛，详见NEXT-DESIGN。

## 用户补充与设备映射（硬要求）

U10：A=macserver（Mac），B=Turing（Linux），两服务器天然异网；P=iPhone。不得再等另找两台电脑，实际两服务器跨网尚未运行。Mac SSH单次banner timeout不证明离线；原GH guard构建通道可用，209/212已实际通过。

U11：Peer relay是可信设备carried候选，与Direct IPv4/IPv6、公共Relay、既有SSH承载共同按实际完整路径/双向体验竞争。覆盖多middle不同速度、middle自己换网/离线；尊重nonrecursive/trust/paused/only_mine/n0。

U12：IPv4/NAT外部映射或端口与IPv6隐私地址/前缀分别变化、同时变化、Wi-Fi↔移动网/地址族切换、双方与peer-relay移动、连续变化后旧结果晚到、休眠/恢复与旧缓存都是硬验收。通知合同只是前置一项；物理注入/发现发布/首个业务/最好路/长流/全部outcomes/成本与iPhone实测未完成。旧地址不能吃完整预算，旧epoch不能撤新路，写请求不隐式重放，健康共享连接不随任意hint全清。

U13：利用Turing128逻辑CPU、约504GiB RAM加速开发，先减少重复工作；不污染计时或缩短发现/hold/故障/稳定门，不默认派发子智能体。当前jobs16保持。

## Turing缓存加速已实证

控制9135b6d已push：原owned root/flock内复用cache/source，先核owner、非软链、tracked clean、准确source tree，未知/脏目录保留拒绝；不reset/clean/删除已有目录。稳定路径与未变文件mtime可复用Cargo，web占位内容相同不重写。Mac保持原隔离checkout。

206 `…184220Z-81707871`与207 `…184314Z-9e2b17cb`同205完整source，两次fmt/net66+1ignored都过，构建17.05→0.41秒，job51.52→35.84秒，实际tests11.79/11.93秒；仅一组同快照初始/复用对照，不承诺通用倍数。`turing-source-cache-pair-206-207.json`留证。driver2/runner6共8 Python控制检查含小Git fixture、mtime与拒绝脏/未知/软链目录，不是本机产品执行。206原输入短control SHA在启动前已锚完整SHA/ref/逐blob，原输入保持，后续stage自动40位。

## 必须保留的先前实证

- 189旧watcher真实反例→193 Linux/194 Mac原整fixture转绿：D077（功能62cf）`Links::detach_attached(peer,carrier,serial)`同锁核当前owner；只有真移除当前owner才lost。整fixture SHA f620fc97…，不回填182/162因果。211/212继承hop10绿。
- 178共享消费者真红→179 Linux/180 Mac整fixture绿：3948872d核Main/Tunnel/served全部live消费者、stable ID去重，unknown保守。iroh peer共享selected FourTuple但每conn open paths不同，不能说每conn独立selector；211/212 wires11仍绿。
- 195 held_precondition_v2三轮3/3：实际hold10444/10460/10455ms后故障，原1500ms恢复/30秒稳定门通过，73/73全ok。负向timeout/reset未冒称覆盖。190原诊断0/3、182原hop8/9、162/168历史均保留，不被合同绿抹去。
- N0 recorder v4真实183校准：5请求ok2/error1/timeout1/cancelled1，SIGSTOP/CONT与原期限实核；caller取消不证明远端停止。paths/hop全outcome保留，旧v3遗漏error不能补零。
- F8真红→150/151；F9 tick/F10长Accepted取消真红→160/161；当前211 Linux24/212 Mac20重新覆盖原合同，四Linux namespace合同不靠Mac替代。
- 176/181固定核stalls均原门2/2，但candidate p1s18 punch/728ms/更多bytes，对照3 punch/537ms，host load不同、单顺序pair，不定单补丁因果或总体提升；真实resident约275–280 threads，尚无能耗结论。

## 下一步与完整范围

1. 继续N1：以208原始quality/样本/年龄与逐wire时序补“稀疏/静默/真正更差”正反合同，分清验证/确认/成本回收；冷/暖Direct IPv4/IPv6、Relay、SSH、peer relay含多middle的有界竞速/持续优化，不盲改gain/时间门。再按准确源跑hop/recovery/dynamic/full、同版本baseline和固定好路。
2. U12物理校准：真实namespace IPv4映射/IPv6地址与前缀分别/同时变化、旧结果晚到，完整phase/outcomes/资源回收；现p4仅来源翻转的无误报控制。先冻结基线/目标，再改通知之外的策略。
3. N4实际macserver↔Turing：Mac主动配对fresh Turing身份的私有临时能力方案，Node::serve/open_as/invite/pair/devices/ls_of与同进程stop_serving API已审计，尚未实现。需要lease/绝对期限/身份/角色同步/真正join/owned-root清理合同，ticket不进源码、普通日志或通用artifact。完成服务器后接iPhone；不碰已安装App/个人网络store。

| 工作包 | 仍必须完成 |
| --- | --- |
| N0 | 最终目标/容差/样本量及p99充分性，全负载所有outcomes、v4/strict hold/reference v2同版本baseline/candidate/固定路与负向校准；不混旧量测版本 |
| N1 | 201慢轮/208当前owner确认，162/182未定因，冷暖多候选/双向与多middle、稀疏/噪声/CPU/防抖/故障/恢复、探测成本与完整矩阵 |
| N2 | IPC剩余预算/取消贯通（正常half-close≠取消、写不重放、读≤1重试、0RTT信任保持），长多阶段原by、两middle合计3秒、输者/晚候选回收、近期地址/重启、jitter/事件合并、quiet epoch与close-vs-offer竞态 |
| N3 | 真实Node32席位/content32、Custom1024帧/256KiB外层队列、SSH/hop混合公平/共享vs分连接、1/3/5设备后台、CPU/字节/唤醒/手机能耗；Noq FIFO_v2低层数据不能替代 |
| N4 | U12完整移动矩阵与Mac↔Turing↔iPhone真跨网，冷暖/双端不对称、登记/出口/整机故障域、布局/容量/运维实测；不是采购部署 |
| N5 | 48 vendor补丁（Iroh37+Noq11）逐项覆盖/上游等价/退出，Noq1/5等缺口，准确最终源可信Quality、Mac原生App/代理/多机/手机/能耗；证据前不升级/删补丁 |

## 工作区与执行规矩

- 产品：`/Volumes/disk0/Khor/worktrees/current/khor-net`，feat/net；control：`…/khor-build-net`，feat/net，9135b6d已push。先核实际HEAD/diff；docs提交不改变上面的211/212准确源适用性。
- 旧baseline `khor-net-baseline` detached6485六fixture WIP、CPU baseline `khor-net-baseline-cpu-v3` detached6485七staged fixture/helper均未触动，原refs/ledger保留。
- 本机source-only，禁Cargo/Rust/Node/Xcode编译、产品测试、formatter、下载可执行物。只用Git/Python stdlib、脚本stub与数据核；不得改installed policy、ensure-budget、已装App/生产store/其他worktree。
- Turing root `/tmp/khor-netlab-feat-net-1110`，owned单flock重型槽，Rust1.95/jobs16/nice10，50GiB准入/40GiB stop/90min，计时不并跑编译/其他重负载，无sudo/宿主网络改动，仅user/netns；SSH BatchMode/ForwardAgent=no/StrictHostKeyChecking=yes，不传个人密钥/token/agent/签名/模型凭据。
- Mac走macserver/khor-build现有guard单槽/jobs2；临时ticket未来只能给可信controller，不复制个人能力。202两次pre-job GitHub内部错（jobs=[]/无artifact）留为执行器事件，209/212后来成功；203错误cwd快照仅stage未启动、source预检已修，均不得算产品结果。
- `evidence/net/source-refs.json`、`turing-next-matrix.json`、各receipt/collection/report hashes为句柄事实。源码/控制/输入先锚，再改动；实际/proc cmdline＋terminal group，不信状态文件代替存活，观察超时不等于停止，不盲重发。
- ops/submit-turing.py stage显式task/revision/control/role，snapshot固定产品cwd并核预读HEAD，复用旧source用--source pure metadata或Turing receipt；launch需前驱完整终态采证并核group/flock/inputs。collector返回101可能是成功采到产品红，先读collection，不重复覆盖。
- ops/verify-mac-record.py只核GH/Git/artifact身份，分类要读日志；Mac临时source/bundle分支只有终态、留证/所有权/SHA一致才清理，复用者不删原请求分支。当前未清理，不擅删。

完整计划与硬要求见PLAN/REQUIREMENTS/ACCEPTANCE；决策边界见NEXT-DESIGN，原始演进见RUNS/REPORT。本页重写前逐字状态保存在HISTORY-20261109-before-mobile-interface-verification.md，再沿历史链；历史在途只代表当时。外盘缺失遵守恢复入口，不按摘要覆盖WIP。最近给用户完整计划粗估3–5开发日，受新回归、真机与能耗影响，非完成承诺。
