# feat/net 当前状态与恢复入口

更新：2026-11-09 14:43 UTC；root串行，无子智能体。活动目标“完成计划”仍active，完整N0–N5未完成。没有合main、发布、中继采购或生产部署授权。

**当前：176原版完整CPU/stalls已采证2/2；178共享载路合同取得真实policy红；修复已写、尚待新准确源Linux/Mac复验。** 当前无Turing重型任务，下一步用相同源码跑net-node，先对178原fixture红转绿，再跑准确源CPU候选/原hop矩阵；不重发旧run。恢复先核实际worktree/HEAD/diff、source/ref、receipt及/proc，不以摘要覆盖WIP。

178 Mac42634473431，baseb9e7d4a3/treef7a24347/snapshot0872b5bb/control7534：root fmt/net66+1ignored/hop9保持，wires10过1真policy红。两真实连接都echo过，outgoing选IP，incoming只有Custom且selected=false，carried-only client选Custom。不同wire IDs、sole-direct允许释放与incoming关闭允许释放两个控制先通过，最后“live incoming仍用carrier不得释放”目标失败。完整身份/transport/control/hash采核；test整段SHA `3a03d41a0cafcbf0e061e072db2a4a424d9fef876d6b99a3f60b47da05ad77b0`见`shared-owner-red178-function-sha256.json`。**这证明一处共享资源判断错误，不证明162原f4i因果已定。**

当前修复只改Wires::selected_beats_carried：检查live Main、Tunnel及served incoming，去重stable_id，closed/expired不计；任一有活跃Custom且不能证明有可替代快路就保留共享carrier。没有Custom的非空path集合不占用该carrier；空快照保守保留；全无live消费者可释放。仍用原path::selected_beats_carried的质量/gain，原2×5秒门、selector/CC不变。诊断新增shared_comparison（all_live_connections_v1，release与各wire facts）；178整fixture和五CLI timed fixtures均逐字不变。尚未远端验证，不能预记绿。

176 turing-20261109T141037Z-b813a5b3，原生产6485/tree49211b55/snapshot31db523a/control7534，1189.88秒，原stalls两门2/2。CPU64（allowed_midpoint_v1）共22条admitted/20spike/end，实际610846ms；两resident各275–280存活thread全部mask=[64]、PID/start稳定，两busy每期CPU ticks严格前进。p1s alpha1/beta0移动、零退役、最慢537.2ms、3打洞；p5三档零relay/打洞、最慢273.8/501.0ms/约1.1s。所有report/group/source/driver/bundle/hash已采核。**p1s原门仅稳定性、原代码丢弃echo error值，不能宣称完整失败分布或延迟SLO绿。** 候选完整对照仍待。

完整范围：[PLAN](PLAN.md)、[REQUIREMENTS](REQUIREMENTS.md)、[ACCEPTANCE](ACCEPTANCE.md)。原始运行：[RUNS](RUNS.md)、[REPORT](REPORT.md)。本次重写前STATE逐字保存在[历史](HISTORY-20261109-before-shared-owner-fix.md)，更早状态沿HISTORY；历史在途只代表当时，以本页及实际进程为准。

## 授权、环境与工作区

速度/体验优先，个人可信设备网。U9优先Turing Linux；U10明确两台异网电脑就是macserver（Mac）与Turing（Linux），加iPhone为P/A/B。先做macserver↔Turing实际跨网，再补手机；物理矩阵已确认，尚未执行真实跨网验收，不能拿同机namespace代替。仅手机型号/iOS/网络等余项待采，不重复问已确认设备，也不阻断服务器独立工作。

本机source-only：不运行Cargo/Rust/Node/Xcode编译、产品测试、formatter或下载可执行物。Git/Python stdlib/脚本检查可用；本机控制Python检查只用stub。重型任务远端；Turing Rust1.95/jobs16/nice10，原profile线程/重复/门；Mac沿macserver/khor-build既有guard单槽/jobs2。不改installed policy，不执行ensure-budget，不碰现装App/生产store/个人会话，不转发个人agent/凭据。

| 用途 | 位置与恢复注意 |
| --- | --- |
| 产品 | /Volumes/disk0/Khor/worktrees/current/khor-net，feat/net；核实际HEAD/diff。此前功能df592已有Linux24/Mac20；本批所有权修复尚未验 |
| 控制 | /Volumes/disk0/Khor/worktrees/current/khor-build-net，feat/net，7534d6e已push；SSH-only cpu-check/hop-lifecycle＋原profile，Python驱动2/runner4检查已绿 |
| 旧基线 | khor-net-baseline detached6485，原六fixture WIP未动；immutable source90126283/tree97040f31保留 |
| 当前CPU基线 | khor-net-baseline-cpu-v3 detached6485，仅七fixture/helper变化，staged树49211b55/source31db523a已用于176；前版186d/63043993也留ref，不覆盖旧WIP |
| 证据 | /Volumes/disk0/Khor/evidence/net；source-refs.json178个run refs，均终态/明确未启动，snapshot/tree双ref全批核；另有fixed-core源准备ref |

Turing root /tmp/khor-netlab-feat-net-1110，每run新目录/owned资源，flock单重型槽；计时场景不并跑编译/其他负载。无sudo/宿主网络改动，user/netns隔离；50GiB准入/40GiB停止/90分钟预算，原pin/hash不变。SSH BatchMode、ForwardAgent=no、StrictHostKeyChecking=yes。collector终态核原group后留完整报告；101也可能采证成功，不能把采证rc当作未下载。

## 关键证据与下一步

- N2 df592：158真实F9/F10红→160 Linux24/161 Mac20绿（相同core helpers/F8逐字），旧F8真实143红→150/151绿保留。当前所有权修复必须新准确源重验这些合同。
- N1 162原hop三轮8/9，第二轮f4i真same-wire carried撤回，X相隔7347ms、carried候选消失；Y前后wire不同。f4s旧门过但三轮最慢按键9.9秒。168 diagnostic3/3未重现，27生命周期事件无measured release/detach，不能据绿收口。
- CPU169实际Node admission ENOENT、未进600秒窗口，170从未launch已superseded；172真实短case定位消失task status，174 exit-aware修复4/4（CPU89），176固定CPU64完整原门2/2。166只是sleep-child3项校准，不回写成Node验收。
- Mac171/173/175仅format红，patch已应用；177 fmt/net66/hop9过但incoming selected-carried前置红，目标未到；178 real_connections_v2全物理前置/两清理控制先过，再policy真红。
- **API事实**：iroh peer共享selected FourTuple，各connection有不同open paths；record_selected无法在某conn找到共同tuple时该watcher selected可为空，仍可经可用Custom收发。不能将per-connection paths误说成独立selectors。v2同时证明server只Custom、client确选Custom与真实echo，且留三方rawfacts。
- 先把当前生产修复提交新source，原178 fixture保持逐字同，Linux/Mac net-node准确源复验；再准确源hop/recovery/dynamic、CPU候选与176对照。比较前核五CLI fixtures、CPU ID/allowed集合、control/线程/重复一致；同host仍需记录load，不宣称单补丁因果或整体最优。
- p1s丢弃echo error值是现有量测缺口，旧绿仅稳定性。后续补全尝试/失败/取消/超时记录并另记measurement revision，不能从旧日志伪造零失败；保留旧原门与所有失败。

## 完整未完成范围（保持N0–N5）

- **N0部分**：exit-aware真实Node短校准174已4/4，原版fixed-core176完整原门2/2，候选相同fixture完整对照仍待；原p1s遗漏echo error须补完整尝试记录。最终、最终各场景目标/容差/样本量与p99充分性仍要收口。失败/取消/超时不能从分布删除。原门改动须另记revision及理由。
- **N1仍红**：先验证178共享所有权修复并追原162撤路原因（168三轮未重现）；准确修复源重跑hop/recovery/dynamic/full，慢直连/快relay及反向、稳定小噪声、恢复后不滞留/不回退。旧recovery候选6/6、基线5/6有host load变化，不能算单补丁因果。Iroh37/Noq2旧vendor scope绿只覆盖其合同，不能覆盖当前应用表现。
- **N2部分**：df592已有Linux24/Mac20预算/流/探针/共享确认合同。仍缺长工作与多阶段原总by、两middle合计3秒、dial输者/延迟候选资源和过期、独立后台预算、近期地址/重启、退避随机扰动/事件合并。无pending proof时resident tick silence/network与served入站独立close仍要核；长期多个quiet epoch/probe开销也未测。
- **IPC边界未贯通**：client与serve的期限起点不同，剩余预算/取消未跨进程传递；正常发送半边shutdown不是取消，不能把EOF当取消，也不能重放已发送写请求。普通读最多一次重试、Mkdir失答一次、Accepted/0RTT准入与信任边界保持。
- **N3部分**：Noq独立FIFO_v2模型校准绿只提供低层数据，Cubic优先p95约890→390ms而BBR3约450→612ms，不据此改CC。真实Node32席位/content独立32预算、Custom1024帧/256KiB外层队列、SSH/hop公平、共享vs分连接、1/3/5设备后台、真实手机CPU/字节/唤醒/能耗仍需实际负载；不增加总资源伪造胜出。
- **N4未完成**：源配置显示北京HTTP/TLS同机、默认自家登记集中北京、香港无默认独立pkarr；不当实时部署事实。双端/不对称路径、冷暖发现、整机/登记/出口故障域与容量实测、替代路径及有测量支持的拓扑方案仍待。复用多登记/选家/继任，尊重only_mine/n0；不采购或生产部署。
- **N5部分**：48初步补丁（Iroh37+Noq11）仍需覆盖/保留退出条件，上游Noq1.4逐行为等价、Noq1/5专项与剩余边界缺；UPSTREAM静态hash不是运行证明。证据前不升级/删补丁。最终准确集成源可信Quality、原生Mac/代理/多机/iPhone与能耗验收仍缺，实验室绿不替代真机。

## 证据与恢复规则

最新全量审计source-refs-audit-20261109-owner-red-and-cpu-baseline.json（178 run refs）。每个refs/khor/net-evidence/<run>必须等于账本snapshot，:source等于tree，不checkout/push这些本地refs。CPU固定核fixture身份见cpu-v3-fixed-core-paired-fixture-identity.json；178 fixture基线SHA见shared-owner-red178-function-sha256.json。后续准确候选需再次核同fixture/hash。

辅助编排evidence/net/ops/submit-turing.py仅Git/Python/SSH：stage可--source复用完整snapshot、--control显式7534、--role先标baseline/candidate/diagnostic；默认control仍b998。launch必须已采证前驱，再核原group/flock/输入hash。helper旧版/SHA保留，观察超时重查同句柄不重发。当前Turing队列已清，Mac最近178终态已收，旧local exec sessions都已完成。

Mac临时source/bundle分支只在终态、完整留证、所有权/SHA一致后清理；复用者不删原请求分支。外盘缺失只读Git恢复，不按摘要覆盖WIP；其他治理任务沿中央STATE/RESUME，不接管其树。完整goal仍active；工期暂估数天、最近对用户粗估3–5开发日，取决回归与真机，不是完成承诺或缩范围。
