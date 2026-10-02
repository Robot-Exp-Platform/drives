> 阶段说明：以下保留第一阶段只读审阅原文，行号和“未修改/未编译”指当时快照。后续Jaka JSON、Hans查询unwrap、RsBullet交互测试标记已完成限定小修；实际验证与最终状态以[主报告](2026-09-21-core-alignment.md)及[小修记录](2026-09-21-local-fixes.md)为准。

# drives 语义适配只读审阅

审阅日期：2026-09-21。核心参照 main `c595393df441056a25b2af8bedfcbd0f598b3fec`。本子任务只读审阅 robot_behavior、roplat_exrobot、roplat_rerun、rsbullet、Franka/Aubo/Hans/Jaka 及相关例程；K1/Go2 另有分工。读了根 AGENTS，所列范围未发现额外 AGENTS。没有运行 cargo、连接设备或编辑任何仓库，以下是源码发现，未将推断写成编译/实机验证通过。

根工作区已有脏 Cargo.lock；ref/libfranka、ref/libfranka-rs、utils/copp、utils/topp 有现存子仓脏状态，全部未触碰。各驱动和适配器本身多数是独立 submodule；改动后需要分别提交/推送及更新父 gitlink，不能只在根提交。

## 总体定位

当前适配的第一阻塞在 robot_behavior 的 roplat glue，而不在所有设备都需要重写 SDK：`robot_behavior/src/lib.rs:12` 无条件公开 roplat 模块，三个节点和 ControlRhythm 使用旧核心 trait，所以下游 crate 即使不直接使用节律也会受到影响。Node 的结构性适配较机械；ControlRhythm 的退出、同步驱动与异步 executor 关系需要先讨论，不应只补签名冒充完成。

核心要求：Node: Lifecycle + Send + Sync；Error 归 Lifecycle；Rhythm::drive 接收 ExecutionContext，闭包也接收 context，并以 Execution<Feed>/Execution<Output> 区分正常值、停止、框架失败。全部合作出口归还原 N；Input 中的资源不在这项保证内。创建层启停与外部状态保持不变；协议停止不等于设备 safe hold、断电或急停。

## 可直接做的机械适配与局部修复

### A1. 三个普通 Node 仍把 Error 放在 Node，缺 Lifecycle

- `robot_behavior/src/roplat/node.rs:26-40` MotionNode、`:60-74` SpaceMapNode、`:96-106` SafetyNode 均有旧 `type Error`。
- `examples/jaka_roplat_multilang/src/nodes.rs:21-36` MotionTickNode、`:50-74` JakaMotionCommand 同类问题。
- 最小改法：分别显式实现默认 no-op Lifecycle，将 Error 移过去，Node Input/Output/process 和现有泛型约束保持。不要借此次适配把 MotionNode 改为自动启动/关闭它经 Input 收到的机器人。
- 验证：小范围 cargo check/Clippy；真实 System 图创建层启停一次、外部重复传入不重置；业务 RobotResult 未用 `?` 时仍是普通数据。
- MotionNode 在 `robot.move_to(target)?` 失败时丢弃输入中的 robot，ControlRhythm 驱动失败亦如此。这符合作者此前接受的“Input 资源不在 N 归还保证”边界，不应据此擅自扩展资源返回协议。

### A2. 两个清晰的 Result 路径仍 panic，可小范围消除

- `libhans-rs/src/robot.rs:70`：`state_read_cur_fsm(0).unwrap()` 位于 `RobotResult<bool>` 方法内。改 `?` 即可保留既有错误类型，避免网络错误变 panic。
- `libjaka-rs/src/types/robot_type.rs:858,888`：Request/Response 的 `deserialize -> RobotResult<Self>` 对 `serde_json::from_str` unwrap。改为已有 `RobotException::DeserializeError` 映射；不改协议语义。
- 测试：畸形、截断、空 JSON 返回 Err 而不 panic；合法报文解析不变。Hans 用最小 mock/纯解析路径，不启动网络连接。
- 同类后续：`libhans-rs/src/types/command_serde.rs:106-107,135-137,161` 元组缺字段/数组坏字段会 panic；可保持 API 改成完整 Result 传播并补无设备反例。`types/move_command.rs:263-268` 还存在 MovePaths 六维点被单个逗号 token 解析的问题，应单独用往返测试定位，不把它和退出设计绑在一起。

### A3. 文档/构建依赖不对应实际仓库

- 根 `AGENTS.md:49-53` 仍说 robot_behavior 被 exclude，实际根 Cargo.toml members 含它，exclude 中对应行已注释；文档还举 roplat 0.1，实际是本地 0.2.2。
- `examples/jaka_roplat_multilang/Cargo.toml:17` build 依赖 `roplat_build = "0.2.1"`，根仅 patch roplat/robot_behavior；运行核心和构建器不是统一源码来源。最小做法是按本机统一开发约定明确本地 roplat_build path/patch，避免生成文件使用旧宏/构建器；先查看现有 lock，不覆写脏锁。
- `libaubo-rs/Cargo.toml:9` description 写 Hans，为机械文案缺陷。
- 各 crate 能力说明不足：roplat_exrobot README 仅一行；roplat_rerun README 空白。可以先补“当前真实能力/未实现/有限测试入口”，不能按 crate 名推导所有设备特性已经存在。

## 必须先讨论定位的核心控制适配

### B1. ControlRhythm 不只是旧签名，设备执行接口阻塞异步任务

出处：`robot_behavior/src/roplat/rhythm.rs:42-88`；底层 `robot_behavior/src/robot/control.rs:109-137`。

旧 drive 没有 ExecutionContext/Execution；closure 只能返回 `(Command, done)`。`control_with_async` 虽接受 async closure，但自身返回 RobotResult 而非 Future，默认每周期 `futures::executor::block_on`。drive 在异步任务中同步调用完整设备循环，因此同任务 join 的其他分支、外层 current_thread 时间驱动或提供闭包所等待数据的 sibling 可能无法推进。已有测试的闭包立即就绪，仅证明同步 happy path，无法验证等待/停止/并发。

需要作者决定/确认的最小边界：

1. 控制域返回 Stopped/Err 时没有 Command，应如何把它转换为厂商本周期最后一条指令并终止 session？已有 `ControlWith::hold_command`（control.rs:110-116）明确只保持命令连续，不承诺安全；不应把它改称 safe hold 或一律自动急停。
2. 设备错误是框架失败并 request_stop，还是 Output 中业务 RobotResult？当前是业务结果；核心对子域 `.output` 的默认传播只作用于 Execution。推荐设备 session 无法继续时作为框架失败，但这是语义变化，须先确认。
3. 等待设备下一状态包时外部 stop 如何被观察？仅在算法闭包前检查不能唤醒已经卡在 SDK/socket read 的设备循环。应明确可中断 I/O、超时或驱动等待契约，不宣称“加一个 cx.is_stopping 就完整合作退出”。
4. 建议保留现有阻塞 ControlWith 入口供同步设备用户，并讨论新增真正返回 Future 的控制入口或独立 scoped 阻塞适配。不能静默 spawn_blocking：F/N 允许非 'static 借用，强行线程化会打破同任务借用设计并增加每设备调度开销。

无设备验证至少包含：预先停止不开始session；在途域必须完整返回N；域失败后session按选定结束命令退出；driver failure及cleanup failure的保留策略；新context二次drive状态不刷新；借用N/owned N；两个真实System sibling（一个要等待另一个信号）确实前进。先测试证明当前“block_on+立即就绪测试”无法覆盖的问题，不需要连接机器人。

### B2. Franka 的所谓 async 控制在 Tokio 调用链上还有嵌套 runtime

出处：`franka-rust/src/robot.rs:715-750`（其余五种 channel 同类），`src/realtime/tokio_udp.rs:44-53`。

`control_with_async` 调用 `block_on_control_async`，内部创建 current_thread runtime 再 block_on。直接从 Tokio 正在执行的 ControlRhythm 调用将触发嵌套 runtime 问题；runtime 只 enable_io、不 enable_time，控制算法若依赖 tokio timer 还缺时间驱动。已存在真正异步底层 `control_async`（同文件:19-41），适合在未来异步 trait 中复用，但不能通过改一处函数就宣称整个接口完成。

异步接收 `run_udp_loop:112-119` 可一直等新包；完成命令阶段 `:128-137` 也等设备确认。同步 `std_udp.rs:31-80` 同样由收包与motion状态驱动。实际结束的边界应由设备session确认，roplat负责等待它返回N，不可强行drop冒充停止。网络超时/缺包/状态错误/结束确认需要mock协议测试和之后真机验证。

可保留的现有优点：同步/异步底层均接收状态→执行闭包→prepare/send命令，done命令发送后等motion结束；没有再加一个ROS式中转节点队列。补异步协议时应维护这一数据流及命令限幅路径。

### B3. Jaka 已定义 done 最后命令语义，但属于阻塞接口

出处：`libjaka-rs/src/robot.rs:438-501` 关节、`:519-582` 笛卡尔。

当前每周期 `_get_data`→算法闭包→`servo_j/servo_p`，即使 done=true 也先发送该周期命令，再退出servo；退出路径尝试关闭servo并复位is_moving。应保留这一明确行为，不能把域stop改成直接不发送任何结束命令。Jaka 未覆写 async闭包适配，继承上面的 futures::block_on。

当前主循环错误+退出servo错误同时发生时只保留前者（:497-499、:578-580）；是否聚合设备清理错误，需与Execution错误模型一并决定。当前向controller提供的是固定period，而非实际间隔（:458-471）；改变dt语义影响控制器，应讨论，不能按“计时优化”顺手改。阻塞sleep与TCP等待的中断也需定义。

### B4. 生命周期不能简单把所有 Robot hook 自动映射到域

Robot `init/enable/disable/shutdown` 是设备能力（`robot_behavior/src/robot/mod.rs:74-131`）；Roplat Lifecycle 是创建层对象启用/关闭。ControlRhythm 当前仅PhantomData、机器人从Input进入，不能on_init就初始化尚未传入的机器人，也不能每次drive结束就shutdown外部资源。

还存在不均衡实现：Robot默认stop等是no-op；Hans shutdown调用disable但自身未覆写disable（`libhans-rs/src/robot.rs:56-59`），read_state/急停等未实现（:101-110）；Aubo new忽略IP、read_state返回默认值、move/load unimplemented（下节）。应先做能力矩阵，拒绝“只要实现Robot就保证可停止/可关闭”。是否修改Robot默认方法本身属于公共契约，须讨论。

## 示例、仿真、可视化和占位驱动

### C1. 多语言例程绕过 System 且没有调用外语初始化

- `examples/jaka_roplat_multilang/examples/real.rs:24-48`、`sim.rs:51-75` 直接 while 中手写 tick→Python→C++→command.process()，未调用Lifecycle；当前核心Python生成器 `roplat_macros/src/puppet.rs:544` 对 process-before-on_init 有明确panic，因此仅修节点trait不够。
- 循环用固定125Hz的period构造虚拟时间，却没有在这阶段按period等待；先累积整条trajectory，再调用move_traj。它是离线路径准备，不是状态驱动闭环。不能把这一例程作为控制节律已经集成的证明。
- sim.rs:35-46 spawn的物理步进线程没有停止/JoinHandle回收。仿真器语义作者已明确另行校订，不应现在强行改为真机接口。
- 建议明确两类例程：有限System图完成离线规划（先可机械改为DSL、创建层管理外语节点）；真正控制例程等ControlRhythm决策后再实现。测试只用ExRobot/假驱动，编译真实入口但不运行设备。

### C2. SimRhythm 的编译适配与语义缺口需分开

`rsbullet/rsbullet/src/sim_rhythm.rs:35-64` 仍旧Rhythm签名、无Lifecycle/context/Execution；循环永不返回N，sleep不可合作停止，`let _ = engine.step()` 吞掉引擎错误。sequence是u32，长运行还有溢出边界。

可以识别机械编译需求，但不要直接规定仿真的step-before-domain、wall-clock追赶/丢步、多机器人共享一个physics clock的最终语义。作者明确仿真器是例外，当前以实物为准。最小候选是保持顺序并补失败/stop的返回N，作为有限兼容修复讨论后再做，不新增通用仿真设计。

`rsbullet/rsbullet/src/rsbullet_robot.rs:67-78` 已明确它的control_with只入队、不等待完成，并刻意不实现ControlWith；这不是遗漏trait，应保留，不能为了通用ControlRhythm强行补一个立即返回的trait impl。

### C3. RsBullet 默认测试含GUI与无限循环，不可直接全量cargo test

`rsbullet/rsbullet/src/rsbullet.rs:148` builder测试依赖robot.urdf；`:183-205` 是普通#[test]、Mode::Gui且无限engine.step循环。这是明确的工程验收阻塞。可将交互演示移examples或标记ignored并提供有限DIRECT测试，需确认资源可用；不能让全workspace测试阻塞后误判驱动本身出错。只编译与执行无设备有限单测应分别列明。

### C4. Rerun 可视化目前在物理步进闭包里执行，可能拖慢同一仿真

`roplat_rerun/src/rerun_robot.rs:22-85` 每帧同步遍历状态、分配HashMap/字符串、重复ordered_links、打印各link，并逐项log；多处unwrap会令可视化错误直接panic。`rsbullet/rsbullet/src/rsbullet.rs:49-74` 同步调用所有active_controls，然后才step_simulation，可视化延迟因此落在物理步进上。

机械改进：关闭逐帧println，预计算稳定ordered_links/路径、补错误返回。独立域旁路Latest/SharedFrame下采样则是较大行为设计，应先讨论丢帧策略、snapshot所有权、仿真任务移除、Rerun失败是否中断控制；目前API没有独立可视化节律，不应称“已与roplat节律对齐”。`:55,75` 所有link把velocity记到同一路径，会覆盖/混合不同link，可用明确link路径做局部修复。URDF遍历顺序与Bullet link index是否一致还需模型反例验证，不能只凭“确定性顺序”推出映射正确。

### C5. Aubo、Hans、ExRobot 的实际能力需要如实分级

- Aubo：`libaubo-rs/src/robot.rs:27` 忽略IP；`:121` 返回默认状态；`:140-158` move/load未实现；没有ControlWith。当前是类型/模型占位，不能当真机已支持或拿默认值证明读取正常。实现SDK是独立工作；将未支持调用改为明确错误/文档可先讨论小修，不编造设备状态。
- Hans：有运动和部分arm.state读取，但Robot.read_state/急停仍unimplemented、waiting_for_finish无退避无限查询（:74-80）；未实现ControlWith。其定位应为部分阻塞运动能力，实时控制需要独立设备协议确认。
- ExRobot：`roplat_exrobot/src/exrobot.rs:318-349` 名为spawn_realtime_loop但实际不spawn、不sleep，默认状态立即循环并每轮println；传给callback的Duration累积0、100ms、200ms而ControlWith文档规定是elapsed cycle time。它适合无硬件能力/数据形状演示，不能代表实时性能、并发性或真实dt。修名/说明是局部工作；把Duration改成稳定周期属于修契约一致性但可能影响现有示例的累积时间算法，应补有限3周期测试并明确此次改变。

## 推荐工作顺序

1. 先在隔离工作树/快照修Node机械适配、极小的Result-to-panic缺陷和现有文档失真；保留原脏锁与gitlinks。只能做有限无硬件检查，别先全workspace run/test。
2. 将ControlRhythm执行模型、停止/失败到设备命令、Robot生命周期映射和driver error分类整理成一次作者讨论；推荐保持N协议，不扩大Input资源保证。
3. 决定后用fake device的真实多层System覆盖ready与pending闭包、兄弟域、创建层、停止/失败及二次drive；再接Franka真正async和Jaka阻塞实现。测调度路径/复制/分配，不让适配层丢失此前核心优化。
4. 更新推荐多语言例程为真实System；仿真/可视化单独标明语义范围，有限DIRECT验证先于GUI。
5. 实物验收和性能实验属于后续明确授权与可用设备条件，不把本次静态审阅称为设备功能通过。
