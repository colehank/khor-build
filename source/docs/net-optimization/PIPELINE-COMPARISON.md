# 批量屏障与滚动补齐的独立对照

本协议在任何rolling计时数据前保存。priority两套18轮已完整结束；批量prefetch1/2/8矩阵正在执行，
其准确源与control保持冻结。滚动实现仍在独立experiment/net-pipeline-v1，不能混入该矩阵。

## 假设与匹配输入

原pull_into等整批join_all结束再补下一批；候选在按序提交一个前缀后补一个请求。
目标是减少批量屏障造成的空槽，同时控制交互长尾、乱序响应内存和取消所有权。
四个逻辑测试与跨窗口真实文件wire已在Mac执行通过；files库的Finder回收站超时两次复现，
整轮验证仍失败。此范围不证明Noq上的实际补齐时刻、服务端取消、吞吐或最终N3接受。

先完整复核批量prefetch矩阵，再在首个rolling数据前记录比较宽度与理由。
primary宽度沿用原8作为共同参考；若prefetch显示更小宽度存在值得保留的交互/完成时间取舍，
在首个rolling数据前确定对应宽度的独立补充对照，不能看rolling结果再选择性补样本。
priority固定0，两种实现完全相同；当前Mac候选仍为-1，不能直接拿来与prefetch源比较。

匹配baseline与candidate均包含相同私有Window模块、四个逻辑测试和同一份大于八片的files_wire载荷。
baseline保持原batch pull_into，私有helper不被生产代码调用；candidate只替换该调度循环及必需短读重启。
核完整树、原batch函数体与全部fixture，明确相对已测prefetch源新增的未调用helper和测试载荷。
不通过只看两行常量来宣称两个不同实验源等价。Noq/iroh、切片大小、priority、并发上限、文件内容、
控制入口、网络配置、caller期限、诊断开关和记录方式全部匹配。

## 首轮性能顺序与结果

固定A=batch、B=rolling，顺序 **A B B A A B**；每block先两个真实w3守卫，再low/high各一个fresh。
每profile每arm三场景，共12场景；low/high独立报告，不能将旧prefetch场景充作本轮baseline。
守卫失败不进入计时；业务开始后low失败仍执行high，保留首非零和失败原位置。
任一场景缺少完整逻辑人口时先调查，不自动补跑或将缺失当成功。

原low v4与cold high v3保持16/192MiB、120/240秒file caller、100/400ms计划、每类单个在途、
三phase每类各120、2秒operation caller、原PTY边界、20ms借用probe/Skip/cap和512行日志。
每场景720个逻辑请求、实际传输/失败attempt人口、全部结果和调度滞后均保存。
成功文件核完整字节和原路径/关闭门；超时只报告被原caller截断的attempt，不能生成verified payload rate。
记录实际配置和路径，不能切断Relay或强制Direct来制造候选胜出。

采用准确冻结source metadata复算、独立原始人口/单调时间戳、收据/原group为空和全部report文件hash复核。
报告逐场景服务/计划到完成分布、文件完成时间、全部失败、成本及对应位置差值；
三场景/arm只作探索，不宣称显著性、p99或全局最优。

## 行为、成本与最终接受

计时之外仍需真实文件合同：有序写入、首片cap、逐片版本/mtime、短片续读、断线续传、
远端变化后重启、拒绝/取消清理、landing提交与原本地复制/sponsor SCP。
慢头/慢尾必须观察真实RPC的已发/已完成/待提交数量；逻辑future被poll不等于网络请求已发。
取消须覆盖用户动作到client/remote读任务释放、外层host/child退出、晚到旧版本响应及资源上限；
本地drop计数不能替代这些端到端证明。

完整前台文件/预览、公平持续推进、共享/分连接、背压与1/3/5设备CPU/字节/唤醒/能耗仍是N3必需范围。
若rolling只改善文件速度但增加交互长尾、CPU或缓存，应记录取舍并继续设计，不能以吞吐一项宣布完成。
最终集成还需准确源码的Mac/Quality、全部相关回归、真实手机自动换网/peer relay与能耗验收。
Finder红项单独调查并保留，不在本协议中豁免；没有合并、发布或候选App安装授权。


## 完整prefetch后的宽度冻结（首个rolling数据前）

18场景/12960交互/153原件已完整复核，结果见[PREFETCH](PREFETCH.md)。
主比较宽度8保持；预先增加独立宽度2补充，原因是low文件完成接近8且目录延迟较低，high则原三次超时，
须验证滚动补齐是否改变取舍。每宽度仍各自A B B A A B、每block low/high各1，共12；先完整8再完整2，合计24。
不将原prefetch作为baseline、不混合宽度/profile/失败，也不在看rolling结果后选择补充宽度。
宽度1不增加滚动计时：单个在途没有本轮要隔离的批量空槽并发差异。
上述是实验输入选择，不是最终产品参数；准确源对、逻辑/wire/取消与全部N0–N5门保持。
