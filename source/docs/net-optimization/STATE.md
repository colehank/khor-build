# feat/net 当前状态与恢复入口

更新：2026-11-09 06:19 UTC；root串行，无子智能体。活动目标“完成计划”仍active。
完整范围：[PLAN](PLAN.md)；约束：[REQUIREMENTS](REQUIREMENTS.md)；运行记录：[RUNS](RUNS.md)；原始结果：[REPORT](REPORT.md)。
此前STATE原样归档[Accepted轮历史](HISTORY-20261109-before-accepted-green.md)与[本轮历史](HISTORY-20261109-before-probe-green.md)；历史在途/候选文字仅代表当时状态，恢复以本页和实际工作区为准。

## 授权与工作区

速度/体验优先，个人可信设备网；活动目标授权N0–N5实施验证。没有合main、发布、中继采购/部署授权。
本机source-only：不运行Cargo/Rust/Node/Xcode编译、测试、formatter或下载可执行物；Git/Python stdlib/脚本检查可用。重型沿macserver/GitHub，不碰现装App、生产store、个人会话或其他任务树。
图谱先用，覆盖不足才定向读；不新建CodeGraph索引。外盘布局/治理入口遵守AGENTS，本任务不接管代码治理。

| 用途 | 位置 / 当前基准 |
| --- | --- |
| 产品 | `/Volumes/disk0/Khor/worktrees/current/khor-net`，`feat/net`；最新已验证生产d4a2e115b898fc337f5091b51fa71e45740c667f；随后只有证据/计划文档提交，恢复核HEAD/diff |
| 控制 | `/Volumes/disk0/Khor/worktrees/current/khor-build-net`，`feat/net`，0cff9671fb2f80945423132e0df46995f6ff3198已push，现场clean |
| 基线 | `/Volumes/disk0/Khor/worktrees/current/khor-net-baseline`，detached6485cb83d3b1f0bb93d4633825dc956623d78af9；仅paths/hop/lab/nat.rs、dev.py/netlab.sh六fixture，生产原样 |
| 证据 | `/Volumes/disk0/Khor/evidence/net/`；source-refs.json现121条不可变refs，全部终态；本轮121个REF:source逐项复核一致，无活动run |

## N2本轮：普通wire覆盖与配对探针三红→绿

**42571393016 Mac net-node SUCCESS**：based4a2e115b898fc337f5091b51fa71e45740c667f clean、treef278b9330f51268182b142729db2c86687f4af4b、snapshot3459894c81cb994310a97bfcba4ca39ae97e569b、control0cff9671fb2f80945423132e0df46995f6ff3198。
mac-probe-owner-candidate-42571393016保存source/environment/raw/classification；根格式、net66pass+1ignored、hop9/wires10/journal28/dial_budget14全部绿（预算/流/探针9.93秒）。仅Mac便携子集，不是完整Quality或N2全包。

| 边界 | 实际红 | 实际绿 / 限制 |
| --- | --- | --- |
| normal/early拨号锁期限 | 42556768499 / 42557961726 | 42557583517 / 42558820303及最新；40ms到期不改共享结果/peer down/backoff，过期空锁不拨号 |
| 操作scope到真实ask | 42562932053两红：忽略40ms、300s被28s截断 | 42566727617首次绿，最新保持；只是持锁边界，不代替长工作/全Op |
| Accepted回答所有权/期限 | 42567527752两红 | 42567946141及最新；取消/到期STOP本流、同wire独立Sessions、相同stable_id/严格3请求/零dial_failed |
| 普通交换/一次重试/写失答 | 新增覆盖，未声称旧源行为红 | 42569954515四项首次绿，最新保持；1秒普通exchange到期不伤并发请求；2秒原by剩余700ms进入真实读重试并到期；失答Sessions严格2次、Mkdir严格1次 |
| probe直接取消、配对外层取消、首成功 | 42570772550实际11pass3fail | 42571393016全部绿；真实HTTP GET和fast已204前置后取消，800ms内TCP EOF，第二URL不被slow first挡住 |

resident入口一次建立原Op档5/30/300/3600秒减.5秒的不可变Tokio by，任务局部只承载值；锁/hop/有界拨号/延迟候选/普通交换/一次读重试显式沿用较早截止点。CallerExpired不记peer失败/退避。
独立ask无Op时由Request工作预算加现有dial25s/hop3s/最大road120s合成外层上限，各阶段仍可先结束。更新dial保留原10秒自身上限；已编译但专门运行覆盖仍缺。
Accepted交接保留Aborts，adopt后按原by等答；取消/到期停止本流、保留共享健康conn。Rejected和未adopt新conn清理、读写/0RTT准入/frame/IPC半关闭未改。
新普通wire控制包含真正同时等待的第二请求，首流STOP后才回答；零读重试/answer_late/peer failure/down/backoff/未答退役。真实retry仅一次closed退役，第二条仍live。Mkdir为代表写请求覆盖，未宣称穷尽所有动作的幂等专项。
probe内部JoinSet按完成取首成功并取消输者，pair_with用Aborts闭合外层；dial成功先drop probe再pair_over，所有早返也清理。原并发量/2.6秒单HTTP/proxy/空与全失败false/诊断fallback不变。图谱还确认ears_heard直接await该函数，返回值映射未改，不能据此声称其完整故障/回声场景通过。
RetireUnheard仍要求armed、heard_count不变、elapsed>=Patience.ack；短取消不退役，真正长黑洞保护不能移除，其真实故障覆盖仍缺。

Accepted测试证明应用的真实Accepted后等待路径；Noq is_0rtt只是应用accept时握手状态的观察，不是包级重放证明。旧前置42566727617失败与格式/编译红均独立分类，不能混算。117th run42569493787、119th run42570381896仅格式门，patch原样应用后继续；本机未运行Rust工具。

## N1仍有Linux红，不能用Mac单元收口

Iroh37/Noq2已由42555870179实际绿：base d2849d4732c7e9b6dd2e953616ce583f0e520391、tree4a9c4d89cc01e33a480375d50cc9f77b6d2d7a83、snapshotb6a4e9240786b1b178aee1e11284e6164aecc377、controldd57be6c57090a811726983e700d0b39a40fd1e8；根格式438Noq/36路径/9home。37当前tuple近期copy接收只作复测提示，无提示允许故障后备用ACK快速Go；有提示保留决定后验证，原250ms节奏/4s兜底/正常两样本保留，无应用重放。

以下原Linux同basea087 clean、treec807d328e78bdfda1dbebd7bdd4c14b0eaa8cd25、controlc68b644，普通日志关闭：

| Run | 结果 |
| --- | --- |
| 42542824071 stalls | SUCCESS2/2；600秒/20次300ms CPU，alpha/beta0/0移动、零退役/打洞，最慢865.4ms、3019包312293B；p5原150/300ms门绿，600ms仅观察 |
| 42542984111 dynamic | FAILURE25/27；p3三轮1.1/1.2秒/818.4ms，前两轮超原1秒门；p2及21相位绿，零错/退役 |
| 42543254686 hop | FAILURE8/9；f4s首轮0.7秒回carried、10.9秒后退Relay、28.8秒再回，最终LinkRelay；另两轮0.4/0.6秒无回退，f4/f4i绿 |

f4s的Relay2.5秒双向注入在断腿恢复1.5秒后清除，后续整段不能当5秒Relay RTT。该回退未独立定因，不宣称37已解决。
42542228149诊断2/2不是SLO；旧Watch较快但CPU3/1和hop1513ms红仍留，不能跨源/机器算改进。恢复Linux后原p3三轮<1秒、CPU0/0、hop无回退，再最终准确源完整矩阵，不放宽门。

## N3测量模型已校准，真实应用验收未完成

42559476637实际UDP道路校准红：旧单FIFO把反向1B挡在未来正向包后；此前42559155803只新fixture字节类型编译失败。
仅测试模型改每方向独立FIFO，保留lane/几何/seed，road_model=independent_fifo_v2、schema v2，产品CC/优先级未改。
**42560015454 SUCCESS**：base1478bf9712ad50c00b502c78613cd932f20586b3 clean、tree33d5ef6d47cde04620db1baa470bd7fefdf0c99d、snapshot9a6f9717470e0a741aedbdd9dd1ecf0a32d18fc8、control0cff9671fb2f80945423132e0df46995f6ff3198。
root fmt/编译/校准1pass、三轮各2pass（167.64/167.09/167.29秒）；704完整samples/51摘要/15bulk、零失败/超时，每轮235/234/235，计数/索引/分位数逐项重算一致，sample-audit.json存hash/方法。
固定300msRTT/每向2%/20Mbit/200ms队列/20s bulk/2固定seed；只是Noq模型，不是Node/SSH/hop/手机SLO。bulk-echo合并p95：Cubic普通890.3ms/优先389.9ms，BBR3普通449.6ms/优先612.4ms，效果不一致，不据此改产品CC；screen每CC仅9样本，不报p99。旧shared_fifo_v1不能和新模型差值算产品改进。

## 执行通道与Linux限制

托管Linux prepare因GitHub账单/额度拒绝（42545673450两attempt、42545712302，无步骤/产品测试/source artifact）；已异步询问额度或已有Linux通道，未有回复，不改账单/采购。直接SSH macserver原iroh-ssh路由banner超时，未改路由/信任。
Mac自托管worker已实际跑产品，现API Git bundle通道多次验证：私库无父source/bundle快照、64MiB上限；worker按精确GITHUB_SHA取可信bootstrap/driver，校blob/SHA256/snapshot/tree后仅本地Git clone。临时repo-read token不继承Git/prepare/构建。
沿development.yml、mac-mini-worker单槽，仅net-unit/net-node/net-vendor/net-congestion，Rust1.95/jobs2；原安装guard/内存/空间/缓存策略未改，无个人key/token/签名/模型凭据或生产store。标准executor旧checkout/default inputs保持兼容。
host实际VZ=1、16GiB/10CPU、约76GB空闲，只有brew，无Lima/QEMU/容器；VM仍是NEXT-DESIGN的候选（固定Lima2.3.0/Ubuntu24.04 checksum，2CPU/6GiB/30GiB、无home/key/agent挂载、前台hostagent归guard）。固定Lima2.3.0原生镜像转换的源码调用链已核，缺qemu-img并非硬依赖；未安装或启动VM，不能说Linux恢复。仍须验证foreground/guard退出清理、资源/隔离配置和实际guest能力探针，且新后端同条件重跑基线/候选。

## 完整未完成范围与下一步

N0部分、N1仍红、N2部分、N3部分、N4未完成、N5部分；完整PLAN范围保留，不以本批单元绿结束目标。

1. **紧接N1 Linux能力探针**：本批普通wire/probe已绿，Linux原红仍阻断选路SLO，优先推进项目隔离的VZ开发guest。先核guard生命周期/磁盘与工具配置，再macserver实际boot/namespace/veth/netem/nft探针；不是直接跳完整矩阵或部署生产。固定版本与源码审查在NEXT-DESIGN及linux-vm-source-audit。新后端必须同配置重跑原基线和候选。
2. **N2剩余合同**：原F7真实长黑洞、长工作/更新10秒运行覆盖；hop两middle合计3秒、dial输者及延迟候选的资源/过期验证；前台多阶段及独立后台整体预算。pair/doctor探针两层已实际修复，不重复排成未修。代理listener/传输job/Visit/nudge有意独立，不能继承已结束前台by。普通wire四项与Accepted覆盖不能证明完整N2。
3. **IPC边界**：client Op外层与serve接收后deadline起点不同，尚未跨进程传递剩余预算或取消。正常exchange本来就shutdown发送半边，服务端EOF不是取消；不得为贯通而重放已发送写请求。具体原源码hash见deadline-continuation-20261109。
4. **N1**：恢复Linux，独立定位f4s回退、原门复验及准确源完整矩阵。Mac单元不能替代namespace/netem/nft实测。
5. **N3**：真实Node32席位/content独立32预算、Custom1024帧/256KiB外层队列、SSH/hop公平推进、共享vs分连接、1/3/5设备后台、真实手机能耗，均需独立负载数据；不增加总资源伪造调度胜出。
6. **N4**：默认北京HTTP/TLS同机、自家登记集中北京、香港默认无独立pkarr；复用已有多登记/选家/继任机制，尊重only_mine/n0配置。双端完整/不对称路径、冷暖发现、整机/登记/出口故障域及容量实测和拓扑方案未完成，不采购/部署。
7. **N5**：48初步补丁（Iroh37+Noq11）仍需覆盖收口；Noq6计时42543928044、Noq4备用ACK隔离42539256792、Noq2/37本轮前绿保留。Noq1/5专项、其余边界、固定1.4.0逐行为等价尚缺；UPSTREAM静态hash不是上游运行，不升级/删补丁。原生Mac/代理/多机/手机/能耗与最终可信Quality仍缺。

本轮用户询问进度/工期时，已给出当前N2这一批粗估1–2小时、全N0–N5粗估3–5开发日且以Linux与真机条件到位为前提；这是粗估，不是缩范围或固定完成承诺。当前十四预算/流/探针测试现已实际绿，但后续范围仍如上。

## 证据与恢复规则

先核worktree/branch/HEAD/diff、当前run与准确回执；本轮无活动run，继续上述Linux能力探针准备及N2剩余范围，不重发已终态旧run。121 refs已逐项核，source-refs.json全部pending=false。
`git rev-parse refs/khor/net-evidence/<run>:source`必须等于回执tree；只git show读容器，不checkout覆盖/push本地证据refs。失败明确分format/compile/fixture/行为/执行器；tests_executed只按实际子集，历史红不删除。
Mac source/bundle分支只在终态、证据留存且所有权/SHA匹配后清理；复用源的请求不拥有原源分支。原attempt不覆盖。
外盘缺失只读`git show feat/net:docs/net-optimization/STATE.md`与PLAN；原外盘未提交内容不能按摘要覆盖。其他治理沿中央STATE/RESUME，不混授权。


## 本轮边界与下一入口

产品已提交、当前无未验证生产候选；control仍0cff967 clean且未改。没有执行安装/启动VM或改宿主网络/现装App/guard/空间策略。对控制仓库scripts/network.sh/bootstrap-network.py已只读复核：当前Mac只有四portable task，重用Linux profile前需显式executor/任务入口与准确source/control receipt；不能把Linux命令直接跑在Mac。
Lima v2.3.0树800a96b14e96b5d2ceb20ab2602147c1e79224ea的9份源码逐blob校验，具体Linux VZ CreateDisk→driverutil.EnsureDisk→proxyimgutil.Convert在qemu-img不存在时走native；此前仅看到Mac guest native分支不能代替该Linux链证明，现已补齐。默认ssh/hostLoopback配置已读，guest配置必须显式关闭个人key/agent/home及自动端口暴露，不把hostLoopback:false误当阻断全部宿主LAN地址。foreground与真实guard进程清理尚需验证。
用户已回复确认真机组合为“iPhone＋两台处于不同网络的电脑”（PLAN U8）；已异步补问iPhone型号/iOS与两电脑OS，尚待版本细节。此前Linux额度/已有通道也无明确答复，不能仅由设备可用推定有Linux执行器。继续独立工作，整体goal active。

下一步具体入口已进一步核实：控制仓库没有AGENTS.md，图谱对scripts/network.sh未覆盖后已定向只读；worker-storage.yml的inventory虽不改policy，却也依赖Ubuntu prepare，因此不能作为当前账单条件下的Mac直达采集。可在既有network-mac只读能力记录中增加有界guard文本/hash采集，核已安装run-build/build_guard.py的进程/信号清理；不得调用ensure-budget或改installed guard。此workflow补充尚未实现/调度，control仍0cff967 clean。读取的控制文件包括development.yml、network.sh、bootstrap-network.py、worker-storage.yml及inventory分支；当前hostcap没有guard逻辑正文。用户OS细节仍待答复，Linux能力工作不等该回复。

## 新接续：Mac只读guard采集

上一goal轮为progress（普通wire四项新增绿、probe三红→绿、真机设备组合确认）。控制60821accc131e837da476706cb6b3ad93812d30e已push feat/net，只在既有network-mac hostcap增加固定两份admission脚本的有界只读文本/SHA256记录；不调用ensure-budget、不执行被读取脚本、不改installed guard/资源/缓存策略。YAML、16 Bash块和2内嵌Python语法已本机轻量检查通过；实际采集待新的Mac net-unit验证。产品生产仍d4a2e115，既有14绿的控制版本仍0cff967，不能覆盖历史回执。下一run先锚准确source/control再读artifact；VM尚未安装/启动。
