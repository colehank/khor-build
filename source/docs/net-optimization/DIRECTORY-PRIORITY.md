# 目录与首块文件的发送顺序实验

H9acc原三场high2160交互全成功、3完整192MiB文件与6post-health通过，但bulk目录scheduled p95为613863/471558/342102μs，首场超原592518.1μs。不是N3接受；全部原数据留存。

同H普通Release一次lifecycle诊断065748Z-b5ef24fa实际720业务及完整文件成功，采集器因普通CLI没有net-diagnostics初始化marker而native101，保存x后未保存y。原失败保持，不把它作为普通性能或完整双端诊断。已有x记录361目录回复、768文件回复，共192MiB；最初offset0/262144两个256KiB块和全部目录走同一Main，余766文件回复走唯一另一条连接。前两bulk目录603622/598904μs，首块1211281μs。支持首块调度假设，未证明全部后续~950ms卡顿的唯一根因。

隔离实验 `/Volumes/disk0/Khor/worktrees/current/khor-net-directory-priority-v1`，9e3e06092d82f12bc4f95bda401562a93592670e / treebbf26bf09beb8850e4e10a1a76b96992d5937ace / snapshot5d905075d8cc1a739ff94b13394804240b26cba5。只改link.rs：有效Ls/Dir响应第一次write前设相对QUIC priority1，文件和其他响应保持原优先级；无新任务/计时器/探测，pool/窗口/首内容准备及原期限不改。相对优先级不保证跨连接或网络QoS，持续目录负载的文件公平推进要实证，尚未采用。

冻结计划E/directory-priority-plan-v1.json，普通Release编译控制b30647868f666f4a02ed8f3856df79317ffdd830，只迁移准确1文件并保留H原ELF；Mac用原control68cadb57完整12组316＋原ignored1，同32明确separate/14contact合同，不改测试名或门。

下一步收齐Turing070611Z-f9eb7a56编译与Mac43821604934；若同源/回归/owner有效，原high固定3次普通Release方向筛查，任何原失败保留。公平性、所有负载/1/3/5成本、最终同期main对照与N0–N5仍缺，不把此小实验缩成完整任务。

## 已闭结果与不采用决定（2026-11-16）

X普通Release556.403秒完成，Mac316/0/原ignored1、32明确separate/14contact全过；原high三场2160读/3文件/6post-health成功，逐场目录p95为614088/336144/315554μs。原协议实际评级是合并360样本：H448002μs、X353557μs，均过592518.1μs原绝对参考；H/X有限人口p99为836037/751934μs。逐场614ms保留；先前逐场参考解读过严，未改原协议、门或失败分母。证据E/current-high-pooled-verification-v1.json。

无同期配对，不能证明目录priority单独收益或必要性；为减少custom，最终候选J恢复H原link.rs，不采用此实验。X原源/产物/全部结果不删。完整共存/公平性/设备成本与最终同期main比较仍要验证，但不再因这条已过的pooled参考追加调度补丁。
