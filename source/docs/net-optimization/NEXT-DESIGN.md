# 下一批网络设计与证据计划

2026-11-09 · 本文主体保留基于33202c22的历史设计与源码事实，不能当当前源码。N2期限/取消已部分实现验证、N3 Noq模型已执行；最新证据以STATE/REQUIREMENTS/REPORT为准。
活动目标/执行器状态只查[STATE](STATE.md)，完整范围见[PLAN](PLAN.md)，验收协议见[ACCEPTANCE](ACCEPTANCE.md)。
本次八个源文件blob收据在`evidence/net/design-source-audit-20261109/inputs.json`；下面区分源码事实、设计提议和待测项。

## N1当前边界

36普通p1s已0/0移动，但p3两轮1.1/1.2秒超原1秒；f4s一轮回carried后又退Relay。
它们是独立未解决项，不由单元或日志模式绿覆盖。
37现已实施并取得Mac36路径/438Noq/9home绿；Turingrecovery6/6，corrected hop仍8/9，有独立f4i回退与91.4秒慢路滞留。下述提议已实现的部分见STATE，不能重复作为待开发项。

下一修正的提议：当前路任一live copy在故障开始后且最近一个原观测tick内收到包，才启用36的双路探测与决定后备用ACK要求。
没有这条提示时，故障后已获得的备用往返可用来快速离开。收包只触发验证，不能证明正向发送可达。
不新增计时器、不改变PTO/收益/正常两样本、4秒兜底或应用重试。

远端顺序：回归实际红→最小修复单元绿→原recovery三轮→原stalls与hop→最终准确源完整矩阵。
f4s回退先在现有三端诊断中关联回退时Silence/Reading/副本/实际Go；测试会在断腿解除1.5秒后恢复Relay正常延迟，不能把整个30秒观察期当成Relay一直有5秒RTT。
恢复后仍要求30秒稳定与1500ms门，历史红不回填。若要修正实验解释，必须先明确拓扑和实际完整路径，不因某版不过删门。

## N2：总期限与取消的实施设计

### 源码事实

- `ipc::Op::deadline`按操作分30秒、300秒、3600秒等档；`run_handoff_within`当前另起该档减500ms的相对timeout。
- `ask_on`的固定28秒by现在只约束早拨和读重拨；普通首拨没传，交换又使用`Request::budget + RTT余量`。
- Request有15/25/30/60/280/3600秒不同工作预算；因此把整个ask强制28秒会破坏长操作，不能当成预算贯通的正确实现。
- `line_to_inner`等拨号锁、`hop_ahead`都先于受by约束的dial。hop现有3秒是两个middle合计预算，不能误读为每个各3秒。
- `dial_with`最多2个JoinSet任务。首试包含已知IP/Relay/Custom；第二identity尝试的1500ms不等于“中继延迟1500ms”。
- 早数据任务在握手前由Aborts拥有，Accepted后take并裸await；调用者此时取消会丢掉JoinHandle所有者。
- `is_reading`不全是纯读取：含按token去重的OpenOnce/Drop等既有准入。`rides_early`是更窄白名单，二者不得合并。
- RetireUnheard原本覆盖取消时真正无应答的坏wire，不能为了取消安全一律禁用；操作到期本身也不能证明wire坏。

### 提议的边界

在操作入口建立一次本进程单调时钟截止点，沿锁等待→hop准备→有界拨号→握手→交换→允许的一次读重试显式传递。
每段截止取操作剩余预算与该段现有安全上限中更早者；不得在第二次调用重新获得完整时长。
保留请求本身的工作预算和现有慢/传输IPC档，不新设统一28秒总上限。
现有无前台操作上下文的后台入口自己建立一次有界期限，不继承已经结束的前台请求。

跨进程不能序列化Rust Instant。首批在resident操作入口确定本地截止点并向下贯通；若以后扩展IPC预算字段，应传剩余时长并保留版本兼容及调用方原外层限制，不能伪称跨进程共享同一个Instant。
不通过一个包住所有工作的timeout来代替分类：到期要能正常返回操作期限错误并解除该次guard，不将短操作超时写成peer失败退避。

| 阶段 | 到期/取消的所有权 | 保留的共享状态 |
| --- | --- | --- |
| 等同peer/proto拨号锁 | 只释放本等待者；到期后不进入hop/dial | 正在服务别人的锁持有者、已有wire及peer失败次数 |
| hop准备 | 总期限与原3秒中较早者；不再尝试下一个middle | 已成功建立且可共享的健康middle wire；调用方到期不计作middle/peer故障 |
| 两个拨号候选 | 同一截止点；胜出后中止输者，测试临时连接最终释放 | 已adopt的胜者；不竞速应用写请求 |
| 0-RTT Accepted后等待回答 | Aborts所有者持续到回答结束；到期/取消中止本回答task与流 | 已adopt且仍健康的连接；原rides_early/is_reading门 |
| 普通交换/一次读重试 | 共享截止点；到期不启动新stream/拨号；保持原重试次数 | 其他正常请求与wire；真全路无应答仍由原独立活性证据退役 |

内部需要区分“调用方期限耗尽”和“拨号自身达到失败上限”，再分别映射用户文字、journal和退避。
禁止把前者伪装成DialFailure::TimedOut后执行Wires::failed；具体错误类型在实现时按调用面最小调整。
到期与成功同时ready时，测试明确截止语义；不承诺非实时runtime在墙钟上绝不晚醒，记录实际调度滞后。

首批回归应调用真实line_to/ask_early/exchange边界，用barrier控制等待、握手接受与回答，不只测一个remaining函数或Aborts包装器。
至少证明：锁到期零新拨号、另一个调用照常完成；早数据Accepted后取消流确实关闭但共享wire仍可请求；一次读重试不能重置期限；token写操作只生效一次。
对通用计时使用可控时钟，若现有std/tokio时钟混用不能暂停，应先统一内部边界或用事件同步，不能把本机sleep误差当成网络结论。

### 缓存、退避和恢复合并

先复用Wires/roadbook已有近期往返与失效机制，测地址/端口变化和冷启动，再决定新增缓存；身份键稳定不代表旧地址稳定。
保持现有最多两个拨号候选，先测失败提示地址对首份响应的影响，不直接调1500ms或再加并发。

随机扰动只能在一次实际失败确定时选取一次，journal和retry_at共用同一结果；不能在next_backoff这个预测读取函数中每读一次重新随机。
推荐让失败提交返回实际次数/延迟供journal记录，或者使用该次失败的唯一预留记录；先审计同peer/proto锁与其他失败更新的竞态再定接口。
睡眠/relay离开已有静默规则、前台不受后台退避拒绝的语义保留；扰动范围需设备同时恢复基线后冻结。

## N3：混合负载设计先定位四处排队

| 当前源码事实 | 要测的因果对照 | 设计决策的依据 |
| --- | --- | --- |
| Node::handle每连接32个处理席位，accept_bi后先acquire再读请求 | 长请求占满席位时，短控制/目录请求在哪一段等待 | 先分清准入排队与网络排队；仅加QUIC stream priority不能解决席位等待 |
| content Window/Record另有32个响应预算，并会提前放掉普通seat | 单独测内容读取及其他bulk，记录permit拒绝/持有时长与内存 | 不假定所有bulk都共享一种席位；不盲目再加同用途Semaphore |
| Custom承载1024帧队列、256KiB BufWriter，满则丢；write_frames尽量排空后flush | SSH/hop外层慢读时的队列深度、丢包、flush间隔与交互尾部 | 测外层按序阻塞与内部流优先级的边界，不能把1024帧当固定字节上限 |
| 现congestion测量有固定丢包seed、bulk+250ms echo、priority与idle→burst | 先按当前精确源复跑，再加真实Node/SSH/hop端到端请求 | 已接入net-congestion三轮profile和v2完整记录候选，尚未编译/执行，不是N3通过证据 |

第一轮比较固定相同路径/带宽/队列/损失、相同工作量：无bulk、原共享连接、共享连接调度/背压候选、有限分连接。
每组记录首份内容、echo所有样本/失败/取消、批量字节与完成时间、队列/permit/重传/总开销；报告p50/p95/max和样本量，样本不足不报p99。
先顺序三轮固定seed定位，再多seed评价稳定性；不把同时编译造成的争用归因网络策略。

决定优先级时先解决实际阻塞层；高优先级只能影响尚未发送的字节。分连接会改变拥塞公平与每连接32席位总量，不能把增加总资源伪装为调度胜出。
候选需要有界总并发/排队字节、取消释放、控制及时性与批量持续进展；具体权重、块大小和保留席位在baseline后冻结。
后台成本另测前台/闲置/恢复、1/3/5台个人设备，记录CPU/字节/连接/探测；手机能耗只由真实手机证据确认。

## N4：现有中继与登记的真实故障域

以下是默认配置源码，不是在线探测结果；用户自定义Relay列表须另按实际配置检查。

| 入口/服务 | 源码所属故障域 | 判断 |
| --- | --- | --- |
| 北京HTTP与北京TLS入口 | 同一机器；不同进程/端口 | 两个URL不能当两台独立中继 |
| 香港TLS入口 | 另一机器 | 可增加地域选择，但不能假定每一端都可达 |
| 默认自家pkarr登记 | 北京HTTP所配置服务；香港入口未配置pkarr | 关闭n0兜底时，默认自家登记集中北京；已有地址/已有会话可用性与冷启动发现分开测试 |
| n0预设（用户开启时） | 另一套发布/查询与备用Relay | 是受用户配置约束的兜底，不能为过门绕开“只使用自己的中继” |

已存在：所有配置登记并行发布/查询、home变化触发Wake::Home、home_first发布、同机HTTP→TLS successor、home dwell。
因此不重复新建发现/换家机制；先实测它们在整机故障、单端不可达、旧地址/重启与冷启动中是否闭环。
显式公共DNS resolver也已配置，源码承认只允许内网DNS的出口可能失败；属于受限网络场景，不能由多个公网DNS地址推定已解决。

下一实验以A/B双端为观察点，逐一切断北京整机、香港整机、A单侧出口、登记服务、仅数据面转发；区分TLS/HTTP可建与真实加密请求可完成。
每种故障同时测暖会话和清缓存冷启动，记录每端home/发布位置/查询来源/完整A→Relay→B时延/首次有效响应。
“接入第二个Relay”只有在两端能互通且发现不依赖同一个失效域时才构成替代路径；单端到Relay延迟排名不构成证明。

当前设计倾向：先让登记与Relay具备独立失效域并验证客户端实际发现/使用，再按双端数据选地域/线路。
是否给香港加独立登记、增加第三地域或第二运营商、同时维持第二家，都仍是待实证选项；不以此文授权采购或部署。
容量按实际同时中继会话/方向流量/每端服务字节、CPU/FD与排队水位建模；相同源三轮压测和真网测量后再给机器规格及余量，不编造QPS或可用性承诺。

## 验收和交接

U9已恢复Turing Linux主通道，账单不再阻断当前Linux验证；PLAN范围未变。本文历史N2源码事实需按STATE对照，已实施并部分验证；N3真实Node与N4仍待实测。
恢复后仍先完成N1必要红绿与普通复验；可并行安排独立N3 baseline，但root不派子智能体，也不让性能测量与构建争用同一执行器。
每批绑定产品tree/control/原始证据；适当短回归通过后再最终全矩阵、Mac/手机/多机与可信Quality。无合并/发布授权。


## 执行器接续

Mac便携入口已经实际执行回归，标准Linux托管lane仍因GitHub账单被拒绝。37候选当前run只查STATE，不能用Mac单位结果替代命名空间SLO。
同时只读采集Mac Hypervisor/已装工具/内存/空闲盘信息，评估在既有macserver内使用独立Linux开发VM；这是执行器候选，不是已部署或已证明可用。
[Lima VZ官方说明](https://lima-vm.io/docs/config/vmtype/vz/)支持基于Virtualization.framework的同架构Linux guest；仍须核实际宿主能力和资源。
若具备条件，先在共享build guard与现有空间门内做隔离Linux namespace/netem/nft能力探针，再考虑矩阵。VM状态/缓存归khor-builder独立目录，源/driver只读挂载、报告目录单独写，不挂个人home或传个人密钥/令牌，不改宿主网络/空间策略、不采购新机器。
基线与候选必须在同一后端/资源/协议重跑，不跨GHA Linux、Mac loopback和VM混算性能。如果无可用Linux后端，则该验收仍需外部执行条件恢复；不通过猜阈值完成计划。


## 2026-11-09执行器恢复设计候选（尚未安装/启动）

Mac实际host-capabilities见42555870179：VZ=1，16GiB、10CPU、约76.2GB空闲，未安装Lima/QEMU/容器工具。可在既有共享guard内用项目专属Lima VZ Linux，避免依赖被账单拒绝的托管prepare。此节为具体设计候选，不是已运行证据。

- 锁定Lima v2.3.0官方Darwin-arm64资产717479690，50317981字节，SHA256 b7af444bd8f91d3c307477e562f2cf3bd7595d62b9429185e4fa841563578cfc；官方release元数据发布日期2026-11-06。工具仅装khor-builder/netlab项目子目录，不用brew全局安装/不改launchd。
- Ubuntu24.04 arm64固定release-20261017官方cloud image，SHA256 50e24e1fb79e89116e939d90aa7738f7af48c442fa1bc2510131e0b39c50348e；不退回无digest的latest。2vCPU/6GiB/30GiB磁盘为候选上限，以42573745492实际50/40GiB空间门和单槽为准，旧35/20不再适用且不改installed policy。先证明namespace/veth/netem/nft/conntrack能力，再编译。
- 显式无host home挂载/无个人公钥载入/无agent或X11转发/无任意端口自动转发；只拷本run已校验source.git、可信driver与返回报告，guest只用项目生成控制key，不携GitHub token/个人SSH key。guest Cargo/registry缓存独立于Mac和可信发行。
- VM生命周期必须由guard持有。官方v2.3.0 start.go及pkg/instance/start.go确认--foreground会以前台SysProcAttr执行hostagent；不能用脱离guard的后台默认start。实现前核foreground/信号关闭链路，trap正常退出/取消停止该实例，最终报告确认不残留；不影响他人VM/worker。
- 首次先工具/镜像checksum+启动/停止+能力探针，不称产品通过；候选与6485基线使用同一VM配置/后端重新测，不能把旧x86托管结果与新arm64差值当策略收益。

依据：[Lima安装](https://lima-vm.io/docs/installation/)、[VZ](https://lima-vm.io/docs/config/vmtype/vz/)、[默认home挂载及禁用](https://lima-vm.io/docs/usage/)、[固定2.3.0 Ubuntu24.04模板](https://github.com/lima-vm/lima/blob/v2.3.0/templates/_images/ubuntu-24.04.yaml)、[前台hostagent实现](https://github.com/lima-vm/lima/blob/v2.3.0/pkg/instance/start.go)。


N3在首个真实测量前新增模型校准审查：congestion.rs双向Lane共用单个按入队FIFO sleep_until的发送任务，存在跨方向队头阻塞，不能代表独立全双工调度。ACCEPTANCE已在改派发器前冻结20ms/1000Bps/两个500B包+反向1B的因果回归及模型版本要求；尚未写测试/修模型或取得运行证据。v2记录器与该物理模型问题分开，不能凭记录完整认定模型可信。


N3模型校准42559476637已实际取红（类型笔误轮42559155803仅编译失败，不能混算），v2记录器已编译但零计时轮。候选独立方向FIFO保留各Lane几何/seed，显式independent_fifo_v2；schema仍v2，所有采样记录带model。产品CC不动，下一准确源先校准再三轮全量测量，不能跨物理模型算产品改进。


## N2接续核查：IPC边界与取消（2026-11-09）

准确源hash/摘录存evidence/net/deadline-continuation-20261109/inputs.json：ipc::exchange明确write_all后shutdown发送端，再read_to_end回复；Node::handle_handoff读取请求至EOF后才run_handoff_within。正常请求本来就半关闭，不能把服务端read EOF当用户取消，否则正常操作全部被撤销。要传播调用方取消，先定义和验证独立协议/所有权信号；不得静默改变半关闭协议或重发已发送操作。

客户端call_refreshing现以Op::deadline包住连接/刷新/交换，serve run_handoff_within在请求完整解码后重新取该档位-.5s。只有拒绝TCP连接可刷新重试，连接后一律不因失答/写失败重放。后续贯通需明确caller/serve各自起点和剩余预算，而不是在每层重新开始同一duration；长档仍保留。

RetireUnheard已经同时要求armed、heard_count自请求开始不变、elapsed>=Patience::of(conn).ack，短取消不构成退役证据；不是“所有取消都退役”。后续普通交换的deadline/取消测试应同时证明短取消保留wire/其他并发请求、真实足够久全无收包仍退役。不要以caller到期不是peer失败为由移除F7保护。此核查仅静态，无新端到端运行或期限总链路绿。


N3模型校准与v2首批三轮已由42560015454实际通过：704全量请求0失败/超时，摘要与raw逐项一致。当前固定Noq模型的priority效果不一致，先以REPORT基线定位真实Node32席位/content预算和外层流排队；不直接上产品CC/分连接或新权重。N2接续继续遵守上文半关闭/取消/RetireUnheard合同。


## N2本轮实现选择：操作期限载体与真实ask回归

为避免在百余操作分支重复生成duration，resident入口用任务局部只承载一次建立的不可变Tokio Instant；网络阶段仍显式传递绝对by，并与自身上限取早。它不是Node全局可变状态；并发操作互不覆盖。跨spawn不自动继承：属于同一前台操作的子任务须显式带by，独立后台工作另立预算，不能继承已结束的前台上下文。IPC半关闭与外层Op档不变，不声称跨进程序列化Instant。

先仅添加deadline载体（current/scope，嵌套不延长父期限）和真实Node/Endpoint持锁的ask_on回归；网络消费者/Op入口暂不接入，原28秒行为原样，以取得实际红。测试绑定后才暂停Tokio时间：40ms操作/推进50ms必须完成，300秒操作/推进29秒必须仍可等锁；用5ms观察窗口驱动ready任务后清理。Tokio test-util仅dev feature，无新crate/本机构建。测试覆盖载体到真实ask边界，不把载体存在等于全链路完成。红后再接入口和各阶段，长工作预算与七类合同保留。


本轮scope双回归已由42562932053实际取红。候选连接Op入口的原档-.5s截止点、锁/hop/拨号/普通交换重试；网络阶段显式取早，拨号保留独立安全上限并区分CallerExpired。独立ask无Op时以Request自身工作预算加现有dial25s/hop3s/最大road120s组成外层上限，各阶段自己的上限仍可先结束；原套IPC档位提议经复核撤回，以免压缩工作加路程预算。实际Op保留其原档截止点。更新拨号另保留原10秒上限，不把它误归为CallerExpired。退役仍凭原RetireUnheard证据；IPC半关闭/准入未改。该候选尚未编译/验证，Accepted后回答所有权故意保持原状，下一独立真实0RTT测试先取红后修，前台spawn/后台入口继承审计继续。

## N2跨spawn所有权初审（本轮静态，未作通过证明）

图谱先查run_handoff_within/ask_early；doctor::any_relay_answers不在当前图谱，随后定向源码读取。此次只覆盖link.rs显式spawn及配对探针，不代表全仓异步任务审计。

| 任务 | 目前归属与接续 |
| --- | --- |
| ask_early回答 | 属于正在等待的前台ask；握手Accepted后原take裸handle会丢失abort owner，真实回归先取红。回答显式沿用by，不依赖task-local自动继承 |
| dial_with两候选 | 已是JoinSet，父未来退出会abort输者；候选显式by、延迟候选在开始connect前检查；尚需真实资源释放与不额外拨号的验证 |
| pair_with中继探针 | 父任务手动abort只覆盖正常分支；外层取消会drop裸handle。any_relay_answers还创建Vec裸JoinHandle，成功/取消也可能留下其它HTTP探针。需验证并贯穿取消所有权，不能只包最外层就称解决 |
| reach/code/borrow代理 | 已建立的常驻listener归Node映射/HostedBorrow；前台操作结束后应继续服务，不继承已结束by。创建阶段/中途取消的资源释放仍需测 |
| nudge/Pump/Visit/配对后visit | 独立同步工作，现有频率/合并机制不改；不自动继承前台by。后台多阶段的整体预算与风暴测试仍缺 |
| drop_here/pending_takes | 持久任务/传输job独立于第一次查询；不能因前台完成就取消。run_drop/run_take整体预算/取消边界继续核查，不能将每次ask的新预算当完整job上限 |
| serve_tunnel/handle/serve_updates | 远端响应任务归连接/服务器生命周期，不继承本机前台Op；应保留各自服务预算、信任与0RTT准入 |

上述表把“要随调用停止”与“有意常驻”按已有产品语义确认，没有新增全局任务deadline或扩大请求重放白名单。

42567527752已取得Accepted回答取消/期限双行为红，且同wire独立请求/请求数/peer控制全部通过。最小候选以Aborts作为Accepted返回值保持任务所有者，adopt后的await显式被原by包住；该任务只做流IO、不再发起Node操作，因此由父等待的by和abort owner覆盖，毋须假设task-local继承。父未来在握手前、adopt前或回答期间被drop均保有abort owner；CallerExpired不重放/不记peer失败，已adopt健康conn保留。42567946141七测试与既有子集已实际绿，仍不能收N2全包。

## 配对探针：三红后的最小所有权候选

42570772550已实际证明直接取消、配对外层取消均留下HTTP请求；第二URL已204、首URL仍等待时整体也不及时返回。旧11预算/流控制全绿。候选用doctor JoinSet按完成取首成功，并在赢家/父drop时取消输者；pair_with再持有Aborts，闭合两层所有权，不能只修一层。诊断探针不做Node操作，因此由owner生命周期覆盖，不依赖task-local继承。原并发量、2.6秒单HTTP上限、proxy策略、全部失败/空false及pair超时措辞语义保留；不把HTTP可达当真实中继数据面性能。42571393016十四专项及既有子集已实际绿；ears_heard亦直接await该helper，boolean映射保留，未据此收echo故障场景。


## Linux执行器：镜像工具依赖的固定源审查

Lima v2.3.0 Git树800a96b14e96b5d2ceb20ab2602147c1e79224ea，9份源码已与tree逐blob/SHA256核对，保存在evidence/net/linux-vm-source-audit/verified-source-manifest.json；本机仅下载/读源，没有执行Lima/Go或安装工具。
Linux VZ [CreateDisk](https://github.com/lima-vm/lima/blob/v2.3.0/pkg/driver/vz/vz_driver_darwin.go)调用[driverutil.EnsureDisk](https://github.com/lima-vm/lima/blob/v2.3.0/pkg/driverutil/disk.go)；raw转换经[proxyimgutil](https://github.com/lima-vm/lima/blob/v2.3.0/pkg/imgutil/proxyimgutil/proxyimgutil.go)在qemu-img缺失的ErrNotFound时回退[nativeimgutil](https://github.com/lima-vm/lima/blob/v2.3.0/pkg/imgutil/nativeimgutil/nativeimgutil.go)，native也支持创建/resize。不是仅用Mac guest的native分支推断Linux；不需要为该方案先全局brew装QEMU，其他转换错误仍正常失败。该静态链不是实际镜像可启动/磁盘预算证明。
[默认配置](https://github.com/lima-vm/lima/blob/v2.3.0/templates/default.yaml)支持显式ssh.loadDotSSHPubKeys/forwardAgent/forwardX11=false及networkOpts.hostLoopback=false（限VZ默认用户网络，不能配共享user-v2）。后者只限制默认网关到宿主loopback，不阻断宿主其他LAN地址，不能误当完全网络隔离。仍需明确无个人home/key挂载、仅指定只读source/control与独立报告目录、关闭自动portForward/proxy传播、项目专属LIMA_HOME及原guard下资源/进程生命周期。
`instance/start.go`确认foreground走ForegroundSysProcAttr，但真实guard对子孙进程、SIGTERM/SIGKILL、正常退出/失败清理仍需审查并探针验证；不将该flag本身当生命周期已证明。下一步先isolated guest能力探针，再同后端基线/候选与netlab原门；现未启动VM、不宣称恢复Linux。


## 实际guard与VM生命周期约束（42573745492）

已只读取得run-build（115B，SHA83c9aa2c98dd60d9eaab670d0fbcd1ae4e20460320a963f1db7d480f4d36b7ce）和build_guard.py（2138B，SHA998f5723eb447a1634411397c18a524e5ed0d50bd6dbd81001a16626219f64eb）。实际空间门50/40GiB、Mac target100GiB；原35/20设计文字撤回，不修改实际policy。guard以新session启动命令并传递build.lock FD；中断/低空间时TERM整组，10秒后KILL。若leader已退出，stop直接返回，因此只有foreground+finally还不足以防控制器SIGKILL后的VM残留。
固定Lima非WindowsForegroundSysProcAttr为空、start_unix.go使用syscall.Exec替换为hostagent，故foreground可继承guard组。实现将用同组lease supervisor持有从guard继承的lock FD：控制器关闭lease或死亡EOF即停止自有VM，正常退出也停止并wait；所有hostagent继续同组，不启动后台默认start。实际探针需证明正常清理及受控杀死子控制器后lease清理，最终Stopped/hostagent退出；这还不是已实现或已运行证明。首capability guest不挂载任何宿主目录，不进入产品编译/个人网络；保持2CPU/6GiB/30GiB候选上限和实际50/40门，磁盘增长仍由原guard监测。


## VZ能力与生命周期已实际证明（42579909366）

controlbedd896，source tree2b5bfce2/snapshot6a046588，固定工具/镜像已实际校验。短实例名修复macOS临时socket限制；按本次IID与/run/lima-boot-done相等验完整初始化。Linux aarch64 6.8.0-147、namespace/veth/netem/nft/conntrack实际成功；正常停机与子controller真实SIGKILL后lease EOF→stop rc0→Stopped均成功、同guard PGID、无本次helper。空host mounts与无个人key/agent/proxy/自动应用端口保留，未改installed guard/策略。前两执行器红在RUNS保留。

下一具体实现为把原scripts/network.sh放入该隔离guest，沿同源码/可信control回执，固定Rust1.95四component校验和，先实际构建/冒烟；随后新后端基线和候选同条件原门。首构建不预设额外全局缓存或资源扩张，计实际磁盘/时间后再决定项目专属缓存。当前能力任务product_tests_executed=false，N1原p3/hop红和N2–N5剩余范围均未结案。


### 2026-11-09续：取消与全路径活性的未验证边界

当前RetireUnheard::drop在无收包且超过ACK floor时退役；exchange_inner在同一floor之后先发全路径PING再等grace。下一反例应在Linux独立netns建立真实wire后，对loopback注入约4.8秒往返、让一个调用4.1秒到期且另一请求同wire仍等待；验证第一调用只STOP本流，活备用在grace内回包后第二请求成功。不是仅用ACK一直活跃的长等待替代该反例，也不能直接把4.1秒无ACK当全路死亡。具体实现需统一活性证据所有者，同时保留真正长黑洞取消后的退役，避免重现F7长期重复交出死wire；不增加每请求无谓探测或异步清理泄漏。此项尚未实现/运行，不冒称行为红。


### F8若取得真正行为红，修复的所有权约束

不要仅把Drop判死阈值加一秒来追当前测试，也不能一律禁止长静默退役。共享wire的活性应有单一证据所有者：接收计数变化清除当前怀疑；quiet达到原Patience后发一次全路径probe，按该probe的grace判断；旧连接/旧probe不能退役替换后的新wire。取消本调用只STOP其流，不能跳过probe宽限判共享wire死。

可沿Wires已有Arc/Slot与resident1秒tick承载，但必须覆盖直接Node调用没有resident tick的情况；若用独立有界确认任务，应每wire合并、只持Weak、不继承已结束前台deadline，且replacement/新收包/Node退出时取消和释放，不能一请求一套无界任务或重复PING。正常exchange和取消路径应共享同一proof，裸未持有wire的配对连接保留本地边界。原F7长静默测试需要验证完成确认后的快速退役，不把“无probe证明的Drop必须同步关闭”当不可改实现细节；修改观察协议须明确记录并同时保留真黑洞和F8阳性控制。此为待F8实际结果后收敛的设计，尚未实现/预记通过。


F8现已在Turing143rd取得真正行为红（2401ms到达、4101ms过期、零回包、两请求、unanswered1、共享wire None），上述修复约束开始实施，尚未改生产。既有Wires为Arc、resident tick1秒；直接Node调用无resident tick也须完成有界确认，不能只依赖tick。建议将当前wire的probe proof/count/grace与有界任务所有权放到Slot，正常exchange和RetireUnheard委托它；任务仅持Weak<Wires>/WeakConnection，按stable_id防替换误伤，收到新包/替换/退出则清除和取消。实现时检查在自身完成路径移除JoinHandle的行为、锁顺序和无runtime的Drop，验证无额外每请求PING/无无界后台任务。


### 共享确认本批已实证，后续仍须保留的边界

150/151同源Linux21/Mac19全绿，F8不改函数，F7 v2与Weak owner/replacement通过；实现与证据见STATE。只覆盖本批直接Node及其流/确认生命周期，不能把下列复核遗漏：resident tick仍有独立silence/network条件，当全路grace比其silence门更长时需验证协调；Accepted后长无应答的取消也需独立真实覆盖。Idle、正常运行中多次quiet epoch、替换和退出后的任务/流量成本在N3计量，不用字段数量替代实际成本。


### 2026-11-09 11:30 UTC：两处剩余活性边界静态复核

准确生产源25d0c803中，Wires::tick对held slot仍直接依据quiet/silent或network change门close，未先检查/加入Slot::probe；独立确认的grace可长达30秒，而network change低RTT门可为3秒，因此必须以真实连接/网络场景验证两者不会抢先判死。这是静态风险，尚未取得行为红，也未改生产。served入站弱连接也有独立静默close，不能只检查held路径。

ask_early的Accepted分支在adopt后继续持有Aborts并等待原by，已有短取消/到期证明STOP本流和保留健康wire。该分支尚未建立RetireUnheard/Slot确认，内部spawn只以req.budget等read_to_end；长时间无收包后取消的死wire保护不能由普通exchange F7推定。下一独立回归应先到真实Accepted/adopt/响应pending，再私有netns按该UDP端口双向drop，确认heard不变/真实包命中/drop前远端open，取消后观察owner全路grace内恰一次退役，同时保留现有健康Accepted控制。不要把局部静态发现记作已重现故障或修改已在途CPU/hop快照。


### 2026-11-09 CPU对照的负载证据缺口

原p1s实际为持续CPU争用加20×300ms道路延迟尖峰，不能写成20次CPU暂停；门是每端最多一次selected move、零retired，未约束最慢按键或punch数。152/153都2/2但候选3.8秒/12punch、原版732.4ms/5punch，尚无整体加速证据。原busy_beside以pid % available_parallelism得到CPU编号（不一定是有效allowed CPU ID），忽略每次taskset status，并filter_map忽略busy contender启动失败。因此当前report只证明该fixture观测结果，没有强制亲和/2busy进程有效负载的证据。保留原运行，不改其门或回写为通过新负载合同。

最终性能复验前需新测量revision：从真实allowed CPU集合选ID；断言每个resident全部现有thread的taskset成功与实际亲和、两个busy contender存活且同CPU；观测前后留PID/亲和/状态与CPU时间证据。不得只增加打印、改变竞争总量或改道路/时长/原门；同一新fixture逐字应用原版/候选并各自锚源。先短校准实际强制成功及非法/失败配置不继续计时，再独占串行新对照。此为待实施的实验有效性修复，当前产品归因保持未证。


### 2026-11-09 F9/F10实际红后的候选

158th两项前置先过后实际红。源码显示，启动probe时slot.rx仍可能是旧采样，下一tick会将probe开始前的ACK当新回答并清task；这是控制流解释，未直接打印私有rx，不冒充运行字段观测。候选在proof开始同步slot.rx到count，不伪造heard时间；tick读到真正新count仍清proof/mark heard，否则有未完成proof时不由独立silence/change期限抢先关闭。

Accepted在adopt后持有与普通请求相同RetireUnheard，正常响应disarm；取消只触发/加入owner确认，不直接判dead。长等待期间可能收到过早期握手ACK，不能将一次历史count变化当永久健康：原ACK时间无进展或整个等待达到原silent时间后，允许用当前count做全路验证，是否退役仍由之后原grace内有无新包决定。此条件可能为仍健康的长取消增加一次验证，按wire合并有界；新增16秒健康Accepted阳性并等待grace后才发独立控制，N3实际探测开销仍需计量。

这不等于所有tick/served静默策略已统一：无pending确认时现有silence/network条件、入站served的独立close、长期后台与跨进程期限等剩余合同继续保留，不能用本批绿结束完整N2。


CPU新helper的首轮Mac163只因模块路径解析停在formatter，无编译/运行。已增加显式path，路径相对声明文件目录，依据[Rust模块路径规则](https://doc.rust-lang.org/reference/items/modules.html#the-path-attribute)。三项短校准明确ignore，单独profile加--ignored执行全部，避免full netlab并行运行短校准的busy loop污染计时场景；这不是忽略产品失败，N0校准必须独立有证据后才开始新CPU对照。


### 2026-11-09 162nd实测与168th待因果诊断

162nd第二轮X同wire139750384102992的carried(quality85.61ms)在7347ms后消失，仅relay133ms，故这部分不是journal聚合假象。Y两事件wire不同，仍不能合成同连接回退。x/y open_hop后的两次5秒检查时间与两次撤路相近；源码Wires.selected_beats_carried只挑Main再Tunnel一条，而Link/close_carried按peer共享；未知/缺carried时目前返回true。这是待证的主动清理解释，不排除stream结束或旧attachment清理。

当前只加opt-in生命周期事实：Attached.serial、Link attach/detach/IO side/current serial、outgoing comparison wire/facts、hop2look release与close request，默认不构建details、不改变政策或journal词。f4i诊断在断言前复制x/y/m serve.log，新增SSH-only profile三轮diagnostic，非SLO；168当前在跑。实际触发点/所属generation须先与Path/Retired wire对齐，再决定共享连接证据范围与cleanup所有权，不能仅把所有未知路径永久保留来骗过门。


## 178共享carrier所有权反例与候选

peer级质量/selected tuple共享，不代表每条connection都有共同selected路径。入站可能只剩Custom且watcher无selected，仍在传真实数据；178 v2已证明只看Main会错误允许释放。候选释放条件须对Main/Tunnel/served所有live消费者成立，并去重stable_id、排除closed/expired。非空无Custom集合无需carrier；空快照保守；有Custom仍用原测量gain。2×5秒节奏不动，诊断shared_comparison留全连接事实。实际红→绿、完整Hop、未知快照/多quiet epoch/资源成本与generation竞态仍须逐项验证；没有据此宣布162根因或N1/N2完成。
