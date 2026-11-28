# I8 差异复查

2026-11-16，复查范围为已测试5f51e340到I8 b899669a；不冒充全仓审计或完整N0–N5接受。

按 review-changes 先运行最小上下文、差异、影响流、测试与半径查询。图仅覆盖该worktree两个测试文件，构建点仍是I7，不能从其“0额外影响”推断生产无影响；随后直接核对五个源码/脚本文件的准确diff与原调用。

| 风险 | 结论与证据 | 尚需处理 |
| --- | --- | --- |
| 高 | 本次差异未发现新的信任绕过、写重放或取消所有权缺陷。删除的生产逻辑只有DNS覆盖和冷失败hint清理调用 | 这是限定差异复查，不是全部写family/包级重放的最终接受 |
| 中 | 系统DNS恢复使固定NAT64冷配对实际RED→3次GREEN；但锁定n0-dns-resolver0.1.0同步读取Apple SystemConfiguration。上游#4246仍开放，不能声称旧大目录初始化问题已修复 | 正常打包路径的冷建联/运行时响应应继续实际验证；不得以重新固定公共DNS规避NAT64，也不在无新反例时加另一套解析器 |
| 中 | SSH世界现在只在local客户端写owned Relay配置，far通过已有环境传递，保持no-install空home。Relay由World持有并kill/wait；原认证/签名/member kept0断言保持 | Turing原OpenSSH8.2 broken-jump提示失败保留；准确I8完整Quality43873334184已首个实际run原13/13通过，原Ubuntu24签名SSH151.86秒成功 |
| 低 | 共享取消测试显式shared，landing/resume与原入口一致；原八STOP/5秒/同wire业务保持。原separate专属测试继续证明extra闭合、handler回收、cached STOP与健康 | I8原12组已323pass/0fail/原ignored1；N3性能和手机验收独立 |
| 低 | 两个独立fixture纳入完整Rust格式检查，未增加排除；未修改应用UI或生产Relay配置 | 新fixture是证据入口，不是新增运行中的产品组件 |

hint清理调用删除后，认证仍由TLS目标身份验证，准备承载使用已有每连接initial Custom地址，不清全peer共享选择。跨peer/协议联系代次、caller deadline与读重试许可列表没有改动。旧vendor清理API暂留但已无生产调用；进一步退出要同时记录旧API测试的退役与地址缓存生命周期，不把死接口当作运行时收益。

准确I8原完整网络回归见 `E/turing-20261116T131052Z-bd3bf61a/product-v8-separate-node-verification-v1.json`。DNS Mac319是组件源3bd998e8，net-unit完整73pass/1ignored只用0.36秒；它不覆盖任意巨型binary目录。实际源码审计保存在 `E/system-dns-initialization-source-audit-v1.json`。上游问题：[iroh#4246](https://github.com/n0-computer/iroh/issues/4246)。

当前建议：准确I8完整Quality与native Mac Lab104case已闭；N3扩展场景、真机/成本与完整N0–N5仍未关闭，不建议据本页合并main或发行。后续背景错峰独立差异见[BACKGROUND-RETRY](BACKGROUND-RETRY.md)，不继承I8准确tree的Quality接受。
