# 恢复同步与缓存额度读取的锁冲突

2026-11-16。准确运行与当前接受状态仍见 STATE；以下定位不代替修复后的原生验收。

Mac 完整回归 43898654929 中，新 endpoint 约 38ms 就绪，但第一轮恢复同步在 devices 仍 pending 时返回；本端没有带回对端在离线期间保存的改名。原测试的立即改名断言失败。Linux 完整358和后续固定10次诊断通过，不能抵销 Mac 的实际失败。

定点诊断 43913474400 保留原30秒 ears、60秒 synced和立即改名谓词：第1次通过、第2次失败后停止，4次未执行。失败时对端名称已更新，本端仍是旧名，证明不是改名没有写入。43915075545 在同样人口和原门下，进一步看到 devices receive/apply 的 `RootPermit::try_acquire` 返回同 root 已有 owner。43916219184 第1次即失败，5次未执行；实际持锁栈为 `dispatch_inner → Node::open → open_hosting → RootPermit::acquire`。

同步轮会读取缓存的 Quotas。该只读请求在 blocking worker 里重新打开 Node，因而进入完整初始化和 root 变更锁；同时该请求启动的额度后台刷新也会另开 Node。初始化与同一轮 devices 的接收/应用发生竞争。传输泵保留 Busy 的公平让步和部分同步结果，本轮结束时未应用改名。不能把这条证据解释成 iroh 没建联、网络丢包或需要更多 Relay。

最小修复在独立 `integrate/net-final-v1` 候选中：额度缓存读取及既有后台刷新复用已准入、正在服务的 resident Arc。没有 resident 的原调用仍走原 Node::open fallback。读取仍在 blocking worker，后台刷新仍按原每 home 一个 flight、结束后30秒合并；不增加任务、定时器、网络/写请求重试、同步预算或传输额度，不改变信任门和 Busy 的锁顺序。

因果回归在真实 root 变更锁保持期间调用 resident 的 Quotas handler，要求缓存读取成功。旧实现应拒绝，修复应通过；5秒仅是测试防挂总门，锁直到实际 handler 返回后才释放。然后必须通过原 Mac 恢复用例及必要的网络、额度/准入回归。原失败、每次样本和未执行人口保留。调试 observer 的日志/Backtrace 改动不进入产品候选。
