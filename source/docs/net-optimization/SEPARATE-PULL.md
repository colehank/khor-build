# 单文件独立QUIC实验

准确隔离源dad4306132a51e1807da2058988e13f315b262ac/tree0db6508aa2bd2094b9dfcfee6c35715a0af20f17；默认shared，只在内部实验布局启用一条额外现有Main ALPN连接。目录继续cached Main，终端独立Tunnel，文件走pull所有的Main；不增加endpoint、协议或vendor补丁。保留原deadline/读最多一次retry/写不重放，RAII结束与取消关闭；Weak side覆盖pause/background/peer retire，其他peer不误关。

已闭证：Linux原12组293/0/原ignored1（045906Z-abc0de50），Mac准确同tree便携289/0/原ignored1（GH43563143698），原4Linux-only按源cfg排除，全部7新mode-ON和14contact合同都过。Release050842Z-6b257594为普通features[]/原thin LTO，CLI9add2509af6c3d96d3ac82783bc7bbc9803a931ffca34f21355ee8dc897404ca，netlab4ca36fb23937b0ceecff08c7d5b3b7bac64f2cbe071e79ccc3fa957ff7c0b3bd。未安装或发布App。

固定两场有日志实际负载诊断051918 shared与052331 separate各720ok/192MiB完整，共2256响应stream配对：前者directory/file一Main，后者两Main且pull_wire_opened仅1、与filewire匹配，两者PTY独立Tunnel；采集3校准、CPU/owner/flag finally恢复完整。只证明布局，不用这些日志样本接受性能或能耗。

普通无日志三组已经先冻结plan/control，再fresh分层校准；A原public main、B原型shared、C同ELF separate，ABC/BCA/CAB九块各原low/high，共12960读/18file。原relative10%+20ms/90%文件率、original-main固定absolute、全部caller/file/CPU/geometry/owner门保留。按固定hard-fail规则停止在第7C：10080/10080读、14完整文件，但最后high原生direct_after=false，整体未通过；第8A/9B未执行，不补替代。E/separate-pull-three-arm-stopped-summary-v1.json保持失败，不能从不完整同期对照接受N3。后续独立诊断只补最终Link观测；速度优先是否符合预期与原screen结果分开判断。 具体观测边界与固定复查见[FINAL-LINK-DIAGNOSIS](FINAL-LINK-DIAGNOSIS.md)。

## 采用之前仍需解决

- 当前只测单文件。并发多文件时每pull各一条额外连接，尚无每peer/全局连接上限或多文件公平结论；需要明确共享bulk连接与受限池的取舍，再实测。
- 已建立transfer连接的读失败沿用一次cached Main retry；**初始额外dial失败当前会返回错误**。需要真实cached Main健康而新dial失败/拒绝/期限到达的正负控制，决定同一原期限内的可靠后退；pause、trust拒绝、caller已过期不能借后退绕过。
- 额外握手会增加首次文件数据的准备。当前架构是否值得该成本，只能由完整screen及首次字节/小文件/冷热连接证据决定；不从大文件吞吐推断小文件体验。
- 原286/282便携旧合同默认shared，新7明确separate，不称旧所有path/satellite/resume/错误/取消合同都覆盖mode-ON。生产server资源租约、真实停止与边界还需对齐。
- 需要1/3/5个人设备的CPU/唤醒/闲时字节与连接成本、多负载公平、移动与真实iPhone功耗。现有Turing大量resident线程观察不是根因或移动功耗结论；运行时worker预算只能先测再改。

这份分支仍是实验，不代表最终采用独立连接。若完整screen拒绝，保留原数据/源与原因，重新定位；不能因为组件绿或升级到最新iroh就晋升。

## 下一版准备阶段的约束

当前原型在任何文件slice前等待额外dial，沿用DIAL_MOST=25秒并受继承期限约束。即便cached Main健康，也可能被这项优化前置等待；这是源码风险，不是已观测的生产故障。下一版应先让原有连接在原期限内交付首个slice/metadata，小文件不为额外连接付握手等待，大文件在准备就绪后才接用额外连接。

准备必须随文件取消而释放，失败或饱和不能延长原期限或绕过pause/trust/身份检查；共享bulk连接须服从实际Endpoint的所有权，不能仅按peer身份跨生命周期复用。默认启用前明确每peer/全局的连接、准备和请求上限，并用多文件取消/公平与1-3-5设备成本选择设计，不预先用任意常量当最佳值。预先列出的真实QUIC与负向合同见E/separate-pull-next-preparation-contracts-v1.json，尚未实现；原S与所有冻结产物不改。

首内容准备的后续实现与准确红→绿证据已在独立[PULL-PREPARATION](PULL-PREPARATION.md)收口：Linux294/Mac290及13个窄合同通过，完整采用条件仍未齐；旧S和已停止性能screen不改变。
