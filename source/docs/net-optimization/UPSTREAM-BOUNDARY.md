# 上游职责与 Khor 定制边界

2026-11-16，回应用户“iroh没有处理？”及“iroh也有iroh relay”。本页纠正后续实施顺序；完整GOAL与全部原验收门不变。本轮仅静态核对与官方文档复核，没有新增产品代码、编译、部署或运行样本。

## 已核对的事实

- iroh本来就实现NAT穿透、直连、Relay传输兜底与QUIC多路径迁移。Relay不只是寻址服务，直连不能建立时也承载加密业务流量。见[官方Relays](https://docs.iroh.computer/concepts/relays)与[1.0架构说明](https://www.iroh.computer/blog/v1)。
- 当前官方最新稳定release仍是[1.4.1](https://github.com/n0-computer/iroh/releases/tag/v1.4.1)。W隔离候选vendor版本为1.4.1，带本地补丁；不代表canonical feat/net、main或已安装App均已升级，也不代表与原版等价。
- 原版1.4.1的默认选择器把IP归primary、Relay归backup；同tier以带偏置RTT和5ms滞回减少抖动，IPv6有3ms优势。原版源码`src/socket/biased_rtt_path_selector.rs:18–35,79–104,136–183`。这是一般默认策略，不证明满足Khor全部体验合同；尤其Relay明显更好仍会受tier压制，与U1有差异。
- [官方Builder](https://docs.rs/iroh/latest/iroh/endpoint/struct.Builder.html)已提供`path_selector`与`add_custom_transport`，均属unstable-custom-transports接口。候选**已经在使用**它们：`crates/net/src/endpoint.rs:595,686`。`RelayMode::Custom`配置的是iroh RelayMap（同文件733），不是另一套Relay协议。
- 因而不能把“接官方接口”当作尚未做过的新修复。真正待审的是：为何官方接口之下仍需要大量vendor改动，以及应用连接/队列生命周期是否妨碍上游自动接管。

## 职责边界

| 能力 | 默认归属 | Khor仅应承担的部分 |
| --- | --- | --- |
| Direct与iroh Relay传输、打洞、QUIC重传/拥塞控制、路径迁移 | iroh/Noq | 正确配置、地址发布、生命周期接线、观测和回归；发现缺陷优先最小复现与上游修复 |
| 路径偏好 | 优先原生默认；明确产品差异通过公开selector | 稳定体验合同、显式策略；不能另起与上游竞争的迁移/健康循环 |
| 可信设备peer relay、SSH承载 | Khor提供业务特有承载，通过custom transport复用iroh连接 | 设备信任、可达middle发现、完整两腿准入与资源边界；不因此自造QUIC/NAT协议 |
| 目录/终端/文件调度与写入重试 | Khor应用层 | 有限预算、取消、公平、写不重放；底层迁移不能自动修复应用卡死/连接被错误退役 |
| Relay地点、容量与可达性 | 基础设施配置和运营 | 实测现有Relay与故障域后再决策；无需自行发明Relay协议，也未获准购买/部署 |

## 定制负担必须显式处理

当前v5账目60项：1项已有上游替换与合同证据，3项在指定版本/专属ablation中删除后失败，56项仍待等价与退出审查。**56项待审不表示功能从未测试，也不表示已证明必须保留。**已迁移成功不等于应长期维护这些补丁；3项局部保留证据也不证明完整fork必要。

刚发现的正常切换确认、近期RTT窗口和快速离开规则属于本地补丁体系。当前SCCSSC失败不能直接归为上游缺陷，也不能仅凭一个相关事件再叠加“波动置信”补丁。暂停该新增提议，先建立责任边界。

## 接下来先做的验证

1. 固定上游原版能力基线：相同地址/Relay/故障条件下验证Direct、仅Relay、Direct失效接管与恢复/地址变更；记录真实业务、连续性、期限、清理和成本。裸iroh基线只能证明底层能力，不能冒充完整Node/PTY验收。
2. 对比当前Khor接线：官方Relay是否实际允许且双方可达，候选地址何时进入连接，上游路径变化是否被Khor缓存、排队或退役覆盖。peer relay与公共Relay分开证明完整路径，不能把首腿可达当两腿可用。
3. 按依赖闭包逐组消减本地选路/恢复补丁，用现有失败合同验证；不能只删一行selector却把其下本地Moving逻辑叫作“原版”。不能为了编译通过静默删业务或反向控制。
4. 原版满足的能力直接复用；公开API能表达的产品差异留在薄适配层；确实缺失且有行为反例的最小补丁保留并准备上游复现。不得用“已有复杂补丁”反证其必要，也不得无证据一次清空60项。
5. 回到原N3目录/终端/文件失败与N4换网/peer relay验收。原绝对/相对门、完整样本分母、写入/取消/能耗边界不变；不因策略简化而宣告完整目标完成。

本次source hashes与文档核对记录为E/`upstream-connection-boundary-review-v1.json`。E是`/Volumes/disk0/Khor/evidence/net`。没有新运行证据，不改v5账目状态。
