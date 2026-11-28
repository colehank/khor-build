# 普通 Release 混合负载生命周期诊断

2026-11-14。三场固定诊断完成，**2160 个逻辑请求全部成功，3384 个响应完整配对**。这些有日志样本中的服务端应用处理和客户端开流时间很短，但没有重现此前无日志性能对照的目录 p95 回退。因此没有定位该回退的根因，也没有接受 H2 或关闭 N3。

准确源码是实验 H2 `db21c2beeecb1d6b1823589267b2f98d34fa5e38`，tree `5cf730ab964c52fd613953251000ba2057e0f2bc`；它不包含后来的 N2 联系代次修复。复用 `turing-20261114T011935Z-60f3d677` 的普通 Release CLI/test ELF，features 为空，无 qlog 或诊断 Cargo feature，本轮重新编译次数为零。

| 固定场 | run | 原结果 | 目录客户端 elapsed p95 | 服务端 accept→本地 write p95 | 目录计划→完成 p95 |
| --- | --- | --- | --- | --- | --- |
| 1 | `024533Z-50822f42` | 720 ok | 355.501 ms | 1.929 ms | 381.737 ms |
| 2 | `024937Z-96d5dcf3` | 720 ok | 353.847 ms | 1.373 ms | 419.509 ms |
| 3 | `025345Z-16ed1d5c` | 720 ok | 344.306 ms | 1.476 ms | 429.886 ms |

完整 run 前缀均为 `turing-20261114T`。每场实际匹配 360 个目录响应和 768 个 256KiB 文件切片：两端 ALPN、wire、stream、offset 均核对；目录与文件共用一条 Main，终端使用另一个 Tunnel。目录阶段只在全部 360 个逻辑请求及两端响应成功、无额外响应时按原串行次序分配；失败或额外人口不强行配相位。

每场服务端 permit 等待增量 p95 为 4/4/2μs，客户端开流 p95 为 65/46/66μs。分析按同一个 RPC 的持续时间计算残差，未用两个 p95 相减，也未减两端未同步的绝对时钟。服务端阶段从 accept_bi 之后开始，本地 write 完成不代表对端交付或 ACK；残差还包含传输、accept 之前的等待、交付及未测应用区间，不能独断为网络、拥塞或某个 vendor 补丁。

每场 2256 个业务观测点均显示同一条 Direct，观测点间记录到的 selected 变化为零。这是业务时刻的快照，不能证明未观测区间没有切换，或确定实际每个包的路径。文件 goodput 20.803/17.458/23.616 Mbit/s 只描述诊断，不参与性能接受。

运行方法保留原 720 结果、192MiB、2 秒 caller、240 秒文件期限、400ms 业务节奏和 owned 600 秒外层所有权上限。先通过原来源及资源校验，再仅在 owned.run 期间设置既有 `KHOR_NET_DIAGNOSTICS=lifecycle`；`NETLAB_DIAGNOSTICS=0` 保持旧 feature 初始化与采集分支关闭，finally 恢复原环境。外部 observer 在原采样循环只读打开私有 serve.log，持有 fd 到全部原生 owner 退出后再 drain，保留 Scratch 删除路径后的尾部。

三场都通过 unlink 后续写、同路径换 inode、跳过外部 symlink 三项采集校准，以及原始来源、CPU、geometry、所有权退出和全部业务人口核验。最多 8 份日志、每份 16MiB。既有 flag 还启用只读 tuple 诊断及短暂 weak 升级，日志和外部采集有运行扰动；不能与无日志样本混算，不能证明性能或能耗不变。

准备版 `023714Z-08b04294` 在逐场来源校验前设置 flag，环境 hash 正确拒绝：原退出 1、5.698953 秒、native 0、compile 0、cases 为空。该失败、原日志和 v1 三场未执行记录均保留。v2 只收窄运行期环境作用域，保留全部来源断言，没有改 Rust、fixture 或期限。

证据在 `/Volumes/disk0/Khor/evidence/net`：

- `h2-lifecycle-shadow-summary-v2.json`、`h2-lifecycle-shadow-{plan,control}-v2.json`。
- 三个完整 run 下各自的 `h2-lifecycle-shadow-verification-v2.json`、原始 workload 日志、两端 lifecycle-shadow 和采集合同。
- `ops/verify-h2-lifecycle-shadow-v2.py` 为本地 stdlib 数据分析，不执行产品。

H2 原无日志正式对照仍按其预先声明的 baseline 硬失败规则停止，第 5/6 块未执行；目录固定门的不可挽回越界结论保持。下一步测量已通过 Linux/Mac 标准回归的准确 I4＋N2 修复源码 `96e999da`，不带入未接受的 H2/PULL4。新普通 Release 来源校验和无日志低/高负载验收单独记录；上述诊断不能替代它们。
