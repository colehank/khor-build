# 上游状态与补丁退出核查（进行中）

2026-11-09。本轮只读核查，不升级依赖、不把上游发布当作本项目验证通过。N0.1/N5.1仍部分完成；原45条盘点后新增iroh35/36/37修正，当前48条的上游等价核查仍未完成。

逐项覆盖/退出账目见[VENDOR-AUDIT](VENDOR-AUDIT.md)：全部45项已做第一轮覆盖盘点，明确Noq 1/4/5/6的专属覆盖缺口。随后Noq4独立ACK与Noq6保活计时、iroh选家/QAD/网桥9项已有对应源绿，具体边界查VENDOR-AUDIT/STATE。逐函数上游等价性和隔离候选行为复验尚未完成；泛化单元绿不替代缺失的专属回归。

## 固定来源与证据

通过GitHub连接器读官方源码/PR，官方发布页及crates.io registry交叉核对。GitHub Noq的Latest标签仍指向1.1.0，不能代替registry的最新稳定版本。
下载的只是源码包，不执行其中代码、不运行本机构建；每包SHA-256与registry checksum一致。
原始registry JSON、包、解包源码与source-manifest.json保存在 `/Volumes/disk0/Khor/evidence/net/upstream-20261109/`。

| 组件 | 当前vendor/使用来源 | 已核对发布版本 | 发布时间（UTC） | 发布源码commit |
| --- | --- | --- | --- | --- |
| iroh | 1.2.0，17c0612f80f78f5288e97b818b1360ae6ea0a51a | 1.4.1 | 2026-10-21 17:08:50 | d6c6963b9602d1b8dd2ba53b6519e7eb19cc7940 |
| noq-proto | 1.3.0，c1f411562e6078852749b8bcf1190523096a107f | 1.4.0 | 2026-10-20 17:16:21 | 34f75c49f93de5e2ff943483cabc5b6d0c5df918 |
| noq driver | 已核对1.3.0发布源码，与本地proto原始VCS一致 | 1.4.0 | 2026-10-20 17:16:39 | 34f75c49f93de5e2ff943483cabc5b6d0c5df918 |

来源：[iroh 1.4.1发布](https://github.com/n0-computer/iroh/releases/tag/v1.4.1)、[iroh registry](https://crates.io/api/v1/crates/iroh)、[Noq registry](https://crates.io/api/v1/crates/noq)、[Noq-proto registry](https://crates.io/api/v1/crates/noq-proto)。对应源码包均未yank；这是观察时状态，不是永久承诺。

## 补丁1：OPEN历史已过时，有等价去重但未证明整块可删

[PR #4522](https://github.com/n0-computer/iroh/pull/4522)于2026-09-29关闭，**未合并**；head仍为本地最初引用的eaca51d4。原KHOR.md的2026-09-21 OPEN是历史观察，不是当前状态。

[维护者评论](https://github.com/n0-computer/iroh/pull/4522#issuecomment-5893467914)认为提案超出临时修补所需，建议先以去重止住增长，并计划更根本的actor状态管理改造。这是维护意见，不证明后续架构改造已经发布。

已检查[1.4.1固定源码](https://github.com/n0-computer/iroh/blob/d6c6963b9602d1b8dd2ba53b6519e7eb19cc7940/iroh/src/socket/remote_map/remote_state.rs)：
遇RemoteCidsExhausted/MaxPathIdReached仍沿333ms重试，加入pending_open_paths之前先contains去重。该文件未见本地同名64上限/PathIds等待释放实现。
因此可确认存在同类去重，不能只因#4522未合并就说完全没修；也不能只凭contains断言本地有界/路径ID恢复与其他补丁都可以删。
下一步要把本地队列有界、路径ID耗尽后释放重开等行为回归移植到隔离的上游升级候选，验证通过后逐块减补丁。

## N1观测接口与时钟边界

1.4.0源码中的latest_rtt搜索命中包含qlog RecoveryMetrics，不能把日志字段自动当成有样本计数/时刻的公共选路接口。
本轮尚未证明其替代Noq补丁8/10或iroh共享质量缓存的能力；需逐项检查公开API与等值ACK、积压ACK、稀疏恢复回归。
来源：[Noq-proto固定paths.rs](https://github.com/n0-computer/noq/blob/34f75c49f93de5e2ff943483cabc5b6d0c5df918/noq-proto/src/connection/paths.rs)。

当前driver 1.3.0的UDP读取把传入的now交给proto::Endpoint::handle，所查路径使用meta.addr/dst_ip/ecn而未使用内核收包时刻。
这支持目前将采样received称为协议接收路径的时间，不冒称已取得内核时间；CPU影响仍须实际验证，不能由字段名消除。
来源：[Noq 1.3固定endpoint.rs](https://github.com/n0-computer/noq/blob/c1f411562e6078852749b8bcf1190523096a107f/noq/src/endpoint.rs)。

## 升级与退出规则

- 先列全iroh1–37、Noq1–11补丁的目标、当前回归、上游等价性和仍依赖的接口；本页只开始了补丁1和观测接口核对。
- 升级候选与本轮N1运行源隔离。iroh/noq/noq-proto/noq-udp的版本与锁文件需整体核对，不能只换proto造成两套类型或漏用本地patch。
- 逐条确认等价后减补丁，保留原失败回归与新旧同fixture体验对照；升级或删补丁后需要相应Linux/Mac/真实设备验证。
- 只有全部剩余行为都不再依赖本地vendor实现时，才移除整个vendor子树及Cargo patch；补丁1去重等价不构成整树删除条件。
- 目前不声称1.4.1已解决本轮p1/p3/p7/f4问题，也不以仍需维护本地补丁为由拒绝评估上游升级。尚未改变本轮依赖或部署。


## Noq 1/2/5：固定1.4.0调用链复核（2026-11-09）

本节基于校验过SHA-256的同一1.4.0发布包；未执行该上游，也未升级依赖。
逐文件SHA-256、准确行号摘录和来源在`evidence/net/upstream-20261109/noq-1-2-5-call-chains/`。
这是具体链路的静态差异证明，不是上游问题已实际重现或补丁整体等价审计完成。

### 1：BBR应用闲置通知的时序不同

1.4的poll_transmit在connection/mod.rs:1444更新path.app_limited，Controller接口没有即时idle通知。
内置BBR在on_end_acks(:1731)接收这个标志并同步自身inflight；on_packet_sent(:1622)先调用handle_restart_from_idle，
其分支依赖自身inflight为零且自身app_limited非零，随后记录该包的is_app_limited。
因此transport已经判为空闲、而下一批ACK尚未发生时，不能仅凭路径的app_limited字段证明BBR已收到该状态。
本地补丁1在原发送轮询得出app_limited时立即调用Controller::on_app_limited，明确覆盖这段时序。

loss链也不同：1.4连接层先remove_in_flight，再调用controller.on_packet_lost(:3523)；内置BBR该方法(:1800)累计lost并处理记录，
没有本地在该事件中立即扣减C.inflight的一行，后者仍在on_end_acks同步。这个判断来自调用链和函数体，不是只搜不到方法名。
静态不能给出吞吐倍数，也不能证明上游其他BBR改造没有缓解最终体验；仍需同源、同seed的idle→burst/丢包与Cubic对照。
来源：[固定连接层](https://github.com/n0-computer/noq/blob/34f75c49f93de5e2ff943483cabc5b6d0c5df918/noq-proto/src/connection/mod.rs)、
[固定BBR](https://github.com/n0-computer/noq/blob/34f75c49f93de5e2ff943483cabc5b6d0c5df918/noq-proto/src/congestion/bbr3/mod.rs)。

### 2：缓存参数→pending→0-RTT封包链仍不同，旧测试漏了一个关键配置

1.4 init_0rtt(:4021)恢复TLS ticket的transport parameters，显式清除项不包含address_discovery_role；
set_peer_params(:6753)按双方角色设置pending.observed_address。space_can_send允许客户端有ZeroRtt密钥时发送Data，
已排队的STREAM足以触发封包；populate_packet的OBSERVED_ADDR分支(:6339)没有!is_0rtt。
接收端process_data对frame.is_1rtt检查，Frame::is_qad_frame将ObservedAddr归为1-RTT，不能以接收端宽容解释该差异。
双方一直启用reporting、客户端在服务端握手响应前发送早数据的链路，静态上没有与本地两个门等价的保护；运行结论仍待上游候选。

原address_discovery_zero_rtt_accepted在第二次拨号把客户端角色改成default/disabled，测试目标是配置变化被接受；
它不能单独证明reporting保持开启时OBSERVED_ADDR不进入0-RTT。拒绝测试也不替代这个配置。
本轮新增address_discovery_reporting_waits_for_one_rtt_on_resumption：双方保持both，
先不驱动服务端，确认客户端已发STREAM却零OBSERVED_ADDR；再完成握手、确认早数据被接受且字节完整，并确认1-RTT仍发送地址报告。
**只加测试，未改生产帧准入；未编译/执行，不能记为通过或旧源红。**
来源：[固定封包与参数链](https://github.com/n0-computer/noq/blob/34f75c49f93de5e2ff943483cabc5b6d0c5df918/noq-proto/src/connection/mod.rs)、
[固定帧准入](https://github.com/n0-computer/noq/blob/34f75c49f93de5e2ff943483cabc5b6d0c5df918/noq-proto/src/frame.rs)、
[固定原测试](https://github.com/n0-computer/noq/blob/34f75c49f93de5e2ff943483cabc5b6d0c5df918/noq-proto/src/tests/mod.rs)。

### 5：新最大号与最大号本次新确认不是同一条件

1.4 inner_on_ack_received(:2972)在ACK抬高largest_acked_packet_pn时就得到new_largest_pn；
只有sent_packets.get(ack.largest)成功才更新largest_acked_packet_send_time。
随后newly_acked由还在记录里的范围求出，可能只有更低号的ack-eliciting包；:3056采样门只查new_largest_pn与ack_eliciting_acked，
没有像本地largest_newly_acked那样要求最大号有有效本次记录。缺失最大号记录时，旧发送时间仍可能进入采样表达式。

[RFC 9002 §5.1及附录A.7](https://www.rfc-editor.org/rfc/rfc9002.html#section-5.1)分别描述最大号本次新确认和至少一个新确认包需ACK的条件，
并以该最大包的发送时间计算样本。本地保护对应的是这个差异；不能从“ACK抬高最大号”自动推出可安全读取旧时间。
专属回归还应构造合法已发送但已不保留的最大号及仍在记录中的更低需ACK包，验证不产生假RTT；
并用合法保留的最大号作正常采样对照。不能直接ACK未发/被刻意跳过的号，那只会测到PROTOCOL_VIOLATION。
PacketNumberSpace::sent本身有非ACK-eliciting尾部记录回收，说明“丢失发送记录”不能一律解释成从未发送；
实际最小序列/测试入口仍需准备与执行，当前没有该项行为红绿。
来源：[固定ACK链](https://github.com/n0-computer/noq/blob/34f75c49f93de5e2ff943483cabc5b6d0c5df918/noq-proto/src/connection/mod.rs)、
[固定包号空间](https://github.com/n0-computer/noq/blob/34f75c49f93de5e2ff943483cabc5b6d0c5df918/noq-proto/src/connection/spaces.rs)。


## Noq接口、ACK、保活与备用恢复事件补查

同一固定1.4.0包checksum复验通过，源hash与行号摘录在`evidence/net/upstream-20261109/noq-interface-and-recovery-call-chains/`；未执行上游。

- 补丁3/8/10：公开PathStats(:217)及path_stats(:2702)只有原传输RTT/统计/带宽等，没有本地PTO计数、最后收包、最老未确认包、无退避PTO或ACK计数/采样发送接收时刻。
  paths.rs:RecoveryMetrics虽有latest_rtt和pto_count字段，但结构本身私有且受qlog feature控制，按变化保留；它不是iroh可读的逐路径采样API，也不能证明等值ACK被计数。
  因此现有调用者不能通过直接升级就删除这些接口补丁；需要上游等价API或整体迁移后再跑对应证据，不能以日志字段同名替代。
- 补丁4：may_send_data由握手/验证/路径status决定；存在可用Available时Backup不获该许可。populate_packet(:6149)只在may_send_data路枚举各包号空间pending ACK。
  本地能独立发包的路径带自身ACK、不能时才兜底的规则在该链路中不同。源代码已证明路由决策差异，尚未用1.4候选复现单向故障/测ACK成本。
- 补丁6：ConnTimer::KeepAlive(:2488)调用ping，ping(:2715)遍历所有路径包号空间置pending_ping；该链没有按路径已有keep_alive分开。
  路径自己的PathKeepAlive另调ping_path；本地跳过规则与之不同。当前本地两计时器用例已437绿，上游同协议复验/设备成本仍未执行。
- 补丁7/9/11：set_path_status(:832)记录local_update和待发status；LossDetection(:2570)进入原loss/PTO处理，后者安排丢失检测或两份尾探测。
  普通handle_lost_packets重传和路径discard时释放仍存在，所以不能把“没有同名resend_off_backups”写成“上游没有恢复”。
  所查状态改变/PTO入口没有本地“最早慢/不应答Backup在飞帧主动重排”的同一事件联动，但完整交付速度是否等价必须跑两种故障时序与健康同速/MTU/成本对照；当前结论仍是保留、待行为验证。

来源：[固定连接入口与恢复](https://github.com/n0-computer/noq/blob/34f75c49f93de5e2ff943483cabc5b6d0c5df918/noq-proto/src/connection/mod.rs)、
[公开PathStats](https://github.com/n0-computer/noq/blob/34f75c49f93de5e2ff943483cabc5b6d0c5df918/noq-proto/src/connection/stats.rs)、
[内部qlog metrics](https://github.com/n0-computer/noq/blob/34f75c49f93de5e2ff943483cabc5b6d0c5df918/noq-proto/src/connection/paths.rs)。
