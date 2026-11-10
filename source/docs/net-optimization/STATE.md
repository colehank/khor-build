# feat/net 当前状态与恢复入口

更新：2026-11-10 14:45 UTC。**完整N0–N5仍未完成，用户已授权持续实施；不得因分项绿、旧goal paused或compact停下。** 速度/体验优先、个人可信设备网。root串行，无子智能体；没有main合并、发布、采购、生产部署或改已装App/store的授权。长期范围见[PLAN](PLAN.md)、[REQUIREMENTS](REQUIREMENTS.md)、[ACCEPTANCE](ACCEPTANCE.md)、[NEXT-DESIGN](NEXT-DESIGN.md)。上版STATE完整逐字保存在[历史](HISTORY-20261110-before-peer-contact-state.md)，含更早历史链；本页替代旧段落的当前状态，不删历史失败或矩阵。

## 恢复先核实际句柄

- 主：`/Volumes/disk0/Khor/worktrees/current/khor-net`，branch **feat/net**。最新已接受分项源码 **47aaa3b9bb16cf2c11326ed6def1d9b51437d4d7**，tree **4a4055196af13894f9a71b582285119434373c96**、snapshot **a61643ae2027086bb845bdb16d7087c4b7b0eb71**。双OS同canonical全12节点绿/hop27，加Linux原vendor439/50/9绿，主53补丁不变，未部署。当前HEAD a462只再加sole-peer fixture与U17文档，产品准入尚未修。
- **无活动heavy/Mac/phone/物理计时任务**。sole-peer旧准入fe6ce1de已真实32项29/3红：Relay第二腿、缺第二腿、历史Direct均不admit/不echo，Direct正控/recursive负控/原27全过；140.253秒/11artifact/identity核/group空。当前仅写Recovery准入候选，尚未运行：无当前target road时允许有限真实尝试，不让历史Direct、配置Relay或缺hint封死；原NoRelay/单跳/信任/paused/两middle/总by/MISSED等保持，无新增任务/定时器。并行普通拨号与epoch仍未做，不能部署此片即报最优。首02f32be2仅六处远端格式，43.707秒/10artifact/group空原件保留。U17产品准入仍未修。
- **actual-actor关闭guard已同canonical闭合本scope**：47aaa3b9/tree4a405519/snapshota61643ae/control68，Linuxb9f10cb1全12节点绿/hop27/wires15/budget30，250.769秒/21artifact/group空；Mac42833751584同source全12节点绿/hop27/wires15/budget26，exact13772163647/20files/11controlblobs核；Linuxvendor99315801同source **Noq439/Iroh50/home9** 全绿，61.826秒/14artifact/group空。两个freeze仅late-close/已入队close与无replacement控制scope candidate_verified=true，field false；pending Noq开关/Abandoned、ears/network/address epochs仍未闭合。Iroh13加法条件API不增加补丁编号，总53仍不变。
- 原actual旧8048/73ff/5e19在761c4291为hop24/1红（已offer的新carrier仍被Closed），独立旧Node原调用817a/35e9/036a在2cf6b5ef为27项25/2红（queued同样Closed=true/carried=false），actor真暂停/先enqueue后新stream与两无replacement控制均过；126.109/126.082秒/各11artifact/group空。首03a/317仅格式原件保留。baseline worktree已核clean/仅本次graph与Python ignored三文件后非force移除，base817和所有snapshot/run refs保留，queued-close-baseline-worktree-cleanup.json证明。
- incoming-only69a6/8536/bb754此前Linux4c316093与Mac42825268619同source全12节点绿/hop23，旧07e04308实际22/1（IP仍通、仅Custom无selected被误拆）与正常控制原件保留，scope freeze true/field false。
- **a35/95e/c74此前同canonical全12组绿**（Linux1952febb265.399秒/21artifact/group空、Mac42822434248 exact13766619208/20files/11controlblobs）：完整准备单by=min(原3秒,caller)、正确MISSED与已登记selected-carried保留；hop21、wires15、budget30/26。69仅再涵盖非空all-custom incoming，不改comparison/空snapshot。原offer前缀、新prepare/published/incoming whole及原hop/wires测试尾逐byte保存，freeze各scope true；后来附加子模块只能作为prefix保护，不能把旧whole hash改成新blob。原921两端原IPC32栈溢出/SIGABRT保留且不接受；a35两个cold open_hop调用Box::pin后原IPC32/remote32/STOP32/resume/canonical全绿，不改栈/fixture/时钟。
- **未覆盖**：late-close/new-offer/pending-Noq-open/ears/network/address epochs与未登记成功连接所有权，尚未并行寻路。field自动恢复仍失败/人工开关App恢复，不把当前各单元合同绿替代手机验收。所有终态source/run/control已pin并核，ledger当前无pending。
- **第四个取消所有权分项已同source双OS闭合**：c071/treeeda37c362b793ef63001fb4876be3ce4f44d58e4/snapshotdfc36e759e887a202864dd01390a0bb28bb5c694/control68，Linux147ab8a0全12绿/hop15/wires15/budget30，261.689秒/21artifact/group空；Mac42812850230全12绿/hop15/wires15/budget26，exact artifact13762956979/20files/11controlblobs核。hop-offer-fixture-freeze.json仅pre-offer取消/正常handoff/同middle新serial scope candidate_verified=true，phone false。全部本地52749等旧session已结束，无Mac重发。
- 旧行为0fb7/treef42/snapshotf954的5ea56c46真实hop14/1红（orphaned=true/stopped=false）、原12+两负向绿，134.349秒/11artifact/group空；c504首538和2b9候选1d66只格式失败原件保留。c071行为与2b9相同只远端格式变化。原一个watcher在首await前登记；准备future持Asking到offer完成再交，取消guard仅撤自己serial，watcher沿既有let_path_go清路，pending无LOOK timer、无额外cleanup task/协议/探针。
- **新prepare旧行为4例**：真实admit后保持原gate，无caller原3秒总期限/短caller更早期限应结束；正常短caller放gate可成功；关闭fixture旧cachedTunnel并实际持有新Incoming不admit，caller先到不应记MISSED/down。先红再改生产。原offer_tests.rs整个旧文件仍是当前文件**逐byte准确前缀**，只尾加prepare子模块声明；新文件独立，prefix长度/hash与新whole在hop-prepare-budget-fixture-freeze.json。**不能把旧hop-offer proof的旧完整hash改成新blob hash**，旧proof锚原source；追加合同用prefix核。原hop/wires旧tests whole与peer_contact全文件均不变。
- 计划用入口统一by=min(原3秒,caller)约束tries及post-admission gate/offer，阶段不重置；不把caller先到或已admit后的本机公布等待计为middle失败。已有PendingHop/watchers负责回收。不得简单外包timeout导致原真正失败MISSED丢失；未知同步文件IO不可声称可被async抢占。**既有check links.has→await close、新offer/pending Noq open、ears重绑代际仍未验，尚不并行普通拨号。**
- **第三Asleep分项已同canonical双OS绿**：fccc/tree714aaa9373cf3431ece85bb33bee689804fa948b/snapshot2b5af046d9c7b7f11fc95ca2f83538800355b12e/control68，Mac42806745387全12绿/budget26/wires15，Linux **61121d16全12绿/budget30/wires15**，21artifacts/group空/416.794秒，whole原及新fixture与真实Mac旧行为543/42806129212的24/2红逐byte核。peer-contact-fixture-freeze.json仅该scope candidate_verified=true，field_phone_issue_resolved=false；没有手机/生产App部署。当前无Mac或phone任务。
- **99534d56失败仍保留**：新budget30/wires15等绿，但原remoteQUIC32第16行private-node配对60秒准备超时，未到回收断言，775.052秒/18artifact/group空；STOP/resume/canonical未执行。观察过cargo DN/288秒无log后自行继续，因进程已退出无wchan，**未证明该等待导致pair超时**。611按同源/同控制/同fixture/原期限只复测一次后全绿，不关闭独立pairing稳定性问题或丢弃此失败。
- portable旧行为baseline **42806129212** source543/treee07/snapshotd16，实际budget24/2旧Asleep红、四负向/setup全绿；候选Mac **42806745387** exact artifact13757749259/20files/11controlblobs全12绿。旧测试only scratch已核clean/只有本次graph与Python缓存后非force移除；base543保留`refs/khor/net-baselines/peer-contact-portable-v1`，source snapshot/run refs仍在，cleanup证明外部peer-contact-baseline-worktree-cleanup.json。所有本地80252/99550等旧会话均已结束。
- 更早Mac42805079369 source959的4个line127反向握手setup超时原13artifact保留；fccc仅计时前关闭未用全网卡endpoint，两端都显式loopback并assert全部公布地址为回环，原5秒/行为断言不变。新旧whole修订与各次真实红/绿都保留，不将旧Linux959全12绿移花接木成当前fixture绿。
- f9退役顺序前一Mac42798075914 attempt2已全12/wires15；attempt1 bootstrap截断JSON无源/compile，原件与name误选旧artifact的逐byte核及exact artifact13754344749都保留。这不是当前活动任务。
- 隔离：`/Volumes/disk0/Khor/worktrees/current/khor-net-recovery-deadlines` detached **2ef6bee5a5aaf841af066f4edb5c894769d4539d**，treef1da6579/snapshot48e9e24b。Iroh42–44与actual carrier/connect-stage诊断，**未接受**；无本轮IPC/Asleep修改。最近物理bf6105dd已结束采证，不重刷旧profile。
- 控制：`/Volumes/disk0/Khor/worktrees/current/khor-build-net` feat/net **68cadb57a0ed72d40a4ca9a843e86b9c6797bf34**，已推；net-node12项自动覆盖wires::tests前缀与link::dial_budget子模块。physical peer-mobility仍用ef0b1ab021802a17710a6748e373ea12b87bf52d。原untracked两个__pycache__保留。

## 最新现场 steering：U17/NET-16

用户新截图（22:07、5GA）hinton UI直连29ms、MacLab/Turing/lecun等不通；认为单个可达peer应自动救通。图与SHA/字段在phone-partial-reachability-20261110-IMG_5608，右侧秒/分钟字段语义未核，不当精确故障持续时间；第二腿/策略未证，未启动phone sampler。源码已核Legs::Dial默认拒绝Direct→Relay（只有声明NoRelay/Measured才放）、要求已有第二腿道路记录，wants_hop还用历史Direct作准入。这是候选筛选缺口，不能直接证明截图命中。**优先补部分可达的有限reachability兜底及单存活middle回归，与现有健康路的性能优化分开，保留总预算/并发/信任/暂停/单跳与quiet门。** PLAN/REQUIREMENTS/ACCEPTANCE已加入U17/NET-16。queued与vendor现已闭合其scope，优先推进该场景和并行寻路；完整N0–N5范围不缩减。

## 用户最新事实与手机结论（优先于旧解释）

- **同LAN的是MacLab本地电脑与Turing**。构建macserver为另一台z13 Mac mini；macserver/Turing异网沿用用户报告，尚无独立NAT/Direct穿透证据。A=构建macserver、B=Turing、C=MacLab LAN参照、P=iPhone。120/120双机cohort实际A↔B，不把C映射成A；公共Relay本身不证明物理跨LAN。
- iPhone是用户确认的 **localhost、0.2.0**，已配对、可切Wi-Fi/蜂窝；用户曾短暂锁屏，首次基线作废后重采。**回Wi-Fi后不通MacLab但可达Turing/hinton/lecun，Relay/peer relay未及时接管；反复关闭并打开App后，某次才经Turing peer relay恢复。自动恢复失败，不能把最终人工恢复算自动收敛。** “关闭/打开”未独立证明OS强退或process restart，具体时点未知。
- 手机采样全部结束，用户可正常用手机；**不要重复问准备/保持亮屏，不再发换网指令或启动sampler**。未将候选安装手机，现场根因仍未定，不把几个单元修复当完整解决或简单归咎LAN/非服务器/中继数量。
- 新session `phone-20261110T090200Z`：Wi-Fi30calls18ok/12busy；蜂窝transition90秒10calls8ok/2busy，前3成功CLI耗时27916/29285/19093ms；回Wi-Fi39calls21ok/8busy/9paused/1cli_error。全79outcomes=47ok/22busy/9paused/1error。CLI耗时含启动/NFS/请求，**非RTT**，busy归属未证，人工reopen/前台连续性未控，不作候选性能、准确换网SLO或能耗接受。
- 证据：`manual-app-restart-erratum.json`、`summary.json`、user confirmations与正常化被动journals均在该目录。首session085800用户报告锁屏，旧cellular label未先发切网指令，不能作蜂窝实测。原v1忙错误归local的解释已留erratum，不覆写原件。
- MacLab真实peer449aed48…、resident PID87295，其他GUI进程是_host/_ghost/_cagent/_codexagent角色，不据此杀进程。Turing54df7c1e…、phone530b4d17…。现行Turing installed0.1.15，运行源未证明等于本分支。MacLab/Turing时钟bracket：Turing快18.603–18.644秒，跨host时间线须校正。当前防idle-sleep断言不证明故障时没有休眠。
- 已装CLI help/missing-target负控确认`journal --on`确实指定远端；0分钟只读返回不含用户正文。一次历史phone→MacLab日志请求仅busy，**未取得phone侧历史**。保留被动MacLabwhy=asleep/空窗作为线索，不称已证明某个竞态是唯一现场原因。

## N2当前因果修复：旧休眠状态与新连接

1. **同tick顺序已双OS闭合**：真实旧suspended wire与新live wire实际收包，85d/7da54d78运行wires12pass/1fail →3a3/1ea89122 +Mac42793316303同source全部net-node绿/wires13。tick先处理resting再back(heard)，新正向证据最后生效。wake-epoch-fixture-freeze.json仅该scope candidate_verified=true，phone未解决。
2. **持槽退役提交已双OS闭合**：真实retire在释放slots后测试gate内暂停，新wire adopt/echo后放旧retire；2e/09a93c9e实际wires14/1红 → **f9ced4ad、Linux61aa0e2b+Mac42798075914 attempt2同source全部绿/wires15**。Asleep移到retire_slot持槽锁提交，after_retirement不再迟写。retirement-epoch-fixture-freeze.json仅该scope true，phone false。原slot稳定ID/Unanswered/锁序不变。
3. **旧Asleep拨号及共享失败：portable fccc已同source双OS全12绿，仅该竞态分项闭合**。普通line_to_inner与ask_early真Incoming CONNECTION_REFUSED；owner持原dial lock在cfg(test)一次gate，排队者真实poll等待，随后同peer真实反向握手/echo/serving。旧结果再提交不得left_alone，waiter须实际重拨/echo；无新接触与只有另一peer接触四负向必须共享拒绝、保持安静。
   - 653a/82adb5aa仅远端assert格式失败，不计行为；dce/97c96cff实际预算28pass/2fail，原24+四负向绿，但scratch cleanup先关endpoint再abort出现None task panic，原件保留。
   - f475仅改先abort/join并检查task不panic，**clean old baseline0e62a55f实际28/2红、两例均旧Asleep覆盖且waiter复用/dials1，无cleanup panic**，原net/fmt/hop/wires/journal绿；14artifacts/原group空/sourcecontrolbundle核。
   - 之后0e0b仅生产：gone与per-peer正向contact版本同锁，dial开始捕获版本，Asleep按版本有条件提交/复用。11894ba7原fmt/net绿、pulse字段遗漏E0609，**编译失败不是行为结果**。959只更正pulse锁引用，959f58bb已原全12绿、预算30/wires15；后Mac428050的4反向握手setup超时促成只改fixture fccc，不改生产或期限。
   - 原whole wires SHA18acdf83…、旧dial_budget whole（仅去新mod声明）、wake/retirement/probe/wire及新peer_contact整个fixture均与clean red逐byte保持；见peer-contact-fixture-freeze/source-audit/source JSON。portable修订保留此前whole hash与实际红绿，当前whole已在旧Mac source543和新sourcefccc核验；portable Mac实际24/2红→26绿、全12绿；新Linux995仅预算30等绿、原remote32在配对准备超时，611同源原门一次复测已全12绿，原995失败保留、原因未证。新peer state是**正向接触版本，非网络/address epoch**；非Asleep失败、120/600秒安静、5–300秒backoff、caller deadline/2拨号候选/accepted写jobs/信任暂停不改，无新生产timer/probe/task。
   - Wires::back调用者经图谱确认adopt/heard/finish_probe/serving/tick；图谱flows0/未测提示存在缺口，不当作安全/覆盖证明。实际旧行为红与whole原回归为证。LOST/MISSED、地址epoch、其他late-close/failed状态竞态仍需各自证明，不偷带清空缓存或扩大根因。

## N2此前已验证的分项（非整包完成）

- **ba28暂停恢复准入**：Linux旧c2ec在canonical已pause→resume后立即Cancel被旧门拒绝。独立held-next-poll真Node合同Mac427690实际红；ba28 treecf0b29d4/snapshotf9efe7c2/control68，Mac42770379226+Linux4bcfad82全12项绿。serve初始poll初始化共享cache、准入读node.paused缓存，无热路径磁盘IO；原半秒poll/写job不改。四whole fixture保持。resume-admission-red-green.json scope true。
- **实际32STOP**：同ba28双OS32Window encoded barrier/permit0，caller abort/join，32Ok(Some(STOP))/零Close替代、同stable wire，barrier仍持有时同wire Begin/Cancel成功。quic-explicit-stop-wire-proof.json；不替代WAN取消/写jobs/能耗。
- **757e013f remoteQUIC32回复所有权**：Mac427545原5秒回收红→Mac427556+Linuxd964同tree2d218/snapshot3f0全net-node绿。仅Resolve/Window/Record响应/帧持至finalwrite，Begin/Cancel/写job/Watch保持。quic-response-owner-32-red-green.json。
- caller/本机32/process lease真实红绿与child reap已留证明；初版SIGABRT、fmt/setup错误保留，不重复全部测试除非新改动/失败需要。剩余accepted写job、两middle3秒总预算、输者/晚候选、network epoch/重启/jitter/quiet/close-offer、WAN取消未闭合。

## N1隔离与真实双机证据边界

- 主Iroh41物理首ad627双端3816/3613ms绿、repeat34871 Y7238红；更早baseline/39/40/quiet和全部失败留历史。Iroh42只候选confirmation copy沿恢复copy rank，单对quiet改善但样本/统计/能耗不足，未合主/未接受。
- Iroh43自然新RTT先清Silence导致恢复episode漏建：旧52/1→新53与Noq441/home9实际绿，物理f2b仍X7728ms红。Iroh44只在**已有pending**与原有限窗口稀疏测量，不把采样当连续确认；原54/2红→56/441/9绿，但首a4双端7664/7774ms红，从未先进入Moving，不能称修好。
- actual4 outer carrier事件仅opt-in/weak stream、原hop owner select内，无生产timer/task/强owner；connect stage同已有DEBUG target，不改await顺序/错误/请求内容。e819物理e0完整124/124且X3223/Y3227ms绿；repeat4dc **cold-start首目录10033.849ms红、未replug**。2ef诊断bf6105dd124/124但请求5bad、X5505.782/Y3496.330ms红。成功轮不是缺失分支已解决。
- bf6105 actual-carrier auditor已补全：四实际承载最晚+1274ms选IP，无lag/end；X174.60→103.17→89.55ms，日志明确在103.17重开quality窗口，此后reading quality103.17保持到+3502ms、+3752才Moving；原4秒红保留。**不是始终三点median**，重开后稀疏且从未合格所以44 sampling_only没行使。carrier-quality-restart-correlation.json锚raw/SHA；下一独立合同/有限总包成本设计，不直接加预算/删样本/缩确认/盲刷。
- 原Noq441/Iroh whole50+epoch+sampling/physical whole与3个checker SHA冻结，不放宽gain/两fresh/连续2秒/4秒/3预约250ms2秒；预约门不等于总包CPU/手机能耗门。旧2c缺RTT与cold-start仍独立未证。
- 真实双机A=构建macserver/B=Turing：757/tree2d218/snapshot3f0/controlccd，coordinator4cee410c97601afe/Mac42757732776，双向各30目录+30固定29B正文逐byte **120/120**；公共Relay101.200.80.77:3340。3/3Mac+3/3Linux stop/EOF/kill/worker0/锁释放，fresh身份与8lifetime副本清理、一次GH secret删除已直接核。仅业务/该Relay与生命周期证据，非独立跨LAN/NAT/Direct/v6/peer/mobile/吞吐/p99/能耗接受。拓扑erratum保留原历史文件名与解释。

## 全范围仍要完成（不得缩减）

| 包 | 余项 |
| --- | --- |
| N0 | 最终SLO/容差/样本量/p99充分性，同测量baseline/candidate/固定好路/全失败/负向校准；原checker与18数据正负控制保持，回溯绿不冒充prospective |
| N1 | 新测量/自然epoch合同与原两门同source至少3次；旧2c X/独立cold-start/quiet黑洞/总包CPU能耗；冷暖/多候选多middle/慢方向201/208/244/242、原hop/recovery/dynamic/stalls/full及162/182红 |
| N2 | 全prepare总3秒/调用者正确记账、accepted写job、输者/晚候选、address/network epoch/重启/jitter/quiet/close-offer、并行寻路、不重放/legacy、WAN取消 |
| N3 | Custom1024帧/256KiB外层队列、SSH/hop混合公平/共享vs分连接、1/3/5设备CPU/字节/唤醒/手机能耗；Noq704模型不替代 |
| N4 | U12 v4/v6独立/同时/切族、peer/middle移动、Mac/Turing/iPhone冷暖不对称、登记/出口/整机故障域；中继布局容量运维测量，非采购部署 |
| N5 | 主53/隔离42/43/44逐项vendor coverage/上游等价/退出、Noq1/5缺口；最终源码可信Quality/native app/代理/多机/手机能耗；证据前不升级删补丁 |

## 执行硬约束与采证

- **本机source-only**：禁Cargo/Rust/Node/Xcode产品build/test/formatter或下载执行物；Git、Python stdlib元数据/数据审计/纯stub控制合同可用。源修改在本worktree；远端重型验证，commit hook实际完成再snapshot。
- Turing owned `/tmp/khor-netlab-feat-net-1110` UID1110，单flock build.lock/Rust1.95/jobs16/nice10，50GiB准入40GiB停/90min，仅fresh user/netns，无sudo/宿主网络更改。计时实验不夹编译/其他heavy。Mac普通单元与Linux普通单元可并行，计时物理不夹Mac heavy。
- Mac既有run-build/build_guard共享单槽/jobs2/50–40GiB/100GiB target；不改installed policy/ensure-budget、生产App/config/store/resident，不绕过installed CLI自shield或复制生产keys。标准SSH BatchModeyes/ForwardAgentno/StrictHostKeyCheckingyes，无个人key/PAT/agent/签名/模型凭据给worker，不重设HOME/CODEX_HOME。
- SSH/GH超时先核receipt/run/proc/PGID/锁；已有bundle/run先recover，不盲重发。stage/launch显式完整source/control与前驱terminal采证/原group空。exec session_id不是完成，要write_stdin取实际终态。ledger禁止并发writer。artifact按attempt精确取，不覆盖原失败。
- Graph先，缺vendor覆盖才定向读；无.codegraph不自建，无子智能体授权。source dirty/diff先核，保留其他WIP/旧baseline/control；只清本次明确owner资源。
- 证据根 `/Volumes/disk0/Khor/evidence/net`，ops均该根下。`source-refs.json`每次pin；最近全审370refs/64controls/45ops；审计发现六个已采证turing row残留pending，按原identity/collection/group空仅纠正状态，before/after SHA与逐项class在source-refs-status-reconciliation-20261110.json，不改源身份/历史失败。新增538逐条pin，稳定后再全审。历史短control唯一解析erratum保留，不改原label。
- collect：control cwd运行`python3 scripts/collect-network-ssh.py /.../receipt.json`；101可为实际产品失败。终态/group空后才取报告。`ops/verify-turing-record.py FOLDER`或`verify-mac-record.py RUN_ID`只核身份，**必须另读fmt/compile/实际测试/原门**。未取得source.json不是身份已核；执行器失败与产品红分开。
- Mac提交canonical已存JSON，经`ops/submit-mac-source-ssh.py`，原可信API bundle/guard；register-mac-run pin后采attempt精确artifact，再verify。所有重型只经已授权路径，合并/发布须另行满足可信Quality与授权边界。

下一步：先核sole-peer 5个真实Node准入旧行为，取红后补缺失的有限Recovery准入、再补缓存epoch/并行寻路；原queued baseline可安全清理但base817/源/run refs必须保留。N1物理恢复/quiet红、N3共存能耗、N4真实两腿/地址族/真机、N5最终Quality全部继续。进度估计仍按原下一批4–6小时、完整计划与验收3–5天的工作量估计，不是完成保证，不能把当前单元绿当已解决手机现场。

U17只读补充：Turing一次installed CLI journal-on-hinton成功返回54条格式化记录，CLI总1629.165ms（非RTT），未保留text/raw事件，不能完整追因。只证明采样时Turing↔hinton能收发此RPC，不证明截图时刻、新Tunnel或转发准入。截图目录hinton-journal-readonly.json已记录，无phone RPC/sampler。
