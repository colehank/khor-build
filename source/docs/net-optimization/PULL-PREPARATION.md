# 文件连接准备：先交付内容

首内容修复基线源码7542ea8960c2bd35df534c437f7d05ddd0b69cdc/tree fe61be7532678bf40cce65d30f1286f51b4e815b，worktree khor-net-pull-preparation-v1。默认仍原shared，只有内部separate实验模式使用此设计；没有修改vendor、ALPN、身份、原期限、写入重放边界或已安装App。

## 问题与行为

上一版在任何文件slice前等待额外QUIC握手，沿用DIAL_MOST=25秒并受继承期限约束。真实QUIC新回归070345Z-658df53f证明：主连接健康且仍能以原stable-id响应Sessions，而没有额外握手acceptor时，5秒观察内没有首个完整slice。夹具先取消实际pull、验证缓存连接、完整关闭后才按新断言失败；这是准备阻塞的直接证据，不是先前混合场最终路径状态失败的根因。

修复先在主连接交付并提交首slice。文件已完整便结束，不启动额外准备；大文件的准备future由pull直接持有，与真实主连接请求并行。准备阶段先保留1个在途请求，随后一条缓存请求完成或握手结束时扩到原8窗口；已有pending和乱序完成结果保留，不重放切换前请求。额外连接就绪后，新请求持有该连接的Arc；最后owner释放才关闭额外连接。取消同时释放窗口和准备，没有分离任务或新增探测计时器。

此fe61基线只有明确的准备TimedOut后退到原缓存行为；Refused、Asleep、CallerExpired保持终止。后续eacc候选已将具体QUIC运输故障单列并用准确RED→同测试GREEN验证；详[PULL-FALLBACK](PULL-FALLBACK.md)，标准回归状态见STATE。未知connect/auth错误仍保守终止，不称所有额外连接失败都透明后退。原暂停/peer retire的Weak side跟踪与按请求期限保持。

## 已闭证的范围

- 准确RED新用例：1失败，主连接和清理正控制通过，产品当时未改。
- 同一新测试及真实QUIC基础fixture逐字相同，072300Z-4cf5e570修复后8个隔离与5个Window合同全部通过。新增首内容已成功；全部唯一文件offset/精确bytes、实际额外连接、取消STOP、缓存身份、暂停、一次读retry和写不重放仍有强断言。两项完整pull fixture依预先设计允许缓存首块及切换前在途请求；原S分支不改。
- Linux标准12组072757Z-352286a4：294通过、0失败、原忽略1；Mac同完整源码GH43571504228：290通过、0失败、原忽略1，4个原Linux-only按cfg核验。8个明确模式ON及14个contact合同都过；原大部分旧合同默认shared，不称全部旧场景模式ON覆盖。
- 源、全部控制blob、owner清理和Mac artifact大小/SHA/CRC、host与原资源准入门已核。独立Window5与标准组有8个isolation重叠，不双计为新样本。

5秒是单元夹具观察界，不是生产延迟验收；这些结果不构成吞吐、能耗、移动网络或完整N3结论。详细proof与当前下一步见STATE，E/pull-preparation-*-v1.json保存原始计划和核验。

## 采用前的剩余项

小文件/暂停/过期/取消8项边界已在59b源16/16通过；后续eacc源20/20还补实际协商失败后完整读完及真实拒绝/应用/本机关闭终止。每peer/全局连接和准备上限、多文件公平、实际Endpoint生命周期隔离，以及更全面准备故障/期限覆盖仍需完成。其后才用发行配置做首字节、布局、低高profile对照和1/3/5设备成本，最终同源完整质量和iPhone换网/功耗验收保持。
