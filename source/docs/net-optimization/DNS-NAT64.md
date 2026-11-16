# 系统 DNS 与 NAT64 边界

2026-11-16静态审计发现当前`crates/net/src/endpoint.rs`仍绕过系统DNS，固定四个公共服务器。源码注释明确这是旧版Apple SystemConfiguration初始化慢的规避；它也明确承认内网DNS独有的域名无法解析。网络DNS64的合成AAAA同样可能被绕过。随后实际夹具已取得产品RED；单改动因果GREEN尚待验证。

当前iroh-dns1.4.1已经使用n0-dns-resolver0.1.0，并公开系统默认、备用nameserver层与网络变化reset。已从准确锁对应的registry源码读取并保存 `E/n0-dns-resolver-010-system-source-audit-v1.json`。Apple实现仍调用SCDynamicStoreBuilder并读Global/DNS，不能仅因更换底层库就宣称旧初始化问题已消失。默认resolver也不覆盖所有VPN scoped resolver；不扩大承诺。

优先验证现成能力，不新增解析器。新 [NAT64夹具](../../tests/net-nat64-coverage-v1/README.md) 使用patchbay已有IspV6转换器；私有DNS提供固定A/合成AAAA，IPv6-only client的UDP nonce必须实际到IPv4 server并返回，双侧确认地址族，系统DNS先能解析，然后才测试原产品冷配对与双向目录。原十秒冷建联、两秒业务期限保持。前置oracle失败不能当产品红。

这是固定DNS64响应，不是动态DNS64算法、CLAT、IPv4 literal支持或物理手机。夹具已按冻结协议编译并执行：`turing-20261116T104129Z-da0f8846`独立网络/DNS oracle全部通过，产品冷配对在原10秒超时（native101/runner1），0目录操作被执行。原件`E/RUN/nat64-cold-red-verification-v1.json`保持。若复现，设计需同时满足系统DNS/internal DNS64、受控public fallback、换网重读，以及Mac/iOS不阻塞运行时与首次建联预算。隔离候选3bd998e8仅去除固定公共DNS覆盖，交回iroh现成系统DNS与fallback；Mac GH43854682670原12组319pass/0fail/ignored1已核（artifact/每个名称/原cfg排除不变），普通Release同fixtureGREEN未执行。canonical未采用。

参考：[iroh DnsResolver](https://docs.rs/iroh/latest/iroh/dns/struct.DnsResolver.html)、[旧Apple初始化问题](https://github.com/n0-computer/iroh/issues/4246)、[patchbay IspV6](https://docs.rs/patchbay/latest/patchbay/enum.RouterPreset.html)。版本结论以上述锁定源码审计为准，不以docs latest别名推断。
