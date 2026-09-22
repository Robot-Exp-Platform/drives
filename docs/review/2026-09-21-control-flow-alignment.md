# ControlFlow 与驱动工作区对齐记录

开始于 2026-09-21，完成验收于 2026-09-22。此记录覆盖此前审阅快照后的实施；不是冻结版本或真机认证。本轮用户授权所有必要修改及本地提交，明确不推送。

## 已确定并实现的契约

- robot_behavior 将 roplat 设为可选 feature；默认能力库的 normal 依赖树不含 roplat/Tokio，Tokio仅用于开发测试。Node 的 Error 移入显式Lifecycle，默认hook不做设备启停。
- ControlWith 规范入口为 `control_with_flow` / `control_with_flow_async`；旧 `control_with` / `control_with_async` 便利包装继续可用。驱动实现者需实现规范入口，应用tuple闭包不需改写。
- `Continue((command,false))` 继续，`Continue((command,true))` 发送有效最终命令后正常结束；`Break(())` 没有本周期算法指令，执行设备协议收尾。保留0.6阻塞会话/async周期闭包，不引入spawn、频道或新调度模型。
- ControlRhythm 的 Input=RobotResult<R>、Yield=(Obs,Duration)、Feed=(Command,bool)、Output=R。完整drive返回Execution<R>及N；域失败时先收回N再Break，设备失败也进入Execution错误。
- 成功Feed按周期提交：本轮计算中请求外部停止但返回Completed(feed)，仍提交该有效指令，下一callback前观察停止；若done=true，则正常Completed(R)。需要立即不发指令的域必须返回Stopped/Err。
- 对象由创建层启用/关闭；多次drive不默认刷新。N在合作退出时归还；Input中的设备不在N内，失败/停止不保证设备本身返回。panic、丢Future、不返回的驱动不属于合作保证。
- 域单错保持原始RoplatError。设备错误以`Io(Error::other(ControlSessionError))`承载可downcast的完整设备错及可选原始域退出；无需修改roplat核心。设备执行与收尾双错由RobotException::ControlSession保留。包装只在错误路径分配。

## 驱动与生态修订

Franka六类通道和Jaka两类通道共用flow规范循环。Jaka Break进入servo退出，done仍发最后有效指令；传输/UTF8/JSON错误改为返回，保留主错与收尾错。Franka修正StopMove独立状态枚举及v5/v8序号、TCP分帧/命令编号路由、夹爪10字节头兼容；取消路径用3秒**总网络等待期限**，恢复socket设置。未确认终结时保留motion ID并拒绝再次开会话，要求重连。该期限不是物理安全停止期限；正常周期的网络超时策略不在此次重设计。

Franka负载惯量另作独立纠错：两体惯量应合并到总质心而不是停留在法兰原点；修正旧测试错误的简单平均质心/惯量期望，补零质量、单体非原点、平移与交换不变量。依据本库Model接口与本地官方ref/libfranka，非新设计偏好。

ExRobot 17类能力接入flow；Go2两个可选节律机械对齐Execution并保留类型化设备错误；SimRhythm增加新协议及合作退出，既有步进错误忽略和计时策略留作后续讨论。Hans状态读取移除panic。保留此前Jaka解析/Go2 CRC/GUI测试ignore小修。

Jaka多语言样例改为有限PlanningRhythm + System DSL，完整规划后才调用move_traj。测试实际运行Rust、Python、C++节点，修正同一Python消息以两个模块名导入导致ctypes对象身份不同的问题。生成工具对齐本地roplat_build0.2.2，仿真依赖移至dev。

行为库Python/C++旧接口示例更新；清理明确未用依赖与lint，PyO3派生显式保留既有FromPyObject行为。中英文档、两份AI技能同步，skills按skill-creator规范验证；不安装插件。根AGENTS纠正旧事实并保留设备安全约束。

构建支持显式跳过Franka/Jaka/Bullet资产导出；Bullet不再删除已有用户资产目录。去掉无gitlink/无目录的franka_letters残留声明。第三方ref/libfranka、ref/libfranka-rs、utils/copp、utils/topp仅将既有.DS_Store规则移入本地info/exclude，维持上游commit；不制造不可获取的父gitlink。未初始化的common/主题子模块不因整理而下载，LFS大数据未拉取。

## 验证范围

| 项目 | 结果 |
|---|---|
| cargo check --workspace --all-targets --locked --offline | 通过；16个实际成员，默认feature，不执行例程 |
| robot_behavior 默认无feature check及normal依赖树 | 通过，normal依赖不含roplat/Tokio |
| robot_behavior --features roplat --lib --tests | 3原有单元 + 4回调 + 13节律/System，合计20项 |
| robot_behavior --all-features --all-targets check / Clippy -D warnings | 通过；nightly内部trait solver提示仍存在 |
| ExRobot ControlFlow / all-target Clippy -D warnings | 4项通过 / 通过 |
| Go2 roplat_execution / command_routing / feature Clippy | 7 + 32项通过 / 通过 |
| Jaka --lib | 10项通过，含真实本地servo协议模拟 |
| Jaka多语言有限规划/System | 5项通过，含实际Rust/Python/C++节点；real/sim examples仅编译 |
| Hans --lib / K1 safety | 8 / 12项通过 |
| Franka默认与fci_v8 --lib | 默认27/27、fci_v8 27/27全部通过，无跳过；含新增协议13项及惯量5项 |
| Bullet资源导出 | 实际函数临时目录验证，已有文件不覆盖，缺失目录导出；不运行GUI |
| 文档/skills | 两技能quick_validate通过；提交后原配置MkDocs --strict通过（含git-authors） |

Franka/Jaka协议测试仅绑定127.0.0.1，未访问真机。最初沙箱不允许本地监听会产生EPERM；允许loopback后重跑通过。这种环境失败与语义失败分开记录。全仓check的首次生成阶段因不能写仓目录EPERM，禁用资产导出并取得已授权仓目录写权限后通过。

原生SDK、Windows/Linux、真实设备、GUI和模型资源不是本机测试覆盖。全仓Clippy仍有既有warning（manifest建议、nightly兼容及部分旧代码lint），不称为全仓零警告。后续Rust兼容提示涉及binrw/re_data_loader/re_video依赖，需另行升级验证。

复现脚本：[check-core-alignment.sh](../../scripts/check-core-alignment.sh)。macOS本机多语言运行用`DYLD_LIBRARY_PATH=/tmp/drives-python-runtime`，该临时目录只含指向已安装libpython3.14.dylib的链接；不要把整个Conda lib加入以免影响cargo的libiconv。其他主机应按本机Python路径配置，不能照抄临时路径。

## CPU性能对照

见[环境参数](2026-09-22-control-performance-environment.json)、[第一次摘要](2026-09-22-control-performance-1.csv)、[第二次摘要](2026-09-22-control-performance-2.csv)。同一优化二进制，每配置21组交替先后顺序的旧/新配对；每样本100万周期，先4次预热，重复整套两次。表取第二次的批均值中位数，单位ns/周期。

旧仓无法与当前核心直接编译，故按be5974bf旧代码形状重建泛型legacy_async/drive；该限制已写入bench，不声称完整旧仓二进制比较。探路版本曾把旧路径简化为inline async block，已弃用，其结果未纳入正式数据。

| 路径 | 命令字节 | 旧 | 新 | 变化 |
| sync_tuple_vs_flow | 48 | 1.090 | 1.090 | -0.02% |
| async_tuple_vs_flow | 48 | 3.003 | 5.748 | 91.43% |
| legacy_rhythm_vs_execution | 48 | 7.077 | 8.952 | 26.49% |
| sync_tuple_vs_flow | 56 | 1.341 | 1.336 | -0.39% |
| async_tuple_vs_flow | 56 | 3.028 | 6.490 | 114.35% |
| legacy_rhythm_vs_execution | 56 | 7.466 | 9.482 | 26.99% |
| sync_tuple_vs_flow | 1024 | 11.114 | 11.099 | -0.13% |
| async_tuple_vs_flow | 1024 | 26.859 | 26.691 | -0.63% |
| legacy_rhythm_vs_execution | 1024 | 24.198 | 26.600 | 9.93% |

同步便利接口在本机测量中基本持平；48/56字节的异步便利包装增加约2.7/3.5ns，比例约91%/114%；完整节律增加约1.9/2.0ns（约26%/27%）。1024字节完整节律增加约2.4ns（约10%）。这些比例**没有满足所有路径低于10%**，不能掩盖。两次重复方向一致，原始极小基线使相对比例敏感，未给出置信区间。

从结构可定位的新增工作是async便利包装的额外Future适配、ControlFlow结果判别、Execution状态/节点保存，以及ControlRhythm周期开始的停止检查和context.clone。绝对成本不应外推到真实机器人端到端时延；没有汇编或硬件计数器证据，不归因为缓存命中率。正常通用成功路径无新增错误包装分配/锁/线程；Franka TCP控制平面新增帧Vec和队列锁，不在正常UDP周期热路径；整个驱动不应称为零分配。

本实验没有逐周期CDF/p99、Python/GIL、旁路通信或跨线程吞吐量，因此不代替此前通信实验或服务器测试。本轮先落定语义与正确退出，不作性能冻结承诺。后续可针对async包装与具体驱动节律单独优化。

## 剩余工作与提交边界

- Franka local Tokio runtime在已进入Tokio上下文中可panic；需要另行讨论能保持0.6契约的执行上下文方案。不能将Mock System验收视为此问题已解决。
- 真正异步会话、公平轮询、状态有效性/新鲜度、输入设备失败归还、各设备截止时间/物理收尾保证、仿真语义仍需讨论。未擅自推进这些设计。
- 官方SDK/多平台/真机验收、历史lint与依赖兼容清理、完整Python部署包和C++应用链接测试仍需继续。当前外语行为库示例check不等于部署验收。
- 本轮仅本地提交；根引用尚未推送的受管子仓commit，是本机检查点。以后发布必须先推子仓，再推父仓。


最终验收摘要见 [verification.json](2026-09-22-verification.json)，已执行152个测试实例（含Franka两个协议配置的重复覆盖）；不是152个独立功能。子仓提交与分支见 [child-commits.json](2026-09-22-child-commits.json)。本机/tmp日志用于追查，跨设备应使用仓内命令和摘要复验。
