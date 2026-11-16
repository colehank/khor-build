# 速度优先的文件与交互共存验收

本页补充PULL-POOL，保留GOAL/PLAN/REQUIREMENTS完整范围。当前执行、准确run及失败见STATE。

产品源ff3360的普通Release CLI SHA256为7454caa6b7347669b322c64fafa16ac2ae7c4eb86e699f5b3496578552847845。新源6f7bbf43/treeaddd2218只改workload测试文件，重新普通Release编译后CLI逐字相同；没有安装/发行。

原三个workload测试仍使用原fixed-Direct条件。另加ws2/wsh2两个独立命名的speed-first case：保留low/high几何、16/192MiB字节、120×三阶段×两操作共720样本、原caller2秒与file120/240秒、原observer/完整文件/真实重叠/PTY关闭及owner清理；起终允许健康Direct/Relay/SSH/Forwarded，结束时再做两个有界的相反方向目录读取、核对完整payload，两端Link必须live且无down_ms。direct_after继续保存。路径标签不是速度证据，最终还须比较真实业务延迟/吞吐与成本。

先用准确普通Release做shared/separate各一次有日志布局合同。shared应1对Main服务目录及完整文件；v1对separate误预测offset0后立即切extra；实际首2块在缓存Main、后766块在唯一extra，原业务/健康全过而布局门拒绝。准确源码在准备等待时继续原window、保留已准入请求，故v2改为：offset0起的非空连续缓存前缀，随后非空连续后缀只用唯一extra；不固定缓存块数，目录仍在原Main，PTY是独立Tunnel。新合同固定1场确认，不改产品或旧拒绝。额外两个健康读带来客户端/服务端各2个回复，全部另存，原1128测量RPC继续精确配对。日志扰动场次不作性能晋升。shared及separate v2均已闭，原v1预测拒绝保留；当前性能campaign运行状态见STATE。

E/pull-coexistence-comparison-design-v1.json固定下一步同ELF设计选择：S shared/C separate顺序SCCSSC，每块low/high，合计12场、8640原交互样本、24个post-health读，每个arm/profile/operation/phase共360原样本。布局已闭，准确同源5个observer/kernel case按原MODEL-DECISION分层准入已闭：4原生过、high冷TCP平均率原门仍失败，失败与原退出1保留；其完整内容/echo/RTT/队列几何及独立UDP容量均核对。最终protocol/control已冻结并运行；计时不开lifecycle日志、不编译、不替代失败，固定串行控制器逐块collect/验收后推进。模型准入不意味着冷TCP门或N3通过。

原工程门保留：C每cell pooled scheduled-to-end p95不超过S×1.10＋20000μs，只有原360样本全ok才可评级；完整文件三次中位goodput不低于S×0.90；原公开参考的所有绝对延迟上限和文件率下限仍须通过。报告service/queue拆分、全部失败/缺失/取消和有限人口p99，不做通用WAN或能耗保证。C硬业务/健康失败或固定360人口的不可恢复绝对p95越界停止后续块；完整最后块的拒绝不能写成数据未齐。

这是共享/分连接设计选择。最终集成源码对公开main的同期对照、1/3/5 peer CPU/RSS/字节/唤醒、小文件首内容、SSH/hop/同步/预览共存、手机能耗，以及N1/N2/N4/custom/最终完整验收仍必须完成。同ELF对照不替代它们。

旧三臂screen停止第7C以及原有/无日志固定诊断、全部失败和未执行人口保持。120420Z-f2ed80e5为错误task的零执行启动拒绝，核对输入与PID/PGID消失后闭证；未产生测试或性能数据。后续用既有net-node入口，协议与case不变。
