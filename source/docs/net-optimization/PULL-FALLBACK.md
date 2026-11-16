# 可选文件连接失败后的回退

当前隔离候选 e4e433a57964495d202d6f0eecdf8e3fe65fc323 / tree eacc11bf9cd05dd50862bbe78e92d2b88a60d653，分支 experiment/net-pull-negotiation-fallback-v1。默认仍 shared；这里处理内部 separate 模式下的额外准备。产品只改 link.rs，无 vendor、ALPN、计时器、写重放边界或安装改动。

主连接能够完成请求时，额外连接的运输协商失败不能丢掉正在读取的文件。原实现把具体运输错误和应用/策略拒绝混为 Refused，随后作为终止处理。真实 QUIC 夹具仅改变新握手的服务端 ALPN，原认证主连接继续服务：准确 RED 085432Z-798e08a1 出现两次实际协商错误、文件提前 Err；其后主连接仍以原 stable-id 完成 Sessions，夹具关闭后才按新断言失败。此结果只针对这个候选 bug，不是旧混合 screen 最终 direct_after=false 的根因。

候选用 Transport 保留具体 QUIC 错误类别；可选准备的 TimedOut/Transport 回到 cached Main。应用主动关闭、本机关闭、暂停/后台拒绝、未知连接/认证错误和 CallerExpired 保持终止。多路错误聚合保留已知显式拒绝，另一条运输失败或 dial 自身超时不能抹掉它。原 primary dial 的展示、backoff 和 early-carrier 重试条件包含新类别，原期限继续向下传递。

同一个三项运输测试模块逐字未改，修复后 085928Z-39e34269：20/20，通过原全局格式和 owner 清理核验。故障场先观察到实际协商错误，再观察 cached offset2 在 offset1 仍被 hold 时发出；释放后文件完整、精确字节、13 个 offset 各一次，part/note 清理，cached Sessions/身份保持。新增真实 Incoming.refuse 负控制及原 ApplicationClosed/LocallyClosed 负控制均终止，旧16项隔离/小文件/暂停/过期/取消合同保持。

此前 084909Z-ec61253e 把 Incoming.refuse 当普通运输故障，这是夹具语义错误：wires::resting_close 和生产暂停处理明确把 CONNECTION_REFUSED 当 Asleep。该源、原计划、2过1失败与纠正说明全部保留，未据此放宽暂停；没有把它冒充产品 bug。

原12组标准090158Z-0ddf1257已306通过、0失败、原忽略1，全部原case/忽略/源码/owner与格式核验已闭；20项明确mode-ON及14contact都过，原大部分旧case仍shared，见STATE；原fe61的Mac290不是当前eacc的证据。每peer/global上限、实际Endpoint归属、并发文件公平、Release共存性能、1/3/5设备成本与真机换网仍未验收；20项观察界不是生产SLO，完整N3与N0–N5保持未完成。

证据在 /Volumes/disk0/Khor/evidence/net：pull-negotiation-fallback-*-v1.json，原收据目录下有身份、原始日志和严格verification。所有观察/人口在相应运行前冻结，没有修改门槛重跑直到绿。
