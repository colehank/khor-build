# 文件取消与连接所有权

2026-11-16。生产取消逻辑未改；本轮修正测试的布局前提，尚待准确候选的完整回归。

原整组 separate 回归 `turing-20261116T114927Z-5192725a` 在等待八个 STOP 时失败。仅增加 cfg(test) 连接身份与关闭观测后，固定最多50次、首失败即止的诊断 `125932Z-e422f307` 在第5次复现：4通过、1失败、45未执行，原5秒和全部断言保持，未补样本。

失败现场中，RPC0/1走 cached Main，RPC2–8走额外文件连接。取消后原 Main 未关闭；RPC1–7观察到 STOP，RPC8先观察到额外连接的 `closed by peer: pull ended (code 0)`。Responder 只接受 STOP 的断言因此失败。`TransferLine::drop` 原本就关闭自己拥有的额外连接；QUIC 连接终止与流 STOP 的到达次序不能作为资源泄漏证据。原失败不抹掉，也没有靠增加期限解决。

两个合同分别保留：

| 布局 | 必须观察到的结果 | 实际测试 |
| --- | --- | --- |
| 共享 Main | 八条 held RPC 全部 STOP，原连接不关闭，随后 Sessions 成功且 stable_id 不变 | `dropping_pull_stops_every_held_rpc_and_keeps_the_shared_wire` |
| 独立文件连接 | 实际 extra 收到非零数量 RPC；取消关闭该连接；owned responder 及全部子任务结束；cached held RPC 全部 STOP；原 Main 继续业务且身份不变 | `isolation::dropping_a_separate_pull_closes_its_real_held_rpcs_and_keeps_cached_main` |

I8 从 I7（0ef5d876）只修改第一个测试的调用入口：显式使用已有 `pull_into_with_layout(..., resume=true, separate=false)`，landing 和其余参数与原 `pull_path_with` 展开一致。八个 STOP、请求人口、原5秒、原 Main 健康断言均不变。第二个测试原本已显式 `separate=true`，继续真实验证额外连接及 handler 回收；不忽略任何测试，不改生产代码或新增配置。

原全局 separate 回归仍应执行全部12组，显式共享/负控制不被环境变量覆盖。该修正不意味着 N3 性能或真实手机取消已接受。原始事实、源码与所有权证明见 `E/turing-20261116T125932Z-e422f307/cancel-observer-captured-verification-v1.json`，E为外部证据目录。
