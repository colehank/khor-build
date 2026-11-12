# 预取并发量单因素协议

在首个prefetch数据前冻结，延续HIGH-BDP与MODEL-DECISION；当前优先级18轮仍使用原冻结输入。

待完整优先级对照复核后，明确记录一份固定priority策略及理由，再冻结PULL_AHEAD=1/2/8三树。
三树只差该常量一行，全部fixture、metadata、控制输入与其他生产代码相同。
每个profile的预定顺序均为 **8、1、2、1、2、8、2、8、1**，每值各三fresh。
没有策略选择、准确三树/单因素审计与campaign记录，不提交prefetch数据。

固定task每个值先两个真实w3守卫，再一个原low w2和一个原high wh2；分别写low-prefetch与high-prefetch。
两profile按值交错运行，各自仍保持上述九轮顺序，共18个fresh场景。守卫失败不进入业务；
业务阶段一旦开始，low失败也执行原定high并保留首个非零退出，不替换失败轮。
这一安排复用同一编译输入，不合并两种profile的结果或把编译混入测量窗口。

原low v4与cold high v3的网络、文件、记录器、请求人口、重叠门和期限全部保持：
low16MiB/120秒/100ms计划，high192MiB/240秒/400ms计划；三phase每类各120请求、
单operation一个在途、2秒caller、原PTY10秒、20ms借用probe/Skip/cap和512行阶段日志。
完成仍需全部回答、完整内容/关闭/Direct及实际配置和下载路径字节。不能延长high的期限让并发1过关。

文件失败也须保留720完整逻辑人口和全部caller结果。失败文件的接口/qdisc仍严格核几何、
配置漂移和单调计数，但不伪要求未完成文件已经流过完整192MiB；这只是让失败可分析，
不改变任何通过条件。文件成功仍须原完整路径字节门。原优先级/preflight validator默认门不变。
失败文件不报有验证的payload吞吐，其时间区间标作caller attempt，不称完整实际传输；
timeout标明完成时间被原caller界截断，不能把它当作一份已完成文件或从分母剔除。
所有phase/开始在attempt内/完整在attempt内的人口、成功条件分布、失败/缺失、调度滞后均保存。

本实验只判断固定批量实现的在途量取舍。join_all批量屏障是另一个待测因素，
不改入这三树；滚动补齐若要实验，另冻结相同上限、顺序写入/mtime/续传/取消/所有权合同。
prefetch实验不是完整前台预览、公平/背压、设备规模/能耗或N3最终验收。
