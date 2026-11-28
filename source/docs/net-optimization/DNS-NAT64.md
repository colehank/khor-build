# 系统 DNS 与 NAT64 边界

2026-11-16静态审计发现当前`crates/net/src/endpoint.rs`仍绕过系统DNS，固定四个公共服务器。源码注释明确这是旧版Apple SystemConfiguration初始化慢的规避；它也明确承认内网DNS独有的域名无法解析。网络DNS64的合成AAAA同样可能被绕过。随后实际夹具已取得产品RED；单改动GREEN已按原夹具连续三次验证。

当前iroh-dns1.4.1已经使用n0-dns-resolver0.1.0，并公开系统默认、备用nameserver层与网络变化reset。已从准确锁对应的registry源码读取并保存 `E/n0-dns-resolver-010-system-source-audit-v1.json`。Apple实现仍调用SCDynamicStoreBuilder并读Global/DNS，不能仅因更换底层库就宣称旧初始化问题已消失。默认resolver也不覆盖所有VPN scoped resolver；不扩大承诺。

优先验证现成能力，不新增解析器。新 [NAT64夹具](../../tests/net-nat64-coverage-v1/README.md) 使用patchbay已有IspV6转换器；私有DNS提供固定A/合成AAAA，IPv6-only client的UDP nonce必须实际到IPv4 server并返回，双侧确认地址族，系统DNS先能解析，然后才测试原产品冷配对与双向目录。原十秒冷建联、两秒业务期限保持。前置oracle失败不能当产品红。

这是固定DNS64响应，不是动态DNS64算法、CLAT、IPv4 literal支持或物理手机。夹具已按冻结协议编译并执行：`turing-20261116T104129Z-da0f8846`独立网络/DNS oracle全部通过，产品冷配对在原10秒超时（native101/runner1），0目录操作被执行。原件`E/RUN/nat64-cold-red-verification-v1.json`保持。若复现，设计需同时满足系统DNS/internal DNS64、受控public fallback、换网重读，以及Mac/iOS不阻塞运行时与首次建联预算。隔离候选3bd998e8仅去除固定公共DNS覆盖，交回iroh现成系统DNS与fallback；Mac GH43854682670原12组319pass/0fail/ignored1已核（artifact/每个名称/原cfg排除不变），普通Release同fixtureGREEN **turing-20261116T114034Z-0988b52f**原0/35.977s，固定3次fresh native、6目录全成功，原10秒冷配对与2秒读门不变，原fixture两文件逐字相同。三场配对＋两次读的总时间1.245/1.242/1.260秒（不是纯pair延迟），各读27.976–31.059ms；每场独立无IPv4、SIIT nonce与系统AAAA oracle全过。证据`E/RUN/system-dns-nat64-green-verification-v1.json`。这是固定DNS64/NAT64场景因果修复，非真实移动网/手机全N4接受。I6已纳入候选，canonical尚未采用。

参考：[iroh DnsResolver](https://docs.rs/iroh/latest/iroh/dns/struct.DnsResolver.html)、[旧Apple初始化问题](https://github.com/n0-computer/iroh/issues/4246)、[patchbay IspV6](https://docs.rs/patchbay/latest/patchbay/enum.RouterPreset.html)。版本结论以上述锁定源码审计为准，不以docs latest别名推断。


夹具链接实际在`112435Z-5e68985a` Cargo成功，但控制器校验时未清除fixture专用`CARGO_BIN_EXE_khor`而被原guard拒绝，0业务。`113814Z-f285349f`只恢复原product环境并证明唯一差异就是该变量，原provenance完整通过、0编译，才接受已经链接的ELF **305004bd75c99cbf6f5fd936ba47c4e89b9d47aee01aa9127038fea16369b390**。两阶段的原runner1/runner0与各自职责保持，未重新编译或改写原拒绝。
