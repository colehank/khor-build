# 运行中直连静默与Relay存活对照 v1

用户认可后继续UPSTREAM-CONSUMERS的关键缺口。协议在运行前冻结于E/direct-fallback-design-v1.json：两臂各5case、单次固定顺序、共同夹具与原版patchbay0.7，同Relay设备双向各25ms延迟以确保初始Direct可比较。A为原版iroh1.4.1/Noq1.4.0默认；C为当前vendor加Khor公开RoadOrder/transport配置，尚不运行Node生命周期或peer relay。

准备阶段证明主连接双方已选择Direct且8字节回显完整、存在Relay路径；另用禁用IP传输的新身份连接同一目标，认证并完成真实Relay-only回显。静默故障只丢Direct UDP，保留DNS/TCP Relay，不关接口/端口/路由，也不主动调用network_change。故障后主路与Relay见证并发实际请求，同一主连接、无重放；2秒沿用原交互caller参考，逾期后仅为了诊断继续等待同一个future至30秒，不把迟到成功改成按时成功。反向控制切断Relay TCP而保持Direct，另有无故障控制。

原版与候选的编译和计时串行，只在Turing执行。source/control/依赖闭包与nft1.1.3隔离工具均须闭证；默认debug诊断不能用来宣称Release性能或完整目标完成。准备失败、原生失败及所有迟到样本完整保留。

首轮184713Z-22ffd315为编译准备失败（原101/31.97秒，A编译24.66秒），std Elapsed上下文与LinkCondition非穷尽构造用法错误，0 native/0业务样本，16报告文件及owner已闭证。仅修正夹具API调用，不改变模型/期限/断言；新源66b766c0/tree39e52802，control4d7566eb5731b85e697ae863f875e25cdf6bcdc8，原准备失败保留。

第二场185001Z-54d69c91（原1/275.09秒）两臂编译过A11.12秒/C26.17秒；首个A healthy准备等待到180秒owner，0preflight/0故障业务，余9未执行。控制器SIGKILL后5秒wait又超时，原native日志/37报告保留，原root组与额外精确owned binary进程另审均已消失。不是产品故障恢复结果。新源bf4ef784/tree610ebd48仅夹具准备：直接拨明确Relay-only地址代替等待home在线，所有可疑online/lab等待按原15秒setup预算约束、加共同诊断trace；control新增parent-death ownership并将kill后reap预算改30秒（不改业务caller）。E/direct-fallback-design-v2.json在新执行前冻结A healthy→C healthy先行门，原故障条件/2秒参考/30秒同future观察/两臂各5case保持；无故障准备或业务不通过即停止余case。Control0af43976144487a408b5ada938a79caed91096ec，实际receipt见common ledger。

第三场185943Z-68483c79为编译准备失败（原101/64.10秒），A过11.47秒、C在mdns看到相同iroh-base1.4.1的不同类型身份而编译失败，0native；24报告identity/owner已闭。原因尚未确定，不能算产品连接故障；下一控制0da0dbe6a0a71b771995d099ce9f06ea27a19697为两臂各用初始不存在的独立target完整重编，source/夹具/design-v2不变，排除共用产物干扰。

第四场190352Z-1ac4a84d的两臂全新独立target均已编译过：A28.63秒/C30.25秒。原shared target的类型身份冲突没有在本次重现，不能据旧编译错误改产品。两臂healthy均已过并各含15个真实阶段事件：双方Direct主连接、独立Relay-only预备/故障后见证、主连接真实8字节往返与不换连接。正在按design-v2执行余8个故障case，结果以实际receipt/collector为准，不能提前验收。


## 已收齐的原版/当前候选结果

190352Z-1ac4a84d已收集闭证：原101/196.84秒、178报告文件；固定10case/全部进程清理/原生footer/预备两端Direct＋独立Relay实际往返/真实静默UDP丢包计数/同一连接/无重放全部核对。E/run/direct-fallback-verification-v2.json。两臂各一次，debug诊断，不是p95或完整App验收。

| 条件 | A原版 | C当前vendor＋Khor选路/传输配置 |
| --- | --- | --- |
| 无故障 | 5.556ms，Direct | 3.799ms，Direct |
| 客户端静默Direct UDP | **15292.660ms，超过2s参考**，最终同连接经Relay恢复 | 601.735ms，同连接经Relay恢复 |
| 服务端静默Direct UDP | **15296.229ms，超过2s参考**，最终同连接经Relay恢复 | 605.340ms，同连接经Relay恢复 |
| 客户端丢Relay TCP | 12.301ms，保持Direct | 12.087ms，保持Direct |
| 服务端丢Relay TCP | 13.831ms，保持Direct | 13.404ms，保持Direct |

静默Direct两种条件下独立Relay-only实际往返约108ms且完整，故障端规则实际有丢包计数。原版的兜底存在，但本模型下约15.3秒，不能把最新版已有Relay等同于已满足快速接管。C约0.6秒证明整套候选在这些条件下更快，不证明全部60项补丁或任何单项都必要。

重要归因边界：两臂核心版本均iroh1.4.1/Noq1.4.0、模型/共同夹具相同，但锁文件种子不同，其他共享包版本有差异（完整列表在proof），并且C同时含vendor补丁、RoadOrder、BBRv3与observed-address配置。因此这轮是bundle诊断，不能作严格单补丁ablation或最终性能晋升。反向控制安装Relay TCP规则但没有主动发送post-fault负向Relay请求，某些故障端计数为0；只证明规则存在时Direct业务保持，不能冒称该轮已实证Relay TCP收到并丢弃业务包。

下一隔离先固定C的完整依赖闭包与同一夹具，关闭仅测试根的khor_policy（即不调用RoadOrder/transport，使用现有vendor的默认selector/传输配置），以B判定快速接管是否依赖Khor策略层。B仍有全部现存vendor补丁，绝不能叫原版iroh；若B同样快，再继续削减vendor的存活/迁移子集。不得把本轮A的旧失败移出目标。

B策略层隔离已在运行前冻结E/direct-fallback-policy-isolation-plan-v1.json，control76d077a27518e6425990f52c6c631a4866d81b9e、sourcebf4ef784不变：只跑B五case，使用C原锁且metadata --locked，强制所有非根package/依赖边/features与保留C metadata逐项相等；同夹具，根khor_policy关闭，默认vendor selector/transport。单独全新B target；原A/C十case不重跑。任何依赖差异先拒绝，不容忍静默更新后作归因。尚未得到B结果。

B首场191210Z-016c949a在metadata阶段拒绝（原1/3.85秒，0编译/0native/12报告已闭）：原C metadata对iroh同一依赖重复输出相同kind/target项，而B去重，原严格列表比较误拒绝。data-only核对全部非根package IDs/features和唯一(name,pkg,kind,target)边均相同，E/direct-fallback-policy-graph-review-v1.json保留；不放宽任何版本/特性/边语义。新controlb93ae14170127cf8629474e71811843dc7be255c仅归一重复项且保留原件，source/夹具/五case/期限不变，执行前已冻结。


## B策略层隔离结果

191518Z-3a4d5411已闭：0/64.22秒、95报告文件，单独全新target编译30.86秒，固定B五项全过。E/run/direct-fallback-policy-isolation-verification-v2.json。B复制C原锁并metadata --locked；全部非根package IDs、features、唯一依赖边与C严格相等，实际root metadata与compiler-artifact均确认没有khor_policy。源码/共同夹具完全相同，保留全部现有vendor。没有重跑A/C原十项。

| B条件 | 主业务结果 | 保留Relay-only见证 |
| --- | --- | --- |
| 无故障 | 4.273ms，Direct | 115.992ms |
| 客户端静默UDP | **603.066ms**，原连接经Relay | 108.388ms |
| 服务端静默UDP | **605.311ms**，原连接经Relay | 108.115ms |
| 客户端丢Relay TCP | 14.179ms，Direct | 未发负向Relay请求 |
| 服务端丢Relay TCP | 18.084ms，Direct | 未发负向Relay请求 |

可得结论仅限这组模型：C额外的RoadOrder与BBRv3/observed-address setter并不是实现约0.6秒静默故障接管的必要条件；B保留的vendor组合已具备这一能力。并未证明全部vendor补丁必要，亦未证明可以从产品删除RoadOrder/BBRv3：原U1“90ms Direct不压过5ms Relay”、peer relay、bulk公平/损耗、换网等合同仍需保持。B的默认tier策略已知不等价于U1，不能直接推广成产品默认。

接下来的隔离顺序：在保留本共同夹具和完整依赖绑定的条件下，拆分vendor的存活判断/迁移/在飞数据处理与质量优化/观测；每个候选先保留真实备用见证与健康反向控制，改动后运行对应故障合同，再按原专属回归决定退出。优先验证更小的快速故障处理是否足够，避免再向正常质量确认体系加参数。获得最小组合后，才回到原N3真实目录/终端/文件负载和N4移动验收。两次Direct故障各1个诊断样本，不是分布/SLO证明；原N3未达标、全部旧失败和完整N0–N5保持。
