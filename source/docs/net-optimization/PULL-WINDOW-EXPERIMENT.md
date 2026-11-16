# 文件预取窗口4已否决

当前结论：保留窗口8，不采用窗口4。两个窗口4场景都违反全部调用成功的原硬门；计划中的六场对照提前结束，不能声称获得完整六场性能结论。N3原交互回退仍未解决。

## 假设与来源

此前同期完整ABBAAB中8640/8640请求成功，但high bulk目录p95从原main520ms升至当前963ms、PTY338ms升至695ms；文件吞吐18.657→24.413Mbit/s不抵销交互回退。应用预取过大只是待检假设，未据此归因BBR或vendor。

E/pull-window4-design-v1.json先冻结；独立experiment/net-pull-window4-v1从DB7bac只改PULL_AHEAD8→4，单片仍256KiB，待传窗口2MiB→1MiB。准确dca27a72edd6d91d710283b24631d3b77294e7f2/tree91b8577e30bddac900f0a8ab4c8d42ba1a672612/snapshot211c9058d966c52ee9804f744522844c83373eca。无CC/队列/fixture/features变化。实验源基于DB，不得从它回退I4的新浏览器锁。

普通Release producer233938Z-8f18bccd已0/550.476037秒闭证，22files/identity/strict native完整，Rust8m19、实际CLI fresh=false。CLI0b0968df21921930f338df819715a91bbafe4e51eb9f74ee71a78ea138df19c1、fulltest5aa21a498f587a8339683f3d777ccf548dc6b838e9dd51f5b2bcd78c875f123e。相对DB只一行常量，反向还原同DB tree；E/run/pull-window4-native-compile-verification-v1.json。

## 原计划与实际执行

E/pull-window-comparison-plan-v3.json / control0a0b87d68bc045c7a4cb7b61db6af60831b57378 / protocol6b29723a1770c91881025e44d82fe9f121bb9de5bd1db8a47955e050e3553d20，原文保留。v1/v2的旧CLI/entry/执行说明残留在任何校准或业务前已修正，未执行准备均保留；实际夹具/顺序/断言不变。

原计划为两臂各5项新校准，再8/4/4/8/8/4六个high场景、4320请求/6个192MiB文件。原2秒caller、240秒文件、400ms采样、owned600秒和CPU/磁盘/来源/清理门不改；不在计时中编译或替换失败场次。原8/4校准004224Z-92e24180与004551Z-ea603d11均17files/identity/owned闭，high冷TCP吞吐原红保持，既有分层准入成立。

| 窗口 | 实际run | 原退出码 | 720次逻辑结果 | 文件完整 | 有效吞吐Mbit/s |
| --- | --- | --- | --- | --- | --- |
| 8 | 004921Z-b43d35b9 | 0 | 720成功 | 是 | 23.925 |
| 4 | 005307Z-492fb1fb | 1 | 676成功、2超时、42阻塞 | 是 | 13.988 |
| 4 | 005746Z-e7e8830f | 1 | 667成功、2超时、51阻塞 | 是 | 10.925 |

三场各13files/identity/原生来源/CPU/owned清理全闭；所有原始720结果均保留。两个4臂硬失败后，root等第三在途场自然结束并完整核验，决定提前拒绝参数支线；第4/5/6原8/8/4三场未发起，明确记unexecuted，无替代。

**提前停止并非原v3预先声明的规则。** E/pull-window4-rejection-decision-v1.json明确为观测数据后的程序变更，progress-v3为stopped-candidate-rejected。拒绝依据是已观察的原硬功能失败；不计算或宣称完整六场pooled p95、统计显著性或因果性能接受。今后的对照须在执行前声明候选硬功能失败后的停止规则。停止此参数支线不暂停完整目标。

## 后续边界

下一假设线索是实际选路变化。E/high-mixed-path-churn-observation-v1.json逐核9场：原main三场selected-change均0、当前8三场2/4/2、另8首场0、4两场6/26。它们是实际path-stream事件，不能当作选择器内部决定输入或包路由；缺少统一wall↔workload时基，相关性不能代因果。相对收益门诊断已先冻E/relative-path-margin-design-v1.json，窗口保持8。

原main工程界限继续保持：high bulk目录p95<=592518.1μs、PTY<=391538.2μs，完整文件median>=16.791371Mbit/s。当前未完成的窗口对照不能通过这些门；最终需要同源、完整人口和明确的设计取舍。

PULL_AHEAD同时用于read_all预览、pull_into和带digest的接收文件窗口。本隔离实验只覆盖192MiB pull_into，未接受预览首内容、offer吞吐、多文件公平、共享/分连接或手机成本。所有旧红和必要验收保持，不将窗口4带入正式候选。
