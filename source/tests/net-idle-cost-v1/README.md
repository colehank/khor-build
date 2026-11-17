# 个人设备网的静默开销夹具

只由有 owner、原 CPU/磁盘门和进程树清理的 Linux netlab 控制器运行。本机 source-only，不执行本夹具。`Cargo.toml.in` 与 `lib.rs` 的 `@ROOT@` 在远端私有副本替换为准确产品源码路径；复用已核验的普通 Release CLI，不编译或替换用户现装程序。

固定 1／3／5 个真实 resident，每组先完整配对并验证所有有向目录请求，等待 30 秒，再静默 60 秒。Direct-capable 和 Relay-only 两种模型各固定三轮，不能替换失败 trial。静默期间夹具不调用 Node 网络接口；前后在实际 namespace 读取接口计数，并按 endpoint 中的 resident PID 读取进程 CPU ticks、RSS、线程和上下文切换。结束后原两秒期限读取真实远端目录；一个节点没有远端，因此该组不产生伪造的业务样本。

CPU ticks 按内核 HZ 换算，并保留采样边界时间，不把一个进程占全机百分比当作单核百分比。上下文切换只是共同线程 ID 的快照差值，遗漏退出线程且不能独立证明线程代际；它不等于硬件唤醒或手机能耗。模型标记表示配置，不把它当成静默期间的逐包实际路型证明。

当前源的 producer、固定协议、全部原失败和测量收据见 `docs/net-optimization/COST.md`。预算不能看完结果再设置；本夹具先量化桌面开销，不单独决定 N3 或手机通过。
