# 交互与文件调度实验

2026-11-11 16:28 UTC，`workload_response_priority_v1`。本协议先于任何本策略计时结果保存。
完整N0–N5与WORKLOAD中的高BDP、共享/分连接、公平推进、规模和手机成本要求保持。
这一步只隔离一个因素，不能以它替代完整N3或最终用户SLO。

## 依据和候选边界

0ff8c054的三轮实际响应流证明目录/文件共享Main，终端是另一Tunnel。
bulk目录native exchange p95约101–103ms，server accept到本地write约8ms，seat等待约10us。
尚不能将剩余区间都叫网络排队；本轮先比较同连接的响应优先级，文件在途量固定八。

Noq 1.3.0的[固定源码](https://github.com/n0-computer/noq/blob/c1f411562e6078852749b8bcf1190523096a107f/noq/src/send_stream.rs#L209)
定义同一连接中较高优先级先发送。优先级要在首次write之前设置；它不是公平调度或跨连接QoS保证。

| arm | PULL_AHEAD | 后续PathSlice响应priority | 首切片/其他响应 |
| --- | --- | --- | --- |
| A：匹配控制 | 8 | 0 | 原默认0 |
| B：候选 | 8 | -1 | 原默认0 |

两份冻结Git树只允许`CONTINUED_FILE_RESPONSE_PRIORITY`这一行不同；全部测试/校验器与控制版本相同。
控制A也经过相同setter/readback代码，隔离优先级值和相关实际成本。它是匹配控制，不冒充未修改旧版。
不增加用户配置、运行时调参入口、连接、请求、定时器、共享map或重试，不改caller/信任/写入/0RTT门。

候选只对`FetchPath`/`SatFetchPath`且offset>0、成功返回PathSlice的response设置低优先级。
首切片保留0不自动证明整个前台文件及时：需要整文件才能展示的预览也可能用后续切片。
首次可用内容和完整前台文件完成时间必须另外验证。持续高优先级流可能饿死低优先级流；
没有饥饿/背压对照前，不将这个候选当作最终产品设计。

## 前置与计时顺序

先验证Mac固定net-node任务、同source实际生命周期readback：
每轮360目录priority0、offset0切片priority0、其余63个切片为该arm声明值，setter成功、实际getter值匹配；
保留双方实际ALPN/wire/stream、16MiB完整性与全部720人口。诊断日志有开销，单独留证。

随后只用`netlab-workload-policy`计时，显式关闭lifecycle/sampling；使用同一冻结control、UID/锁/资源门。
先w3真实拥有者/取消记录guard，再三个Buffered完整场景，失败不删除后续轮，首非零状态保持。
低profile保持v4：单向20ms/每方向8Mbit/40ms队列/16MiB，三阶段各两类120、100ms计划、每类1在途，
原2秒调用/120秒bulk/10秒就绪总界，实际TC读回/探针/完整字节/关闭API门都不变。

固定block顺序 **A、B、B、A、A、B**；每block三个fresh Lab，共18个完整场景、每arm九轮。
三对block按(A1,B2)、(A4,B3)、(A5,B6)报告，block内逐轮保留。开始前冻结源码与control；
若实现/入口失败导致换source，旧原件保持，明确新revision后再开始，不选择性重跑性能差的轮。
源码格式/编译失败不能计作已有计时样本，也不能把不确定观察当成重发理由。

报告每轮全部结果/成功率/错误和超时人口、service与计划到完成n/p50/p95/max、实际payload率、完整文件完成时间。
同时报告bulk整阶段、实际开始时文件在途、完整落在传输区间三个人口，避免用文件已结束后的样本稀释负载影响。
不删失败，不混合各轮分位数当独立样本，不报告p99或显著性。此首轮只估计效果，不凭单一数字自动选胜。

## 后续必须完成的对照

1. 将prefetch1/2/8作为独立因素，不与首轮priority结果混成同一个改动。
2. 除40ms/8Mbit外，覆盖300ms/50Mbit的大BDP，先做独立kernel校准及场景自身TC读回。
   计划高profile使用96MiB、400ms请求计划和预先冻结的240秒bulk边界，保持每类一个在途/完整失败人口；
   它是新模型，不能改写原v4的120秒门。容量/重叠人口与新的模型版本需要实现前再冻结，不提前称已运行。
3. 连续控制流、前台完整文件/预览、后台批量、取消与背压、共享/分连接、Direct/Relay/peer relay/SSH，
   验证批量持续推进而不是靠饥饿换延迟。
4. 1/3/5设备的CPU/字节/唤醒、真机换网与能耗，以及完整集成源码回归。

即使B在此小链路更快，也仍须上述验收、N0最终体验门与决策报告。没有合并、发布或已装App变更授权。
