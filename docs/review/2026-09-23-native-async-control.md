# 原生异步控制与独立依赖边界

本轮依据作者确认的方案实现：保留独立驱动、同步 move_to 和 0.6 阻塞 async 回调入口，新增由外层运行时推进的完整异步会话。代码与验收完成后推送 GitHub；不发布 crate、不打稳定标签、不合并 main。

## 执行契约

`AsyncControlWith<S>` 提供 `control_native_async(&mut callback)`，返回完整会话的 Send Future。`AsyncControlCallback::call(&mut self, obs, dt)` 可以借用控制器内部状态跨 await，返回 `ControlStep<Command>`；普通 FnMut 到 Send Future 有便利实现。基础行为能力不依赖 Tokio 或 roplat；驱动声明自己的运行时要求。

roplat feature 下新增 `AsyncControlRhythm`。业务端口与旧 ControlRhythm 相同：Input=RobotResult<R>，Output=R，Yield=(Obs,Duration)，Feed=(Command,bool)。合作退出归还 N，先完整等待在途域；创建层管理生命周期；成功 Feed 按本周期提交，停止在下次 callback 前观察。Input 中且不在 N 中的设备仍只在 Completed 时返回。panic/drop/abort 不属于合作关闭保证。

原生节律不建立运行时、不 spawn、不增加每周期 Box 或节点复制。借用回调在驱动会话返回后仍由调用方持有。AsyncControlRhythm 的实现显式返回 impl Future+Send，以避开当前 nightly 对 async fn 实现的借用生命周期推断问题；真实两层 System 的整图 Send 与 spawn 回归用例防止误改。

## Franka 双内核

同步 move_to/control_with 继续调用 std_udp。旧 control_with_async/control_with_flow_async 保留内部运行时和阻塞整次会话的契约，仍不适合直接在外层 Tokio task 中调用。新原生入口使用应用 Tokio 的 I/O 和 timer；六种控制通道与旧路径共享协议和状态转换。

异步 TCP 会话启动、UDP 状态与命令、正常结束和取消/失败收尾使用异步等待。异步内核由共享会话对象组织，旧 async FnMut 和新借用 callback 仅保留各自很短的调用循环。私有 Network 由独占对象持有，避免跨 await 持同步锁；没有增加中转线程、队列或第二个设备 UDP 端口。

TCP 保持完整分帧、命令 ID 路由和待处理终态回复；取消使用覆盖协议网络等待的总期限。套接字非阻塞模式在正常/错误及 Future Drop 后恢复；未确认结束的运动与未完整消费的 TCP 操作不能假装可复用。Drop 恢复本地状态不意味着已经物理停止。连接建立及轨迹规划不属于本轮异步会话范围。

## Feature 与来源

robot_behavior/roplat、libgo2/roplat 默认关闭；本轮补齐 rsbullet/roplat，对 SimRhythm 的模块和导出一起门控。RsBullet 其他异步运动仍需 Tokio，不能简单把整个 Tokio 依赖删除。基础驱动没有自己的图适配时不添加空 feature。

独立子仓保留完整 Cargo 声明，不继承外层 drives 工作区。已发布 roplat 0.2.2 与当前内部 API 不同，robot_behavior 0.6.0 未发布，因此内部依赖使用 GitHub 固定提交；集成根对相同 Git source 做 path patch。实际版本、来源和 package identity 一并校验。详见 [依赖说明](../dependency-sources.md)。

为使 Cargo 能检出 roplat Git 依赖，另建 codex/git-dependency-packaging：b47f7b6aa54a230417331e4b85cefcc8eff9e128 仅移除历史 cmake-gen 失效 gitlink；Rust 源码与核心 c595393df441056a25b2af8bedfcbd0f598b3fec 相同。该记录没有 .gitmodules URL，且当前构建不引用它。Go2 的真实独立 Git 构建用于验证修复。

## 验证结果

本机有限测试全部通过；没有连接机器人、运行 GUI、访问厂商 SDK 或修改 roplat-exp。

| 检查 | 结果 |
| --- | --- |
| workspace `check --all-targets --locked --offline` | 通过 |
| workspace `clippy --all-targets --locked --offline` | 通过；历史 warning 仍存在，不称全仓零警告 |
| robot_behavior 默认与 roplat feature | 默认 3 项；启用 roplat 后 31 项；all-features/all-targets Clippy `-D warnings` 与严格 rustdoc 通过 |
| Franka 默认 / `fci_v8` 协议 | 各 42 项，性能样例默认 ignored；最终默认 release 也通过全部 42 项 |
| Franka all-features lib/tests Clippy | 通过，保留 6 条历史 warning |
| JAKA / Hans / ExRobot | 10 / 8 / 4 项 |
| Go2 command routing / Execution | 32 / 7 项 |
| K1 离线 safety | 12 项 |
| Rust/Python/C++ System 示例 | 5 项，包括实际三语言图执行 |
| robot_behavior_page / 两个行为 skill | MkDocs `--strict`、skill 校验通过 |
| 依赖 feature 矩阵 | behavior、Go2、RsBullet 各基础和 roplat 配置，以及 roplat_rerun 集成 check 通过 |
| 独立源码构建 | behavior、Go2 基础与 roplat；Franka、JAKA、Hans、Aubo、ExRobot、K1 基础配置通过 |

独立检查使用原始 manifest、真正的 Git 依赖获取，不借用 drives 父 manifest 或 sibling path。部分检查以集成 Cargo.lock 播种以固定 registry 版本；这是独立源码/来源验证，不是全新无锁解析验收。Go2 另核实 core 来自 Cargo Git checkout，默认普通依赖树不含 roplat。夜间版工具链、既有原生库和本机平台是验证前提。

RsBullet 两种 feature 和 roplat_rerun 已通过根工作区构建。完整独立 RsBullet checkout 未另验；独立 roplat_rerun 的 Cargo 在递归获取 Bullet 时触发 240 秒时限，未取得完整构建结论，已停止进程。不能据根工作区通过就宣称这两项独立构建已验证。

Franka 用例包括：真实 `System → AsyncControlRhythm → Franka`、整图 Send/spawn、原 N 对象地址归还、创建层生命周期；同一 current-thread executor 上在 TCP 启动、UDP 等待、正常结束、Stop ACK、idle 状态与取消终态等待期间推进兄弟任务；同步→原生异步→旧阻塞 async→同步；非 Send 旧闭包兼容；网络错误、双重报错、清理期限、socket 模式恢复与未完成 Future Drop 后的复用拒绝。

## 性能方法

Apple M5、macOS 26.6.2、aarch64，Rust `1.100.0-nightly (67eda617e)`，release；所有 Cargo 编译和其他验收测试停止后串行计时。没有 CPU 绑核、实时调度或真机周期约束。旧 behavior 使用 `38094b2`，旧 Franka 使用 `70e0e02`，core 源码两边相同。registry 包版本从本轮 lockfile 播种，并审计 Tokio 1.48.0、nalgebra 0.34.1 等不漂移。

旧 Franka 从真实提交导出，仅植入 cfg(test) 工厂和同一回环测量体，删去尚不存在的 Native 分支。为消除新 System 测试 dev-dependencies 带来的 feature 差异，旧基线仅补齐测试依赖；旧生产 src 去掉测试接线后逐文件字节一致；确切测试编译图中 149 个 target/feature 项的多重集合一致，122 个 registry 包与锁种子版本及 checksum 相同。首次未对齐 feature 的回环结果已弃除。复现工具见 [基线脚本](../../scripts/native-control-baseline/README.md)和 [测量脚本](../../scripts/measure-native-control.py)。

CPU 测试：每批 100 万周期，21 对交替测量，完整程序再交错运行 3 次，表内取三次批均值中位数的中位数。它不是逐周期 p99。旧 control_flow 测量器内部还有更早的 tuple 对照；本轮只比较两个实际版本各自的 `new_median_ns_per_cycle` 列，避免混淆前轮 ControlFlow 改造成本。

Franka 测试：本机模拟协议对端单独线程，与被测异步执行器不同；它代表设备网络对端，不是将图内旁路通信换成跨线程测量。每次 3 轮预热、11 轮轮换后端，完整程序交错运行 3 次。持续会话各 1000 周期；另测 1 周期短会话。UDP 往返包含编解码、过滤与 OS 调度，不等于 roplat 通信延迟。native 复用调用方已创建的 runtime；旧阻塞包装每次内部创建 runtime，这是真实 API 使用方式差异。

### 既有接口的本轮回归

单位 ns/周期，CPU 批均值。

| 路径 | 命令尺寸 | 修改前 | 修改后 | 变化 |
| --- | ---: | ---: | ---: | ---: |
| 旧 async 回调 | 48 B | 5.738 | 5.723 | -0.26% |
| 旧 async 回调 | 56 B | 6.473 | 6.446 | -0.42% |
| 旧 async 回调 | 1024 B | 26.273 | 26.250 | -0.09% |
| 旧 ControlRhythm | 48 B | 8.723 | 8.779 | +0.64% |
| 旧 ControlRhythm | 56 B | 9.270 | 9.215 | -0.59% |
| 旧 ControlRhythm | 1024 B | 26.690 | 26.697 | +0.03% |
| 同步 ControlFlow | 48 B | 1.027 | 1.051 | +2.34% |
| 同步 ControlFlow | 56 B | 1.343 | 1.341 | -0.15% |
| 同步 ControlFlow | 1024 B | 11.069 | 11.078 | +0.08% |

### 新原生异步节律

同一当前版本内比较两个节律，单位 ns/周期。mock 的域 Future 立即就绪；该表衡量 CPU 路径，不证明网络场景也有同等比例收益。

| 命令尺寸 | ControlRhythm | AsyncControlRhythm | 变化 |
| --- | ---: | ---: | ---: |
| 48 B | 8.917 | 5.086 | -42.96% |
| 56 B | 9.289 | 5.478 | -41.03% |
| 1024 B | 26.645 | 16.914 | -36.52% |

### Franka 回环持续会话

每种后端 33 个 1000 周期会话。p50/p99 是各会话分位数的中位数，**不是合并全部周期的 p99**。

| 版本 / 后端 | 会话总时长中位数（ms） | 周期 RTT p50（µs） | 周期 RTT p99（µs） |
| --- | ---: | ---: | ---: |
| before / blocking_async | 14.148 | 12.167 | 26.666 |
| before / std | 14.591 | 11.167 | 25.750 |
| after / blocking_async | 15.169 | 12.167 | 27.208 |
| after / native_async | 15.017 | 11.917 | 27.084 |
| after / std | 15.781 | 11.333 | 26.083 |

同步内核修改后，会话总时长变化 +8.16%，周期 p50 +1.49%，周期 p99 +1.29%。

旧阻塞 async 修改后，会话总时长变化 +7.21%，周期 p50 +0.00%，周期 p99 +2.03%。

当前原生 async 相对旧阻塞包装，会话总时长 -1.00%，周期 p50 -2.05%，周期 p99 -0.46%。原生方案的主要效果是网络等待能交还外层 executor，兄弟图任务不被整次会话堵住；它不承诺消除网络与调度开销。

单周期会话的启动/结束总成本单列，不能视作正常持续控制的每周期成本，也不给单个周期命名 p99：

| 版本 / 后端 | 1 周期完整会话中位数（µs） |
| --- | ---: |
| before / blocking_async | 75.625 |
| before / std | 66.291 |
| after / blocking_async | 76.792 |
| after / native_async | 73.666 |
| after / std | 63.458 |

这些有限结果不等于对所有设备、平台、负载或尾延迟承诺 10% 上限；上下文中的核心图长期性能目标仍需继续验证。本轮没有新增每周期 Box / spawn / runtime；Franka 原有 UDP 序列化 Vec 分配仍在，因此不能宣传整条驱动零分配。

紧凑 CSV、环境与精确二进制 hash 保存在 [结果目录](native-async-control-results/)。原始日志、逐轮测量、target 和临时旧源码均不入库。

## 发布范围与待完成边界

受管修改仓库使用 `codex/native-async-control` 分支，先推子仓再提交父仓。此前父仓引用但未推的多语言示例 `9037e58` 已推到其 `codex/control-flow-alignment`。无修改的 K1 和 rerun_urdf 与 GitHub main 核对一致；第三方参考仓维持上游提交。roplat 打包修复单独在 `codex/git-dependency-packaging`，不合并主分支。本次是内部开发提交，不是 crates.io 发布或冻结版本。

| 仓库 | 提交 |
| --- | --- |
| robot_behavior | `79a3820af143a3ab75dfd3b308469482aac534c0` |
| franka-rust | `c198cba427ad90734d51bca1f6aab7711afa6025` |
| rsbullet | `804518bacad7fec80e065e5f95e1c34524cf8fdc` |
| libjaka-rs | `9ff806e441acbaa43ae91789b13c7b05f444c597` |
| libhans-rs | `18c9b2baa42468df07404ccbbb6dce482b3cf98b` |
| libaubo-rs | `62f235239c333fd3c1be4ea5891a265d92a9d239` |
| roplat_exrobot | `c953ff3e32c6647c2f8b7aa4ffd1d448be5f4281` |
| unitree-go2-rs | `22a2d462e16c41bf46770d0efeba74e1469de319` |
| roplat_rerun | `e593ff2a30b37564be66ff0bf8dd9f4131d791a2` |
| robot_behavior_page | `c343b229c3651ff619ac6a8f6ac0290def46ffda` |
| .github/skills/robot_behavior-skills | `32faa6f3687f33bb797a77d7af13b3b39eea664e` |
| examples/jaka_roplat_multilang | `9037e5896545738d5c552cbc3c6f60f8764cc84c` |

后续需要设备/平台验收的内容：Franka 真机的正常与取消协议、各 SDK feature 的目标平台构建；完整独立 RsBullet 与 rerun 获取/构建；状态新鲜度、正常会话超时策略和仿真语义。Input 设备失败时的归还及强制 Drop/abort 的异步收尾不在本轮扩大；不能以本地清理成功替代物理停止确认。
