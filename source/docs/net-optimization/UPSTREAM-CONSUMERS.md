# 定制消费边界：原版能力通过后先拆哪里

2026-11-16，准确候选6f7bbf43。图谱仅3个测试文件且stale，production semantic查询0；本轮按允许的定向源码fallback检查。以下为静态调用与行为归属，不是补丁退出结论，未更改产品。

## 已接入上游，并非缺Relay

`crates/net/src/endpoint.rs:595`注册custom transport；686以RoadOrder覆盖默认selector；733给iroh RelayMode::Custom传RelayMap。`Custom`在此表示自选Relay服务器列表，不表示自造Relay传输。796–803另外选用BBRv3并开启observed-address双向报告。812–843组装自家/公共Relay名单，only_mine边界仍需尊重。

## 当前决策比公开selector更深

- `vendor/iroh/.../remote_state.rs:1012–1079`对selector输出再作confirm_move，再传播status；1248–1255每250ms周期读路径、探测、refresh和重新选路。Moving的2秒/4秒/30秒确认与rest是本地规则。只删除endpoint的path_selector调用仍会保留这套底层行为，不是原版对照。
- `crates/node/src/wires.rs:129–135`另有应用连接存活预算，2142–2193接收计数更新、网络变化/静默判断后可主动退役连接。它与QUIC逐路径失效的对象不同，不能未经合同验证直接删；也须查会否先关闭仍可被上游迁移救活的连接。
- `crates/node/src/link.rs:1325–1334`消费自定义relay_departures并传给Wires::departed；13790–13857的真实业务等待又有每beat活性判断、Relay checker和总预算。可达性、业务慢、Relay重连与应用deadline必须分别归属，不能靠多个层次相互重启修复。

## 官方1.4.1没有的10个Endpoint/ConnectOptions方法名

仅比较`pub [async] fn`的方法名，不把文本搜索当完整调用图。候选定义而发布版同文件未定义的方法为：

| 方法 | 已定位消费/职责 | 优先审查方向 |
| --- | --- | --- |
| watch_direct_addrs / holepunch_rounds | journal.rs835/837，网络事实与观测 | 官方watch_addr/path_events能否保留需要的语义；单纯观测优先退出，不能丢网络epoch唤醒 |
| relay_departures | link.rs1325，触发应用退役 | 是否本应由上游连接恢复负责；保留休眠/断开与暂态Relay变化的区分 |
| network_change_major | link.rs2011，恢复/重绑 | 官方network_change及OS通知是否足够，尤其无interface变更的App恢复 |
| check_relay_connections | link.rs13839，业务暴露静默Relay后唤醒checker | 原版Relay恢复实际通过，但静默TCP与主动断线不同，须用原silent合同决定退出 |
| offer_addrs | carry.rs385、hop.rs1424/1681，给现存连接提供新承载 | 官方address lookup更新/custom transport如何即时接入，不以首连成功替代活连接晚到候选 |
| close_carried / close_carried_if | carry.rs375实际用后者，peer relay承载结束 | 优先公开Path::close与应用已有连接集合；保留旧close不能关闭新承载的竞态合同 |
| discard_ip_hints_if_idle | link.rs4239，失败后清过时地址 | 官方地址生命周期能否处理原冷hint故障与迟到结果，不能以新peer已知地址替代 |
| with_initial_custom_addr | link.rs967，承载握手独立初始路径 | 官方ConnectOptions/custom transport能否处理另一活IP连接的共享首选，不许为复用上游丢peer relay |

部分同名调用位于cfg(test)/单独测试模块，未计为实际生产触发。10个方法不对应10项独立补丁；内部选路/Noq改动还需按依赖闭包审查。56待审账目保持，不因静态同名或这轮库能力测试改写为“全部可删/全部保留”。

## 上游仍须验证的快速故障边界

发布版Noq1.4.0 `src/connection/mod.rs:1174–1224`只要仍有validated Available路径，Backup不承载普通Data；PTO增加本身未在这一判据中排除该Available。原版iroh默认selector又按tier/RTT选路。这是静态事实，不能独立推断完整失效时长，因为还有路径超时、网络通知和其他事件。

现有16项原版合同中的UDP封锁在连接前施加，Direct outage用例会恢复接口后再要求ping；它们没有直接证明运行中只静默Direct而TCP Relay完好时能在Khor的原预算内接管。下一对照应先固定这个场景：双向业务证明Direct和备用Relay均真实可用，再静默丢弃Direct UDP而不关闭端口/接口、不发网络变化提示，保持原总期限与业务内容，记录同一连接上的恢复、路径事件和原生清理；原版与Khor使用同一模型与观测，不先给原版加watchdog。反向控制覆盖Relay故障/Direct健康和无故障，后续再覆盖peer-relay第二腿与移动。没有这组证据，不追加本地置信逻辑，也不承诺直接清空底层定制。

## 旧N3失败仍保持

原共享high终端超时轮整形出口TBF捕获drops3881；其他五业务块为978/799/1124/745/931，均为前后计数差（before0）。数据记录E/pull-coexistence-qdisc-review-v2.json。v1数据解析器错用calibration marker得到空集，已显式标invalid，不作为零丢包证据。当前只能证明失败轮丢包较高、存在Direct/Relay往返切换，不能在没有时序细分下确定谁导致谁。原性能失败、数值门与完整样本分母保持。
