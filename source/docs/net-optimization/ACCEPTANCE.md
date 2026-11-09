# 验收协议 v1（N0，策略修改前）

2026-11-09。实现前冻结本文件的行为要求；实际运行状态与证据索引写入 STATE。

## 首批选路验收

- 单元：对 Direct / Carried / Relay 的所有配对，90 ms 与 5 ms 的有效测量中始终选择 5 ms；从活的慢路和刚切换的慢路都允许显著改善，快路不能被类型较优的慢路抢走。
- 真实网络 p6：设备各有独立 Strict NAT 的默认出口，另共享一条可直连的网络。中继设备单腿配置 40 ms RTT（端到端经过两次，约80 ms），共享网直连分别为约 300 ms 或局域网速度；两种场景应分别稳定使用中继和直连。
- p6 每例等待 10 秒供候选建立，再给最多 15 秒达到目标；连续 30 个目录请求（200 ms 间隔），零错误，p95 < 200 ms，观察期末两端仍在目标路径。报告样本量、p50/p95/max 和路径。30 例不报告 p99。
- 现有 p2（打字时慢直连回局域网）保持 10 秒内、无连接退役；p3 黑洞的最慢请求保持 1 秒内、无退役。先验证执行环境及这些基线，失败保留并诊断，不能删除或扩大原目标。
- 所有配对、暂停、only-mine 和 0-RTT 原有安全回归继续成立。

策略层初版使用所有路径一致的相对/绝对收益门槛，调用 iroh 的新鲜样本确认；不在此预先定死最终参数。每次参数改变记录对这些目标和 p1/p1j/p1s、p5 的影响，选择有对照证据的版本。

## 对照、重复与结果归属

- 先在未改策略的源码上运行新回归，取得预期产品红；编译/工具/执行器错误独立记录，不算复现。
- 然后对相同场景、工具链与负载运行候选；p6 与恢复场景至少 3 个独立重复。记录每次结果，不仅保留最好一次。
- 固定好路径作为可达到的参考：后续测量需在同一拓扑中单独屏蔽其他承载，不能把另一台机器的 ping 充作基线。
- 正常性能对照顺序运行，CPU 忙测试显式施加负载；执行器被构建挤占的结果需注明。
- 后续工作包的新增数值目标在其修改前按已取得基线补充；缺失的全矩阵、真机能耗和中继故障域测量仍是未完成项。

## 远端入口

控制仓库独立分支 `colehank/khor-build:feat/net`，通过现有 `development.yml` 的 Linux `network` job 执行；Mac job 对网络任务跳过。任务：

```sh
python3 scripts/dev.py check --task net-unit --builder-ref feat/net
python3 scripts/dev.py check --task netlab-smoke --builder-ref feat/net
python3 scripts/dev.py check --task netlab-compare --builder-ref feat/net
python3 scripts/dev.py check --task netlab-dynamic --builder-ref feat/net
python3 scripts/dev.py check --task netlab-hop --builder-ref feat/net
python3 scripts/dev.py check --task net-node --builder-ref feat/net
python3 scripts/dev.py check --task net-vendor --builder-ref feat/net
python3 scripts/dev.py check --task netlab --builder-ref feat/net
```

`net-unit` 验证格式与 khor-net 单元；`netlab-smoke` 的初版覆盖 p2/p3/p6，control d7d1af1 起加入 f4/f4s/f4i，显式2个场景并行用于功能回归；正式性能对照仍用单场景compare。`netlab` 完整矩阵保持原8并行。Linux 上先实测 user namespace、netem、nft 的能力再编译。源码使用原有 parentless snapshot + tree 校验，结果下载沿用 `dev.py result`，control commit 也写入回执。控制分支未经检查不得冒充已落地的默认入口。

`netlab-compare` 将 p2/p3/p6（含两个固定好路径参照）以单场景并行度1顺序重复3次，全部结果保留，即使baseline某轮产品红也继续其余重复。固定参照在同样双路拓扑完成准备后阻断竞争路径，再进行同样10秒预热和30次请求；Relay参照阻断唯一共享直连腿，Direct参照阻断中继数据面。参照是稳态可达到值，不计为首次拨号或故障切换性能。两版使用同一fixture，生产源码仍分别为旧/新策略。

control781a907增加`netlab-hop`，同一f4/f4s/f4i fixture按单场景并行度1顺序三轮，保留每轮失败；不改其网络/门槛。`net-node`补跑journal单元，验证path事件诊断字段可读旧journal且不改变显示句。221b98fc起path事件附缓存的传输RTT与选路quality/age/count，只在已有事件读取，不新增timer或探测；这是事件处理时的连接缓存快照，不冒充选择器内部决策时刻。

## 动态恢复目标（p7，修改采样实现前冻结）

与 p6 相同拓扑，先确认两端都由约80ms端到端中继承载（设备单腿配置40ms），再预热10秒，将300ms直连恢复为局域网速度。持续200ms间隔目录请求时，两端10秒内回到直连且无请求失败；闲置时20秒内回到直连。观察最多45秒并记录实际恢复时间，包括超限/未恢复，不能只报告通过者。

专用 `netlab-dynamic` 以 `NETLAB_TEST_THREADS=1` 顺序运行 p7，避免普通性能验证夹入其他场景的CPU负载。`NETLAB_TEST_THREADS` 默认仍为8，完整矩阵保留原有并行压力；报告必须写清执行模式，不能混用数字。

control c6a20c5 起，`netlab-dynamic` 将 p2/p3/p7 一起顺序重复3次，覆盖变慢、黑洞及好路恢复；老control的动态任务仅含p7。具体filters/线程数/重复次数由每run的measurement-mode.json固定，恢复时按准确版本解释结果。

2026-11-09 几何核对：`lab::slow(relay,40ms)` 给经过中继设备的单腿设置40ms往返，设备A→中继→B端到端约80ms。早期结果行“relay40ms”是设备配置值，实际路径读数84/88ms与此一致。已改标签和说明，拓扑、负载、200ms请求门槛及10/20秒恢复目标均未改变。

## p2协议纠错 v2（改测试前冻结，速度优先）

原p2的Relay没有延迟注入，LAN与Relay都可能约1ms；把“必须回LAN”作为通过条件与用户速度优先决定冲突。旧版结果全部保留，未回LAN不能单独证明请求恢复慢，旧日志缺逐次延迟分布也不能回填为新协议通过。p3的真实请求卡顿独立有效。

保持原拓扑、预热、约21ms慢腿注入与10秒收敛要求。新通过条件：故障前至少一端确在将变慢的second腿（否则是准备失败）；注入后两端10秒内进入已知快路径集合（未注入延迟的LAN或Relay），不能仍使用second腿。没有连接退役。

v2首次执行前补充审计：Link.rtt_ms是各连接的旧平滑RTT，缺样本年龄，不能拿它当新鲜RTT设门；曾拟的≤10ms门不采用。路径是否在快集合由本实验的明确拓扑决定，实际用户体验由下面的逐次请求延迟/成功率证明；平滑RTT保留为诊断值，不冒充新鲜质量。这个纠错在任何v2实跑前记录，旧记录不回填。

前台p2持续200ms间隔请求，包含收敛过程并在第一次达到快路径后继续至少30次请求；零错误，最坏请求<1秒（沿p3体验上限），全部样本p95<200ms（沿p6），结束两端仍在快路径。超时/错误保留进样本与结果，报告路径、收敛时间、样本量/p50/p95/max，不只报接口名。新增延迟要求比旧p2更完整，不放宽10秒/零错误/零退役。

p2i仍是原闲置观察项，记录相同快路径收敛与退役，无额外业务请求，不把观察red自动转成测试pass证明。两版正式对照需复制完全相同v2 fixture后重跑；旧LAN-only结果不替代新协议证据。

## p7相位扩展（新增fixture前冻结）

为覆盖闲置恢复19秒附近的裕量，在原10秒预热后增加故障注入相位偏移：交互0/2/4秒，闲置0/5/10/14秒。每个相位新建相同Lab，不改变路径、请求间隔、45秒观测上限或交互10秒/闲置20秒门。变化前再次确认两端确在Relay，否则记准备失败；不把未打中的变化算成功。

七个独立Rust case均由p7_前缀纳入netlab-dynamic：每轮单场景顺序、三轮共21个p7观察；另有原p2/p3，各轮保留失败。结果写明相位，不能将同一VM的21个案例称为21台独立机器。扩展后的完整矩阵是40项，扩展前35项结果仍按原源码解释。此扩展只增加覆盖，不推断探测开销已达标，也不由老phase0数据回填其余相位通过。

## 独立恢复与稳定性执行模式

为定位相位批次里的p2偶发16.2秒，以及完整矩阵的无收益切换，增加两个固定执行模式；不改变fixture、网络注入或原断言：`netlab-recovery`将p2/p3单场景顺序重复3轮（6项），`netlab-stability`将p1/p1j/p1s/p5单场景顺序各1次。后者仍保留p1s自身的忙CPU与20次300ms注入，不夹入其他场景的并行负载。每run保存measurement-mode.json。p2记录路径事件质量，p1失败及双路准备失败也保留两端诊断；输出发生在测量结束或失败之后，不新增探测。

## p1稳定性准备 v2（改fixture前冻结）

42524625685的p1j在准备期未选Direct，Relay约23–25ms、Direct约21–22ms，差值未必达到原收益门。
要求未明显更快的Direct必胜与速度优先冲突，不能拿准备失败当作600秒抖动证据，也不能恢复产品类型硬优先。

只对one_speed_stalling共用的p1/p1j/p1s准备增加受控引导：Relay设备先配置100ms往返（完整设备间路径约多200ms），
在原90秒准备上限内确认两端Direct；随后清除Relay设备双向延迟，成功清除后沿原30秒settle，再开始原600秒测量。
观察期不限制Relay或Direct选择。两条IP路的原1±0ms/10±2ms配置、最多一次移动/零退役、
p1s忙CPU与每30秒300ms停顿均保留；流量/打洞计数仍从settle后起算。原p5、p2/p3及转发fixture不改。

结果明确标注bootstrap版本；旧前置失败和旧切换红保留，不回填新协议通过。baseline6485cb83生产代码保持原样，
仅复制相同fixture/诊断及远端脚本，与候选按单场景各运行完整稳定性模式；不能混用旧准备与新准备的数字。
本修正保证实际测量能开始，不预认定选路或忙CPU抖动已解决，不把不同VM的一轮对照称为统计显著性。


## 忙CPU/短停顿定向复验模式

Watch确认改变使用netlab-stalls：单场景顺序运行原p1s与p5，各一次，保存measurement-mode.json。
p1s仍600秒/20次300ms/原忙CPU，p5仍150/300/600ms各4次、每次30秒间隔；不缩短计时或改门。
已有完整稳定性A/B与最终full仍需保留；此模式只省去未被Watch改动的普通空闲/抖动准备轮次，
不能代替全范围验收。源码与control commit单独回执，不将不同执行模式的数字混为同条件统计。

## 可选诊断模式与vendor覆盖补齐

`netlab-diagnostic`只用于定因：原p1s/f4s完整时长、负载和断言，各一次1线程。
显式NETLAB_DIAGNOSTICS=1才以CLI的net-diagnostics可选feature编译并启用subscriber；
记录ACK raw/delay/corrected、实际迁移Reading/Silence及恢复hold，测量结束后保存各节点原serve.log。
默认构建不安装subscriber。日志改变CPU/IO开销，诊断结果不能作为普通SLO或与普通模式混算。
生产策略变化后须关闭日志重新跑同条件回归。

control322835b起，`net-vendor`除原Noq全单元与iroh remote_state过滤器外，额外运行iroh `--lib khor_`，
覆盖此前遗漏的net_report家中继/QAD与socket网桥测试，分别保存iroh-home-unit.log。
这不代表iroh全158个单元都执行，也不能从旧33个路径单元绿回填这些遗漏测试。
补丁逐项缺口见[VENDOR-AUDIT](VENDOR-AUDIT.md)，原生代理/真机/能耗仍独立验收。

## 观察客户端 v1（fixture改动前冻结，非N2实现）

42536677230的f4s准备在LabNode.with反复Node::open_as处报document transfer state busy，未测恢复。
Node打开会申请本地状态permit并进行初始化，不应每次观察都重复；生产app本来复用Node。
LabNode改为在本设备runtime构造一次Arc<Node>，后续with在同一runtime使用克隆的Arc。
不改生产状态门、不加Busy重试、不忽略IPC/状态错误，初始打开失败仍报告准备失败。
明确输出observer client v1；网络注入、settle/600秒/停顿/原断言不变。
该fixture改变本地观察开销，未来A/B必须复制同一lab.rs；此前四文件匹配A/B只对旧观察方式成立，不能回填。
先用原时长诊断验证助手，选路策略暂不改；netlab-hop-diagnostic只跑原f4s一次1线程，
用于避免准备修正的验证被p1s十分钟等待挡住，普通SLO仍须关闭日志复验。

## N2 行为协议（实现前冻结，未执行）

当前link.rs已核：普通ask首轮line_to未传by，锁/hop等待不受by约束；exchange重建自身budget；
0-RTT握手后交出的asked JoinHandle直接await、失去原Aborts所有者。这些是静态缺口，尚未取得行为红。
N1诊断在途时先冻结下面协议，N2代码仍待N1收敛后实施；不把本节记成N2已完成。

| 场景 | 通过行为 | 对照/证据 |
| --- | --- | --- |
| 同一操作先等拨号锁，再准备hop、拨号、交换和一次读重试 | 共用单调时钟的绝对截止点，每段只用剩余预算；不得重新获得完整28/30秒。到期后不再启动新网络尝试 | 控制各段完成/阻塞的时序，覆盖普通首拨和早数据；在旧源取行为红，再验证候选。保留现有IPC 30秒外层上限及请求自身预算 |
| 等锁时取消/到期；同一对端另有正常调用 | 取消者释放自己的等待/任务，不污染peer失败退避，不终止别人的健康连接 | 两个调用者并发、单个取消；记录锁/任务/活连接数量及远端收到的请求数 |
| 两个有界拨号尝试，一胜一仍在握手/失败 | 胜者可用，输者任务与临时连接最终释放；候选不超过现有2个。失败地址不占用整个操作剩余预算 | 分别控制direct提示尝试/identity尝试；胜者标签与实际路径分别记录，不能把Won::Direct当成实际直连 |
| 0-RTT握手已接受、回答未到时取消/到期 | 回答任务仍有所有者，取消可释放流/任务；已adopt的共享健康连接保留。不因早回答缺失自动放宽写请求重试 | 控制握手接受/拒绝和回答时机，检查远端执行次数；原rides_early/is_reading准入不扩展 |
| 读请求在失联后允许的一次重试 | 重试沿用原截止点，最多原一次；仅原有可重试分类，不能把远端活着但处理慢当成路坏 | 故障在写入前/后、已收到响应前缀、远端慢/全路无答分别测试；超时和失败不从延迟样本删除 |
| 已写出的命令/文件修改在重连、取消、早数据拒绝附近 | 保持原语义准入，远端执行至多原请求允许的次数；结果未知不能伪报失败未执行，更不能隐式重放 | 带可观察副作用的请求计数与内容校验，应用重试和QUIC重传分开记录 |
| 预算耗尽而连接仍有正常往返 | 结束该操作，不能仅因调用方预算用完退役健康连接或给peer加故障退避 | 持续其他请求/保活对照，区分Miss::Late、Unreached及取消；包括到期前一瞬间/已过期入口 |

计时单元用可控时钟验证先后和预算不重置；实际网络实验记录计划截止点、完成时刻和runtime调度滞后，
不能把非实时执行器的绝对墙钟零超时作为保证。首次连接p50/p95、换网/睡眠恢复、后台扰动分布与成本目标
仍需各自基线后冻结；此协议不提前批准1500ms竞速间隔、缓存TTL或退避随机范围。


N2源码复核补充（33202c22，尚未实施/执行）：Op::deadline含30/300/3600秒档，Request::budget含长操作；ask_on固定28秒不能被直接提升为所有工作的总上限。
共享的是调用方该次操作的实际截止点，保留原请求工作预算及外层档位；后台入口独立建立一次有界期限。
hop现有3秒覆盖两个middle合计。is_reading含既有按token去重操作，rides_early是更窄集合；不合并或放宽。
取消/到期不应清除别人的健康wire，但RetireUnheard对真正无应答坏wire的独立证据仍保留。具体下一批设计见NEXT-DESIGN。


## N3底层congestion测量协议 v2（改记录前冻结，尚未执行）

现有ignored `crates/net/tests/congestion.rs`只将成功echo加入answers，失败被丢，摘要却标“asked”；空成功列表还会报告零延迟。
它不能证明无错误或完整尾部，也不是实际Node/SSH/hop验收。本协议纠正后续采样，不回填旧摘要或把修记录算生产性能改善。

保留已有模拟几何：默认RTT300ms、每方向2%损失、20Mbit/s、200ms队列；环境覆盖仍明确记录。
保留两方向各自固定seed，以及Cubic/NewReno/BBRv3与现有priority五组；CC_ONLY仍按旧约定只取所列控制器、priority关闭。
每一组按原20秒bulk、3秒预热、结束前2秒不再发起echo、前一请求结束后等250ms再发下一请求；每个echo原10秒上限保留。
报告实际运行时长，不把序列请求误称固定每秒四次或承诺恰好20秒完成。

记录规则：

- bulk保留收到字节、实际读取/总case时长及结束原因；EOF/读错误不能被静默当成跑满时长。
- 每个已发起echo都保留实际elapsed和outcome（ok、timeout或open/write/finish/read阶段错误），不删失败、不把超时伪成一个成功延迟。
- 汇总分别记录attempted/answered/failed/timed_out；成功响应的分布和全部尝试结束时间的分布分开标注，空集合为无数据，不显示0ms成功。
- 原始逐次记录在该测量阶段结束后输出，避免在计时中的每次echo打印；保留controller、priority、index、elapsed和outcome。
- idle→burst仍每组3轮、每轮先typing10秒再读80×80字节screen；typing的失败也要保留。screen整个open/write/finish/read采用明确10秒上限，原1MiB接收上限和完整6400字节校验保留。
- 所有组/轮尽量保留记录后再汇报任何请求错误；编译/fixture失败与有效测量中的请求失败分开。不通过筛选成功者得到“无卡顿”。

初次复验顺序执行3轮，相同参数、源tree/control/seed均入回执；然后再定义多seed和实际Node负载对照。
这是baseline记录协议，没有批准新的p95/p99、吞吐/能耗阈值，也不通过改记录、改控制器或增加并发来制造改善。
样本少只报n/p50/p95/max，不报p99；失败/取消/超时保留在结果里。可信SLO与真机验收仍按PLAN另做。


v2测量候选f17b8a31已实现，未编译/运行。cc-sample-v2保存逐次结果，cc-summary-v2区分成功与全部尝试分布，空集合null；
cc-case-v2保存几何/seed及bulk字节、读取/总case时长和结束原因。每组完整输出后，有错误、零样本、bulk提前结束或零字节使工具失败退出。
工具绿也只证明该次基本完成，不等于吞吐/交互SLO。失败之后其余controller和重复轮仍尽量继续保留。

control 6447289f4f9260df4f5887adc7c1e2f2270bbcce已push feat/net，新增入口：

```sh
python3 scripts/dev.py check --task net-congestion --builder-ref feat/net
```

这是底层Noq loopback UDP模拟，顺序3轮，每轮--ignored --nocapture --test-threads=1，固定默认几何并清除继承CC_ONLY，
保存congestion-N.log和measurement-mode。它不走namespace前置，不代表netlab或实际Node/SSH/hop已通过。
本机仅用Python桩验证driver，未运行真实Cargo/Rust、格式器或测量。远端恢复后须先格式/编译，再核原始记录，不能直接收口N3。


Mac便携网络入口（control6b1616c，已实际执行）通过--executor mac选择同一development.yml，不走托管prepare/retire。
原Mac入口仅允net-unit/net-node/net-vendor/net-congestion；现另加netlab-vz-probe（见末尾实际验证），产品netlab/native仍拒绝；默认standard不新增dispatch input以兼容旧控制main。
42551075037已实际完成根格式/437Noq，Iroh新故障边界35pass1fail为预期产品红。Mac平台不替代p3<1秒、转发/CPU及全Linux矩阵。


## N3模拟道路校准补充（2026-11-09，修改派发器前冻结）

静态发现：congestion.rs的双向Lane虽各有free_at，却共用单个FIFO发送任务。该任务等待队头未来due时，会阻塞队尾已经到期的反向包。现v2仅完整采样记录协议，不能据此把该模拟器视为已校准全双工道路或决定产品CC/优先级。

先新增实际UDP因果回归：RTT20ms、每方向1000B/s、队列1000ms、0%损失；client连续发两个500B包，server收到第一包后立刻回1B。反向包应在后续正向序列化结束前独立送达。绑定/设置在计时前；给反向独立送达250ms调度容差（理论约11ms，相对下一正向约500ms间隔），仅为模型校准，不是产品SLO；外层防挂并清理测试runtime。

旧共享FIFO先远端取红，再独立方向派发或按due排序并取绿。记录schema可沿v2，必须显式区分road_model版本；之前同源数据原样保留，不把旧模型与修正模型差值当成产品策略收益。完整采样/默认300ms/2%/20Mbit/200ms/20s/seed不因校准失败而放宽；实际Node/SSH/hop/能耗继续独立验收。

## N2真实Accepted回答取消回归（执行前冻结）

`link::dial_budget::cancelling_an_accepted_early_answer_stops_only_its_stream`使用scratch Node、Minimal且禁relay/discovery的端点，server只绑127.0.0.1；先完成一条真实Sessions请求与应答，为同一client/server身份建立resumption cache。
第二条请求必须由server实际收到、已被Node adopt且真实ask_early在额外20ms观察内仍等待，才进入取消观察；生产的fresh/Rejected路径在adopt后直接返回NotTaken，只有Accepted带回答任务才继续等答。未满足前置是fixture失败，不是所有权行为红。server is_0rtt只保留观察值。
取消并join真实ask_early调用后，500ms观察server发送流收到STOP_SENDING（Ok(Some(code))）；连接关闭/EOF不能替代该证据。随后普通ask_on Sessions必须在5s防挂内成功，Main连接stable_id保持相同、peer dial_failed为0、总请求严格3次（warmup/被取消读/独立读），最后关闭端点并停止fixture任务。
500ms是取消观察容差而非产品SLO。server延迟回答、不关闭共享wire；原生产Accepted裸JoinHandle代码保留，先执行取实际红，再修所有权。已有scope40ms/300s和两锁回归仍同批运行，不扩展0RTT/重放白名单。
STOP_SENDING观察按[Noq 1.3 SendStream::stopped](https://docs.rs/noq/1.3.0/noq/struct.SendStream.html#method.stopped)；本地只读源码、未运行Rust formatter/编译/测试。

Accepted前置42566727617未成立，不能算取消行为红。为让本地调度下真正发出early数据，server接到第二次Incoming后延迟100ms才accept/发首TLS握手包，仍以is_0rtt检查为准；该fixture延迟不参与产品性能判断。新增同fixture的deadline变体：warmup后为ask_early传1秒绝对by，必须先adopt且仍在by之前；在by+500ms防挂内必须正常返回CallerExpired，并仍停止回答流、同wire独立请求成功/计数3/零peer失败。旧裸回答所有权保持待红。

**前置修正依据（取新行为红前冻结）**：42566727617只有旧is_0rtt前置红；42567204795格式门失败未测试100ms延迟提议。准确依赖[Noq1.3 poll_accept源码](https://docs.rs/noq/1.3.0/src/noq/connection.rs.html#1147-1159)在应用accept时取is_handshaking，无法保证历史数据包的加密级别。撤回100ms延迟及is_0rtt硬前置，改上述实际Accepted路径与20ms pending观察；没有删除STOP_SENDING、CallerExpired、共享wire/请求计数/peer验收，也不声称该测试验证包级重放安全。详细推理保存evidence/net/accepted-answer-fixture-notes-20261109.md。

## N2普通wire/读重试/写失答覆盖（执行前冻结）

新增`link::dial_budget::wire`四项真实连接覆盖，继续由现有net-node profile选中。复用scratch Node身份fixture；server仅loopback、Minimal无relay/discovery，先完成真实握手并adopt Main wire，再开始预算。生产实现本批未变；这是补足既有贯通候选的运行覆盖，若绿不宣称这些新增用例在旧源实际取红。

- 普通exchange_on在1秒scope内等待一个延迟Sessions回答，同时另一无该scope的Sessions已被同一server接收并等待。仅收到首流STOP_SENDING后才回答第二流。首调用须正常CallerExpired，第二调用成功、stable_id不变、请求数2、零读重试/拨号失败/answer_late/未答退役/down/backoff。by+500ms只是防挂容差，不是产品SLO。
- 真实ask_on首个Sessions在原2秒scope耗至仅余700ms时关闭连接且失答；第二次真实握手和Sessions必须在原by之前进入，回答保持等待。调用在原by到期而非另起完整预算，第二流收到STOP，恰好两请求/一次读重试/一次真实closed退役，无dial_failed/answer_late/未答退役；第二条live wire保留。
- 两项失答对照都由server接收并执行后立即关闭conn，不发送回答；server仍继续接收可能的重放（每轮800ms观察、最多4条），不以不服务第二连接隐藏重发。Mkdir只执行一次/零读重试；Sessions严格两次/一次读重试；调用正常错误、server完成，零dial_failed。

所有绑定/初次握手在计时前；任务用Aborts归测试所有，超出观察会abort并join，最后关闭端点。短预算场景不替代真正黑洞持续超过Patience.ack后的F7退役验证，该项仍待Linux/可控故障通道。

## N2配对诊断探针所有权（修复前冻结）

新增`dial_budget::probe`三项真实HTTP回归；生产pair_with/doctor::any_relay_answers保持原样，先取得实际红再修。fixture仅loopback临时TCP listener，匹配doctor的GET /probe且排除relay upgrade；完整收到请求后不响应，用对端TCP EOF证明该请求已停止。若环境存在HTTP(S)_PROXY即fixture前置失败，不修改全局env，也不把前置失败算行为红。

1. 直接取消正在等HTTP的any_relay_answers父任务并join后，800ms内其请求应EOF；server不提前关socket。
2. 真实pair_with使用scratch身份/无实际秘密ticket、127.0.0.1:9不可达issuer和临时probe URL；收到GET且配对仍pending20ms后取消配对并join，800ms内probe须EOF且零dial_failed，最后才关闭endpoint/server。
3. slow URL排第一，fast URL第二；两边均已收到GET才释放fast响应。要求fixture在任务启动1秒内发出fast 204，随后500ms内any_relay_answers正常true，800ms内slow请求EOF；这两个观察合计早于原2.6秒HTTP自身超时。不能等待自身超时假装赢家/取消释放，也不能用关闭测试server制造EOF。

以上为功能/资源释放观察，不是产品RTT SLO。每任务有Aborts所有者、观察结束后abort/join fixture；不加并发数、不改原2.6秒单探针上限或proxy策略。空/全部失败的boolean语义需保留。


## 真机验收设备组合（用户确认）

2026-11-09用户确认iPhone＋两台处于不同网络的电脑；随后明确“macserver、linux就是天然的不是一个网络的设备”，“linux指turing”。矩阵固定为P=iPhone、A=macserver（Mac）、B=Turing（Linux）。型号/iOS及手机网络条件仍待采集；两服务器可先执行A↔B真实跨网验证，不能继续把“两台异网电脑身份未知”列为依赖。

先用准确源码、可信构建与隔离测试身份/数据执行A↔B双向冷/暖连接、前台交互与批量共存、断连/进程重启恢复及空闲开销，保留实际端到端路径和全部失败。随后补P↔A、P↔B及iPhone支持的换网/休眠恢复、后台能耗。设备确认和测试准备不代表已执行或通过；不得以同机namespace、Mac loopback或Noq模型替代真实跨网结果。

记录三端OS/硬件、网络条件、源码/可信构建身份、双端路径、工作负载、失败/取消/超时和原始数据。沿现有远端构建/隔离Lab机制，不替换现装App或使用生产store/个人会话。原1/3/5规模测试须标明物理设备、独立Node进程和模拟拓扑；已确认三台物理设备不能写成五台。


## 隔离Linux能力入口（42579909366）

`python3 scripts/dev.py check --task netlab-vz-probe --executor mac --builder-ref feat/net`：control bedd896，Mac原单槽/guard下启动项目专属Lima2.3.0 VZ，Ubuntu24.04固定镜像，2CPU/6GiB/30GiB，空host mounts/无个人凭据或自动应用端口。先固定完整临时socket<104、当前boot IID就绪、guest实测userns/veth/netem/nft/conntrack、正常和控制器SIGKILL后lease清理、同PGID/lock fd/最终Stopped无helper，才允许后续产品任务接入。

42578713267：socket107字节，VM前失败；42579282147：实际Linux启动/正常清理绿但guest探针空输出非零、完整就绪未证，未验父死；42579909366：所有上述能力/生命周期绿。三run准确tree2b5bfce2/snapshot6a046588，同源不同可信control97334be/8d39080/bedd896；前两红仍保留，不算产品红。最后一次两不同boot IID、真实controller=-9、两个stop rc0、双Stopped、无残留，原始证据在mac-vz-ready-42579909366。

此任务不运行产品编译或测试。Linux aarch64 VZ上的netlab接入、编译及原基线/候选门仍待执行；旧托管Linux硬件/架构不可直接比较。源码/结果回执和正常/失败清理不得因接产品而略过；默认standard仍沿原执行器。用户iPhone/两台异网电脑验收独立保留。


## U9 Turing主Linux执行器

用户明确授权Turing并优先其性能。SSH既有alias/known host，不forward agent或token；项目owner root `/tmp/khor-netlab-feat-net-1110`，每run不可重用目录/receipt，固定Rust1.95四component SHA、CARGO_HOME/target本地盘、原network.sh filter/repeat/gate、flock串行，编译jobs16/nice10、测试线程沿已冻结profile。只在新user/net namespace操作nft/netem，不改宿主网络/默认Rust/系统包/现装App。

source snapshot无父+bundle SHA256、远端tree/metadata/materialized commit、可信driver字节与control、environment均核；本地ref位于refs/khor/net-evidence/<turing-run>，不冒充GitHub run/release。终态采证使用control scripts/collect-network-ssh.py receipt.json：先terminal/原job group，再不跟链接的tar/身份核验及逐文件SHA。报告留存失败与所有repeat。

turing-20261109T075531Z-b6277f48候选recovery3轮6/6实际绿（p3最慢518.5/597.9/706.9ms）；同host baseline127th ref在途，尚无速度差值结论。后续CPU/hop/full和N2–N5矩阵不因此缩减，旧托管Linux红仍保留。


## N2全路径探测宽限前的取消（F8，修复前冻结）

Linux-only真实fixture，先启动新unshare -Urn子harness，核network namespace不同与单uid映射，才up lo/加tc；不改主机网络。先完成原生UDP握手并settle 1秒，再只在该namespace的lo加netem每向2400ms（约4.8秒往返），启动两个真实exchange_on Sessions，共享同stable_id；其中一个by=4.1秒，控制调用by=12秒。

前置必须证明两请求恰各一次到达server、其发送响应时刻>=2.2秒，响应再经过固定2.4秒路程晚于首调用到期，首调用正常CallerExpired且到期观察在by+400ms内、到期前heard_count未增加。前置失败独立归fixture；未取得实际运行红前，不把静态风险写成行为红。

目标：首调用到期时wire仍live，控制请求收到真实延迟回答、heard_count随后增加，同stable_id继续保留；零retired_unanswered/closed/readretry/dialfailed/answer_late/down/backoff。12秒与400ms为测试防挂/调度观察，不改产品budget或体验门。场景结束捕获全部事实后才删该namespace的qdisc/关闭端点，异步任务均Aborts持有并abort/join。原真实长静默F7和长ACK阳性必须保持；不能一律禁用取消退役来让F8过。


## N2更新连接独立10秒上限（新增运行覆盖）

两个portable回归调用真实Node::dial_update（UPDATE_ALPN），scratch Node/Minimal endpoint、only_mine原测试配置和临时loopback UDP黑洞；sink实际收到UDP才满足前置，未触碰更新包、现装App或生产store。200ms操作scope时须由dial_update自己正常返回错误，30秒scope时仍在原10秒自身上限返回；外层expected+500ms仅防挂，触发外层timeout不算通过。实际elapsed下界expected-20ms，上界expected+500ms，sink首包早于expected；零共享wire未答退役/peer down/backoff/answer_late/dial_failed。此为现有合同新增实测覆盖，未声称旧源行为红，也不改变10秒/200ms/长操作档语义。


## N1路径与退役关联的诊断身份

后续f4i诊断保留现有journal，Path和Retired可选wire编号取同一进程内Connection::stable_id，跨节点/进程不比较编号；缺字段的老日志仍可读，显示句子不带实现字段。只补身份、不改路径选择或现有peer/change去重。以实际路径事件wire与retired wire关联区分新旧连接，再与主wire/Link采样对照；不能仅凭同peer的平面事件假定都是同一连接。字段兼容回归通过也不构成真实切路/无回退的证明。


## F8共享确认修复候选的验证合同（f7_owned_confirmation_v2）

F8实际红143rd的函数保持逐字不变（SHA见evidence/net/f8-red-to-candidate-fixture-identity.json），仍为4.8秒原生UDP往返/4.1秒首调用期限、同wire另请求必须成功。修复将每wire的probe/count/grace和有界任务交Wires::Slot拥有，正常exchange和取消共用；不同调用不重置相同计数的确认，收到新包、adopt替换或清理wire会取消旧任务。任务只持Weak owner/connection，guard捕获原runtime，Drop即使不在entered runtime也能发起确认。

F7保留原私有namespace、真UDP双向丢弃、计数/远端未关闭/恰一次unanswered及零其他故障条件；修改的仅是取消后的观察：从std::thread scoped thread丢弃已poll过且长静默的future，立即仍须持有wire，随后在自身全路grace+500ms内完成确认并退役，无resident tick或应用future继续poll。旧“Drop同步关闭”不具备probe证明，不能作为必须保留的实现细节；真正长期交出死wire的故障仍受原黑洞验证约束。

另两portable真实连接回归：pending确认不得保留Node/Wires强引用；确认期间adopt一条不同stable_id新conn，等超过loopback确认窗口后，新的真实Sessions仍成功、无unanswered/closed退役或out_of_reach。不是只测计时辅助函数，也不以连接被测试server主动关闭制造通过。当前仅候选，尚未格式/编译/运行。


## N2确认期间巡检与Accepted长静默取消（F9/F10，修复前冻结）

F9复用F7真实独立user/netns、原生UDP端口双向nft drop、首次poll后不再poll应用future，保持原ACK+grace等待与外部thread Drop；另在等待开始标记真实network_changed，取消发起slot确认后立即调用真实Wires::tick。网络变化3秒低RTT期限已过，但新确认grace尚未过；tick须不抢先退役，wire仍同stable_id，随后在owner原grace+500ms内恰一次Unanswered、零Network/Silent/Closed/readretry/dialfailed/answer_late。确认期间仍无收到包、真实包命中drop、远端未关闭；前置不满足单列fixture红。原F7不加network change，F8函数不变。

F10先用原真实resumption fixture到Accepted/adopt/响应pending，健康Accepted两控制保留。新增黑洞读取用原30秒预算且准许early的FetchPath（warmup仍Sessions），避免Sessions原15秒工作上限先到；不改产品budget。新增Linux私有namespace场景先通过oneshot显式暂停外层ask_early future的poll（内部响应task与transport仍运行），收到暂停确认后才对该server UDP端口双向drop，稳定300ms后记录heard_count，等待该连接原Patience.silent+100ms，证明长期零新包、任务仍pending、远端仍open且请求严格warmup+早期各一次。然后取消并join调用；立即仍保持wire，随后owner全路grace+500ms内恰一次Unanswered，无resident tick或其他request帮助，零Network/Silent/Closed/readretry/dialfailed/answer_late。测试不把整个操作取消本身当peer失败，也不额外重发该读取；结束前采事实，再关闭端点/abort join服务任务。150/300/500ms等均是观察容差，不改产品期限。未实际跑出行为红前，不把静态风险写成已复现。


F9第一次157th目标失败，但旧共享helper将confirmed断言放在事后unheard/drop/remote-open断言之前；因此157th F9暂只记目标红，下一源将这三项原前置断言前移到目标之前，所有网络/等待/阈值不变，不据尚未检查的局部变量预记完整行为红。F10在同157th已通过所有前置后confirmed断言实际失败，行为红成立。


F10修复候选另补portable长健康Accepted阳性：原30秒FetchPath预算、真实Accepted/adopt后等16秒，传输保持健康；取消须STOP本流，独立控制请求之前先等原all-path grace+500ms，仍同wire且零Unanswered/Network/Silent/Closed，随后Sessions成功、请求总数仍3。避免长取消清理死wire的修复误伤有正常ACK的长工作；原短取消/到期和原Linux F7/F8/F9/F10门不变。


## N0 CPU负载有效性（checked_affinity_v3，实施前冻结）

p1s原门/600秒/每30秒一轮300ms道路延迟尖峰不变，仍为两个resident和两个busy loop共用一个CPU，不增加竞争总量或并发。新revision从实际thread-self允许的CPU ID集合选择，必须核每个resident全部现有thread绑核成功；两个contender必须已exec既定busy shell、存活、全部thread同CPU，且各自CPU ticks真实增加后才进入计时。开始/每尖峰前/结束记录结构化schema/revision/phase/CPU/allowed集合、PID/starttime/thread亲和及CPU ticks；PID变化/退出/亲和漂移/竞争进程不再运行均按fixture失效停止，不记为产品性能红。CPU计时窗口从admission成功之后开始，旧报告不回写成新负载通过。

短校准单独入口，不缩短p1s：以两个自己spawn的idle子进程代替resident，只验证绑核与真实双busy竞争/CPU计数增加/Drop后子进程reap；另验证稀疏CPU ID解析、缺失/非法集合拒绝、一个实际非零pin命令和不允许CPU都拒绝admission且不改变已有子进程的亲和。校准运行不得与计时netlab并发。两版后续对照使用相同新fixture逐字源码/原profile门，并保存完整source/control/report。


## 共享carried所有权合同（real_connections_v2）

Peer共享底层carrier；iroh共享selected FourTuple，但每条connection的open paths可能不同。`PathList::is_selected`是该tuple是否存在于connection watcher，不能单凭入站无selected marker推断数据没走carrier。校准场景在同一个本地endpoint同时保持一条IP outgoing和同remote key的carried-only incoming；远端分别为loopback-only与carried-only隔离endpoint，两条连接都须实际echo字节。两秒内验证outgoing确选IP、incoming非空且仅Custom、carried-only client确选Custom且无其他路径，保留三方facts和不同wire IDs；未满足前置不算清理逻辑红。

目标是仍有这条真实carried消费者时，不能因另一条IP wire胜出而释放peer共享carrier；只剩IP消费者和carried incoming关闭后都应可释放。177只证明v1前置不成立，目标未执行；v2校准理由来自vendor apply_selected_path/record_selected，原ownership目标和两个清理控制保持。此合同不替代162原f4i故障因果与完整hop矩阵。


## caller outcome量测v4（校准待执行）

paths echo与hop typing每个被实际poll的逻辑目录请求留唯一(pid,attempt)的started/finished，成功、错误、超时、调用方取消与panic abort均保留；30秒原外层预算不扩大，记录工作纳入同一总预算。内部重试属于同一个用户逻辑请求。记录包含case、from/to、操作、wall timestamp、单调耗时、deadline及错误原因；f4另留观察/故障生效与清除的实际时间，避免将观察前路径变化误作窗口内变化。

独立ignored校准用真实两Node：同一peer目录请求成功、缺失目录错误、SIGSTOP状态确证后的200ms超时、确认future已poll再取消、CONT后恢复成功。外部schema consumer必须得到五个started和五个finished（ok2/error1/timeout1/cancelled1），不接受重复、孤立、上下文变化或无instrumentation。控制netlab-measure-check单线程一次、非SLO；stub驱动检查不替代该真实校准。caller取消记录不证明远端执行停止，IPC取消传播仍属未完成N2。

旧176/181是v3原稳定性门对照，不能因新v4存在就回写为完整失败分布。新量测用于性能对照时需重新绑定相同fixture、记录开销、CPU/allowed集合及原目标；旧门不据此放宽。


### f4共享载路生命周期诊断（独立非SLO）

`netlab-hop-ownership`在Turing仅跑原f4 filter、单线程三轮，保留原准备、故障、时长与所有断言。`NETLAB_DIAGNOSTICS=lifecycle`仍显式编译net-diagnostics，但native subscriber为off；初始化成功后写`mode=lifecycle samples=off`header，人工生命周期事件单独输出。lab必须实核header并留原始serve日志；端点事件在报告中单独核对，中间splice角色可无本地载路事件，不用高频ACK/selector sample日志扰动该轮。原`NETLAB_DIAGNOSTICS=1`及普通timed profiles保持。

原`link_attached`在map写入前仅代表意图；新`link_attachment_committed`在map锁内、写入后输出serial/previous_serial，配合link_io_ended、link_detached、hop_measured_release、carried_close_requested定位当前世代。f4即使通过也留六个方向journal；准备失败也尝试保存各resident诊断；先写出原始文件，再核模式，保留校验失败证据。v4逐次请求与故障时间点保留；三轮有红继续后续轮，最后必须通过完整性consumer，不将缺失请求算零失败。诊断绿不等于修复或SLO绿。


### f4准备前置 held_precondition_v2

190第三轮在观察t0前81.4/54.8秒已由当前owner释放两端载路，旧f4_lab仍因忽略until结果而返回Relay起点；第二轮则根本未进入故障窗口。这些不能当作有效“当前carried发生停顿”的SLO样本。新revision要求初次Y达到carried后，双方reported Forwarded连续10秒，任一丢失重置；原两次60秒等待采用合计120秒绝对期限，包含状态查询。到期标准备失败并保留三方journal/诊断，不注入故障。phase记录hold起止与失败；原故障时长、1500ms恢复、30秒无回退、30秒请求期限保持。

这是准备有效性修正，非放宽恢复门，也不证明reported状态等同独立物理快路；固定好路径参照仍需补。记录held_precondition_v2，未来baseline/candidate须同fixture重跑，旧v3/v4结果保留其当时含义。首次远端运行前，不预记该修正通过。


### N0 hop稳态参照 hop_reference_v1（首跑前冻结）

新增独立ignored实验，复用f4的严格IPv4 NAT X/Y、公开M、各设备20ms RTT与同一22ms单向中继几何；原f4故障/断言不变。三组使用同一构建器和负载：automatic（全部候选）、relay-only（配对前在两端scratch设置既有peer-hop=off，M保持在线，保留相同三节点/同步背景）、carried-only（配对完成后在X/Y各自namespace双向阻断唯一relay IP，保留M直连腿）。不冻结中间设备，避免把中间设备宕机恢复成本混入Relay稳态参照；不改产品默认配置/selector/CC。严格NAT无IPv6/共享直连网；每组留隔离配置/实际nft表及双端journal，物理前置不成立独立记校准失败。

三组都先真实建立两个方向的目录请求，再在120秒绝对准备期限内等双端有效路径连续稳定10秒；fixed组必须为所选承载，automatic可为任一实际可用Direct/Relay/Forwarded。准备请求同样记录outcomes、单独setup case；不能将其作为稳态样本。之后Y→X与X→Y各30个原30秒预算目录请求，间隔200ms，按方向保留全部耗时/错误并报告n/p50/p95/max与起止路径；样本少不报p99。首个错误后可停止该方向以控制坏路任务总时长，必须记录已尝试数、失败以及覆盖不足，n不足不能算通过。两方向都尝试，case失败不阻止同run其他reference/repeat。

此实验仅要求准备/隔离有效、完整30×2请求零错误及fixed组末尾仍在目标路径，不新增p95阈值来调参。它提供可达到的稳态操作参照，不是首次拨号、f4故障恢复或能耗验收。控制SSH-only netlab-hop-reference按一线程三轮，sampling diagnostics关闭、非SLO；仍用attempt_outcomes_v4完整性consumer。未来baseline/candidate须同fixture/工具链/负载串行，先看真实数据再决定gain/confidence/清理策略。


### hop_reference_v2：候选启动校准（198首轮数据之后冻结）

v1在mesh刚有设备表时便阻断Relay，没有证明carried候选已建立；198前两轮carried准备失败，automatic/relay-only虽然可测，尚不能据其差值评价未建立的载路。保留v1所有数据。v2不改产品策略、几何、负载、120秒准备上限或测量断言；在该总上限内先完成两个方向的可用请求，automatic/carried再等至少一端journal有向正确M成功打开hop的原事件。该事件在attach前发出，只证明opening已开始；阻断后的真实请求还须证明载路可用，不能把事件本身当成功数据面。carried此后才双向阻断Relay。

三组从同一准备起点至少等待60秒（给现有30秒compare两个机会），再各发两个方向的真实准备请求、确认连续10秒稳定后测量；计时总上限仍120秒。Relay-only无需不可能的hop事件，M仍在线；候选开路、阻断和measurement时间点分别保留。若120秒内没建立有效前置，则记未校准，不抬高产品SLO或从失败分布删除该次尝试。v2记录hop_reference_v2，不能混用v1或把对照当单参数因果证明。


## U11/U12：Peer relay与真实移动地址矩阵（2026-11-09补充，未完成验收）

Peer relay就是可信设备承载的carried候选，必须与Direct IPv4、Direct IPv6、公共Relay及已有SSH承载一起按实际完整路径/业务表现竞争。两腿相加只作候选估计，不替代端到端证据；两个方向分别测。覆盖单middle、多middle有明显速度差、middle失效/恢复与middle自己换网；体验接近时才比较成本。尊重only_mine/n0/暂停/信任名单，不能为提速添加未配对设备或绕过配置。201仅证明单middle固定参照可用，不能算该矩阵已过。

移动地址按正常生命周期而非罕见异常验收：①IPv4地址/NAT外部映射或端口变、IPv6保持；②IPv6隐私地址轮换或前缀变、IPv4保持；③两族同时变；④Wi-Fi↔移动网、双栈↔IPv4-only/IPv6-only（NAT64单列）；⑤一端换网/双方交错或同时换网；⑥peer relay发生上述变化；⑦连续快速变化后旧探测/旧发布/旧关闭晚到；⑧休眠/恢复与重启留下旧缓存。区分已有活备用、只能重新发现、完全无网三种起点，不能把无网时段算作可恢复通路。namespace注入与iPhone真实移动分别留证，不相互替代。

每次留注入时间、实际新IPv4/IPv6与映射观测时间、发布/发现时间、首个成功业务、回到最好可用路、所有请求outcome/延迟与长流连续性；记录旧地址尝试次数/总预算、候选/任务/连接最终释放、背景字节/探测/唤醒。旧地址不能吃完整业务预算；已发送写请求不重放，健康共享连接不因单次换网/调用取消被误杀。先保留可追溯基线并按N0冻结各起点的数值目标/容差，再改策略；现有p2/p3/f4数值门与旧结果保持，不能把它们套成所有移动场景的未实测保证。

## Turing开发加速边界

204主机回执为128逻辑CPU、约504GiB总RAM/约468GiB可用，当前build jobs16。先从真实build日志区分编译、链接、业务等待和清理耗时；同一源码的普通/诊断feature与锁文件共享已验证缓存，源码和control身份仍逐run冻结。可评估build-only更高并行度，独占构建槽/磁盘门保持；计时开始前所有构建结束。数据解析、源码准备与独立Mac构建可穿插，网络计时不夹编译或其他压力。编译加速不得减少三轮/样本、60秒发现等待、10秒hold、原恢复门或改变CPU压力；高核数不代表移动端成本/能耗已验收。具体改动先记录配置与编译耗时对照，不承诺未测提速比例。


### U12前置通知合同：稳定接口不能被无关候选更新饿死

在任何修复前冻结：复用生产journal::interface_addrs/Interfaces，分别保持IPv4不变替换IPv6、保持IPv6不变替换IPv4。新接口集合第一次出现后保持相同，期间完整DirectAddr集合中的QAD-only候选每次变化；它们不属于接口集合。接口通知仍需在原ADDRS_SETTLE=1秒到期后可投递且只投递一次，不能每次无关更新从头等；真实接口集合再变仍重新稳定，回到已通知集合则撤销，空集合/同址恢复与Local↔QAD来源翻转保持既有合同。先跑原实现取得实际Rust红，再最小修复。此合同只校准生产通知状态机，不替代真实IPv4/IPv6重编号、NAT映射、发现/发布、业务恢复或手机验收。


### U12物理前置校准：ipv6_renumber_v1（未执行，先冻结）

为响应U12先补N0/U12物理校准，暂先于N1下一轮策略修改；不放弃完整N1。全新patchbay双栈Public router/device namespace，原IPv6、IPv4双向ping可用后，在一个device自身接口同prefix加新的IPv6地址，等待实际DAD完成并验证从另一device可达，再移除旧IPv6。实际kernel JSON分别保存地址前/加后/删后；IPv4地址集合保持且IPv4仍可达，新IPv6双向成功、旧IPv6不可达。每次ping只1packet、2秒命令等待，所有stdout/stderr/status/elapsed都留；共6个IPv6检查（5正1负）和2个IPv4正控，属于fixture数据，不是业务成功率/SLO/分位数。只在新建用户namespace内ip addr变更，宿主网络不改。单独SSH-only ignored profile，一轮/单线程；任何工具/前置失败分类为校准失败，不冒称产品回归。这次只证明注入几何成立，Node/QUIC/地址发布发现/peer relay/移动设备/IPv6前缀变化仍需后续矩阵。


### U12真实Node迁移基线：ipv6_node_overlap_v1（217已校准；原协议保留）

在215同prefix IPv6重编号几何上接两台fresh真实khor resident与各自复用IPC client、唯一marker目录。先mesh，再在双方eth0 input/output实际nft阻断所有外部IPv4（loopback IPC保留），因此IPv4地址保持但IPv4直连/唯一HTTP Relay均不可用、无第三peer/SSH备路。双方实际目录正控＋reported Direct连续10秒，准备绝对120秒；不能拿cached Link单独证明物理数据面。

测前两个方向各30次真实目录请求，200ms间隔、原30秒caller预算、验证指定peer唯一marker及大小。然后只给A加同/64新IPv6，确认assigned/ready，保留新旧重叠后删旧；三阶段kernel JSON留证。删除后立即同时发起双向首请求（先请求，再做任何耗时ping负控），然后两方向各30次同负载。全部started/finished与error/timeout/cancel保留，某方向首错可停但n不足不得绿；首请求与前/后warm分组，单独记恢复时刻、所有路径/退休/拨号与全局network事件。最后再验证新IPv6可达/旧IPv6不可达、IPv4地址未变、实际nft隔离仍在；日志不计入业务分位数。

首轮只校准前置、双向业务可达和记录，不新造恢复延迟SLO、不强断言复用同wire；连接连续性及首请求耗时作为原始基线，之后按N0收敛目标再改策略。make-before-break同prefix、无备路，只是一项；prefix/IPv4映射/两端与peer-relay移动、断网缺口、活备用、真手机仍是独立要求。scenario在resident ready后总240秒上限只作实验资源边界，不扩大每请求30秒或原已有门。


### U12对端先发业务：ipv6_node_overlap_remote_first_v1（新增，运行前冻结）

沿用上述同prefix、新旧重叠后删旧、IPv4双向实际隔离、setup/Direct连续10s、before/after各方向n30、每请求30s及scenario240s资源界限。区别只在删旧后：B→A首请求立即发起；其finished（包括失败）记录后，才发A→B首请求，留first_remote_finished phase。A原有后台同步、keepalive与网络变化处理保留，因此是“无移动端前台业务帮助”，不是静默网络。首请求失败不得被后续A发包恢复抹去；全部124逻辑请求outcomes，kernel/nft/最后ping正负控与分方向首样本保留。

两病例各fresh Lab、居民与身份，按原单线程串行运行；pair profile列明两个scenario revision，原v1数值/门不改。不把一次首样本当尾延迟分布，也不把同pair中的两个不同场景当baseline/candidate策略因果比较。217首轮raw已校准124/124ok，换址首AB509570us/BA1703454us；其Node addrs/network事件晚于两首业务完成，不归功于接口通知修复。


### U12真实接口换网：node_replug_triad_v1（新增，运行前冻结）

复用原uplink::e已使用的patchbay eth0.replug，真实删除旧veth/接入另一router并设置新地址/路由，非只改连接摘要。每病例fresh真实Node/身份/IPC/唯一目录marker；三病例单线程串行：

- node_v4_nat_replug_v1：Home IPv4 NAT→Strict IPv4 NAT，IPv4地址及另一NAT出口映射变化，HTTP Relay仍可用。
- node_dual_stack_replug_v1：Campus双栈→Cellular双栈，IPv4/NAT出口及IPv6 /64前缀一起变化，HTTP Relay仍可用；不能把双族同时变化归因单一族。
- node_v4_to_v6_replug_v1：Home IPv4-only→Public IPv6-only，删除IPv4并获得IPv6，原唯一IPv4 HTTP Relay换网后无法承载；远端始终Public双栈。

mesh后真实双向setup请求，reported双向可用/rtt连续10s（不要求Direct，不能冒称最快路或固定承载），绝对准备120s；before双向各n30/200ms/30s caller预算。开始/完成replug留phase，完成后立即只发B→A首业务（禁止先主动probe/目录/诊断），finished后再A→B首业务，再双向各n30。A后台流量保留。全部started/finished/error/timeout/cancel，kernel before/after IPv4/IPv6、两NAT router前后真实nft、完整journal及first/warm分组留证；总240s实验资源门不扩大caller预算。首错方向可停，但n不足或任一失败不绿，first失败不因后面恢复消失。

独立固定本地UDP端口的reflector在before负载前/全部after负载后验证NAT出口IP确实变更；后探针不帮助首业务恢复。它不是Khor QUIC socket的映射，不能据此声称Khor端口本身已经量测。IPv6-only病例后无IPv4探针，以真实kernel移除v4及新增v6核几何。此批校准完整可达和记录；不新造恢复SLO/同wire保证，无长流、peer-relay移动、真实手机或能耗结论。


### N1冷peer-relay候选排序合同（运行前冻结）

对生产middles_with_cost，以原本已经符合资格的三条非recursive候选、固定完整两腿RTT比较：Relay2ms＋SSH3ms，Direct100ms＋SSH20ms，Direct未知＋SSH1ms。Measured正控应按5/120/65536ms排序，Dial同样必须先5ms；未知不得因Direct越过已测快路。原资格、最多两个middle、合计3s、trust/paused/only_mine/SSH优先承载、选择收益/确认不改。本合同只证明冷候选尝试顺序不被Direct硬优先覆盖，不证明各路roadbook新鲜、额外候选资格、并发拨号/首响应/吞吐最优。先跑旧生产实际红，再最小排序修正，同整fixture复验。
