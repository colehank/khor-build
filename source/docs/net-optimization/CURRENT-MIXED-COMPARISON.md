# 当前普通 Release 与原主线基线的混合负载对照

本轮只测固定网络模型中目录、真实远程PTY和完整文件共存，不代全N3、手机能耗、同步/SSH/hop或任意WAN。原旧I1对照high超时和F3候选单臂high长尾均保留，不能由新的短绿倒推其根因已解决。

协议在新运行前冻结于E/current-mixed-comparison-plan-v1.json，SHA4b1f4b545a7ddfeac4d84ca58105c2eb755ebb44104dde960d5f56cb7af87361；控制fa5f6f6bf0edfb4f1174dea4784eaf9110e13145。原基线c3b098bd经原测试桥构建，treef6c239ae；当前DB7bacfb18/treea2f7a04c的产品代码逐字I3/F3，DB仅新增双端移动夹具。两臂11个workload/observer/calibration文件逐字相同；reduced/full entry均用PublicV4，实际几何另作新校准。完整Release、原features[]、无qlog/诊断生产覆盖、无运行编译；复用闭证ELF与CLI，运行前核源/原收据/选定五文件SHA并仅从原不可变存档恢复。

执行顺序固定：baseline五项preflight、candidate同五项，随后ABBAAB六块；每块low/high各一场，总12场、8640逻辑请求、12个完整文件。每场保留全部起止/失败/缺失/取消、计划到完成与实际服务时间，原2秒caller和120/240秒文件期限不改。每cell每臂预期360样本，有限样本nearest-rank p99不冒充总体尾部保证。原工程回归门仍为candidate p95<=baseline×1.10+20ms、文件完整率全通过且median goodput>=baseline×0.90。root唯一ledger writer，逐块采集/核验后才继续，无后台协调器/替代试次。

新baseline223755Z-939ba0ad：1/166.756497秒，17files/identity/owned全核；两个取消所有权控制、low TCP和独立UDP容量通过，high冷TCP吞吐原断言失败。candidate224138Z-9dcce531：1/172.626989秒，同17files/identity/owned全核；相同四绿一红。依据数据前原MODEL-DECISION分层准入，两臂high的68echo/双向payload/server/实际分离队列和270–420ms RTT及独立UDP40–60Mbit均满足，允许Node业务对照；不把原冷TCP红改绿。两份current-mixed-block-verification-v1.json保留完整原结果，root progress记录为2个已闭preflight。

固定六块已经全部闭证：224503Z-db31994a、225118Z-b47c53d4、225546Z-a7340df5、230128Z-0577ab41、230711Z-84f0dddd、231156Z-0466226b。每块14files/identity/native/CPU/owned已核，原返回码均0；12场8640/8640请求成功、12文件完整，0替代/运行编译。E/current-mixed-comparison-progress-v1.json闭8块（含两preflight），current-mixed-comparison-verification-v1.json保留全部人口/失败语义/原统计门。

**工程回归门未过，不能接受当前候选的完整N3。** Low全部六cell及文件门通过，high空载前后四cell和文件门通过，但high bulk目录与终端均超过原界限。每cell每臂360样本、nearest-rank、全部实际成功，因此这些p95无需剔除超时或条件化；不是总体WAN尾部保证。

| high/bulk指标 | 原基线 | 当前候选 | 原门 | 结果 |
| --- | ---: | ---: | ---: | --- |
| 目录scheduled p95 | 520.471ms | 963.174ms | <=592.518ms | 未过 |
| 真实PTY scheduled p95 | 337.762ms | 695.394ms | <=391.538ms | 未过 |
| 完整文件median goodput | 18.657Mbit/s | 24.413Mbit/s | >=16.791Mbit/s | 通过 |

吞吐提升不能抵销交互回退；两条p95原门保持，当前候选不据此晋升。后续优先定位bulk期间排队/拥塞行为，并做共享/分连接等有假设的隔离对照。Noq版本与当前custom等多分量均可能参与，本对照是整分支效果，不证明具体BBR补丁或其他单点根因。
