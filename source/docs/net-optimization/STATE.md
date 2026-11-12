# feat/net 当前状态与接续入口

更新于2026-11-11 23:20 UTC（本地11月12日）。完整目标“完成计划”仍 **active、未完成、未阻塞**；完整范围为[PLAN](PLAN.md) N0–N5，root串行，不派发子智能体。此页是唯一当前入口。上一页原样保存在[high v2前状态](HISTORY-20261111-through-high-resident-v2.md)，SHA 560d77e0f02b553ebda7b3ac0282a54b0220a0bb1dc1b44a3063f702abf1b172；继续链接更早全部证据。实际worktree/收据/原始日志优先，不按摘要覆盖未提交内容。

## 当前状态与下一步

**当前仅一个Turing任务在途：turing-20261111T231143Z-7bb58177，high priority block2/B。** 无Mac任务；以下cold v3与Mac两轮回执均已真实终态、收件、核验与ledger终态。 high v3冷模型预检 **turing-20261111T223651Z-5d27f45e** 已 **813.212秒、原退出0、25files**，两个真实w3、三low与三high全部通过；共4320业务请求成功、三份16MiB与三份192MiB完整，目录/终端各120完整落在实际文件传输内。独立提取准确冻结metadata复算、原始人口/时间戳、八TC读回/安装/下载路径字节与静态配置先于两resident/配对均核验通过。旧v2的路径失败不改绿，N1延迟突变选路仍待查。

| cold v3准入证据 | 值 |
| --- | --- |
| source | **1c44a5e3e8008a2e9679575ddab50d574d5bf6f9** / tree **b856ce9a5315b745b3570189ec844236e0af248f** / snapshot **90eb94ae8f576bf1c9e815b3dbaef88e246c2992**；candidate-high-resident-source-v3.json，clean/0excluded |
| control | **5eb896e54c9ce28a3b9f1355579f3fb0516dea6e**，原UID/flock/netns/资源/期限门不变 |
| proof | high-resident-preflight-verification-v2.json、原workload-high-contracts.json、identity-verified.json、cold-profile-path-observations-v1.json；preflight_passed=true、full_N3_accepted=false |
| high文件秒数 / payload Mbit | 84.994152 /18.949689；81.521900 /19.756811；81.190881 /19.837360 |
| bulk目录 / 终端 p95 ms | 410.533/313.286；370.721/335.750；369.441/322.165。全部720/轮成功，三个phase/失败分母/计划到完成/调度滞后/探针flush人口齐全，不报p99或最终UX达标 |
| 初始路径 | 两端journal初始Direct约304–321ms，三个cohort均未记Relay选择、结束Direct=true；只说明该冷模型的可比性，不证明旧4–13→300ms突变的唯一原因或产品已修复 |

**高profile A/B源码与control现已冻结，严格核两个完整树只差CONTINUED_FILE_RESPONSE_PRIORITY的0/-1一行、PULL_AHEAD=8与全部fixture相同；预定A B B A A B每block三fresh，共18。** 第1组A已完整收件/身份/准确冻结metadata与独立原始人口核验、ledger/campaign终态通过（3/18已复核）；第2组B在途、其首轮原Rust已过，正在第2轮，整组尚未收件或复核；后四组未提交。新task netlab-workload-high-policy每block两个真实w3→三个wh2，生命周期诊断关闭，失败仍保留在原位置。原低profile18轮独立，不合并到高profile；后续prefetch1/2/8、完整预览/公平/背压/成本与N0–N5仍需完成。

| high priority冻结输入 | 值 |
| --- | --- |
| A / priority0 | **4a9fdc0d7b852abd5f69bf9ae20730616dc5a809** / tree **1ef37909385d96c661897075aa6d68c0a5764b81** / snapshot **86cd81c9375b158c5c9afa035ce543676b9944de**；control-high-response-priority-source-v1.json |
| B / priority-1 | **0c873a13c9489d3a9e873d1b1e87243c63b7ab81** / tree **17b23829edd9fdbe61ebb4f71ebc48387b118d2d** / snapshot **15f1ee6ab3fd719c6dfef67519a9cf1cfc80731e**；candidate-high-response-priority-source-v1.json |
| control / protocol | **5eb896e54c9ce28a3b9f1355579f3fb0516dea6e**；MODEL-DECISION/HIGH-RESIDENT-SETUP，measurement resident_mixed_workload_high_v3，comparison workload_high_response_priority_v1 |
| pair / campaign | high-response-priority-source-pair-v1.json、high-response-priority-campaign-v1.json；pair refs均pin；相对cold v3源码和Mac16c只STATE差异、零runtime，A/B各匹配旧A/B生产runtime；common queues与c50容量/物理geometry证据适用性保留 |
| block1 A终态 | **turing-20261111T225814Z-6038e783**，682.817秒原0、19files/identity/原group空、high-policy-block-verification-v1.json与ledger/campaign终态；三轮各720全ok、三192MiB/关闭/Direct/120重叠全过；file93.709272/81.513137/81.911400秒。不得再次advance 1或重新launch该组 |
| block2 B在途 | **turing-20261111T231143Z-7bb58177**，pid656960/child657174；measurement workload_high_response_priority_v1-block2-B，stage/advance session37929已终态。编译44.11秒、w3守卫过；首wh2原Rust720全ok/完整192MiB/Direct/关闭/重叠均过（file96.103503秒），第2轮进行中；未判整组结果 |

下一步只观察当前block2；终态后CONTROL collect-network-ssh.py→ops/verify-turing-record.py→**ops/verify-high-policy-block-v1.py RUN 2**，以准确冻结metadata复算和独立原始人口/时刻核验。然后 **ops/advance-high-response-priority-campaign-v1.py 2** 先终态ledger/campaign，再唯一提交下一组B/block3；每组同理，不能因观察超时重复advance/launch。advance 1已实际终态，不能重复；缺失完整测量人口时标campaign needs_review并停止下一提交，有完整人口的业务失败留在原位置。stage不确定时按同measurement revision恢复已有receipt，不重跑preparing。第6组终态也不等于完整N0–N5完成，随后需完整对照复核及prefetch/预览/公平/背压/成本等。

全组六block报告器ops/summarize-high-response-priority-campaign-v1.py已准备：重核每个原始report文件hash、复算全部raw population、保留失败与对应位置差值，不选winner/p99/显著性；实际对未完成campaign调用会拒绝且不生成comparison，尚无完整高profile结论。等6组完成再运行，不能拿partial表作最终策略选择。

### 下一因素的准备（未启动prefetch实验）

[PREFETCH](PREFETCH.md)已在任何prefetch数据前70586fd6冻结：固定priority策略须待完整对照后明确，再冻三树只差PULL_AHEAD=1/2/8一行；low/high各顺序8,1,2,1,2,8,2,8,1。每个值同一次编译输入先w3两真实守卫，再low/high各1 fresh、分别记录，共9组18场景；low失败仍执行原定high，保留首非零。原文件/请求期限、全部人口/字节/Direct/重叠门保持。

准备代码 **a0f43b89320f3ac5ec5e383fe72addc995165039**、准备控制 **5d3691f252f9dee0297fa3c494374be6119bca29**，新固定task netlab-workload-prefetch；**当前priority campaign仍显式使用5eb，不切换控制或源码**。元数据13项21.435秒和dispatch1项/38 task-mode路径43.257秒通过。曾发现合成high_timeout重叠计数没随延长attempt更新，已修fixture重算并保留原门，非真实业务失败。prefetch-default-compatibility-v1.json证明新默认validator逐字段复算实际A组三轮原contract完全相同；prefetch-preparation-input-audit-v1.json证明控制只增加固定task/allowlist，资源/UID/flock/netns/期限逐字未动。

失败文件只放行结构化分析其单调网络计数/固定几何，不伪要求已传完全部字节；仍functional=false，未完成无verified payload rate，区间标caller_attempt，timeout明确被原120/240秒caller界截断；720逻辑请求/失败/缺失仍保留。成功文件仍严格完整路径字节门；旧policy/preflight默认仍只准PULL_AHEAD=8，旧输出未改。**尚未选择priority、未冻结三树、未建prefetch campaign、零prefetch真实run；新代码还需实际远端运行，不称通过。** 不得提前启动或把这一准备混入当前18轮。

额外静态发现留在bounded-prefetch-static-audit-v1.json：pull_path_with使用join_all批量屏障，等整批完成再补下一批；有界滚动补齐可能改善有效在途量，但**尚未实现/测量**。保留既定优先级/prefetch独立因素，后续方案须保持原顺序写入/mtime/续传/取消/资源所有权和八片内存界；不能动当前冻结pair或混入这18轮。

### 并行完成的Mac vendor默认库回归

Mac **43114612815** / request e320800ccc684904829cebe7c93b4d55 / actualcontrol **4e34fb7dce174345410a33e892b0eb76a5cc5c23**（隔离私库feat/net-vendor-full-v1，当前Turing控制HEAD5eb未动）。source **16c79bc0e7e826febcbdd97044890d0b6592413f** / tree **b2ced2679af2b19ff544c8b8172aa326a9e17ca4** / snapshot **47ddb7a76057f1301fe34ff2e36b5c188e7c287f**。artifact **13898480553**，ZIP SHA **d35ae705f685649262daf28f4b69a4fc09be268a8f0f722bec07ee2b914f1606**；16files/11controls/source/bundle身份/两vendor锁逐字核验，vendor-full-verification-v1.json与ledger终态。

Noq完整默认库 **447通过/0忽略/0筛除**；iroh完整默认库 **184通过/1原忽略/0筛除**，另保留58路径/11home专项（它们与184重叠，不相加冒充独立覆盖）。唯一原忽略endpoint::tests::connect_via_relay_becomes_direct_and_sends_direct在源中标flaky，未偷偷运行/删除/称通过。该工作补齐默认库覆盖，**不是全部feature/ignored/integration/物理或最终N5接受**。隔离control只改net-vendor分支，在原3suite后补全iroh库；保留全部四scope及首个非零，7条合成control路径8.816秒通过。共享Mac guard/jobs2/90分钟/源码验证与其他11控制文件保持原样。私库调度/源码上传用原项目脚本，查询job/artifact也经连接器核验；无main合并/发布/部署或手机安装。

### high v2失败与不丢失的观察

旧 **turing-20261111T221821Z-2e4c439f**：f428627b354a447f169d86d0843f79b2cf810bd3/tree3707ffd537c5d8d3c758f83d1bbf7edfb62bf57f/snapshotfb9442b69da624075bc596f6e6ed9178a5254636，control30d7a35db340201eed9f80a6161d9de2ce7cfbae；799.391秒原101、25files/identity/原group空、high-resident-preflight-verification-v1.json与route-ambiguity-v1.json/ledger终态。三high虽2160业务全ok、192MiB完整、关闭/重叠全过，但第二轮结束Relay、Direct=false使预检失败。三文件79.183/80.826/79.311秒，约20.34/19.93/20.31Mbit；不并入新v3。

v2是在配对/首次Direct后施加模型，journal先4–13ms再300ms；Relay首次选中时smoothed101ms/Direct126ms而fresh quality约303.077/300.856ms。按[HIGH-RESIDENT-SETUP](HIGH-RESIDENT-SETUP.md)在新数据前d9480418预注册，只将high配置提前，保留可用Relay、原Direct/期限/负载/完整人口门，未强制直连或切断替代路。cold v3通过不消除该N1动态行为待查项。input-audit-v3.json证明实际源码调用顺序和共同queues/observer逐字相同；本机12项6.575秒合成数据绿，未执行Rust/格式器。

15cba7f8是更早仅格式红：725ce682/tree902c1f37/snapshotbe6aa14c，同control30d7；43.848秒1/10files/identity/原group空，零业务，准确四文件格式patch已f428627b应用，format-only-verification-v1.json与原件保留。旧policy自动编排90703已终态，不能恢复。root仍为source-refs唯一writer，所有新操作先核实际收据/进程，不因观察超时重发。

### 已完成的独立容量诊断

独立UDP容量诊断 **turing-20261111T214616Z-c50ab6ef** 已186.409秒、原退出 **0**、17files、identity/原group空，collect/逐原始位图与本地窗口人口核验/source-owned metadata复算/ledger终态。stage session12779与PID553115/child553331均已实际终态；只复用证据，不重发。旧policy编排90703也已终态，不恢复。

| 最新准确输入/证明 | 值 |
| --- | --- |
| source | **a6a119a4d5d3e38197ba1f0da495c6fa8ddf9cef** / tree **29217fb5fb5fa4cafe58c3d69435e4907e9bd2dd** / snapshot **d796dc95fe85326521ff4ed128a7164d7dfefd4d**；candidate-kernel-capacity-v2-source-v1.json，clean/0excluded |
| control | **1a6eee06e2e5df3220acc6e3c6ddcae363ebc3fc**；control-kernel-capacity-v2-input-audit-v1.json：相比ce3e仅measurement revision literal变化，runner字节相同；原资源/UID/flock/netns/期限门不变 |
| 原协议 | [CAPACITY-DIAGNOSTIC](CAPACITY-DIAGNOSTIC.md)，先293aa396冻结v1，1fa4e9df在v2结果前冻结5ms修复；v2 revision workload_kernel_udp_capacity_v2 |
| proof | kernel-capacity-verification-v2.json、原workload-kernel-capacity-contracts.json、identity-verified.json与source-refs终态 |

三fresh、六方向均 **145833/145833计划包成功发送**，共874998；每方向4000个5ms实际slot、无catch-up/跳过/发送失败、max lateness1336–2052us、无>10ms迟到。源packet内容函数/receiver/valid门相对v1逐字未改，metadata只加兼容bin/count与slot关联的4000/37上限，未降低99%/65–75/40–60/期限/拓扑等门。

| fresh | upload sender/receiver Mbit/s | download sender/receiver Mbit/s | upload unique/missing | download unique/missing |
| --- | --- | --- | --- | --- |
| 1 | 70.000320 / 48.309120 | 70.000320 / 48.309120 | 99454 / 46379 | 100929 / 44904 |
| 2 | 70.000320 / 48.309120 | 70.000320 / 48.309120 | 99453 / 46380 | 100929 / 44904 |
| 3 | 70.000320 / 48.309120 | 70.000320 / 48.309120 | 99455 / 46378 | 100929 / 44904 |

全部tuple、bitmap全集/子集、逐包内容检查计数、25/21个本地1秒bucket与人口一致；源实现对每个收到包逐字节检查，独立复算原始bitmap与窗口，未保存/重放网络payload。unique共601149、missing273849，发送/内容/重复/异peer/越界错误均0。70M输入故意大于50M整形，缺失分母完整保留，**不把这一缺失率当用户loss SLO**；不跨host时钟相减。三轮24份实际TC JSON/文本/interface及6次初始root rate安装齐全，profile角色/真实路径byte/drops核验通过。metadata在服务器Python3.8.10也已实际成功产出，完整复算与原contract相同。

**UDP独立容量诊断已通过；完整模型决策/旧TCP门/Noq/Node/N3未接受。** 原contract和独立proof始终physical_calibration_accepted/full_N3_accepted=false。这证明在该分离几何与负载下接收端确实获得约48.3Mbit payload，不能把旧冷TCP平均35Mbit直接等同IP链路只有35Mbit；但也不证明CUBIC、队列或CPU某一项是唯一根因。

分层模型判读已按MODEL-DECISION明确版本/理由/旧新结果；high v2已实际预检失败，冷初始化v3完整预检已通过，以本页顶部为准。后续按同条件priority A/B和独立prefetch1/2/8、完整预览/公平/背压/成本推进，当前priority -1仍实验候选。UDP通过不等同旧TCP通过、应用SLO或N3接受。完整N0–N5保持active、有可执行工作，不设blocked/complete。

### 本轮前置失败与旧诊断（原件全部保留）

- UDP v1真实 **turing-20261111T213626Z-343630a3**：8f2d059ee7ac810df66aa0a57e71fefc888fb098 / tree6e2edb494b48ee03b56010b9b824bb8ddab04a4e / snapshot2d29f27467db05990beebc83155fd5ffd8f24a75，controlce3e0671c9fe5fc397120ebd00d96e889c7c558e；188.648秒101、16files/identity/原group空。计划874998，sent708209、unique601205，六方向只发108219–136200/145833全不足99%；虽raw receiver窗48.309120，**pacer输入不合格，容量不接受**。原metadata又因Python3.8缺int.bit_count崩溃，无contract；kernel-capacity-pacer-failure-v1.json是独立raw reanalysis，不重写原report。5ms slot/兼容popcount是另版修复，不能改绿v1。
- UDP前置 **turing-20261111T212922Z-c5265bb8**：5e04ec3c/tree58a4e705/snapshot06c80c0b，43.774秒1、10files/identity/原group空，仅format红零行为。e92ffbb4应用原patch包含两处可选函数调用尾逗号调整；初始纯空白断言触发，后逐token核只此二逗号与空白变化，format-only-verification-v1.json完整说明。**turing-20261111T213324Z-75020c6f**：38df2f3b/tree6d37f147/snapshot29f72940，33.779秒101、14files/identity/原group空，format过但CLI无直接serde，三入口编译失败零UDP；compile-only-verification-v1.json。8f2d059e改用已有serde_json显式同字段，未增Cargo依赖。
- [队列拓扑ABBA](QUEUE-TOPOLOGY.md) **turing-20261111T210442Z-80ceec22**：1f4d90dc7c31aaf7c315727cc66105ac6f1c8f9f / treea0c19bc7c976f2b631b556b5a3001bcdabd76e5e / snapshot384dec5a7233307d778e116e337ab28b0a621b63，controldde989632732550725f163a1511b0e5ba7d7511b；329.214秒101、19files/identity/原group空。A/B/B/A upload35.368/35.400/35.406/35.383，download28.599/34.485/34.532/28.088Mbit；272echo/八96MiB完整，物理四红。B rate drops79/80与80/78，传播netem均0；kernel-queues-verification-v1.json。旧format-v1 47ae仅格式红零业务、准确patch1f4d90dc，原件保留。
- [burst ABBA](HIGH-BDP.md) **turing-20261111T185007Z-6e04ad5e**：c23cd79e/tree32620a62/snapshot1fbc641b，control0d573ad3；352.084秒101、19files/identity/原group空，272echo/八96MiB全。4KB A upload25.384–25.422/down26.094–26.101；64KB B up35.385–35.391/down28.671–36.229，四红。固定latency使FIFO limit随burst变，不称token-only因果；kernel-burst-verification-v1.json。324格式零行为与81f Device/Router编译零行为等前置沿历史原样保留。
- extended **turing-20261111T182504Z-cde36230**：c5ec8b6f/tree67888ea2/snapshot53132d69，controlc731fb00；266.067秒101、13files/identity/原group空。单hl0/68echo/双向384MiB/24两端快照完整，平均32.900532/32.432250Mbit仍红；kernel-long-verification-v1.json/socket-flow-age-summary-v1.json。upload25–50秒38.213–38.245/50–80秒34.802–34.829，download34.899–34.931/35.293–35.332，不能只用96MiB太短解释。rcv_space不是对端advertised window，缺失字段unknown、不跨host时钟相减。
- socket c21三轮48快照/204echo/六96MiB完整但25.192–25.324/26.035–26.095红；原h0 374三轮25.214–25.301/26.049–26.144红；同准确源LOW 405三w0原35–80ms/6.4–9.6Mbit通过，不能接受HIGH。全部准确SHA/TC/前置格式红/证明沿历史链和ledger保留，不重复旧run或旧自动编排。

本轮v2 source metadata10项1.704秒、兼容popcount专项1项0.542秒、control相关dispatch1项6.696秒（9tasks×3）；此前全control13项13.904秒、source10项1.640秒沿历史保留。实际Turing format/编译8.80秒/三UDP fresh均过，17files校验齐全。本机source-only只Git/Python数据与合成控制检查，未执行Rust/格式器/Node/Xcode/产品。图谱部分/滞后，0flows不是覆盖证明；未运行Mac新任务、未合并/发布/部署/安装候选App。

## 已完成的低profile优先级对照

root串行自动编排 **exec90703已正常终态**，全部六block均原退出0、collect/identity/原group空、逐原始人口核验与ledger/campaign终态。**不要再运行该编排或advance这些组。** 权威campaign `response-priority-campaign-v1.json` 保持预定A B B A A B和每block三fresh，共18场景/每arm九轮；全部12960请求ok、零缺失/重复，18份16MiB一致。六份proof/identity及114个原始文件的hash/source/control已独立交叉核，`response-priority-comparison-v1.json`已实际生成并核。

| block | arm | 准确run |
| --- | --- | --- |
| 1 | A | turing-20261111T170059Z-b3de4646 |
| 2 | B | turing-20261111T170751Z-4fd7ab02 |
| 3 | B | turing-20261111T171328Z-3abf2108 |
| 4 | A | turing-20261111T171702Z-d8d2a9b3 |
| 5 | A | turing-20261111T172209Z-0f937684 |
| 6 | B | turing-20261111T172555Z-52fcee21 |

完整18行表/全部范围与限制见 [SCHEDULING](SCHEDULING.md)。每轮bulk目录/终端各120个均完整落在文件实际传输区间；不是组成变化。九个对应位置B−A目录p95全负，**改善6.725–15.628ms**；A108.931–114.527ms，B97.230–105.405ms。完整文件差值−1.035至+0.857秒，九对中5对B更慢、4对更快，**没有一致文件完成收益**。

A/block1/场景2终端p95 **416.984ms** 原样保留，其余A78.819–83.736ms，B79.896–83.922ms。原因未证；终端在另一Tunnel，不能由一个异常推定同Main优先级对终端的直接因果效果或CPU/I/O原因。全phase/实际transfer区间/九对差值/条件成功分布/完整失败分母/文件时间及payload率均保留。仅小链路目录局部方向性收益，不报p99/显著性、不自动选胜；严格优先级仍是实验候选，公平/前台完整文件/背压/成本和全部N0–N5未闭。

[HIGH-BDP](HIGH-BDP.md) 的原v1/96MiB与旧TCP门失败记录保留；新v2以 [MODEL-DECISION](MODEL-DECISION.md) 的显式分层准入及192MiB输入为准，均在首次高profile Node数据前固定。新high v2已实际编译/预检失败、high v3已编译且完整预检通过；prefetch矩阵未实现/运行。原低profile v4/2秒/120秒门保持，不混用cohort，也不把小链路局部收益当最终设计。

## 冻结源码与前置证据

外部证据根统一 `/Volumes/disk0/Khor/evidence/net`，OPS为其ops目录；root独占source-refs.json。全部run identity/behavior proof按run目录保存，identity绿不单独证明业务绿。

| 输入 | 精确值 |
| --- | --- |
| B/source-v3（priority -1） | **f20461c6d71c390d58fa6d1bc093d83218fcf978** / tree **f634bd8999e047907eed6fe6f5dc884cdc2b1274** / snapshot **93878569c6c16a7ef18c6ff1c5188bc37973d5a7**；candidate-response-priority-source-v3.json |
| A/source-v3（priority 0） | **d507c4e735419114c0613734bd9315fcad9ca04c** / tree **1f1e92bc078186cbf979b456af691c845fa8cf27** / snapshot **7e3dda2cb01d0eed6eef01e7b6bcf974f2303313**；control-response-priority-source-v3.json |
| 两臂控制 | **5f1242f5f4d216dde60ad27be0d17a5958b7ef46**，SSH准确Git输入，本地未push；原UID/flock/netns/资源/期限门不变 |
| pair核对 | response-priority-source-pair-v3.json：两个完整树只有link.rs的priority const一行增/一行删，PULL_AHEAD=8；所有fixture/parser/协议SHA相同；source/snapshot各有隔离ref pin，工作区保留B |

候选只在首次write前降低FetchPath/SatFetchPath的offset>0且成功PathSlice响应优先级；首切片/目录仍0；没有增加连接/任务/定时器/共享map/重试，没有修改信任/写入/0RTT/原caller。它是**实验候选而非已选最终设计**：首切片及时不证明完整前台文件及时，严格跨priority排序可能饥饿，必须完成公平/背压/高BDP/共享分连接/成本对照。

前置均已收件/核实：
- **Mac43061931813**，requestcd1cb21f0dcb4cdb8cd595609ccda8f0，实际control68cadb57a0ed72d40a4ca9a843e86b9c6797bf34；exactartifact **13873209484**，ZIP **ba386502cfa04221d5c647575e2ddbbc0dd941645fb53c8a9bf42903f9b226f1**，20files/11controls/identity。固定12门全绿：budget32/hop89/net68+1原ignored/wires16/journal30/IPC18+六单项；真实六initialCustom/两incoming执行。node-suite-verification-v1.json与ledger终态；不是全部1378 Node或vendor测试。
- **B诊断 turing-20261111T164701Z-a5469ffc**：283.548秒、原退出1，25files/identity/原group空。三个实际场景2160请求全ok、三份16MiB一致、1272实际业务stream配对；1080目录和3首切片getter0、189后续切片getter-1且setter全成功。真实w3/w3-startup两个guard均过（含真实sleep子进程未poll与等待中取消后回收）。原parser错把双test guard摘要按单test处理，原report false/原job失败不改；35412360修为guard严格2、workload仍严格1，合成先红后绿11项1.306秒。wire-priority-readback-verification-v1.json独立核原始行，ledger终态并明确metadata_reanalysis_verified，不伪称原job绿。
- **A诊断 turing-20261111T165434Z-9bc952d4**：310.362秒0、25files/identity/原group空；同control/source-v3，三个场景2160全ok、三份16MiB一致、1272实际配对，1080目录/全部192切片getter0且setter成功，两个实际guard过。原source-owned contract true，wire-priority-readback-verification-v1.json/ledger终态。

Mac/B实际编译源是source-v2 **8371dbb024502c8a0fea952b0ea96fbff6a41f39** / tree51310731fd24ed9e18fa27060770c3ccee9570ce / snapshot5d989256ae5e0362440141dff578ea2c94e87bb7。source-v3相对它**只改两份文档、metadata parser与两个合成测试**，完整Git差异严格allowlist及零runtime文件差异已保存于pair-v3，故按计时前协议复用实际B/Mac运行时证据；不称source-v3在Mac重新全跑。两个arm计时均使用同v3冻结源对。此前v1 Mac43060911492/Linuxbd59仅格式红、零行为样本、相同准确patch已1df76f64应用，原件与终态保留。

## 近期生产参照与边界

生产选路/拨号候选基准仍 **3de372a026488a64a794935600e47db29f2f6faf** / treeae76cd646749ca624d5e3e98af01f96099ebd0e0 / snapshota9f46eaa909b8cf45663395912798effb31b7261，candidate-effective-route-source-v3.json；后续04c67d8e opt-in日志/default env查询与当前priority生产文件有diff，不能把旧矩阵当新source完整复验。

- f2f17750动态三轮27绿，ce3b1222恢复6绿，de04bcc1原p1/p1j/p1s/p5四长稳门绿（前三各600秒）。空闲回好路16–17秒/活跃2.2–3.15秒仍只是旧门内，不是极致UX/完整N1接受。
- 四peer6970ad46三轮6绿，f4r首内容1.263/1.253/1.223秒，实际31响应/stream/单business wire及一个outer commit；原f4尾巴3.6–3.7秒未闭。
- 严重d8c22be9功能门绿、实际目录RPC8.713秒，其中约8.193秒是fixture故意封锁；从物理恢复到回复563ms。不能称8.7秒都是恢复后不明stall；同故障窗口固定备用Relay参照仍缺。
- **0354726c稳态参照九场景6绿/3失败**（一配对90秒、两启动60秒）。384已进入测量RPC全ok不掩盖三次未进入业务的失败。四scratch后代继承原锁；先核UID/PGID/start ticks/exe/环境/锁inode并保留stderr，再逐pidfd TERM，最终原group空才collect。steady-reference-failure-v1.json/cleanup.json/ledger终态；手工清理不是自动回收绿，底层启动/配对原因未证。bc519eeb/8371dbb0改fixture立即持有Reaped、先停keeper再删home，专项取消现已实际通过，不据此宣称原启动根因已修。
- 0ff8c054实际wire诊断1272配对，bulk目录native exchange p95约101–103ms、server accept→本地write约8ms、seat约10us；不直接归因CPU/磁盘/网络队列。bc4dbca9六轮记录器对照4320全ok，无稳定缓冲收益；日志有扰动，不能混作policy性能人口。kernel141b/a9ef读回/物理校准独立保留。
- Noq5 cfg(test)四专项：Mac43050978778候选447绿；隔离单门去除Mac43052478191恰446绿/唯一reclaimed采样计数6vs5红，其余三控制绿。主59补丁数量与生产保护未改；不是全部上游/物理/Noq1接受。

全部早期红项、准确SHA/任务/统计与限制沿历史链/ledger保留，包括573/166/eb72/412、严重6aa与quiet/resume/coldstart、Noq13物理长尾、Iroh47原门等；没有删除、放宽门或把截断/前置失败算通过。

## 完整剩余工作（不缩小目标）

- **N0**：最终分场景UX门、足量尾部人口、同条件完整前后与成本报告；当前不是p99或最终最优设计证明。
- **N1**：严重/quiet/恢复长尾、跨连接质量与比较成本，原启动/配对失败定位，同故障fallback参照。
- **N2**：同一真实连接独立/同时IPv4/IPv6地址迁移、LOST身份、完整取消/owner/迟到/撤销，以及carrier在调用开始后到达。
- **N3**：低profile18场景及独立UDP容量诊断已完成；旧TCP门仍红，分层模型判读v2已明确，先真实高profile预检，再独立比较prefetch1/2/8、大BDP300ms/50Mbit（先kernel校准/预注册新模型）、前台完整文件/预览、连续控制下bulk公平、共享/分连接、背压/取消、独立host/child退出、1/3/5设备CPU/字节/唤醒/能耗。小链路收益不能自动接受全部范围。
- **N4**：两腿Direct/Relay/peer relay/SSH、唯一/多middle、prepared/混合版本、middle自身v4/v6移动、整机/出口/登记故障域及基于数据的中继布局/运维设计。
- **N5**：默认库Noq447/iroh184已过（iroh有1原ignored），全部59 vendor的上游/退出与其他feature/integration/物理覆盖、Noq1/物理Noq5、最终准确集成源码Quality/native/proxy、多机/iPhone与能耗、统一决策报告。

## 执行约束与用户状态

本机source-only，仅Git/Python stdlib metadata/shell语法/合成工具测试；不跑本地Rust/格式器/Node/Xcode/产品。Mac走macserver+私库khor-build/原共享guard；Turing严格原UID1110/flock/user-netns、Rust1.95/jobs16/nice10/50GiB开始40GiB停止/90分钟，SSH BatchMode/ForwardAgent=no/StrictHostKeyChecking。构建不混入同机计时。无主机网络修改、无sudo；仅授权scratch fixture。未授权main合并、发布、购买/生产部署或已装App/config/store更改。

MacLab与Turing同LAN；macserver为另一台构建机，沿用户报告与Turing异网。iPhone是localhost、旧0.2.0已配对；历史反复开关App才peer relay恢复是原始问题，不是当前候选验证。没有候选安装/手机采样/能耗通过；旧手动测试窗口已过期。准备可信具体构建后再约最终真机交接，不能假设用户仍在等待切网。

16:41起给用户的估计：当前对照1–2小时、完整收口剩余2–4天，**低置信度**，新失败与真机验收会影响。此估计不是承诺或验收，不随compact/session重置起点。目标保持active，有可执行工作，不设blocked/complete。
