# iroh 稳定版升级与补丁退出

2026-11-12，按用户U18实施。隔离树`khor-net-iroh-upgrade-v1`，分支`experiment/net-iroh-upgrade-v1`，运行时基于已验整合候选dc73837e；canonical feat/net仍未升级。

- 官方稳定iroh1.4.1，配套Noq1.4.0，发布源码来自已保存且SHA256匹配的crates.io包；源清单在E/upstream-20261109/source-manifest.json。
- 以旧发布包为共同基线三方迁移，保留原专属回归，并保留新版阻塞Relay发送不阻塞Direct Initial的回归。E/iroh-upgrade-source-merge-v1.json记录逐文件动作和7处冲突。
- 冲突决议：保留本地速度选路与上游Initial发送任务并存；保留原pending队列硬上限与path-ID释放事件，未以简单dedup冒充等价；Noq统计保留本地公开遥测；vendor iroh锁中Noq-proto继续本地来源。
- 补丁22的IPv6字面地址QAD改用上游relay_tls_server_name；域名/IPv4/IPv6的上游测试与真实IPv6 QAD测试保留。此为源码替代，尚待新源行为验证才记退出完成。
- 其余补丁先作为迁移候选保留，逐项按UPSTREAM/VENDOR-AUDIT原合同决定复用、必要移植或删除。不把三方merge成功当语义验证，也不把原60项全部标成仍必要。
- workspace、net/node的iroh及直接Noq测试依赖统一要求1.4系列；锁文件必须由Turing生成，禁止本机Cargo。
- 准备轮允许远端生成锁/formatter patch并报告编译不兼容，产物不算冻结原源码验收；应用补丁后再冻结源码、运行原回归。不得修改原性能冻结源或原失败日志。
- 完整N0–N5、速度优先、完整peer relay两腿、移动IPv4/IPv6变化及无需重启自动接管要求不变。当前没有发布、main合并、App替换或新真机验收。
