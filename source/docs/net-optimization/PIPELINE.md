# 有界滚动窗口候选

实验副本khor-net-pipeline-v1，branch experiment/net-pipeline-v1，基线797b7650。
不修改feat/net的冻结priority/prefetch输入，不据此预选策略或宣称吞吐改善。

原pull_path_with委托pull_into；后者按最多八片构成一批，join_all等整批响应才写入、补下一批。
候选仍保持最多八个尚未按序消费的请求/响应；首片独立确认(size,mtime)，之后按序提交一片即补一片。
乱序已完成的响应也占窗口额度，慢首片不能让缓存无限增长；慢尾片则不应挡住已提交前缀的补齐。
所有future仍借用原调用者，不spawn任务/增加连接/定时器，不改变优先级、信任、请求期限或切片大小。

本候选先改远端pull_into；本地复制、sponsor自身scp分支保持，预览和offer的其他批量调用仍待独立收口。
必须保留有序写入、首片cap、每片版本、短片续读、断线续传/变化后重启、拒绝/取消清理与原landing提交。
窗口整体重启/丢弃时，未提交的future和已完成缓存一起释放，不把旧版本响应写进新文件。
这不代表独立host/child退出或server读任务取消已通过；完整N3所有权与能耗门仍必须实测。

先做窗口逻辑与既有Node文件合同回归，再用准确源码比较同宽度的batch/rolling。
当前priority18轮与prefetch1/2/8批量基线完成之前，不混入它们的计时数据。
只有真实同条件体验/吞吐/公平/背压/成本数据支持，才能考虑带回主候选；单元通过不能替代上述证据。

任何运行前的实现修订：使用最多八槽的VecDeque，每次next在原caller上poll全部pending，
再交出已完成前缀；避免惰性流只消费ready队列而未及时poll新增请求。慢首片测试额外要求
释放首片后，返回下一份已有响应之前新补的第九片已被poll。无独立任务或周期探测。

首轮Mac门：四个窗口逻辑专项、既有files库单测、files_wire真实配对/字节/拒绝合同、file_handoff_wire原resident保留合同；files_wire载荷扩大到12片以上，实际跨窗口落盘。全部只使用原scratch/loopback，不代表真实iPhone或网络性能接受。
