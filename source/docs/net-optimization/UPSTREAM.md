# 上游状态与补丁退出核查（进行中）

2026-11-09。本轮只读核查，不升级依赖、不把上游发布当作本项目验证通过。N0.1/N5.1仍部分完成；原45条盘点后新增iroh35/36修正，当前47条的上游等价核查仍未完成。

逐项覆盖/退出账目见[VENDOR-AUDIT](VENDOR-AUDIT.md)：全部45项已做第一轮覆盖盘点，明确Noq 1/4/5/6的专属覆盖缺口，以及当前iroh33过滤器未执行net_report/socket的khor_测试。逐函数上游等价性和隔离候选行为复验尚未完成；泛化单元绿不替代缺失的专属回归。

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

- 先列全iroh1–34、Noq1–11补丁的目标、当前回归、上游等价性和仍依赖的接口；本页只开始了补丁1和观测接口核对。
- 升级候选与本轮N1运行源隔离。iroh/noq/noq-proto/noq-udp的版本与锁文件需整体核对，不能只换proto造成两套类型或漏用本地patch。
- 逐条确认等价后减补丁，保留原失败回归与新旧同fixture体验对照；升级或删补丁后需要相应Linux/Mac/真实设备验证。
- 只有全部剩余行为都不再依赖本地vendor实现时，才移除整个vendor子树及Cargo patch；补丁1去重等价不构成整树删除条件。
- 目前不声称1.4.1已解决本轮p1/p3/p7/f4问题，也不以仍需维护本地补丁为由拒绝评估上游升级。尚未改变本轮依赖或部署。
