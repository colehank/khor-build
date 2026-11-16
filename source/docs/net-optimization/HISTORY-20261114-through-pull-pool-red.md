# feat/net 当前状态与接续入口

更新于 2026-11-14T10:14:32.629216+00:00。用户授权持续实施与远端验证，活动目标“好 完成完整目标”仍未完成。root串行、不派子智能体。完整[GOAL](GOAL.md)、[PLAN](PLAN.md) N0–N5和[REQUIREMENTS](REQUIREMENTS.md)保持。未获main合并、正式发布、现装App替换或生产Relay采购部署授权。

此前STATE逐字归档[HISTORY-20261114-through-separate-pull-screen.md](HISTORY-20261114-through-separate-pull-screen.md)，SHA256 `85c364bde3626ce7627a3be9055ad1a477a557a2601cbf682d3b58aafd64ba53`；该档案内的pending、下一步和“尚未运行”均为历史，不能用于重派。E=`/Volumes/disk0/Khor/evidence/net`。

## 当前执行与下一步

**当前Turing唯一在途：turing-20261114T101752Z-6a8e53e2**，准确同一个晚到文件公平性用例/控制/5秒观察，control92940cb697af028a7beb05c45036a5cc169771c3；E/pull-pool-head-progress-native-plan-v1.json。候选 HEAD22863f5f1a647d9a639f1e8d394754c868809e5f/tree8887d284c768afb12695f5b5c63dee942ffaa4be/snapshota7e22ae2f47a44c904729a80e7961a62442a348c，在khor-net-pull-pool-contracts-v1 / experiment/net-pull-pool-contracts-v1。只改link.rs/link_pull_pool.rs：普通bulk可立即取得则优先用bulk；当前文件cursor会被阻塞时使用既有8个foreground额度，尚未发送的prefetch只在成为cursor时提升；watch随commit/reset推进，不重发RPC、不重置原deadline、不加task/timer。此时是否GREEN以实际收据为准，不重派。

**真实公平性RED已闭：101209Z-77cafaa4，101/96.107202秒，准确0pass/1fail/0ignored**；11files/identity/owner/原fmt，E/run/pull-pool-late-file-red-verification-v1.json。原02e产品策略未改，仅0c7819 test/cfg(test) instrumentation：A首块已提交，8bulk持有且7块实际parsed-ready留存，A head从不释放；B首块成功，但5秒内只发出offset0，完整文件不前进；取消后A实际STOP、credits/extra额度全部回收，cached Main原stable-id真实Sessions健康，夹具关闭后才断言失败。**这是bulk-ready占满导致另一个文件饥饿的准确证据，不能拿24 lease绿声称文件公平。** 原源/控制/断言/数据保留；修复当前运行与该test module blob9816a50187c2b457651bbad64a8dbf9aec8769d8逐字相同。

另两个并发完整文件fixture原v1源码已保存0c7819，但从未执行。首内容额度在后续设计中扩为“必要的当前cursor进展”而非仅首块；执行这两例前另固定v2资源记账，区分bulk与foreground持有，保留8/32/8上限、全部精确bytes/唯一offset/真实STOP/一extra与清理门。未运行不是失败或通过，不能改写原late-file RED。完整1/3/5业务、公平/取消等待、成本、Release/Mac/phone仍缺。


**pool24窄门已闭：095903Z-7033f56c，0/108.249557秒，24pass/0fail/0ignored**，11files/identity/owner/原fmt；E/run/pull-pool-native-verification-v2.json。准确runtime候选02e2b4c54efa91fc54030bbf94555ed1e86dd44e/treec37d7032569cec73e13dc916a3ded43b07a17733/snapshotac37754c3065fe1868b076653d08ec9d3c193469，worktree khor-net-pull-pool-v1。原20所有强断言保持，新增4实际QUIC合同：双owner只1extra、drop1不关/drop最后才关，主动退役旧Arc保留仍释放permit且替代不受旧Drop影响，同key新endpoint不跨scope复用，4实际peer extra满时第5不dial后退且释放后真实正校准。只有连接lease层通过；完整并发文件、ready额度、caller/公平/成本仍未验收。原eacc306/20与Macfe61不可外推当前源。

首轮095600Z-da6c1c18是compiler-only（101/53.979810秒），stop_rpc旧fixture把新Arc又套了一层；0个测试执行，不记产品行为红。准确单行修正1eb→02e产品bytes/断言不改，原24人口不变，后续上述唯一有效运行已通过；原compiler收据/原源/plan保留。设计参数4extra/peer1、bulk32/peer8、首内容8仅候选，不称最优。默认shared，vendor/ALPN/现装App不改，完整N0–N5保持。


**上一轮同源标准12组已闭：turing-20261114T090158Z-0ddf1257，0/489.875419秒，306pass/0fail/原ignored1**；21files/identity/owner/原fmt与全部原case/忽略逐项核对。E/run/pull-negotiation-fallback-standard-node-verification-v1.json，准确控制 **5d3691f252f9dee0297fa3c494374be6119bca29**，源码 **e4e433a57964495d202d6f0eecdf8e3fe65fc323** / tree **eacc11bf9cd05dd50862bbe78e92d2b88a60d653** / snapshot **1a7b9dccc9c5b9f5ff075b1d02b45c592b5bfe2d**。worktree khor-net-pull-negotiation-fallback-v1 / experiment/net-pull-negotiation-fallback-v1，clean。原fe61标准294＋新增准备边界8＋运输/政策4=306，原12命令及原ignored不改；20明确mode-ON与14contact全过，大部分旧合同仍shared，不能称全部旧场景mode-ON。当前源尚无Mac/Release性能/真机验收，fe61的Mac290不能代替eacc证据。不重复已闭标准306/窄20/旧16。

下一实现设计已写[PULL-POOL-DESIGN](PULL-POOL-DESIGN.md)：actual canonical Arc<Endpoint>归属、每peer活跃共享extra、node/perpeer连接与slice额度、ready结果持有额度、公平推进与退役/调用者边界；候选4 extra/peer1、bulk node32/peer8、首内容8仅为待测参数，不称最优。此段是实现前设计记录；当前pool候选已实现并在顶层24项验证中，尚未接受。先在新独立分支实现并闭真实并发/生命周期合同，再进入发行性能与1/3/5成本；旧混合screen仍不通过，完整N0–N5保持。

**当前源窄门已闭：085928Z-39e34269，0/104.226576秒，20pass/0fail/0ignored**，11files/identity/owner/原全局fmt；E/run/pull-negotiation-fallback-native-verification-v1.json。准确RED085432Z-798e08a1（原runtime ba423cb/treee692）先2pass/1fail，真实额外ALPN协商失败导致文件提前Err，cached Main仍真实Sessions/原stable-id；清理后才断言失败。修复后原三项transport测试模块blob完全相同：真实两个协商错误后offset2在held1释放前发出，完整13唯一offset/精确文件/part和note清理全过；真实Incoming.refuse暂停信号、ApplicationClosed、LocallyClosed继续terminal，原16隔离/准备边界全保持。只有link.rs产品变动：具体QUIC运输错误独立Transport，optional TimedOut/Transport回cached；原resting/应用/本机/未知auth/config/caller到期终止，聚合不让另一条运输失败或自身dial超时擦掉已知显式拒绝。原primary词句/backoff/early retry条件维持；无新任务、timer、deadline、ALPN、vendor或默认模式改动。详[PULL-FALLBACK](PULL-FALLBACK.md)。这不是生产延迟或完整N3接受。

前一实际探针084909Z-ec61253e已101/90.142939秒、11files/identity/owner/原fmt闭；2pass/1fail。其Incoming.refuse()明确对应既有wires::resting_close的CONNECTION_REFUSED→Asleep，且生产暂停/后台确实drop Incoming；测试把它当可恢复运输故障是错误前提。E/run/pull-transport-fallback-fixture-semantics-verification-v1.json，原26b241源/plan/断言全部保持，**不能称产品回退bug或改掉暂停语义**。原错误假设verifier未执行。新独立design在运行前固定：set_alpns仅让额外握手协商失败，原Main继续服务，实际accept/协商错误、offset2在offset1被hold时发出、完整精确文件与身份/cleanup、app/local terminal全保留。

上一批已闭证： 最新边界任务 **turing-20261114T080729Z-b976002c** 已0/104.196722秒、11files/identity/owner/原fmt全闭，**16pass/0fail/0ignored**；E/run/pull-preparation-boundary-native-verification-v1.json。control **b37158c0e2040aedb4d0d9a65b1671ecd861042e**，独立worktree khor-net-pull-preparation-contracts-v1 / experiment/net-pull-preparation-contracts-v1，formatted HEAD **59b841424a24ca2e2899e5c4489fca16510bab0f** / tree **b4b59602e7dc6280309b236ebeff91fdf349ba9f** / snapshot **892046078337ec757beecc94238a13c35ee19d1c**。相对fe61只两个cfg(test)文件，runtime/vendor/参数未改。新增8项真实QUIC边界：0/1234/262144字节文件不额外handshake（含真实positive observer校准）、本机/peer暂停/休眠标志/过期caller四项terminal且不dial、实际Incoming未accept时取消后cached identity保持并能再新建extra；旧8项隔离/首内容合同保持。不重跑已闭组；本门不证明真实手机休眠、性能、准备传输错误回退或caps/fairness。

下一步：按PULL-POOL-DESIGN在新隔离worktree实施pool/caps、公平性和生命周期合同；原eacc保持不变，已闭306/20不重复。之后Release共存性能/成本与移动验收，完整目标保持。随后继续pool/caps、公平性、Release共存性能、移动与最终完整目标，不能以边界单测代替N3验收。
formatter080133Z-141756d2已1/45.917981秒、10files/identity/owner闭，compile/native0；原全局fmt gate完整导出patch SHA **9b9f3a2a29114f76626b2bc49754ba1c4352b203fae16fc406fe294851da0e83**，只两处test文件；准确apply/commit59b，当前native仍执行原fmt gate。首个本地严格token断言在apply后拒绝Rustfmt对两个peer_pause match表达式加保值块；精确reverse-check及仅这两处归一化后核验，未重apply/远端重跑，原件保留。E/run/pull-preparation-boundary-format-verification-v1.json。

调度准备一次错误控制SHA被本地rev-parse拒绝，run目录/ledger/SSH均未开始；E/pull-preparation-boundary-dispatch-preparation-v1.json。改为从冻结JSON读准确b371后唯一stage/launch当前run，不能重派。

最新同源码标准原12组 **072757Z-352286a4** 已0/487.964780秒、21files/identity/原fmt/owner全闭：**294pass/0fail/原ignored1**，8个明确mode-ON与14contact全过，全部原test名/原ignored逐项核对。E/run/pull-preparation-standard-node-verification-v1.json。准确formatted HEAD7542ea8960c2bd35df534c437f7d05ddd0b69cdc/tree fe61be7532678bf40cce65d30f1286f51b4e815b/snapshot51e316bd77c8df1889053f86aa68ae5831c52880，worktree khor-net-pull-preparation-v1 / experiment/net-pull-preparation-v1，E/pull-preparation-implementation-source-v1.json。另有5个Window原生合同见下13窄门，不在标准12组计数里，不能双算8个重叠isolation。完整N3/Release性能/成本仍未接受。

**Mac同源标准回归已闭证，无在途Mac**：GH **43571504228** / job **132376699636**，control68cadb57a0ed72d40a4ca9a843e86b9c6797bf34；与Linux同完整source snapshot51e316/treefe61，**290pass/0fail/原ignored1**，差4原Linux-only cfg逐项核源，8明确mode-ON/14contact全过。E/pull-preparation-mac-verification-v1.json；artifact **14104736988** / **37828B** / SHA **3088eabca140fa0120f0af081d68ed88f8b11389d59ceb739483f566fcaddd7e** / ZIP CRC/20files、全部11control blobs/source/bundle/ARM64/rust1.95/jobs2/installed guard998f及原50GiB门全部核。host采集free70030983168B；无App打包/安装/发布，其他LIVE23961未动。独立progress已pending false；仅完成严格验证后才completed-only追加共同SourceRefs，该Mac闭证时共同ledger全部pending false；当前在途见顶层。

原12命令2078B与Linux5d逐字同/SHA74d7cb不变；requestf90151d4d99346668208082d9b29f328、bundleSHA ca616894027c7c8337df62dd55348465bca08101a335be99fb5888c253d217b1/22033302B/blob e03996ec4e69d5f84f9ad8d018c0d8f6e1faf060、独立进度与pin ref全部保留，collector/strict verifier均已唯一执行。不要重派标准Linux/Mac或覆写proof。

Mac首次本地手动import缺scripts路径，khor_lab import在upload_snapshot入口、任何上传/bundle/dispatch前失败；原request记录保留，E/pull-preparation-mac-submission-recovery-v1.json。补sys.path后恢复**同一request**、只dispatch一次，已发现上述唯一run，不能重派。

**首内容修复窄门已闭证**：072300Z-4cf5e570，0/118.208476秒、12files/identity/owner/原全局fmt门通过；**8 isolation＋5 Window=13/0/0**。E/run/pull-preparation-native-verification-v1.json，新first_content=true/cached_main_control=true/fixture_closed=true；与070345准确RED的新测试preparation.rs和原真实QUIC Fixture文件blob逐字同，只改产品实现使红转绿。原7event/实际extra完成取消/缓存身份/真实STOP/读retry1/写不重放/peer pause及其他peer负控制全过，新Window扩容保留held prefix且后续只发一次。此处5秒夹具观察不是生产延迟、非完整回归/Release/Mac/成本/N3接受。

formatter **071746Z-25a10da4** 已1/45.915458秒闭证，10files/identity/owner，compile/native0。原顶层check_format先于定制formatter-only分支执行，因此formatter-only.json未执行；原全局门完整导出disposable clone的三文件patch SHA **beed5f9f763bd28fdd214a62c588fc65f51b5f3c936983882407cd2bb5c44f94**，原exit1保留，不称格式已绿。该准确patch已reverse-check/规范化非格式token验证后apply/commit7542。首个本地whitespace-only校验拒绝Rustfmt移除一个optional trailing comma，发生在已apply后；没有重apply或额外native，修正为允许该Rustfmt格式变化后核验并commit。E/run/pull-preparation-format-verification-v1.json。当前原生任务仍必须通过原全局fmtcheck，不降门。未格式26bfe7/0a7a15与原件全部保留。

实现设计E/pull-preparation-implementation-design-v1.json在改源前冻结。首slice先走cached Main；小文件首slice完成则不新dial；大文件追加prepare future由pull持有，与真实cached请求select并行；每请求持有额外Line的Arc，最后owner负责关闭。可选prepare未就绪时先1窗口，再随一条缓存请求完成或handshake结束widen到原8，不移除/复制既有pending；无detach/timer/新deadline，Shared仍原行为。该fe61首内容基线只对typed TimedOut后退cached；后续eacc已在独立分支细分Transport，准确新验证见顶层。Linux294/Mac290及13窄门已接受这一首内容修复的限定合同，未采用为最终方案。

该原生门已完整闭证（见13窄门），不再重派：原7个isolation＋新首内容共**8**，原Window4＋新增“扩容保留held prefix且后续各一次”的**5**，都原有界真实caller/cleanup，不重复已闭标准组；全组绿后再补剩余负向/小文件/caps等，最终仍需完整标准/Release/Mac。新设计两项full-pull隔离fixture明确允许缓存首块/已在途请求，保留合计全部唯一offset、精确bytes、实际extra连接、原8总pending、实际STOP_SENDING与cached stable-id正控制；旧S dad430分支与所有数据不变。不要拿旧“全部file都extra”的架构断言当新首块设计的安全合同，也不能削弱上述完整合同。

**准确RED已闭证**：070345Z-658df53f，原未改产品runtime28e47f/tree d750449（仅加测试）native101/134.268274秒，原fmt通过，准确1test/0pass/1fail/0ignored；marker first_content=false、cached_main_control=true、fixture_closed=true，明确在清理后仅新首内容断言失败。E/run/pull-preparation-red-verification-v1.json，11files/identity/原进程组清理全闭。5秒是夹具观察界，非生产期限或性能门。**这是新发现准备阻塞的实证，不是旧混合场direct_after红的根因**。不重派RED。共同ledger按executor与pending筛选，无在途Mac。

原4个p6速度优先合同同准确普通Release已0/116.199051秒且16files/identity/source/原CPU/owner全闭：**065726Z-daaec6b9**，E/run/current-speed-first-contract-verification-v1.json；快Relay/快Direct及两条固定路参照各30真实读，全部无错误、原p95<200ms保持。快Relay148.075ms、快Direct31.289ms、Relay-only147.675ms、Direct-only28.076ms；双端均持有目标路径/down=None。原max Relay246.701ms、Relay-only266.634ms也保留，不称全部<200。只接受原四功能合同，不代混合screen或完整N1；不能再重派。

最终Link复查已全部结束，不再追加同一混合诊断。有日志3场063128/063616/064113、无日志3场064509/064908/065321均闭证：**4320/4320读取、6完整192MiB文件**；最终两端均Direct/down=None，0/6复现原红。有日志各16files/原CPU/owner/capture三校准/1128stream配对/两Main与独立Tunnel全过；无日志各13files且确认应用详细trace与外采均关闭，原CPU/owner/source/720人口/全部门保持。E/separate-pull-path-diagnosis-summary-v1.json与separate-pull-final-link-unlogged-summary-v1.json均completed-fixed-diagnosis。这是**未复现，不是已修复**，原九块screen继续停止且不通过，8A/9B不运行。详[FINAL-LINK-DIAGNOSIS](FINAL-LINK-DIAGNOSIS.md)。

fixture-only producer **062632Z-8dc26135** 已0/240.932852秒，23files/identity/严格原Release来源闭证。独立worktree khor-net-separate-pull-path-diagnosis-v1，source **ddc62a719cbbc0445617b7940ddea96ca68f85e6** / tree **e27863d51c82001603728bf3e7ff668fa9c4d2c3** / snapshot2c3dcc0bdd8c91219041e29484f86b63b105e05a，只有workload.rs保存/输出原门使用的final Link及对侧Link。产品0变化、门槛0变化，CLI仍准确**9add2509af6c3d96d3ac82783bc7bbc9803a931ffca34f21355ee8dc897404ca**，netlab **3853081a55af2f0bfa1ed5c46fd7592f99306982c58c9a3ba92902572732a8cf**。E/run/separate-pull-path-diagnosis-compile-verification-v1.json。design在改源/运行前固定三次有日志诊断，不作性能接受；compile与runtime freezer均已唯一执行，不能重派。静态分析E/separate-pull-path-failure-static-analysis-v1.json保持证据边界。

**独立文件QUIC三组screen已停止且未通过。** E/separate-pull-three-arm-progress-v1.json phase=`stopped-isolated-hard-functional-failure`；不可运行原第8A/9B或补替代场。停止报告E/separate-pull-three-arm-stopped-summary-v1.json已生成，原计划、阈值、所有失败与未执行人口保持。

历史诊断起点（下述已有固定6场未复现结果，不再追加同场）：定位第7C高延迟原生失败`direct_after=false`。该场720读全部成功、192MiB文件完整、PTY关闭/120+120实际重叠/队列读回均通过；最终是否Direct且非down的组合检查失败。原complete只存bool，未存最终Link对象，不能据此判定实际掉线或单纯显示错误。x journal有一次path selected Relay（RTT302ms，质量估计301.38ms，Direct RTT306ms/质量312.47ms），但这是切换后快照、不是决定输入，也未绑定到最终cached Main；不能直接宣布根因。先查Wires sampling、实际连接编号与选路；必要时只追加独立诊断，不改变已冻结screen结果。用户要求速度优先，修复须符合这一方向。

## 独立文件连接实验的准确源与证据

worktree `/Volumes/disk0/Khor/worktrees/current/khor-net-separate-pull-v1`，branch experiment/net-separate-pull-v1，HEAD **dad4306132a51e1807da2058988e13f315b262ac** / tree **0db6508aa2bd2094b9dfcfee6c35715a0af20f17**。默认shared；内部实验flag启用每pull一条额外现有Main ALPN连接，目录仍cached Main、终端Tunnel。未采用/安装/发布，详[SEPARATE-PULL](SEPARATE-PULL.md)。

- Linux045906Z-abc0de50：原12组**293pass/0fail/原ignored1**；原286默认shared，新增7项明确mode-ON真实QUIC合同。Mac GH43563143698/job132351061154同源**289/0/原ignored1**，差4条原Linux-only cfg；新增7项和14contact全过。E/separate-pull-mac-verification-v1.json，artifact14101007961大小/SHA/CRC/20files/全部11控制blob/ARM64/rust1.95/jobs2/installed guard998f及50GiB启动门已核。无App安装，其他任务LIVE23961未动。
- 普通Release producer050842Z-6b257594，0/562.186521秒，CLI **9add2509af6c3d96d3ac82783bc7bbc9803a931ffca34f21355ee8dc897404ca**，full netlab **4ca36fb23937b0ceecff08c7d5b3b7bac64f2cbe071e79ccc3fa957ff7c0b3bd**。source snapshot0a97f374b33f6d7a3b5840b48c55fbf001cb7700、原Release/features[]/thinLTO/CLI强制refresh actual fresh=false、全部source与复用合同准确核验。E/run/separate-pull-native-compile-verification-v1.json。
- 真实布局有日志诊断shared051918Z-ca81d523、separate052331Z-eea39ea2均通过：1440读、2文件、2256响应stream配对。shared目录/file一Main；separate两Main且pull_wire_opened准确匹配filewire；终端均独立Tunnel。采集校准/界/CPU/owner/flag finally恢复已核，只证明布局，不计性能接受。
- 正式screen control **6ee198d424e0313d71812c1f0c5343947be50f99**，plan SHA **9d32bff84c5cda4e00dbb7bacab1f8d27370ca49a3161a2c18b4f881334ab6b2**。A原public-main runtime c3b098bdbf52a9b3b38e887ae74b90155292b544/treef6c239/snapshote22263；B/C同S普通ELF，仅shared/separate flag不同。
- 固定ABC/BCA/CAB九块、每块low100ms/high400ms、原720读/16或192MiB/caller2秒/file120或240秒/owner600，主门C/A相对10%+20ms与90%率、固定原main绝对门保持。实际仅运行7块，**10080/10080读、14完整文件**；整体功能false，未完成九块对照，不能接受N3。

|块|组|Turing run（20261114）|原生结果|
|---|---|---|---|
|1|A|053604Z-010c3ac7|0，1440读/2文件|
|2|B|054110Z-5872daed|0，1440读/2文件|
|3|C|054711Z-5f383a10|0，1440读/2文件|
|4|B|055222Z-35b35b63|0，1440读/2文件|
|5|C|055734Z-f273a63f|0，1440读/2文件|
|6|A|060328Z-4dd54696|0，1440读/2文件|
|7|C|060906Z-6589b6ce|1，高profile direct_after=false；1440读/2文件均成功|
|8/9|A/B|未执行|按预先hard-fail规则取消，不替代|

C三块每cell360样本的描述性p95：low bulk目录79638μs/PTY77024μs，high目录409698μs/PTY332833μs；high文件中位23.98445Mbit/s。均不抵销功能红，也不构成未完成A/C对照结论。两source fresh preflight052833Z-419e8012、053223Z-815dd1d4各原coldTCP高profile红保留，全部传输/几何及其他4门通过，依原MODEL-DECISION分层admit，不称全绿。

所有v1 freezer、record/verify/summarize helper和plan已冻结；不要覆盖输出、改阈值或重派已闭场。source fixture10个公共文件字节相同，lab.rs只增IPv6工厂、完整还原后逐字等于原V4工厂；E/separate-pull-fixture-equivalence-v1.json，不称11文件全字节相同。元数据准备错误及原生红均在历史保留。

## 已完成且不重复的恢复与基础修复

**纯IPv6固定功能矩阵完成**：E/ipv6-only-mobility-verification-v2.json phasecompleted，5类各3次、15/15场、**630/630真实读取**；双向首次最大1705510μs，原2秒门。实际old移除/new UDP源、无非loopbackIPv4、IPv6 Relay TCP、路由/CPU/PSI/owner核验。Relay-only3场恢复约0.507–0.509秒，nft反射器放行/其他UDP封锁的正负控制及两侧Hop::Relay通过。

边界：UDP阻断在配对前，证明已建立Relay地址变化恢复，不是Direct遇突然UDP黑洞；IID变更允许add/DAD/delete重叠，不证明原子断旧。非NAT64/DNS64、新peer发布、睡眠、手机、peer relay自身移动或成本验收。详[IPV6-ONLY-MOBILITY](IPV6-ONLY-MOBILITY.md)。V4d6341a1/tree1cbe5c0d，producer043812；产品CLI443c6e010293b83e7c0e963b6eb96edb3897dd4c0d6ada0bca9f6fca02666a43不变；前4与后11仅nft literal修正，功能carryover已核、不合池性能。原E0308编译拒绝和第5场nft准备失败保留。

**N2旧拨号超时覆盖新认证联系修复**：TF96e999dae8312d86a38c08095ba58c3e1c1274e1/tree41df16ad，原14合同12绿2红→14绿；Linux286/Mac282原12组全过、各原ignored1。全局contact守Asleep、每协议握手代次守失败出口，保留失败记录/原期限/读写边界，无新timer/packet。S已包含；详[DIAL-TIMEOUT-CONTACT](DIAL-TIMEOUT-CONTACT.md)。Mac原磁盘49.66GiB拒绝编译0，按既有受保护回收规则修启动reserve判定后67.21GiB，同源43542867922通过；原失败不删除。

**I4最终前版**4517d4cb/treecd52c290完整Quality43523128168首轮13/13、native Mac Lab43523194833首轮4jobs/77cases通过；WebKit原四诊断后锁1.65.1/27.2，旧I3红保留。不是当前S的完整Quality、不是手机接受。F3 silent/healthy固定6场168/168成功、silent0.873–0.954秒；双端V4/NAT及dual IPv6固定6场252/252、首次0.507–1.674秒。证据见归档、[RELAY-DEMAND](RELAY-DEMAND.md)、[BOTH-ENDPOINT-MOBILITY](BOTH-ENDPOINT-MOBILITY.md)。

## 已拒绝方案与上游

- I4+N2 shared原完整ABBAAB对照8640读/12文件成功，但N3高延迟目录和PTY长尾回退，E/contact-current-mixed-verification-v1.json为RED。
- PULL4两场硬失败共4timeout/93blocked，停止，未采用；H1收益margin原unit发现Relay200ms错误清195ms hop，未Release；H2两场虽1440成功，但目录超固定门已足以futility，随后原baseline硬失败停止。全部数据/未执行人口保留。
- H2两有日志诊断selected-change均0，仍未复现无日志回退；不能将churn认定唯一根因，不能盲目增margin。
- 2026-11-14T05:37:22Z已在线核官方Release/docs.rs/crates.io JSON：最新稳定iroh1.4.1/noq、noq-proto1.4.0，未yank，与S候选一致。E/upstream-recheck-20261114/verification.json，详[UPSTREAM](UPSTREAM.md)。canonical feat/net产品仍iroh1.2/Noq1.3，未把实验升级合入，现装App也未更新。
- custom历史60项目前1已上游替代、2Noq移除红保留、1 iroh38移除红保留、56待逐项退出核查。E/iroh-upgrade-patch-ledger-v5.json只4条有详细结论，不能称余56均必要。

## 剩余完整目标与资源规则

N1多middle、置信/流量/成本；N2全部预算/取消/资源/写与0RTT/NET16负向；N3当前screen拒绝后的合理架构、初始extra dial可靠后退、caps/多文件公平/小文件首字节/1-3-5成本；N4新地址发布与冷peer、Direct突发UDP黑洞、peer relay移动、睡眠/手机换网、中继故障域/容量；N5全部custom退出核查与最终同源Quality/Mac/mobile及设计报告。不得用局部绿缩减REQUIREMENTS。

iPhone现装仍旧版，之前“就绪”过期，真实候选可用后另约窗口。MacLab与Turing同LAN，macserver是另一异网设备。builder colehank/khor-build public获明确授权；未授权App安装、发布或生产Relay部署。

本机source-only：不运行本地Cargo/Rustfmt/Node/Bun/Xcode/产品；Git与stdlib Python源码/数据/ZIP/hash及远端编排允许。Turing UID1110/shared flock/jobs16/nice10/Rust1.95/启动50停止40GiB、私有user/netns；计时CPU>=80%idle/pressure<=5/无同UIDCargo，与编译串行。Mac jobs2、原50/40GiB与target100GiB及installed guard998f保持。root唯一ledger writer，无后台协调器。

先读本STATE/PLAN，核实际HEAD/diff/共同ledger，不能照历史pending/PID重派。S图谱0节点、不覆盖准确树（不是低风险判断）；按图谱优先后定向读实际源码，不擅自索引。其他任务LIVE23961、main及已安装App保持。



下一产品工作已在改源前记录E/separate-pull-next-preparation-contracts-v1.json：当前原型在首slice前等待额外dial，DIAL_MOST=25秒且受继承期限约束；这是源码风险、不是这次已观察故障根因。先验证健康cached Main的首内容/小文件不受可选dial前置等待，准备失败/取消/政策/期限边界，再设计每peer/全局caps、多文件公平及实际Endpoint生命周期隔离。尚未实现，不改原S或已冻结源，详SEPARATE-PULL。原4p6已闭证，当前按上述单一RED合同推进；完整N0–N5保持。

## 下一步（完整目标继续，前一阶段所有收据已关闭，新边界格式任务以上节为准）

1. 在当前fe61源码上补足小文件确实不新拨号、准备超时/拒绝/暂停/调用者过期、准备中取消与正确后退的实际正负合同；现只对typed TimedOut后退，Refused混合交通/策略错误仍需拆清。13/294/290已过的组不因接续而重跑。
2. 默认启用前实现并验证每peer/全局的额外连接、准备与pending上限，验证多文件公平、单文件取消不破坏其他文件、真正Endpoint所有权隔离；不能拿每文件一连接的原型当完整资源设计。
3. 收敛上述源码后再做普通Release、真实布局/首次字节/小文件及原低高profile性能对照和1/3/5成本。未来验收要明确模型路径隔离条件与速度优先产品行为；原direct_after红及全部旧cohort结果不改、不补替代，不能继续重复已完成0/6复查。所有新协议在数据前冻结，保留失败和原性能容差。
4. 继续REQUIREMENTS中完整N1/N2/N4/N5及最终同源Quality/Mac/iPhone/能耗，未获合并、安装替换或发布授权。当前首内容修复是阶段成果，活动goal保持active，不能标完整目标完成。

图谱针对当前worktree无符号覆盖：本次detect_changes虽显示3文件/0函数/风险0，但这来自0覆盖，不能作低风险或无测试缺口结论。实际审查按Git三文件diff（相对RED28e）与真实QUIC/Window/标准回归完成。源worktree clean，原S/D/RED及所有失败、格式补丁与proof均保留。
