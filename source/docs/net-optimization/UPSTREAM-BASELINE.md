# 原版 iroh 能力基线 v1

按UPSTREAM-BOUNDARY顺序，先在Turing执行原版iroh1.4.1自带的16个patchbay合同，原断言/内层期限、默认features与发布Cargo.lock不改；顺序单次、失败不替换。源码取已核SHA256的官方crate，隔离worktree为`khor-net-upstream-baseline-v1`，仅vendor/iroh恢复官方逐字文件，原产品候选保持。运行时复制该原版子树到run独占目录，完全离开Khor workspace，禁止本地Noq/iroh patch进入依赖闭包。协议E/`upstream-iroh-native-baseline-plan-v1.json`；执行前固定source/control，沿用原owner/flock/收据和资源门。

覆盖：Relay IPv4/IPv6、三种初始UDP封锁、两端Relay链路断开恢复、Relay同址重启、打洞、两端Direct链路恢复、四种双向跨IPv4/IPv6切换、无WAN局域网换址。原Relay恢复合同允许30秒，与Khor更严格的体验门须分开报告；通过不能证明接管足够快。UDP封锁是初始条件，不能冒称已测运行中只切断Direct而保留Relay。

这是库能力诊断，不是Node/PTY/文件/手机或完整目标验收。所有原失败、完整N0–N5保持；不以此自动清空vendor补丁。原版基线后，再按消费API依赖闭包消减Khor定制，运行真正相同的Khor业务合同。


首轮已完成` turing-20261115T180649Z-ae2a9603 `（原退出101/82.20秒）：原版独立编译20.47秒；85原版文件、默认features、发布lock和全部其余registry依赖闭包确认。16项实际native全部收齐，5项Relay IPv4/IPv6、两端Relay断线恢复、Relay重启恢复通过；11项在Nat::Moderate路由器创建时被nft0.9.3拒绝concatenation map，均保留原native101，不归为iroh业务失败，也不写16/16。66报告文件/identity/全部原进程组清理闭证，E/run/upstream-native-baseline-verification-v1.json。

工具兼容处理不修改iroh/Noq或测试：Turing内核5.15.0-139已具备相应能力。隔离nft0.9.8能声明map，但仍拒绝原DNAT表达式；初次单行校准另有分隔符错误，均在工具准备记录保留。随后仅为本组实验解包官方Debian稳定nft1.1.3及其私有依赖/loader至任务tools独立前缀，发行Packages SHA256与全部二进制hash已固定；原始多行NAT规则在原内核独占user/net namespace实际安装成功。全局nft、系统软件和其他任务工具不变。

补测协议E/upstream-iroh-native-baseline-completion-plan-v2.json冻结原11项且不重复5项成功，沿用同一SHA256 `683f36910627f28284e75297e4308b99c62b8ae3afd48ebf69dcb2efcb1ad304` 原版测试程序，0新编译；仅隔离nft工具变化，断言/期限/网络规则仍逐字原版。不得把v1失败追溯改绿，亦不得把不同工具下的5＋11称一次16全绿。


补测` turing-20261115T181626Z-dafd3754 `已闭：原control退出1/66.74秒，11个实际native均退出0，原生footer全部为1pass/0fail/0ignored；全部owned group清理、52个report文件与准确source/control/工具闭证。第10项`switch_uplink::switch_server_v6_to_v4`成功footer在`73 filtered out`与分号间插入后台trace，导致原control严格整行regex记false；原退出1、原row false保留。独立data-only核对完整counts和native0确认11个合同成功，0重跑、0新编译，E/run/upstream-native-completion-verification-v2.json。不能称原control通过，也不能把两轮不同工具环境相加冒称一次16/16绿。

由本轮实际结果可确认：原版库在这些模型中已具备Relay IPv4/IPv6承载、初始封UDP的Relay兜底、Relay断线/重启恢复、打洞、Direct链路恢复、双向跨IPv4/IPv6换网与无WAN换址的行为。仍需单独检验运行中Direct-only静默而Relay持续可达时的业务恢复期限、交互/文件负载、手机与能耗，不能把“有功能”视为Khor原SLO已过。
