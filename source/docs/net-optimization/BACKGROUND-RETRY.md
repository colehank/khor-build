# 后台重试错峰

2026-11-16，独立候选b04e3cb3完整358回归已闭；canonical已由eed1ab9a采用，所有非docs文件与b04一致，最终准确tree Quality/Mac仍待。

原Wires已按peer/protocol串行拨号并指数退避，但连续失败的peer使用完全相同的等待。个人设备同时断网或醒来时，后台可能集中重试。PLAN N2.5要求适当随机扰动，不需要另一套恢复任务或调度器。

保留原5秒首次失败、300秒上限与睡眠设备300秒。连续失败2至6次时，在原10/20/40/80/160秒等待上添加0–25%的正扰动，仍受原上限约束。每个Wires持有标准库RandomState，对(peer id, failure count)取hash：节点间不同，同一状态反复读取稳定，因此journal显示与实际retry_at一致。不增加依赖、任务、timer或按peer保存的新表，也不缩短原后台等待。

前台请求仍走原前台预算；网络/联系代次和睡眠规则原样保持。到达300秒上限后沿用此前各peer累积形成的错峰，不重新增长等待。该策略不声称消除任意大规模网络的重试峰值，目标仍是个人设备网。

先在仅加两项测试的8feaa67b取得实际RED：`turing-20261116T142005Z-4e15b0ef`原101，19pass/1fail；唯一失败为所有peer等待相同。实现后同测试在b04e3cb3的`142930Z-4cf80a91`原0、20pass/0fail。原18合同和新增journal一致性均过，spread测试同时核首次/上限/睡眠、每一重复失败阶段128个peer的边界、跨peer分散与重复读取稳定。完整原12网络组＋13真实Node集成`143457Z-68b8fbe7`原0/956.140秒、358pass/0fail/原2ignored helper已闭，所有原门和owner清理核验通过。该原生回归不是Release性能、手机或完整N0–N5接受。

差异复查先使用review-changes的最小上下文、差异、影响流、tests_for和半径。图只有两个文件且构建点落后一个commit，不能用其零外部影响证明无影响；直接核对原next_backoff→wait_after→failed_at_contact调用与准确diff。生产只加一个RandomState字段、导入和有界计算；`base`在转换毫秒前严格小于300秒，整数转换和加法有上界。全局并发额度、取消、信任、读写重试与底层iroh选路未改。

原始计划、源码描述、格式记录与RED/GREEN proof在E/background-jitter-*，各run ledger保留原退出码和全部owner清理。后续最终Mac与Quality须绑定准确集成源码，不将此候选测试冒称TestFlight真机结果。
