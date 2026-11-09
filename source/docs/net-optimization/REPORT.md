# 阶段对照报告（未完成整体验收）

2026-11-09。当前结论：最新Watch版42532849927动态27/27，p3三轮最坏508–656ms、零错/退役，21相位绿；闲置phase0仍18.42–19.53秒贴近20秒。42532770351的p5短停顿门绿，p1s仍3/1次移动；42532877270转发8/9，f4s一轮1513ms超1500ms，原门保留。ff版与baseline同准备v2的p1/p1j均0/0、p1s均7/0，ff版打洞更多，不能称全面胜出。六项任务已终态，下一步用默认关闭的诊断构建记录实际选路输入；日志模式不作为SLO。旧15秒黑洞失败仍保留其准确源。N1和整计划均未完成，下列历史各组不能互相替代。

## 正式三次对照

同一fixture/工作负载，每个job内单场景顺序、全新Lab/居民重复三轮；baseline与candidate在不同GitHub托管VM。control419b5f0；完整收据见RUNS和证据source.json。三轮同VM不能当作独立机器统计，下面报告全部结果而非汇总显著性。

| 指标 | 旧策略6485cb83，42509612220 | 新采样候选，42509632238 | 判定 |
| --- | --- | --- | --- |
| p6r 慢Direct/快Relay，请求p95 | 323.93–325.44ms，3轮仍Direct | 110.15–117.42ms，3轮Relay | 旧3红、新3绿 |
| p6r 固定Relay参照，请求p95 | 106.55–108.08ms | 110.45–112.63ms | 两版均3绿；候选自动选择接近本机固定好路 |
| p6d 快Direct，请求p95 | 25.70–27.79ms | 29.29–32.26ms | 均3绿，不能因此声称零额外开销 |
| p6d 固定Direct参照，请求p95 | 25.30–44.98ms | 30.00–32.03ms | 均3绿 |
| p2 旧协议，两端必须LAN | 3.4 / 3.1 / 3.1秒 | 13.0 / 16.4 / 6.9秒 | 原门2红；拓扑中Relay同样快，不能单凭接口名判体验回归 |
| p3 黑洞后最慢请求 | 680.0 / 708.8 / 683.2ms | 755.4ms / 9.9秒 / 15.3秒 | 新版2轮超过1秒，最后1次请求超时 |

p6每轮30次目录请求，200ms间隔，零错误、p95<200ms且期末两端保持目标路径。Relay单腿40ms配置，完整端到端约80ms，实测83–84ms。固定参照在同拓扑准备完后屏蔽竞争路径再预热；它只代表可达到的稳态，不是首次建连或故障切换。

候选tree4549630f，baseline treef95f6e54；两者fixture与netlab线程开关一致，生产源码不同。完整18个结果、失败日志与各轮source/environment/mode已保存证据目录，summary.json保留全部报告，不删除失败。

## 其他已取得证据

- p7旧版动态两场景45秒未回快Direct。首轮新采样交互2.6355秒、闲置18.1609秒；另三次顺序重复交互2.5261/2.7413/2.7497秒、闲置19.6758/19.6386/19.7252秒，零错误。闲置贴近20秒，仍需恢复相位/负载与空闲成本覆盖。
- 完整旧版matrix42507224815：31场景8并行、1908秒，23pass/8fail。其中6个测量断言红，2项配对document-transfer busy未进入测量；p2i还有报告red但Rust test ok。候选完整matrix42509652259为21pass/14fail（35场景，1968秒），p1/p1j/p1s明显抖动，另有转发/NAT/p2/p5等红；它早于共享API/ACK校正/失败Backup重排，不能充当最终源码验证。
- 首版统一bind/选路单元62pass；采样/看守统一门槛后63pass；共享quality输入回归先63pass/2fail，随后66pass/1ignored。底层旧采样noq431/iroh路径21通过，共享缓存API版431/22通过。ACK-delay校正候选是后续不同源码，尚待证据。

## 当前判断与下一步

保持速度优先。ACK样本现与平滑器使用同一校正值；同速但黑洞的Backup未释放在飞流字节已取确定性红432pass/1fail，修复后433Noq/22iroh全绿（42513546687），健康同速不额外重排。当前三轮42513580645的p3为530.5/732.0/740.9ms，零错/退役；真实卡顿已得到重复修复证据，后续仍需最终源完整回归。

原p2的Relay与LAN均未加延迟，强制LAN违背用户决定；旧失败及工作流结论保留，但不能把它解释成已证实用户响应慢。v2保留10秒/零错/零退役，加故障前确在second的前置与全部回显p95<200ms、max<1秒、收敛后至少30次；以明确拓扑快集合验路径。没有用旧平滑RTT冒充新鲜样本设门，也没有回填旧数据通过。

## 最新相同v2 fixture三轮对照

controlc6a20c5，单场景顺序三轮，准确tree/snapshot见RUNS。baseline42516040900为4/2、4/2、3/3；ACK/Backup修复候选42516056972为6/0、6/0、6/0，早于后加的真实样本时间窗4fa3f1b3。两job在不同托管VM，不声称独立机器统计显著。

| 指标 | baseline6485cb83 | 当前候选 | 判定 |
| --- | --- | --- | --- |
| p2快集合收敛 | 3.1/3.1/26.6秒 | 1.3/2.9/2.9秒 | 旧1轮超10秒，新3轮绿 |
| p2全部回显p95 / max | 42.1–49.4 / 52.1–61.7ms | 38.6–47.4 / 62.4–90.1ms | 全零错/退役；新n35/41/41，旧42/42/137，不能称所有延迟都改善 |
| p3最坏回显 | 476.8/598.2/546.0ms | 431.1/539.7/463.8ms | 均3绿，零错/退役 |
| p6r慢Direct/快Relay，p95 | 330.4–333.0ms，仍慢Direct | 113.4–128.9ms，Relay | 旧3红，新3绿 |
| 固定Relay参照，p95 | 110.9–113.9ms | 114.4–114.8ms | 旧三轮期末held路径断言红，不能称整项通过；新3绿 |
| p6d快Direct，p95 | 29.6–33.6ms | 35.2–35.8ms | 均3绿，不声称零开销 |
| 固定Direct参照，p95 | 29.3–32.1ms | 34.5–40.4ms | 均3绿 |

新fixture未改变p3/p6的拓扑、负载或门槛。候选18项全绿证明本批恢复和静态选路，不替代后续转发、长时间、混合负载或真机验收。最新p2第一方远端检查42517477095只报格式差异，已应用patch；它未执行单元，单独记工具输出，不算产品红。

共享质量/年龄/数量输入已有66单元与vendor证据；剩余相位、CPU积压样本、转发、取消/写语义、混合负载、中继故障域、真实Mac/手机与最终准确源码Quality仍需完成。N0–N5完整范围见PLAN/REQUIREMENTS。


当前生产代码三轮p7交互2.4233/2.6044/2.4252秒、闲置19.5757/19.5113/18.7489秒，零错误；闲置裕量仍小。最新短矩阵42513843436为7pass/2fail：f4/f4i本次绿，f4s应答后14.0秒才回carried且最坏回显10.1秒，另一红为旧p2类型门。f4i曾有setup/恢复失败仍保留，不能用这次绿称稳定。完整生产代码的第一方格式/单元42514729034为66pass/1ignored；后加fixture尚待复验。

## 真实时间窗版与完整负载

上述p7/短矩阵为当时生产版，不是当前4fa时间窗版。后续三个“初始估计/延迟读stats/积压旧包”回归在42518379039确定性取红；4fa修复版42519038936为434Noq/26iroh全绿。6a45e488的短矩阵42519803507为9/9绿：p2 1.0秒、最坏45.7ms；p3最坏609ms、零错/退役；p6r p95 106.05ms，固定Relay106.27ms；f4/f4s应答后0.9/1.0秒回carried，f4i9.3秒回m。它只有一次、2场景并行，不冒充正式三轮。

221b98fc独立转发三轮42517915206为9/9绿，但同源完整35项42517959505为23pass/12fail、1864.48秒8并行。4项前置未达（3个f4、p1j），其余原断言红包括f3、Plain NAT d2/d4、p1移动5/3、p1s7/9、p2收敛13.5秒、p5切换/打洞及固定Direct参照p95 1.56秒/max2.63秒。不能将这些全归CPU，也不能将“未选Direct”直接当作“未打开Direct”。完整压力与独立转发数字分别保留。

新增path测量诊断的实际Entry回读另有兼容产品红42519750741：核心66/hop9/wires10通过，journal27/1失败，原因是serde内部tag缓冲直接解u128不支持。6b3644f5按JSON u64毫秒读入原算术类型，并测持久化Entry，随后42521515341仅签名格式失败；修正后42524575725核心66pass/1ignored、hop9、wires10、journal28全绿，实际Entry回读验证通过。Netlab读原始JSON Value，既有路径测量没有经此损坏的Entry链路过滤。

上述三项均已终态。相位42519778236为26/27，21个p7全绿（交互2.26–2.60秒、闲置4.39–18.56秒），p3三轮最坏均<704ms；p2末轮16.2秒选路超限，但最长回显63.4ms、零错/退役。full42521539154为28/40，含p3最坏1.1秒、p7i20.054秒、p6r223.21ms及稳定性/前置等红，不能全归CPU。最终源完整检查、CPU/置信、期限/取消、混合负载、能耗、中继故障域及真机仍未齐。f4的最长回显包含人为断腿且备用超慢期间，恢复后1.5秒/无回退目标不变，不能拿p3已有快备用的1秒目标套整个断网阶段。


## 独立稳定性与最新候选

同e480源、单场景顺序的42524625685为2pass/2fail，1880.44秒：p1空闲600秒移动0/0，p5各组短停顿均零Relay切换/打洞；p1j仍卡在要求Direct的准备阶段，Relay实测相近，需审计前置与速度优先的冲突。p1s含自身忙CPU与20次300ms停顿，600秒移动14/2、最慢回显658.8ms、零退役，真实抖动仍需修复。不能用短矩阵通过或full负载竞争掩盖它。

同源独立恢复42524600966三轮6/6绿：p2约0.73–0.75秒且最长回显<33ms；p3最坏0.58–0.71秒。此前p2的16.2秒失败保留，不以不同VM三绿判偶发已消除。

4ec6ab2e修正“窗口旧点已过期、quality未重算”：确定性回归42525350514为Noq434绿/iroh26pass1fail，修复后42526508917为434/27全绿；不刷新ACK/时间/确认、不增加探测。修复后42527316922短矩阵9/9绿（373秒2并行）：p2 1.0秒/max40.55ms、p3最坏642.6ms零错/退役、f4/f4s应答后0.3/0.4秒回carried无回退，p6四项绿；动态27项42527295249仍在途。尚无证据将原p2/p7超限归因于该bug，也尚未对修复后长稳定性作结论。N1和整计划未完成。


随后确认取证另取得确定性红42528150054（434Noq绿、iroh27pass/2fail）：旧快速分支会使用决定前两个包直接切换，而且候选未答时不复测当前路。6eabe635要求决定后包，并在既有确认tick内双路复测；不加固定2秒等待，不改变黑洞leave和收益门。42528714010为434/29全绿；实际长稳42528746575、转发三轮42528783380、相位三轮42528841695在途，尚不能说p1s已修好或恢复无回归。


最新过期修复版42527295249三轮25/27，p2收敛1.5/4.0/1.1秒，最长回显均<63ms，21个p7相位全绿；但p3最坏612.4ms/1.3秒/15.3秒，末轮1次15秒请求超时。原不应答重排只在status改变时触发，已新增42529470019确定性回归核查“移路之后才失效”的窗口；尚未改Noq生产逻辑，也未证明实际长卡顿的全部原因。历史三绿不能抵消这次用户体验红。


后移路故障重检ff399658已42529931673 **435Noq/29iroh全绿**，相同虚拟黑洞时序先434pass/1fail。候选沿既有Backup PTO事件重检，未改RTT/收益/静默门槛、未增加独立timer或应用重放；<300ms保护及健康路完整交付通过。实际恢复三轮42530014989与短矩阵42530060891仍在途，当前15秒实际卡顿尚不能宣布已消除。


最新结果：6e确认版动态三轮27/27，但闲置phase0最慢19.67秒；独立稳定性1/3（两项Direct准备失败，p5多余打洞），转发三轮8/9（末轮f4恢复2.2秒超1.5秒）。ff Noq版独立恢复6/6，p3 429–602ms；短矩阵8/9，唯一f4i准备Busy未进入恢复测量。最终稳定性与转发尚未达标。准备v2已先冻结再实现，正以相同fixture对照baseline648和候选；Watch单次慢quality立即打洞正用新增回归取红，不能把当前未测的p1s说成通过。


519ad4aa探测后打洞修正已435Noq/33iroh绿，首慢值probe/punch同时触发先取得确定性红。没有增加原探测预算；普通质量需要2秒与决定后证据，极慢RTT和真实故障仍有原快速探索。新control6af仅增加原p1s/p5定向入口，实际忙CPU/短停顿、相位、转发三项正在运行，最终N1仍未完成。


42555870179 Mac根格式、438Noq/36路径/9选家全绿，准确tree4a9c4d89/snapshotb6a4e924/controldd57be6。37无接收copy快速离开用例从42551075037实际红转绿；有提示/旧包/兜底覆盖保留，Noq2始终启用reporting恢复实际通过。API bundle远端还原源码与可信控制后复用原guard，旧Git HTTPS两attempt执行器失败不记产品红。Linux真实p3与f4s红尚未复验，不代表N1或整计划完成；N3测量记录器只有格式证据。


## 2026-11-09：已校准Noq层首批基线

42560015454（tree33d5ef6d47cde04620db1baa470bd7fefdf0c99d、snapshot9a6f9717470e0a741aedbdd9dd1ecf0a32d18fc8、control0cff9671fb2f80945423132e0df46995f6ff3198）在Mac26.6.2 arm64/Rust1.95、2构建任务/测试单线程，根格式/校准1pass、三轮各2pass全绿。
校准先取得42559476637共享FIFO模型实际红，再只修独立方向派发；模型independent_fifo_v2，记录schema v2。原几何300ms RTT、每向2%损失/20Mbit、200ms队列、20秒bulk、两固定seed保留。没有跨模型计算产品收益，未改产品CC。

704原始请求（每轮235/234/235）、51摘要、15bulk case全部保留，零失败/超时；对每组索引/计数/成功与全尝试分位数从原样本逐项复核。原日志SHA256、合并分位数方法及详细数据在mac-duplex-congestion-42560015454/sample-audit.json。下表合并三轮bulk-echo；吞吐按实际bulk秒数计算范围。

| CC / 优先级 | echo样本 | p95 ms | max ms | bulk Mbit/s范围 |
| --- | ---: | ---: | ---: | ---: |
| bbr3 / off | 78 | 449.6 | 710.5 | 9.307–10.742 |
| bbr3 / on | 75 | 612.4 | 821.1 | 9.310–11.841 |
| cubic / off | 70 | 890.3 | 1081.8 | 0.322–0.344 |
| cubic / on | 77 | 389.9 | 808.2 | 0.334–0.338 |
| newreno / off | 66 | 973.2 | 1108.7 | 0.211–0.245 |

该固定模型下优先级没有呈现一致改善：Cubic尾部下降，BBR3本轮p95反而更高；这是后续定位线索，不是因果置信结论或产品策略选择。各CC固定顺序、同一丢包seed且仅三轮，不能泛化到真实路径。typing各CC159/161样本，screen仅9样本/CC，原样保留但不报p99或尾部SLO。
下一阶段必须测真实Node准入/排队、SSH/hop外层流、共享/分连接资源公平和后台能耗；N1 Linux原p3/回退红、N2其余边界、N4/N5不由本结果收口。


## N2操作期限与Accepted回答所有权：Mac红绿

42562932053的真实ask/持锁回归两红（40ms未传达、300s被28s截断），由42566727617首次转绿。42567527752随后在实际Accepted回答路径取得取消/期限两行为红，五预算控制绿；两fixture均先通过同wire独立Sessions/相同stable_id/严格三请求/零dial_failed后才在目标断言失败。旧is_0rtt前置失败42566727617不当所有权行为红。

42567946141修复源2fc650b30a3ab16792adb7babe566b8cec1bfeb5 clean、tree314894b33ed72a5ff2aeed47504d6c9e7f575bd9、snapshot78c6fb8eabf86ff755800188d9f92e64f8d4e3cf、control0cff9671fb2f80945423132e0df46995f6ff3198：根格式/net66+1ignored/hop9/wires10/journal28/dial_budget7全绿。Mac26.6.2 arm64/Rust1.95/jobs2；预算子集1.67秒为测试运行时，不是性能SLO。Accepted交接保留Aborts owner，回答等待沿原绝对by；取消/到期停止本流并保留共享健康wire。

这批证明已测的锁、scope与Accepted回答合同，尚未覆盖普通交换/真实死wire/一次读重试/写入失答/跨IPC剩余预算/完整后台及并发释放。未执行Linux最终矩阵、全Quality或真机/能耗，不宣称N2或全计划完成。原始证据在mac-accepted-owner-candidate-42567946141，失败与fixture修订原因保留RUNS/ACCEPTANCE。


## N2普通wire与配对探针

42569954515新增四项普通wire覆盖实际绿（dial_budget11、其余既有子集绿）：普通1秒到期仅STOP本流、同时等待的第二请求在同stable_id成功；真实读重试沿原2秒by；失答Sessions严格2次/Mkdir严格1次。生产不变，未宣称这四新增用例在旧源实际红；长黑洞F7尚未验证。

42570772550以原pair/doctor取得三行为红，旧11绿；完整GET/配对pending/fast已204且在单HTTP2.6秒前等前置已过，失败仅在取消未关闭HTTP、pair取消残留与快second被slow first挡住。
42571393016 based4a2e115b898fc337f5091b51fa71e45740c667f clean、treef278b9330f51268182b142729db2c86687f4af4b、snapshot3459894c81cb994310a97bfcba4ca39ae97e569b、control0cff9671fb2f80945423132e0df46995f6ff3198：根格式/net66+1ignored/hop9/wires10/journal28/dial_budget14全绿，预算子集9.93秒是运行时不是性能SLO。Mac26.6.2 arm64/Rust1.95/jobs2，真实取消后TCP EOF及首成功均在冻结观察内；测试server未提前关闭制造EOF。

修复仅闭合probe两层任务所有权、按完成首成功并取消输者；未增加并发/改变2.6秒预算或proxy。ears_heard也是该函数的直接caller，HTTP boolean映射保留，完整echo/故障场景未由本批证明。原始source/env/raw/classification均保留；没有Linux最终矩阵、全Quality或真机/能耗结论。


## Turing首轮恢复（U9，2026-11-09）

用户授权Turing作为Linux主通道后，candidate turing-20261109T075531Z-b6277f48准确treeef392bb7/snapshot8ff966db/control59dc223，3轮recovery全部6/6绿，原p3最坏<1s保持，实际518.5/597.9/706.9ms、均零失败/退役；p2改变慢路后分别3.1s/6.1s/930.4ms达原断言、typing errors空。每轮样本/原始scenario日志在对应evidence/report/netlab/，非跨host改进结论。首编译6m19s，后续repeat0.58/0.53秒；基线同Turing同工具链/同driver原门正串行运行，首次编译55.57秒。

源码全量tuple、原字节SHA、toolchain、namespace、host资源、执行状态、三轮全部结果、原job group检查均留存；采证collector576c127独立于执行control59dc223。只证明此source在Turing recovery profile绿，N1 CPU/hop/full、N2–N5和真实设备未结。历史托管Linux42542984111两个p3红不删除、不据换硬件称已定因。


Turing recovery同机对照已完整采证：

| 角色 | p2 p95三轮(ms) | p3最慢三轮(ms) | 结果 |
| --- | --- | --- | --- |
| candidate(c55/treeef392bb7) | 70.704 / 71.865 / 70.794 | 518.5 / 597.9 / 706.9 | 6/6 |
| original baseline(6485/tree52a44260) | 73.692 / 71.256 / 247.204 | 787.0 / 787.1 / 704.3 | 5/6，p2第三轮p95超原200ms |

两源同Turing/固定Rust1.95/原fixture/control59dc223，串行candidate→baseline；两源p3均满足原1s。基线第三轮host负载变动（candidate end load1=9.12，baseline end=46.12）与时序未隔离，不能把尾延迟差异直接算源代码或某patch收益。保留基线原失败、全部样本/断言/环境；数据不是稳定性统计证明或完整N1收口。下一为两源hop/stalls与CPU受控/必要复验、dynamic/full；N2–N5仍未结。对照JSON含两边load before/after与全部tuple路径。


## Turing修正占用计量后的hop（136th）

准确source tree78d063d915f033d9bffa58516488b65de9f0f14b/snapshota828463e/control59dc223，原三场景单线程三轮、hop_occupancy_started_v2，结果8/9，三轮2/3、3/3、3/3。新reducer此前独立实际2红→4绿仅证明started纳入占用，旧candidate8/9完整红仍保留。

| 场景 | 三轮结果 | 限制 |
| --- | --- | --- |
| f4 | 3/3 | 场景最大按键3.5/3.3/3.5秒，原门通过不是交互极致 |
| f4s | 3/3 | 回carried相对腿恢复0.2/0.0/0.0秒，无后续回退；最大按键8.6/10.2/10.1秒 |
| f4i | 2/3 | 首轮恢复后X carried→relay→carried，to_relay=1而原门0；后两轮绿，第三次回carried91.4秒 |

首轮f4i首行started relay条件通过，失败来自后续回退；精确X事件carried→relay1697ms、relay→carried5803ms。journal没有connection stable_id，不能只凭该报告区分连接或定因；未归咎reducer或改门。3轮完整log、source/driver/bundle/environment、原PGID无live与逐文件SHA已采；执行1652.41秒。尚无corrected baseline完成数据，不给因果加速/稳定性结论。新Linux net-node与基线hop/CPU对照仍待，完整计划未完成。


## Linux真正长静默与取消覆盖（141th）

Turing141th turing-20261109T095017Z-93c2c1a2，base9eafba74/treef93c32b1/snapshot6708accb/controlb998479，固定Rust1.95，根格式/编译/net66+1ignored/hop9/wires10/journal28/预算取消探针16全绿，完整原始报告已采。新F7在单独unshare -Urn的user/netns核隔离后安装两向UDP端口丢包规则，真实request/probe命中drop计数、heard不再变；ACK+all-path grace后abandon真实Pending future恰一次unanswered退役，远端仍open、其余失败计数零。长ACK阳性同wire并发请求仍成功。

前Mac UDP代理前置12→15收包以及140th nft语法错误均按fixture红保留，没有当产品退役红；新覆盖通过不声称旧源实际红。该结果只证明已测长静默合同，未证明grace之前取消的备用可达性、跨IPC期限/取消、完整N2或全Quality。21.24秒为子集运行时，不作性能SLO。当前基线hop137th已准确启动，仍待完整对照。


## Turing corrected hop两源观测对照

固定相同五fixture/control59dc223、hop_occupancy_started_v2、原三轮/门；candidate136th tree78d063d9，baseline137th tree97040f31（6485生产原样），完整报告均采核。

| 场景 | candidate | baseline |
| --- | --- | --- |
| f4 | 3/3 | 3/3 |
| f4s | 3/3 | 1/3；晚回退两红 |
| f4i | 2/3；一次回退红 | 3/3 |
| 合计 | 8/9 | 7/9 |

baseline f4s首轮Link已恢复、稍后约35.6秒Relay/37.4秒Forwarded，原after-leg门失败；第二轮carried后26.2秒再选Relay、约0.7秒后再carried，原30秒无回退门失败。两源均有约8–10秒按键卡顿，candidate f4i一次回carried91.4秒。结果只证明该批观测，不能按总通过数认定体验完成或单patch原因；多连接事件缺身份，进一步诊断候选已加wire字段、尚待验证。所有tuple/round/host load在turing-corrected-hop-comparison.json，原红不覆盖。


## F8：操作到期过早退役共享连接的真实反例

Turing143rd（tree42050b56/snapshoted287397/controlb998），root fmt/编译及旧16合同绿，新F8真实红。新私有user/netns、原生UDP握手后lo每向2400ms；两个真实Sessions请求在2401ms到达并被server回复。首调用4101ms正常CallerExpired，此时heard16→16、回包仍需经过固定延迟，原全路径grace尚未到；wire已被Drop退役（unanswered1），控制请求也失败。所有隔离/时序/两请求前置都先通过，才在共享wire保留断言失败，不能归fixture红。

根因边界：RetireUnheard::drop仅凭4秒ACK floor与计数不变关闭wire，而exchange_inner会先probe所有路径并等grace；取消不应跳过该证据。尚未修复；保留F7真正长静默控制，不采用一律禁用退役或简单凑新阈值。

Mac146th（treebbed6503/snapshotc8035ce2/controlb998）另取得17便携/29journal全绿，包括真实update_dial200ms早操作by、30秒操作下10秒自身cap，以及wire身份字段旧日志兼容/roundtrip/人类句子不变。其17排除Linux F7/F8，与Turing143rd17并非同一集合，不可替代真实F8红；30.57秒为单元子集用时，不作产品SLO。


## F8共享确认修复：Linux21 / Mac19同源绿

base25d0c803/tree6805b7b7/snapshote0a42573/controlb998：Turing150th fmt/编译/net66+1ignored/hop9/wires10/journal29/预算取消探针21全绿；Mac151st42605391960同一snapshot对应19便携全绿，Linux-only F7/F8排除。F8函数与143rd实际红逐字相同，接收计数/两请求/4.1秒到期前置保持，修复后同wire控制请求成功且零误退役；函数SHA与完整原始报告可追溯。

正常exchange和取消共享Slot拥有的一份全路径probe/count/grace，调用过期不跳过宽限；fresh receive、replacement、wire清理中止旧确认，任务仅Weak持有，原runtime handle允许在std thread Drop发起确认。F7保留同真实黑洞及一次退役/远端open/零其他故障，仅观察改为owner自身确认后退役，已先记录v2协议；replacement后真实Sessions成功、pending不保留owner也通过。独立确认仍可在黑洞证据充分后记Unanswered，不能把CallerExpired本身当peer故障。

150预算子集46.37秒、151子集32.31秒是测试耗时，不是体验SLO。只是这一边界红→绿，resident tick较长grace协调、Accepted长取消、其余N2与原hop回退/长卡顿、N3–N5仍待；新源CPU对照152→153正在推进，不提前给性能结论。


### 2026-11-09 Turing 修复后候选CPU停顿：原两门绿，体验仍有缺口

152nd turing-20261109T110456Z-dee800e0与Linux150/Mac151同准确源码（base25d0c803/tree6805b7b7/snapshote0a42573，controlb998），原single-thread/repeat1的p1s/p5 **2/2**。600秒20×300ms CPU停顿，alpha/beta0/0路径移动、零退役；但按键最慢3.8秒，alpha12打洞/3449包/1335065B。p1s原门只约束移动/退役等原assert，不因绿色忽略长尾与开销。p5 150/300/600ms各4次，三档均零relay移动/零打洞，最慢279.9/518.6/992.0ms。

完整report与source/driver/bundle/hash/原进程组已采核，classification存run目录。总1193.92秒，host load before4.90/6.36/7.33、after5.58/8.26/8.90；baseline153仍在途，尚无本轮前后因果结论，不跨旧硬件/源比较。原报告只保留scenarios/task/build日志，无额外逐包journal；打洞/长按键定因需后续独立诊断。完整N0–N5、hop/f4i、交互批量、真实手机/能耗仍未收口。


### 2026-11-09 F9/F10共享确认红转绿，Linux24/Mac20同源

158th在全部真实namespace/UDP静默/drop>0/remote-open等前置之后，两项owner确认目标失败（原21过）。生产候选df592ad7对齐probe计数基线、巡检等待pending proof，并给Accepted响应等待同样的取消guard；长等待历史ACK不再等同永久健康，但取消只请求全路验证，只有owner原grace内仍无新包才退役。正常完成的guard早返，不额外读取paths。

160th Linux24全绿（合同92.30秒）；161st Mac20全绿（50.01秒），两者准确tree3c8bc03a/snapshote30f6377/controlb998。新增16秒健康Accepted取消，等全路grace后再发独立控制仍同wire、零退役；原短取消/期限/读写/探针/update及LinuxF7/F8保持。F9/F10core helper/F8相对红158逐字一致，身份文件f9-f10-f8-red158-candidate160-function-identity.json。全部source/env/control/transport/hash/原group已采核。

这只收本批Node合同：无pending时tick/served静默、完整resident/跨进程期限、长预算/候选资源、N1切路与N3实际应用/能耗等仍未完成。CPU152/153原两门均绿，但3.8秒vs732.4ms及12vs5打洞不支持整体提速；绑核/busy有效负载仍未被旧fixture断言。新准确源hop162三轮在途，不能预记稳定性或最终体验通过。


### 2026-11-09 同源三轮hop仍8/9，载路消失需定位

162nd turing-20261109T121345Z-8ba7e561同Node160/Mac161准确源（df592ad7/tree3c8bc03a/snapshote30f6377/controlb998），三轮3/3、2/3、3/3。第二轮f4i：X wire139750384102992在1794227442646选carried（quality85.61ms、8samples、age141ms），1794227449993选relay133ms，只有relay留在measurements，间隔7347ms；这是同连接真实回退，不能由跨连接聚合解释。Y该两聚合事件wire不同，必须分开。open_hop后约10秒与2×5秒清理节奏吻合但非原因证明；下轮独立diagnostic记录attachment/IO/清理决定，不先调排名阈值。f4s三轮门过，最慢按键均9.9秒，不是体验绿。

### 2026-11-09 CPU有效性校准3/3，不代替完整性能对照

166th source886b084c/treeed97f217/snapshot349bd7ba/control7534，短校准3pass（0.50秒），Linux实际CPU14允许集合0–127，两个sleep模拟resident和两个busy contender全thread绑同CPU；305ms各增加15CPU ticks，clock100ticks/s，且负例/reap断言过。新机制支持测量前/尖峰/结束的实际证据，尚未跑真实Node600秒p1s或原版同fixture对照。167th Mac同完整source根格式/net66+1ignored绿，仅net-unit。所有report/环境/driver/transport/hash已采。168th同源f4i生命周期diagnostic三轮在途，明确非SLO，清理政策仍原样。


168诊断3/3没有重现162原same-wire撤路，全部27生命周期事件留证，不能用无复现替代修复。N1继续验证peer共享载路的连接所有权范围。CPU169/170用checked_affinity_v3相同真实Node fixture作原版/候选串行对照，生产基线纯净；结果未出，不预报性能通过。


CPU169未进入测量：真实Node CPU admission返回无路径ENOENT，不能与170构造性能对照；170原版未启动已保留并superseded。下一步只增加错误路径上下文和真实Node短准入，不放宽压力/进展/affinity断言。用户U10明确跨网实机A=macserver、B=Turing（Linux），P=iPhone；设备身份已确定，A↔B可独立推进。


172真实Node短准入红把169类失败定位到枚举与读取之间的线程退出（具体task status ENOENT）；先修fixture快照，不改产品或压力条件。修正仍必须覆盖每个存活thread、PID identity与真实CPU ticks。173仅format红，无多连接行为结论；共享载路清理假设仍待可重复合同。
