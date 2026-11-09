# feat/net 当前状态与恢复入口

更新：2026-11-09 04:06 UTC；root串行，无子智能体。活动目标“完成计划”仍active。
完整范围：[PLAN](PLAN.md)；约束：[REQUIREMENTS](REQUIREMENTS.md)；准确源/历史结果：[RUNS](RUNS.md)。
此前STATE原样保存在[本轮历史](HISTORY-20261109-before-duplex-candidate.md)，其中pending/未执行均为当时记录；当前以本页为准。

## 授权与工作区

速度/体验优先，个人可信设备网；活动目标授权N0–N5实施验证，取代最初仅写计划。没有合main、发布、中继采购/部署授权。
本机source-only：不运行Cargo/Rust/Node/Xcode编译、测试、formatter或下载可执行物；小型Git/Python/脚本检查可用。重型沿macserver/GitHub，不碰现装App、生产store、个人会话或其他任务树。
图谱先用，覆盖不足才定向读；不新建CodeGraph索引。外盘布局和治理入口遵守仓库AGENTS，本网络任务不接管治理。

| 用途 | 位置 / 分支 |
| --- | --- |
| 产品 | `/Volumes/disk0/Khor/worktrees/current/khor-net`，`feat/net`；恢复核实际HEAD/diff，本轮最后提交将含N3独立方向候选；N2生产为802e13a8+远端格式修正 |
| 控制 | `/Volumes/disk0/Khor/worktrees/current/khor-build-net`，`feat/net`；0cff9671fb2f80945423132e0df46995f6ff3198已push；历史run各绑定原control |
| 基线 | `/Volumes/disk0/Khor/worktrees/current/khor-net-baseline`，detached6485cb83d3b1f0bb93d4633825dc956623d78af9；仅paths/hop/lab/nat.rs、dev.py/netlab.sh六fixture；生产原样，现场已复核 |
| 证据 | `/Volumes/disk0/Khor/evidence/net/`；source-refs.json现105条不可变refs，全部已终态；下一候选尚待调度 |

## 当前工作：N3模型校准候选待绿

**42559476637实际模型红**：base6880d251edd38218007da99c9d070df2dd1957a5 clean、tree751e95115a7fb3a17faca1a5a44129e00a27655c、snapshote851d256586088d589971c2c1cb60f1461233a0c、controle7258e57155da1618f0f3b00fd1e395f481b5f19。
根格式与congestion目标编译通过，真实UDP校准0pass/1fail：双向各1000B/s、RTT20ms、0损失、队列1秒，正向两个500B，第一包到后反向1B被队头未来包挡住，超过250ms容差。两ignored过滤，零计时轮；完整v2记录器已编译但尚未测量。证据mac-road-model-typed-42559476637已核source/control并下载。
前42559155803仅新fixture误用u8编译失败，修b"e"后才实际取红，不能混算。

ACCEPTANCE在修改模型前冻结校准协议。当前候选仅改测试道路：每方向一个FIFO派发器，保留每lane序列化/损失/队列/seed，移除跨方向未来due阻塞；标识`independent_fifo_v2`。记录schema仍v2，case/summary/sample均标road_model。
产品CC/优先级没有调整。control同步该model字段；先要求唯一校准1pass/0fail，再跑三轮ignored完整采样。driver本机桩检查通过：校准红即止，首计时轮红仍保留三轮及最终exit8；不是Rust实测。
**下一步准确源Mac net-congestion：先候选根格式/编译/校准，再三轮计时；全部结果仍待。** 若format/compile红先按真实类型处理，不冒充模型红。旧shared_fifo_v1结果不能与新模型差值算产品改进。

## 已取得的本轮真实回归证据

| 边界 | 实际红 | 实际绿 / 范围 |
| --- | --- | --- |
| N1 Iroh37：无当前copy近期接收提示时允许故障后备用ACK快速Go | 42551075037：根格式437Noq绿，路径35pass1fail仅新快速Go用例 | 42555870179：根格式438Noq/36路径/9选家全绿；同时Noq2 reporting始终启用恢复测试通过；Mac26.6.2 arm64、Rust1.95/jobs2 |
| N2 normal拨号锁调用期限 | 42556768499：根格式/net66+1ignored/hop9/wires10/journal28绿，dial_budget1pass1fail；40ms调用被锁挡至500ms防挂 | 42557583517：同既有子集及dial_budget2全绿；CallerExpired、owner结果不变、零peer down/backoff/失败，过期空锁不拨号为新增绿覆盖 |
| N2 early-data拨号锁调用期限 | 42557961726：normal控制绿，仅early持锁40ms不返回至500ms防挂；dial_budget2pass1fail | 42558820303：根格式/net66+1ignored/hop9/wires10/journal28/dial_budget3全绿；含两入口与过期空锁；treef3b250de87c64b452ccf2dac6543eb63d677938a、snapshot27bfd24c3233e98ab8998f8056bbba3f8650e5c4、controle7258e5 |

N2新增CallerExpired与peer TimedOut区分；by入门过期、取锁等待和锁就绪后都检查，None保持原来等待。到期不写共享失败缓存/peer失败/退避。fixture为scratch hosts=false Node及无relay/discovery的Minimal Endpoint，绑定在计时前，不碰个人网络。
42556474188和42558490443分别只有新测试assert的格式门失败；均原样应用远端format.patch后复验，不是行为红。本机未运行formatter。

37生产最初72b1d891：当前tuple最近原250ms tick内且晚于故障began的live copy收包仅作为复测提示，不证明正向健康；有提示保留36的决定后备用应答/当前坏copy探测，无提示接受began后的已有备用应答。旧包/4秒兜底/正常两样本/速度优先/原节奏保留，无应用请求重放。
以上都是Mac单元，不能收Linux SLO或完成整计划。

## Linux原红与已有稳定性证据

下列同basea087 clean、treec807d328e78bdfda1dbebd7bdd4c14b0eaa8cd25、controlc68b644，普通日志关闭：

| Run | 结果 |
| --- | --- |
| 42542824071 netlab-stalls | SUCCESS2/2；p1s600秒/20次300ms忙CPU，alpha/beta0/0移动、零退役/打洞，最慢865.4ms、3019包312293字节；p5原150/300ms门绿、600ms仅观察 |
| 42542984111 netlab-dynamic | FAILURE25/27；p3三轮1.1/1.2秒/818.4ms，前两轮超原1秒门；p2及21相位绿，零错/退役 |
| 42543254686 netlab-hop | FAILURE8/9；f4s首轮0.7秒回carried、10.9秒后退Relay、28.8秒才再回，最终LinkRelay；另两轮0.4/0.6秒无回退，f4/f4i均绿 |

f4s中继2.5秒双向注入在断腿恢复1.5秒后清除，后续整段不能当5秒Relay RTT。回退独立定因，不宣称37已修复。
42542228149诊断2/2不是SLO；旧Watch较快但CPU3/1与hop1513ms红仍留RUNS，不混源/机器算百分比。恢复Linux后原p3三轮<1秒、CPU0/0、hop无回退，再准确源完整矩阵，门不放宽。

## 执行通道与限制

托管Linux prepare因GitHub账单/额度拒绝：42545673450两attempt及42545712302无步骤/无产品测试/source artifact；原始拒绝已保存。已异步询问额度或已有Linux通道，无回复，不改账单/采购。
直接SSH macserver经原iroh-ssh路由banner超时，未改路由/信任。Mac自托管worker-smoke42549824680仅工具链探针；实际产品测试现已恢复。
42552723913两attempt在Mac→github.com:443的source/control checkout超时，无编译/测试/源artifact；执行器传输失败不是产品红。

当前API Git bundle通道已多次实用：本机上传私库dev-bundles/<request_id>，仅无父快照对象、64MiB上限；worker经API读bundle和GITHUB_SHA的可信控制文件，校Git blob/SHA256/snapshot/parentless/tree，复用原prepare和已安装build guard。Git只读本地bundle；临时repo-read token不继承到Git/prepare/构建。
沿development.yml、mac-mini-worker单槽，仅net-unit/net-node/net-vendor/net-congestion；默认standard不发新input、旧checkout保留。资源/空间/清理策略不改，无个人key/token/签名/模型凭据/生产store。产品12项、bootstrap3项、prepare1项、driver桩检查及YAML/16shell语法先前通过，后续driver改动亦已重验。

host-capabilities42555870179实际：VZ=1、16GiB/10CPU、约76.2GB空闲，仅brew已装，无Lima/QEMU/容器工具。Linux VM具体候选已写NEXT-DESIGN：项目专属Lima2.3.0/Ubuntu24.04固定checksum、2CPU/6GiB/30GiB、无home/key/agent挂载、前台hostagent归guard生命周期。**没有安装/启动VM**，不能说Linux已恢复；新后端需同配置基线重新测。

## N0–N5未完成范围与接续顺序

N0部分、N1仍红、N2部分、N3部分、N4未完成、N5部分。不能用本轮单元或测量器修复结束目标。

1. 先收当前N3独立方向候选校准/完整三轮；失败/超时保留，成功与全尝试结束分布分开、空分布null，不用少量screen样本报p99；它只是Noq模拟层。
2. N2除已绿normal/early锁准入外：hop准备、拨号自身裁剪/CallerExpired分类、普通交换/一次读重试总期限、Accepted后Aborts回答取消、长操作档位和并发/输者释放仍缺。按Op原30/300/3600真实deadline贯通，不能把长CodeServe/transfer压成28秒。hop3秒为两个middle合计；取消不计peer故障，真实RetireUnheard仍保留。七类合同在ACCEPTANCE，具体调用链NEXT-DESIGN。
3. N1先恢复Linux能力，独立定因f4s再原门复验与最终完整矩阵。Linux新VM尚是设计，不跳过能力预检/基线。
4. N3真实Node32席位/content独立32预算、Custom1024帧/256KiB外层队列、SSH/hop排队/公平、共享vs分连接、1/3/5设备后台与手机能耗仍缺。不能把增加总资源当调度胜出或Noq模拟当Node SLO。
5. N4默认北京HTTP/TLS同机、自家登记集中北京，香港默认无独立pkarr；已有多登记/选家/继任机制须复用，尊重only_mine和用户n0配置。两端全路/不对称/冷暖发现/故障域/容量实测与拓扑设计仍缺，不采购/部署。
6. N5共48初步补丁（Iroh37+Noq11）：Noq6计时覆盖42543928044、Noq4备用ACK隔离42539256792和本轮Noq2/37绿保留。Noq1/5专项、其余边界、固定1.4.0逐行为等价仍缺；UPSTREAM静态hash证据不当上游运行，不升级/删补丁。原生Mac/代理/真实手机/多机/能耗及最终可信Quality仍缺。

## 证据与恢复

先核worktree/branch/HEAD/diff、当前run与准确回执，不按摘要覆盖未提交内容，不重发活动run。source-refs.json现105项终态；下一调度后立刻更新ref/STATE。
git rev-parse refs/khor/net-evidence/<run>:source必须等于回执tree；只git show读容器，不checkout覆盖或push本地证据refs。账单/checkout失败均tests_executed=false，校验成功但format/compile失败与真实行为红分开。
Mac源/bundle分支只在终态、证据已留、所有权/SHA匹配后清理；复用源的请求不拥有原源分支。原attempt记录不覆盖。
外盘缺失只读git show feat/net:docs/net-optimization/STATE.md和PLAN；原外盘未提交内容不能按摘要覆盖。其他治理继续中央STATE/RESUME，不混授权。
