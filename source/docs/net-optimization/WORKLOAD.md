# 真实 Node 混合负载基线

2026-11-11，`resident_mixed_workload_v1`。这是 N3 的第一份可复现实验协议，尚未编译或执行；不替代完整 N3 或最终用户体验门。沿用 Linux namespace / scratch resident / 原资源门，独立串行三轮，运行时冻结产品 tree、control、原始日志。产品调度和连接策略不随此实验修改。

固定两个真实 resident，公开 IPv4，配对后先达到 Direct。仅 x 接口双向配置：单向20ms、每方向8Mbit/s、40ms队列、零随机丢包。该配置的实际吞吐尚未独立校准，日志明确 `calibrated:false`；第一轮是测量入口和功能合同验证，不能把名义速率当实际容量、测得差异当拥塞控制收益。

实际调用链：目录 `ls_of → resident IPC → Main`；16MiB文件 `pull_path → resident IPC → FetchPath slices → Main`；终端 `open_shell_on → reach → resident Tunnel → remote PTY`，使用现行 cookie/Version/Ready 协议。**当前终端本来就走独立 Tunnel ALPN，不能假定终端与文件在同一条 QUIC connection，或把创建第二连接当作尚未实现的优化。** 目录与文件是 Main 竞争候选，需随后按 journal/真实 wire 身份复核，而不是只看 UI 路型。

每轮按 idle_before / bulk / idle_after 顺序；每阶段目录与真实 PTY 各120个逻辑样本，100ms闭环调度，总720个。记录每个计划/开始/结束时刻、延迟、调度迟到、成功/错误/超时/前序终端失败导致blocked，原始记录不删除。2秒是实验调用边界，不是用户批准的SLO；终端失败后不隐式重连或把剩余样本丢掉。三轮用于定位，不声称足量p99或消除了顺序效应；前后空载阶段用于观察漂移，最终比较需要交叉顺序与多seed。

终端在 scratch home 打开真实 shell，关闭本地输入回显后 read/printf；请求不含响应前缀，每个响应含唯一phase/index，避免把PTY输入回显、启动命令或旧环形缓冲当成功。整个测试仅向新建scratch会话输入；现有会话和用户resident不参与。协议、流与测量future由父调用拥有，结束显式关闭会话，scratch清理继续兜底host pid。

文件由固定xorshift64 seed `6d697865646c6f61`生成16MiB，接收后逐字节全量比较；不得只以文件大小或请求返回认定完整。bulk最多120秒，记录完整开始/结束/字节/错误。两类交互至少各30个样本完全落在bulk真实在途区间内，否则本轮不是有效混合负载。每阶段前后记录resident wire计数，末尾保留两端journal。日志本身有开销，明确是diagnostic baseline。

首批只判测量完整性、所有逻辑操作成功、真实重叠、文件完整和scratch会话关闭；**不以其functional green声称延迟、吞吐、CPU、能耗或队列设计达标**。后续必须补独立链路校准、完整实际wire/stream身份、真实队列/许可/背压事实、同条件对照与候选公平调度、SSH/peer relay双腿、1/3/5设备、CPU/字节/唤醒及真实手机能耗。缺项保持在 STATE / PLAN 中，不把本协议缩成 N3 的完成定义。
