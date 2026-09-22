> **修改前的审阅快照。** 后续用户已确认 0.6 的 `control_with_async` 是阻塞会话、异步周期闭包，本轮不改为真正异步会话。本文件当时的“真正异步入口优先”建议已撤回；已修接口和当前验证状态请看 [后续实现记录](2026-09-21-control-flow-alignment.md)。状态有效性、仿真及平台缺口仍按具体记录处理。

# drives 核心对齐审阅与 TODO

日期：2026-09-21。核心基准为 roplat main `c595393df441056a25b2af8bedfcbd0f598b3fec`，drives 根基准为 `9248354fa4ae6c718c7885b8ef2f1e10a269d222`。本报告对应本地 `codex/core-alignment-audit` 工作树，不是已提交、公开发行或实机认可的版本。

**当前优先事项是打通驱动与核心执行协议的真实语义，而非继续优化某个局部通信数值。** K1/Go2 默认无设备测试可用；robot_behavior 的 roplat glue、Go2 可选节律与核心不兼容，且真实设备控制的阻塞入口不能仅靠更换 trait 签名变成合作式异步域。本轮只修了四处范围明确的小问题，把影响控制与退出的方案留作下一次讨论。

## 范围与证据级别

审阅覆盖所有当前 Cargo 成员的清单/入口、主要 public API、Node/Lifecycle/Rhythm glue、实际控制循环、native bridge、状态/命令数据流、相关示例、测试、文档、CI和 SDK/资产来源。重点检查 robot_behavior、Franka/Aubo/Hans/Jaka、K1/Go2、ExRobot、RsBullet/Rerun；工具、模型、参考仓主要检查接入和构建边界，**没有逐行审计全部 vendor 文件**，也没有把文件可访问等同功能验证。

未连接硬件、未运行运动/GUI示例、未构建整个workspace或全部feature、未做性能或ROS 2对照。未修改核心契约、控制周期、设备停止策略、仿真语义、Cargo清单/锁或CI。当前核心仍是内部版，无需追加迁移说明。

| 证据 | 本轮实际含义 |
|---|---|
| 源码审阅 | 可定位旧trait、阻塞调用、默认占位值和缺失路径；不是实机或完整编译证明 |
| `cargo metadata --no-deps --locked --offline` | 成员发现成功，实际16个成员；不代表各crate能编译 |
| K1 `safety` | 默认feature的12项mock测试通过，窄Clippy通过 |
| Go2 `command_routing` | 默认feature32项mock通过；另一项由SDK/平台cfg控制，未编译；小修后窄Clippy通过 |
| Jaka/Hans/RsBullet 窄命令 | 均exit101，robot_behavior旧trait产生13项错误，未完成目标crate验收 |
| Jaka隔离parser | 同一4项回归对旧方法2通过/2 panic，修后4通过；仅证明原方法抽取出的解析路径 |

具体命令和返回码在[平台验证JSON](2026-09-21-platform-verification.json)、[局部验证JSON](2026-09-21-local-verification.json)。原始日志未加入Git，紧凑记录保留其摘要；复现命令不依赖本机日志继续存在。

## 已完成的小修

| 子仓与位置（相对drives根） | 改动及理由 | 验证限度 |
|---|---|---|
| `libjaka-rs/src/types/robot_type.rs:858`、`:889` | JSON解析由unwrap改已有DeserializeError；畸形网络消息应走现有Result，而非panic | 新增请求/响应反例和合法往返4测试；隔离生产方法4/4，整crate被上游阻塞 |
| `libhans-rs/src/robot.rs:70` | `is_moving`查询由unwrap改`?`，保留已有错误类型 | diff审查；窄check被上游阻塞，没有虚构mock/设备通过 |
| `rsbullet/rsbullet/src/rsbullet.rs:183` | GUI无限步进演示标明交互用途并ignore，避免默认测试挂起 | 未改变演示行为；窄check被上游阻塞，未开GUI |
| `unitree-go2-rs/src/types/messages.rs:652` | 字节打包改为chunks(4)+零填充小数组，保持小端、末块补零、无堆分配，消除两项新lint | 相同32项mock重跑通过、严格窄Clippy通过；没有SDK/roplat feature验证 |

各子仓开始均干净，分别建立同名 `codex/core-alignment-audit` 分支；本轮未提交/推送、未更新父gitlink。根也使用该分支写README、AGENTS及审阅资料。原有Cargo.lock和ref/libfranka、ref/libfranka-rs、utils/copp、utils/topp的脏状态保留，不能与本轮新改动一起批量提交。根锁前后SHA256均为 `af981eeb6f1bc9e316ea7e109508264706014655707944ba9776a9815d9ab1db`。来源、子仓HEAD与小修文件摘要见[来源记录](2026-09-21-source.json)。

## 三组需要先决定的问题

### 1. 真正异步入口：控制域如何与同任务其他节律共同前进

**事实。** `robot_behavior/src/roplat/rhythm.rs:42-88`仍用旧Rhythm；`robot/control.rs:126-137`的control_with_async接收async闭包但自身是阻塞方法，默认futures::executor::block_on。`franka-rust/src/realtime/tokio_udp.rs:44-53`还创建Tokio runtime再block_on；在外层Tokio任务中会遇到嵌套runtime边界，且其内部只开I/O、不启用timer。Go2旧drive内部使用thread::sleep（`unitree-go2-rs/src/roplat_rhythm.rs:124,205`）。立即就绪的测试无法证明等待另一分支或timer时还能推进。

**建议。** 保留阻塞ControlWith供同步调用；确定一个真正返回Future的控制入口或明确的阻塞适配层。Franka已有异步底层可以复用，Jaka等同步协议需要分别设计。不要用隐式spawn_blocking糊住：System允许借用N，直接要求`'static`会改变现有所有权策略与成本。Node的Error移到独立Lifecycle是机械适配；控制运行模型不是。

**收益与代价。** 真正异步入口可让多层/兄弟域按照核心模型协作，保留状态到达→计算→发命令的短路径；代价是驱动接口与I/O等待需要对齐。独立阻塞线程可兼容部分SDK，但引入跨线程交接及调度成本，不能被描述为零成本。先以fake设备的current_thread、pending闭包和两个兄弟System分支验证，再选择实现。

### 2. stop/failure：框架退出怎样映射到设备session收尾

**事实。** 新Execution的Stopped/Err没有业务Command，旧ControlWith闭包必须返回(Command,done)。已有hold_command明确只保证命令连续，不是物理安全保证。Jaka done周期会先下发最后命令再退出servo（`libjaka-rs/src/robot.rs:438-501,519-582`）；Franka等设备确认motion结束后才返回。仅检查算法闭包前的stop不能唤醒正等待硬件包的session。Go2默认Robot::stop与README推荐的Sport API stop_move路线不同；部分preplanned/示例用`?`提前返回会跳过尾部stop尝试。

**建议。** 为每类设备明确：停止后最后命令/结束session动作、等包时的可中断或超时边界、主错误与cleanup错误的保留。驱动错误是否提升为框架失败并通知同域，还是保留Output里的业务Result，需要明确；不能只机械包一层Execution。生命周期由创建层负责，子drive结束不自动关掉外部设备或刷新状态。N完整归还保证保持；Input里的机器人不在N保证内，这是既定边界，本轮不扩大。

**收益与代价。** 可解释的结束路径使“已收回资源”“已退出设备模式”“已确认物理停止”各有准确含义；新增设备超时和收尾动作会有不同延迟，必须由具体驱动实现，不能统一用零命令或急停。测试包含预先停止不启动设备、在途Future完整返回、driver/cleanup双错误和新context再次drive；物理结果以后按设备验证。

### 3. 状态有效性及适配范围：什么数据可以进入控制、哪些能力真正存在

**事实。** Go2初始全零状态即使未收到帧也可能作为Ok返回（`unitree-go2-rs/src/robot_impl.rs:34,58`）；状态断流与正常最新值缺乏区分。K1 native保存接收时刻，但guard按Rust poll时间刷新freshness（`libk1/src/robot.rs:48`），积压旧帧首次取出可被当新鲜。K1当前没有roplat adapter且SDK handles的Send前提未证明。Aubo是占位，Hans部分Robot方法未实现，ExRobot的Duration是累积演示时间；不能将统一trait存在当成功能完成。

**建议。** 先定义未收到/有效/过期的快照边界及各command route需要的状态，再决定超时如何进入Execution与设备fallback。Latest仍只捕获最新消息，不强迫消费每条；有效性与消息是否最新是两个问题。K1时龄可从native传样本age，不直接假设C++ steady_clock与Rust Instant纪元相同。决定原始publisher是否是明确的底层入口，以及本轮优先接哪些设备/控制空间；不盲加unsafe Send，不给Aubo等占位赋予真实成功语义。

**收益与代价。** 控制器不再把默认值当真实观测，也能准确选择已完成适配；代价是状态/错误边界与部分公共接口需要整理。建议先以真实可用设备选定一种最小控制通路；K1/Go2各自可选桥接按需要加入，仿真器另行校订。

## 优先级与完成标准

| 优先级 | TODO | 理由与完成标准 |
|---|---|---|
| P0 | 完成上述三组定位，再适配robot_behavior与Go2节律 | 恢复真实合作执行。不是“编译过”即完成；有限fake设备多层System覆盖N归还、创建层、等待、失败/stop和再入drive |
| P0 | 普通Node显式实现Lifecycle，保留业务Output；校订多语言例程为System DSL | `robot_behavior/src/roplat/node.rs:26,60,96`与Jaka例程仍旧type Error；real/sim例程手写process且没on_init，当前Python宏会拒绝。先完成无设备有限图，再接控制域 |
| P1 | 清理测试入口与分层验证矩阵 | RsBullet仍有外部URDF依赖；SDK编译与mock/GUI/设备执行分开。重新跑本轮受阻的Jaka/Hans/RsBullet命令，不把隔离parser结果计为crate通过 |
| P1 | 固定构建来源并修CI | workflow缺同级roplat checkout，nightly和lock未固定，Go2 roplat feature未测；Jaka外语例程registry roplat_build与本地core不一致。干净布局能复现后才扩大矩阵 |
| P1 | 明确每设备能力表与错误语义 | Aubo默认状态/Hans未实现/Go2空成功方法必须准确标注；已有Result解析panic继续做有限修复。Robot默认方法的公共契约另按第三组决定，不一律替换合理no-op |
| P1 | 样本时龄、包错误和结束握手的mock测试 | 可控时钟/报文验证未收到、陈旧、坏包和cleanup失败；不靠短sleep证明时间语义，不将丢帧策略与旁路原则混淆 |
| P2 | SDK/模型来源、构建缓存与资产导出 | K1来源已固定；Go2 vendor/model需明确revision/校验和；CXX/AR/SDK头变更要触发重建。Jaka/RsBullet build会写用户资产目录，建议可配置且内容变化能触发同步；不擅自删vendor/历史 |
| P2 | 仿真及Rerun独立整理 | SimRhythm无context/归还且吞step错误；入队控制不等同drive。Rerun在physics闭包中分配/打印/unwrap，影响步进；先局部错误/路径修复，再讨论独立旁路可视化，不在本次强定仿真时间规则 |
| P2 | 设备上验证、性能及示例教学闭环 | 无设备语义通过后，再按设备读状态→有限控制→收尾/故障验证；另测适配层分配、调用/唤醒和端到端数据年龄。本轮不作性能、硬实时或ROS 2胜出结论 |

优先处理共享glue后再逐驱动对齐，可以避免每个crate重复绕过同一阻塞；把设备收尾与Runtime选择先说清，可避免后续性能优化推翻本轮接口。服务器未就绪不阻塞这次审阅，也不在此新增公开发行前的服务器验收门槛。

## 复现本轮有限验证

在drives根执行；只用本地缓存时加`--offline`，保持现有lock。以下前三个目标命令在当前源码上预期仍被robot_behavior阻塞，保留此事实：

```sh
export CARGO_TARGET_DIR=/tmp/drives-review-target
cargo test -p libjaka --lib types::robot_type::tests --locked --offline
cargo check -p libhans --lib --locked --offline
cargo check -p rsbullet --lib --tests --locked --offline
cargo test -p libk1 --test safety --locked --offline
cargo test -p libgo2 --test command_routing --locked --offline
cargo clippy -p libk1 --test safety --locked --offline -- -D warnings
cargo clippy -p libgo2 --test command_routing --locked --offline -- -D warnings
```

独立parser脚本[reproduce-jaka-parser.py](reproduce-jaka-parser.py)将生产原文片段与同一测试提取到指定的临时目录，分别运行旧方法与修后方法；错误类型也取自生产源码。固定lock位于`parser-locks`，脚本不改工作区锁。它只复现解析回归，不能绕过当前驱动集成失败：

```sh
python3 docs/review/reproduce-jaka-parser.py --output /tmp/jaka-parser-review-unique
```

本轮Rust为`1.100.0-nightly (67eda617e 2026-09-10)`、macOS arm64；临时原日志在JSON中保留补充位置与哈希，换设备无需复制其绝对路径。Cargo manifest/profile警告继续保留。RsBullet构建曾尝试导出Application Support/bullet并被sandbox拒绝；未为此扩大写权限，未声称资产导出通过。

## 详细证据入口

- [语义/设备静态审阅附录](2026-09-21-semantic-appendix.md)：含file:line、具体失败机理和测试建议；初稿写于局部修复前，顶部列出后续已修项。
- [平台/K1/Go2审阅附录](2026-09-21-platform-appendix.md)：保留第一阶段静态判断、第二阶段真实验证、第三阶段Go2 lint修复。
- [限定小修记录](2026-09-21-local-fixes.md)、[平台验证JSON](2026-09-21-platform-verification.json)、[局部验证JSON](2026-09-21-local-verification.json)。
- [实际成员摘要](2026-09-21-metadata.json)、[来源与lock保护](2026-09-21-source.json)。

本报告不替代具体设备协议；后续大幅调整仍先讨论作者需要保留的行为。根与子仓提交需要分开处理，本轮尚未提交/推送，不能声称其他设备已能从Git取得这些小修。
