# 空闲恢复后的首次操作与成本

本协议在新增首次操作数据前保存。旧p7及其active10秒/idle20秒选路门保持；
它们量两端重新选路的时间，不代表用户空闲后首次读取的响应。
源码备用keepalive15秒、STREAM活跃刷新最多每peer5秒一次，与旧idle16–17秒量级相符，
但不能独立证明旧尾部的唯一原因，也不预选缩短生产计时器的方案。

## 所有权准入

`LabNode.with`当前直接await设备JoinHandle。调用者future被丢弃时，内部设备任务可能继续执行；
这是需要真实验证的fixture所有权假设，尚不能凭静态代码把它算作已重现或已修复。

先添加固定filter `paths::idle_action::p8g_`的两个ignored诊断，必须显式`--ignored`执行：

1. 实际LabNode.with启动一个受控pending future，收到started后丢弃调用者；
   在1秒fixture界内必须观察到future Drop。旧实现即使红，也先释放受控等待并收尾后再断言。
2. 原始Device.spawn句柄被丢弃的负控必须能被检测；保留独立abort控制并回收，
   防止一个失灵的观测器把“没观察到任务”解释为自动取消。

两项各用原speed_pair的真实resident/设备运行时，5秒startup/cleanup只是fixture存活界；
测试线程1、诊断关闭、原shared guard/namespace/资源门不变。先取得准确源码的真实红和负控，
再修正fixture所有权，以完全相同两个测试复验。生产Noq/iroh/Node策略和旧场景参数不因此改变。
这只证明测试调用者对设备future的所有权；服务端任务、RPC取消、独立host/child与真机仍另验。

## 随后的首次操作测量

所有权准入通过后才实现并执行新测量。沿用speed_pair：慢Direct300ms、可用Relay约80ms，
两端准备阶段确认使用Relay，STREAM静默至少10秒，然后将原Direct恢复至0ms。
恢复后等待名义0/5/14秒，每方向各一个独立fresh，共六场景；不能先从另一端发业务预热。
首次前台目录读取前不查询Node状态或发其他业务；原echo的30秒caller与全部结果记录保留，
这是测量存活界，不是新UX达标承诺。记录实际单调时间、首请求服务/总时间、错误/超时与前后路径事实。

分开idle和首次操作的wire总字节/包区间；接口只读快照自身的时间与命令失败也须记录。
不能把网络总量当probe-only、把CPU当唤醒或把服务器结果当手机能耗。
观察工具/设备任务须由调用者拥有；计时前不得新增无主任务或预热动作。
需要原始配置/计数单调性、收据、进程组空、全部源/报告hash和source-owned复算；
缺测和失败保留，不按好看的首请求筛样本。

当前只有静态审计与新guard源码；Linux格式/编译/guard红绿均待远端执行，零新增首次操作数据。
当前prefetch9组仍使用原冻结源/control，不能混入新guard或在其计时内编译。
完整N0–N5、严重/quiet/跨连接质量、peer relay与移动、最终UX与真实手机能耗范围保持。
