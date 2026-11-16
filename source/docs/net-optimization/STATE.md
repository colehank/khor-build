# feat/net 当前状态与接续入口

更新于2026-11-17 01:05 UTC（本地2026-11-17）。用户先明确暂停等待main UI修复，随后“好了, main被更新了”恢复该集成与网络任务；问答期间没有继续构建/合并/发布。[GOAL](GOAL.md)、[PLAN](PLAN.md) N0–N5尚未整体完成。root串行，不派子智能体。本机source-only，重型工作限Turing、macserver与授权khor-build。goal服务仍显示paused是此前暂停状态，不能覆盖用户最新恢复指令；不得另建替代小目标。

**本次接续优先**：远端main已同步到 **84e75c57cf87d0dfaffda14149d0d726f15dfe9e**。比df47增加12文件：原生旧macOS透明导航材质、对应UI检查/探针与指南，以及trust::admit区分Pending和Revoked（含新增测试与中英文文案）。这12文件与当前feat/net改动无交叉，必须保留；主仓untracked lab-preview/output不动。当前只集成到开发分支，不直接发布。

**15:50 UTC最新交付要求优先**：用户“后面你会在main去ship对吗”，已答复最终从main ship：feat/net候选验收→保留main同期改动的集成→验证合并后的准确版本→按rc/签名门发布TestFlight。当前不立即合并/发版；本页以下旧“无main/发行授权”文字记录的是此前阶段，不能把它当作永久阻断或要求用户重复确认最终main ship。采购/生产Relay部署和直接替换现装App仍未包含。main目前84e75c57（已含UI/准入修复），其其他任务改动与untracked lab-preview/output不得覆盖。

## 当前运行与真实结果

当前唯一在途是Mac Busy定点诊断 **43915075545**（request26461d13c2394cc39d33858e3167e6ec，复用原control49a52，源6187d90d/treeb4a0f32d/snapshot19ef0267）；最多6次、首红即停，保留原30/60秒与立即改名断言，只加Busy来源/步骤状态日志。`E/resume-busy-observer-{source,plan,mac-progress}-v1.json`；查原run，不重复提交。此前诊断已结束并核证。feat/net已在 **20b354ad012e6c193c5aa2d436e3d6ae3e67fa2c** 合入main84e75；12个main变更文件逐字保留，未改main或发布。下一步定位Mac恢复同步的Busy/未完成路径，再收敛准确候选。

- **恢复诊断闭证**：仅test observer源b9516300（生产逐字3d），Linux `165705Z-3cd67887`原0/152.532秒，固定10次全过；Mac **43913474400** attempt1原101，固定最多6次，第1过、第2红即停，4未执行、0替换。Mac直接观测：beta已是`beta-while-alpha-was-away`，alpha仍`alpha/beta`；本轮353ms明确devices pending。原耳朵30s/同步60s/立即改名断言不变；还未证明具体Busy源。两目录`resume-partial-observer-verification-v1.json`，Mac ZIP14247534585/SHA1bb6d6f2/原11控制/原guard均核。Linux十绿不抹掉Mac失败。
- **准确3d完整Linux25组358已闭GREEN** `153547Z-876c1874`原0/944.251秒、358pass/0fail/原2ignored helper，`E/RUN/relay-home-refresh-full-node-verification-v1.json`；source/control/所有原测试名/原门/格式/owner均核。保持沿用，不因session/暂停重跑。
- **Mac43898654929已闭真实RED** 原attempt1/exit101，**547pass/1fail/原2ignored**。完整iroh211与原12网络321全部通过；原13 integration执行至resume（先前5目标全绿，resume5绿1红）。唯一失败`a_deafened_serve_hears_its_peer_again_after_resume_without_being_reopened`：新耳朵约37.9ms、同步轮159.9ms已返回，但回复明确为devices仍在传输的部分同步，立即检查未看到对端改名。后7 integration＋5 Relay目标共12个未执行；不扩大等待或删断言。`E/mac-43898654929/relay-home-refresh-mac-red-verification-v1.json`；artifact14241042904/95,708B/SHA99d29695、CRC、准确source/all11 control blobs/原guard/所有native名字与cfg排除均核。原进度pending已关闭，common ledger已追加失败，不重复整批。

以下两条为原提交信息，执行状态以上述闭证结果为准：

- Turing当前唯一在途 **turing-20261116T153547Z-876c1874**，PID1430422，revision `relay_home_refresh_full_node_v1`，复用原完整25组control `7c4146df6d1bf0ed90bea386ceb0f29eab8eb18b`，预期358pass/原2ignored helper。`E/relay-home-refresh-full-node-plan-v1.json`；结束collect＋identity后用`E/ops/verify-relay-home-refresh-full-node-v1.py RUN`核证。查原receipt，不重复启动。准确源 **3d62159eaa635f60ef7fc1078a5d809eef6bb8fd** / tree **153f05926125021ee6f09ddb1101cf492f56415b** / snapshot **cd648b7b976d040baf8b0b425f0011fa9ae70c36**，`E/relay-home-refresh-source-v1.json`。独立worktree `khor-net-relay-home-refresh-v1`，branch `fix/net-relay-home-refresh-v1`，尚未采用canonical。
- **Mac同源完整验收 GH43898654929**，job133409000622/network-mac，trusted control **c44d3ec0b78ef2d128c955deec34e4fc74255aa7** / `net-relay-home-refresh-mac-v1`，request `579d064a9ae34eca98416dadcc43dfac`。`E/relay-home-refresh-mac-{progress,plan}-v1.json`；原11控制文件仅network.sh选择组合变化，既有workflow/bootstrap/run-build与其他控制blob不改。原完整iroh库＋Mac原12网络321＋13 Node integration33＋5 Relay/登记目标7（root合计361，原root忽略3）联合验证，global separate/原显式negative override保持。首次push已完成后确认远端精确ref再单次submit，0重复。读原run，不重派。
- 新候选只改vendor socket.rs/net_report.rs及补丁说明#51：同URL实际Connected→非Connected时，现有SocketActor请求一次现有全量net_report；不新增task/timer/依赖/Endpoint/常驻家连接。沿原单pending串行合并，major/full理由不被后来的弱更新覆盖；设原reports.next_full但不谎报网络major，不重置successor cooldown或退役QUIC/重放应用请求。0生产Relay变更；需实际原场景RED→GREEN及后续Node/Mac/Quality，不能因lib211通过即采用。
- 此源独立worktree `khor-net-background-jitter-v1` / `b04e3cb3d80b8e4628e9a35ebc0b7ed98025d871` / tree `08f8d3c9af754c010402a5fbf7734935c30c59bb` / snapshot `5c1fc9ed9ddca176987bc0b7c237d97f49ca0e4e`。在I8＋early fixture上，只增加每Wires的std RandomState与稳定有界正扰动：首次5秒、睡眠300秒、上限300秒不变，连续失败2起增加0–25%，无新依赖、任务或peer表；前台与联系/换网代次不变。
- canonical已以merge **eed1ab9af01d78627b3b73dfac7aab1aa60f8ec4**采用I8/DNS/cold-hint caller删除/early fixture/jitter。相对通过完整358的b04，所有非docs文件逐字一致；不是最终canonical整tree的Quality/Mac接受。新N4测试仍在独立worktree，未采用；main未合并、未发行。

## 最新已核证结果

E=`/Volumes/disk0/Khor/evidence/net`。已闭证项不因compact/session重跑；所有原红、未执行与原门继续保留。

- **后台错峰原生RED→GREEN**：`142005Z-4e15b0ef`原101/114.097秒，原18＋journal控制通过，新增spread测试真实失败；实现后`142930Z-4cf80a91`原0/102.204秒、20/20通过，同测试/断言保持。各run的`background-jitter-{red,green}-verification-v1.json`，源/控制/清理已核。不是完整网络或物理手机接受。
- **b04完整网络＋真实集成全GREEN已闭** `143457Z-68b8fbe7`原0/**956.140秒**，原25组 **358pass/0fail/原2ignored helper**，全部测试名/原门/格式/source/control/owner核验完成。`E/RUN/background-jitter-full-regression-verification-v1.json`；global separate与原显式negative override保持，包括最新early跨族握手门、全部联系代次、共享/extra取消、IPC跨进程与真实文件/同步。不是Release性能或手机接受。
- **N4两次原拒绝已保留并闭证** `145418Z-a06cf28f`原101/34.316秒为测试SocketAddr类型缺失，0native执行；`145735Z-c6ae0446`原101/40.170秒为新增两项首次id-only lookup未找到记录，原pkarr1过/2新增失败/1ignored，0固定fault轮完成、后4组未执行。还不能称产品根因；V3只补原计划20s内的独立HTTP signed-record发布前提，不给客户端resolver预热，不改原15s lookup/2s payload/30s全故障恢复和固定3轮。V3保留首次HTTP状态，等待实际证据；0生产变化。
- **备用登记点真实3/3已闭** V3 `150311Z-bae8c5af`原101/60.013秒，原pkarr控制＋新cold场景2pass/whole1fail/原liveignored1。cold首内容88496/73980/75710μs，各3实际payload，全部登记下线后held连接仍可用、fresh客户端原8秒内失败，初始HTTP404→200直接观测确认发布准备边界。whole第1轮在原20秒门红，0完整轮、后4组未执行，`registry-fault-domain-partial-verification-v3.json`，不是整N4接受。
- **整域RED定点观测已闭** `151121Z-2b066df4`原101/56.031秒，0pass/1fail；只跑同whole，未重复cold三绿。原20秒时仍是失效旧URL，native status=false/Connection refused，存活登记服务200但body没有新家，`registry-fault-domain-observer-verification-v1.json`。当前SocketActor无home状态失败触发report分支，Client.follow_home只记drop，普通20–26秒周期/增量保留可能延迟离开坏家；正在用上述最小事件衔接做因果验证，不直接强选新家。原观察源cbc25177仅加失败后快照，未改门/产品。
- **#51完整底层＋故障域联合GREEN已闭** `152349Z-f341533a`原0/**323.328秒**：完整iroh **211pass/0fail/原ignored1**，原pkarr＋新增两场景与restart/token/jitter/stuck5目标 **7pass/0fail/原liveignored1**；合计218pass/0fail/2原忽略。`E/RUN/relay-home-refresh-native-verification-v1.json`，原测试pkarr字节与定点RED cbc完全相同、Noq源不变、原lock/所有test names/格式/控制/owner/跨graph同内容mtime刷新均核。整域三轮关停→新冷端点取得实际载荷 **497332/525153/513764μs**，未重启原端点；cold三轮76804/79650/70137μs、每轮3实际payload和全登记下线负控制都过。原20/30/15/2秒门与3轮不变。仅Endpoint组件/debug小样本，持续Node业务、发行性能、物理WAN/手机与整N4未接受。
- **74 Request旧静态审计适用性已重核** 3d的整个proto.rs SHA与I3逐字相同，51 safe-repeat/28 early许可、action限制/default deny均未变化，`E/request-replay-static-applicability-relay-home-refresh-v1.json`。这是0新增native的适用性证明，不补token family持久化或完整重放行为接受。
- **I8完整Quality** GH43873334184，准确b899669a/tree68f191b3，首个实际run/attempt1原13/13成功，包括signed SSH151.86秒。`E/net-product-quality-v8-verification-v1.json`；CLI/Node各3563文件逐项核3077 checkout＋486同run frontend生成文件、ZIP SHA/CRC/实际fresh产物均核。第一次派发push未完成而422，0run；原拒绝保留。
- **I8完整Linux原网络回归** `131052Z-bd3bf61a`原0/503.944秒，12组323pass/0fail/原ignored1。`product-v8-separate-node-verification-v1.json`。共享八STOP与独立extra关闭/handler回收/cached STOP均通过；取消旧fixture竞态定因见[CANCEL-LAYOUT](CANCEL-LAYOUT.md)，0生产取消逻辑变更。
- **I8原生Mac Lab** GH43874780032首轮4/4job，104浏览器case/0错误、98tracked＋18generated来源、实际arm64 backend包均核；`E/public-native-lab-net-product-v8-v1/lab-acquisition-verification-v1.json`。本机0产品运行，无安装替换。不是手机或整N5接受。
- **真实Node集成** `140707Z-88cbb82c`原0/467.727秒，13组33pass/0fail/原ignored1。源d53ce645只修early UDP fixture，生产逐字同I8。`node-integration-contract-verification-v1.json`。旧同族代理实际完成握手已直接证明；新跨族代理保留实际握手未完成前提、原no-write/positive-write及1＋3秒原门，不据旧RED声称安全漏洞。
- **同early源Mac补验** GH43884943661原early1/1通过7.25秒，但原workflow在artifact finalize时ETIMEDOUT失败、0可取artifact；raw job log/source tree/control/run attempt已留。`E/early-barrier-isolation-mac-progress-v1.json`已标partial，不是整workflow绿。14:29 UTC原macserver SSH alias仍banner超时。最终候选Mac回归纳入early，不能只为上传重复此旧run。
- **完整同ELF共享/分连接对照** `system-dns-coexistence-verification-v1.json`：原SCCSSC六块、12case、8640交互、12完整文件、24post-health均成功，原相对/绝对p95与文件率门过。低profile目录p95共享273082μs/分连接84206μs；高profile终端p99共享639109μs/分连接916769μs，n360描述。**保留shared默认**，separate opt-in实验保留；收益尚未证明整体稳定性。旧cold TCP准入RED和旧业务RED不抹掉。不是同期main/SSH-hop-sync/物理手机。见[COEXISTENCE-DECISION](COEXISTENCE-DECISION.md)。
- **DNS/NAT64** 原IPv6-only＋SIIT＋系统AAAA独立oracle下冷配对10秒RED；只删固定公共DNS覆盖后3次fresh配对/6双向目录GREEN。Mac原319网络合同通过，普通Release来源闭证，见[DNS-NAT64](DNS-NAT64.md)。系统DNS同步初始化的Apple边界仍在，不声称上游已彻底修复；固定DNS64不是动态CLAT/手机。
- **custom审查** iroh1.4.1/Noq1.4.0已核。60项逐项审查完成，10项专属消融反例、3观测API与其他合同边界详[PATCH-REVIEW](PATCH-REVIEW.md)。cold-hint唯一生产caller删6行已进入I8并通过原全组；vendor API本身尚留，未扩张无理由重写。
- **移动/恢复与成本既有证据** 双端dual/v4换址252读、IPv6-only15case/630读、单middle真实两腿/Release ABBAAB、旧失败缓存3红→3绿、5f恢复/路径控制已闭，详原专项文档。1/3/5设备18场54 resident interval成本已量化；worker数量screen因CPU门未过明确拒绝，未采纳生产cap。硬件唤醒与手机能耗仍缺，见[COST](COST.md)。

## 下一步与未完成范围

1. 底层3d完整iroh＋N4已闭GREEN，先闭当前准确3d的Linux358与Mac联合验收，保留所有结果与未执行组；旧b04完整358及3d底层218不因compact重复派。mac旧early上传失败记录已闭partial，不为上传单独重跑。
2. canonical生产分量已集成b04、核对仅docs差异。#51与N4测试尚未采用，先验证，再冻结最终准确源做Quality和原生Mac。main基线冻结93f6cb23，不追其他GUI任务，不碰LIVE23961。
3. 按[REQUIREMENTS](REQUIREMENTS.md)逐项对账；执行归并见[CLOSEOUT](CLOSEOUT.md)。继续N1/N2的多middle、预算/取消/重放及N3的SSH-hop-sync共存、成本，N4冷Node寻址/发布、非对称出口与容量；现有Endpoint组件合同不冒充持续Node业务或完整场景。N4原测试worktree `khor-net-registry-fault-domain-v1` / cbc25177保持观察源，不覆盖；新#51在另一worktree。原20秒publication/30秒whole/15秒lookup/2秒payload和两场景各3轮均保持；#51是原60项审查之后的新候选，不能冒称旧账已含它。
4. 物理iPhone localhost现装为**TestFlight**（用户本轮确认），旧0.2.0和先前就绪窗口不能验收新候选。候选发行必须按main ancestor rc tag与签名环境政策；不绕过流程。先把授权范围内候选做具体可评审，再请求所需main合并/rc/TestFlight授权，不能现在半途提审批。
5. 最终同源报告、设计、持续矩阵与手机换网/锁屏/能耗仍缺，goal保持active。无main合并、现装App替换、rc/发行/TestFlight上传或生产Relay采购部署授权。

## 环境与调度

Turing UID1110/GID1002，自有`/tmp/khor-netlab-feat-net-1110`，原owner/flock/build.lock；Rust1.95/jobs16/nice10，磁盘start50/stop40GiB。性能计时CPU idle≥.8/PSI≤5/无同UID Cargo，同host编译与计时串行；Mac不同host可并行。stage/launch各一次；先查receipt、execution、原owner，再collect＋identity＋实际合同核验后关闭ledger。不得覆盖原失败或替换样本。

MacLab与Turing同LAN，macserver另一网络。Mac重型工作唯一run-build槽；原SSH不通时使用已授权公开`colehank/khor-build`，不自行安装替换产品。其他代码治理环境与资源按中央STATE协调，不借用其GUI实施权限。

本页此前逐字状态保存在[HISTORY-20261116-through1421](HISTORY-20261116-through1421.md)，SHA256 `2b24e36e7093d0eae10b1c95a1fffec2e3ac71f2d6c2b9dc6c4bf5091b1856ff`。其中在途文字均为历史，以本页与实际receipt为准；全部旧红/未执行/来源继续保留。
