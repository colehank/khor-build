# 正常优化的收益确认

2026-11-16，进行中。用户确认“维持健康主路，正常优化更稳，故障接管更快”；本页仅处理正常迁移的波动缺口。原共存终端超时与切换的因果尚未证明，原N3数值门与失败保持。

同一代表副本已有最多8个、2秒内的ACK RTT样本和对应发送/收到时间。旧普通Moving只确认时间与两侧决定后的样本数，即使候选中位数落在当前路已观察到的波动内仍迁移。提议在普通非far迁移最终Go前，比较决定后样本的候选上界与当前下界；严格分离才确认收益。无新增数值阈值、窗口、计时器、探测或公开API，分配为零。故障leave、远快fast branch与当前RTT超过确认期限的原例外保持。

准确RED `c09851c5`抽取现有测量→Moving入口，行为等价，加入5个固定合同。`turing-20261116T034715Z-c940ffb7` 原101/39.883秒，1红/4绿：当前[280,340,320,330]ms，候选[290,310,300,300]ms，原实现仍Go；收益清晰、远快、当前极慢及故障控制都过。全部原始日志/身份/锁/209个native测试列表/owned group独立核验，原失败保留。

GREEN `5eea07cd` / tree `d1d0a1b8`，仅在普通确认条件加入同代表副本/决定后样本范围；现有内部Moving合同恒真admission用于保留原状态机合同，新5个生产边界合同源码逐字不变。完整库 `turing-20261116T035134Z-5342f6de` 原0/90.102秒、208通过/0失败/原ignored1，209个test名与每项结果独立核验；五个新合同逐字不变。

本次不是新的全网健康系统，也不恢复Direct硬优先；近似体验Direct优选仍须在selector层独立完成。库合同不能替代真实路径、Node、性能、跨网与手机。E方案 `normal-gain-window-plan-v1.json`，原失败 `turing-20261116T034715Z-c940ffb7/normal-gain-verification-v1.json`。

## 物理故障回归

仅追加逐字相同的旧公共夹具后准确源 `7d9f1c07`/tree `cb045c42`，`turing-20261116T035501Z-5c11c26d` 原0/64.030秒/94报告，原5场景全通过。UDP静默client/server同一原QUIC连接恢复1250.410/602.541ms，原2秒门保持；独立Relay-only真实回复106–115ms，实际UDP丢包counter和认证前置成立。健康3.102ms，反向Relay TCP规则故障下Direct10.682/15.526ms；反向场景未主动发负向Relay请求，不能因此接受Relay探测故障。

锁文件、非根features与唯一依赖边对原C完全相同，新增行为位于原同版本iroh vendor，夹具逐字不改。单次固定诊断不代表尾概率或所有网络。证据E/run/normal-gain-transport-verification-v1.json；原A/C/B与Noq移除失败全部保持。

## 原生应用控制未通过（2026-11-16）

普通Release producer `040019Z-6ccd73cf` 原0/602.617秒，准确source7d9，CLI63f14b10与netlab3638a513；身份/原fmt/严格native_provenance及真实新CLI行已核。新source修改vendor，旧317/314不当作新源Node回归。

原4控制 `041241Z-fb745707` 为3通过/1失败，原退出1/128.772秒：两个p6快路相反场景、p7 typing过；p7 idle实际观察20.199002777秒，超过原20秒，无业务errors，两端journal记录second@18s。原owner/CPU/完整结果保留。普通gain未接受，后续preflight5/high3未执行。下一步只用两个已存ELF/原夹具做固定配对，判断是否存在当前源特有退化与选路/应用观测的时间差，不改变20秒门，也不以新成功替换原失败。

数据核验v1误将native负例设为analysis_invalid；v2接受完整原footer101与明确p7i红、保留行为失败，仅修正数据有效性。两个proof都保留。
