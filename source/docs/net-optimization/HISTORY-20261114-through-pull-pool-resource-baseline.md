# feat/net 当前状态与接续入口

更新于 2026-11-14T10:34:58.934097+00:00。活动目标“好 完成完整目标”保持active，完整[GOAL](GOAL.md)、[PLAN](PLAN.md) N0–N5及[REQUIREMENTS](REQUIREMENTS.md)均未整体完成。用户已授权实施/远端验证；root串行，不派子智能体。本机source-only，重型工作在Turing/macserver/授权khor-build。未获main合并、现装App替换、发行/TestFlight或生产Relay采购部署授权。

## 当前执行

**当前唯一在途Turing：turing-20261114T105305Z-6ccfbea6**，control39671f145da7109e204bd12c49dd7c0116f73fcd，固定2个owner取消/到期诊断。独立worktree khor-net-pull-pool-resources-v1 / experiment/net-pull-pool-resources-v1；HEADd4f51fda7189255f043465dd1d15e30cb51c4feb/tree204f4e3f139e3d31b36709de73491aa39c1ea913/snapshotaa47b190f37bebe23c8cc09866fbb4d43832173b。相对2e42仅cfg(test)新资源合同，无产品改动。原500ms owner、30ms waiter pending、Incoming drop/refusal与5秒观察不改，只分开waiter Result和server accept，并在清理前记录实际错误；E/pull-pool-waiter-diagnostic-plan-v1.json。先poll同handle，不重派。

**资源基线104608Z-b34b246d已101/102.148122秒闭，1pass/3fail/0ignored**，11files/identity/进程owner/原fmt。E/run/pull-pool-resource-baseline-verification-v1.json。by-value Endpoint clone实际完整文件cached、无extra及真实后续校准已过。容量用例是准确产品RED：4实际extra占满，remote关闭其中1后client确知closed，旧4 Lease Arcs仍保留、另3健康；peer5却返回RetryCached(capacity in use)，无新Incoming；释放后peer5真实握手、全部额度与cached身份/fixture清理控制通过。不能归为不可达或观察器坏。

另外两项owner用例在旧join(waiter,accept)处5秒超时，未分开保存waiter返回值，当前仅为**未分类失败**，不能推断gate泄漏、调用者污染或直接放宽政策；原失败/源码b292/计划保留。当前诊断只改观察，不改产品或原故障。旧27及全部完整目标范围保持。

截至上一轮27闭证时无在途任务。 E=`/Volumes/disk0/Khor/evidence/net`，实际调度以E/source-refs.json、原receipt和远端execution为准。旧STATE逐字归档[HISTORY-20261114-through-pull-pool-red](HISTORY-20261114-through-pull-pool-red.md)，SHA256 `1ba0106d5788903764b87e7c9e4de7aba69182013dfc900f99e807f728a3681f`；其中pending/下一步属于历史，不能据此重派。更早历史和全部N0–N5证据通过该档案及REQUIREMENTS保留，未被本摘要替代。

当前候选worktree **/Volumes/disk0/Khor/worktrees/current/khor-net-pull-pool-contracts-v1**，branch **experiment/net-pull-pool-contracts-v1**，clean；HEAD **2e42b3468f625c0e01fb13c7217c94fc83e64760** / tree **596ebda4f9cc5a5871155481789a75a9f60374fb** / snapshot **8b990071c37b4db7e3d7f3bf0e91ff1d39ec671d**，E/pull-pool-contracts-source-v1.json。原pool基础源码02e和之前eacc保持独立不动。canonical feat/net当前只有本计划记录的更新；没有把实验产品代码直接并入该分支或main。

**最新窄门102700Z-2091c5d1已闭：27pass/0fail/0ignored，0/110.208659秒**，11files/identity/owner/原全局fmt及全部27 test名/结构marker逐项核对。control **c217a678d41377d3c19c54a642a81d11ed77db19**；E/run/pull-pool-contracts-native-verification-v1.json。内容为旧20隔离/准备＋4真实lease合同＋3真实并发文件合同。**不重复27、24或原RED/同测试GREEN。** 当前源尚未跑原标准组、Mac、Release性能或真机，旧eacc标准306/Macfe61不能外推。

## 本轮真正完成的部分

- [PULL-POOL](PULL-POOL.md)记录实际实现/证据与剩余项。候选默认仍shared，只有内部separate模式使用pool。实际canonical Arc<Endpoint>指针归属；每peer一个准备锁和活动共享extra，node4 extra，满额回cached；最后owner关闭extra，不保留闲置extra。retire_peer/clear/let_go/let_go_served与set_ears覆盖主动退役；旧Arc Drop不能关新连接。
- bulk node32/peer8，foreground node8，独立于服务端response预算；请求及乱序ready结果均持有credit直到提交/丢弃。首块及当前cursor受阻时可用foreground，普通bulk可立即取得仍优先bulk。尚未发出的prefetch只在成为cursor时提升，取消的是sem等待，RPC不重发。caller期限在入队前确定且不重置。cursor watch随commit/reset推进，无后台轮询/额外网络探测。
- 4个实际QUIC lease合同：双owner只1握手；drop1不关/drop最后关闭；主动退役时旧Arc保留仍释放permit、新连接不受旧Drop影响；同key新Endpoint隔离；4peer实际extra满时第5无新handshake并后退、释放后正校准。源码02e，095903Z-7033f56c已24/24，当前27保留。
- 两个真实文件：首块均精确提交；实际1extra、8bulk持有且6块真正parsed-ready保留、总10请求（本轮foreground持有0）。两个完整文件各13唯一offset/精确bytes/part与note清理通过；取消A时A held stream真实STOP，B在同一extra上继续，7ready时额度仍持有，B完整完成。最后owner关闭extra，全部bulk/foreground额度归还，cached Main真实Sessions及stable-id保持。
- **准确公平性RED→同测试GREEN**：101209Z-77cafaa4原策略0c7819（只加test/cfg(test)计数），8bulk/7实际parsed-ready由A占住、A head永不释放；B首块成功但5秒只发offset0，原Main健康，清理后才失败。101752Z-6a8e53e2在22863f仅改Node/pool后，**同一完整test module blob9816a50187c2b457651bbad64a8dbf9aec8769d8**通过：B全13块完整，A head仍未释放，所有清理/身份控制保持；当前27该函数尾SHA75234b64cd139366d227ea66b3b1ceae36d05cb03754b2167cf6803145be478f仍同。5秒是夹具观察界，不是生产延迟SLO。

## 接下来的具体顺序

1. 在新测试/源码版本上补全五个真实对端的节点总资源压力：4个peer各8bulk（总32、28decoded-ready）且head保持，验证第5个文件通过cached＋foreground完整推进；进一步覆盖foreground8占满后的新caller过期/取消与释放恢复。预先写清人口和资源记账，再运行；1/3/5真实业务/CPU/字节/唤醒成本仍必需。
2. 实测准备owner过期/取消时另一个未过期等待者不受旧结果影响、prepared连接/permit无泄漏；unknown/by-value Endpoint走cached不发未纳管extra。Endpoint迟到握手/peer policy在队列前后变化继续补。
3. 额外连接远端关闭而旧Lease Arc仍活着时，全局4槽是否及时给其他peer使用尚无证据。当前仅显式退役/Lease Drop/同peer下一次prepare会释放；需真实满额场景确认，不能凭静态猜测宣布修复。可考虑无后台任务的close观察/事件清理，但先用固定合同实证。
4. 上述资源/公平功能收敛后跑准确源原12组。**截止当前源推算313pass＋原ignored1**（eacc306＋pool4＋并发文件3），后续加case前重新按实际源冻结人口，不盲用313。当前源没有标准/Mac结果。需要适当同源Mac验证；不要先做一轮完整标准然后无原因重复。
5. 普通Release构建，先核真实布局再固定新的共存与1/3/5成本对照协议。保留旧三臂screen hard-fail及所有负结果，新门区分真实业务/存活和速度优先，不能强迫Direct当产品目标。之后N1/N2/N4缺项、所有custom收口与最终集成Quality/Mac/phone按REQUIREMENTS继续。

## 不可误用的既有证据

- eacc运输失败回退：090158Z-0ddf1257原标准306/0失败/原ignored1，20明确mode-ON和14contact通过；多数旧case仍shared。旧fe61 Linux294/Mac290（GH43571504228）已闭，不能当新pool验证。
- 旧S三臂screen stopped：7C high direct_after=false，10080读/14文件成功但原组合门失败；8A/9B不运行替代。D独立固定有/无日志各3诊断合计4320读/6文件全成功，0/6复现，是未复现而非修复。不再重复同场诊断。既有4个p6速度优先普通Release合同已闭，也不重跑。
- 095600Z-da6c1c18是compiler-only（Arc<Arc<Endpoint>>旧fixture），0测试；仅修该行后02e的24已过。两个并发case的v1草案在0c7819保存且从未执行；foreground cursor设计后运行前固定v2资源记账，完整文件/取消/限额门保持。原late-file RED与同测试GREEN始终保留。少量本地source准备断言/formatter记录见E，未当产品红或替换试验。
- 图谱对实验worktree为0nodes/edges/files，允许定向源码fallback；risk0/testgap0不能证明低风险或覆盖完整。canonical main图谱不代表这些实验树，不能擅自重索引。

## 环境与授权

MacLab和Turing在同一局域网；macserver是另一网络。iPhone localhost已配对，旧就绪窗口已过；待具体候选可验收时再安排Wi-Fi/蜂窝/休眠与能耗。khor-build公开已由用户明确授权，当前网络控制/源码包可公开；此授权不等于App发行或main合并。其他任务LIVE23961未动。

调度固定：Turing UID1110、原flock/磁盘/工具门，root唯一ledger写者；先stage一次，再launch一次，未知结果恢复原receipt，不因观察超时重派；只用冻结JSON里的真实control SHA。collect原返回码后核identity/owner和真实合同，再闭ledger。上一轮27闭证时全部pending false；当前唯一在途见顶层。

持久计划worktree `/Volumes/disk0/Khor/worktrees/current/khor-net` / feat/net；本机仅Git/stdlib Python源与证据编排，不本地Cargo/Rustfmt/Node/Bun/Xcode/产品执行。重型验证按原macserver/Turing/GH可信边界；main保留9a4219ffe5e3发布基线，builder主checkout HEAD5d3691不改。所有原证据/日志/失败/未执行人口保持，不把组件绿当完整N3或N0–N5完成。
