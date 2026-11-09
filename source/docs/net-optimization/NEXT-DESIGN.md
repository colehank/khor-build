# 下一批网络设计与证据计划

2026-11-09 · 基于33202c22只读复核；没有执行N2–N4新实验或修改这些生产机制。
活动目标/执行器状态只查[STATE](STATE.md)，完整范围见[PLAN](PLAN.md)，验收协议见[ACCEPTANCE](ACCEPTANCE.md)。
本次八个源文件blob收据在`evidence/net/design-source-audit-20261109/inputs.json`；下面区分源码事实、设计提议和待测项。

## N1当前边界

36普通p1s已0/0移动，但p3两轮1.1/1.2秒超原1秒；f4s一轮回carried后又退Relay。
它们是独立未解决项，不由单元或日志模式绿覆盖。
37目前只传入未使用的当前copy收包提示和新增预期红回归；GitHub账单拒绝任务，尚未实际取红，未实施条件分支。

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

执行器账单问题没有改变PLAN范围；N2–N4目前是设计/源码事实，不是已实施或已通过。
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
- Ubuntu24.04 arm64固定release-20261017官方cloud image，SHA256 50e24e1fb79e89116e939d90aa7738f7af48c442fa1bc2510131e0b39c50348e；不退回无digest的latest。2vCPU/6GiB/30GiB磁盘为候选上限，现有35/20GiB空间门和单槽不改。先证明namespace/veth/netem/nft/conntrack能力，再编译。
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
