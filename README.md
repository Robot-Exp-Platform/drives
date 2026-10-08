# drives

`drives` 汇集 Robot Experimentation Platform 的 Rust 机器人驱动、通用行为接口、物理仿真与可视化库。你可以在自己的项目里单独依赖一个 crate，也可以克隆这个工作区，一起开发多个后端。

第一次使用时，先根据设备或任务选库，再跟随对应 README 完成最小示例。安装某个已发布驱动不需要先构建整个工作区。

## 设计理念：把算法、设备和运行组织分开

机器人应用通常同时涉及三件事：计算目标或控制量、与设备交换数据、组织任务运行。这里把它们分成可以组合的层：

1. **行为契约**：`robot_behavior` 用明确的运动空间、控制通道和状态类型描述能力。不同后端可以实现相同接口，应用按所需能力编写。
2. **具体后端**：真机驱动处理设备协议与会话；RsBullet 提供物理仿真；参考机器人帮助理解接口。统一接口保留各后端的限位、状态和周期差异。
3. **运行与展示**：需要任务图时接入独立的 [roplat](https://github.com/Robot-Exp-Platform/roplat)；需要观察运动时接入 Rerun。普通驱动调用不必先学习任务图。

这种分层方便在参考实现、仿真和设备之间复用接口层代码，但不会自动保证控制器参数或动力学模型通用。下面的库也处于不同实现阶段，应从各自 README 的能力说明开始。

## 选择一个入口

| 任务 / 设备 | 仓库与 crate | 建议先了解的内容 |
|---|---|---|
| 理解共同接口、编写通用控制代码 | [robot_behavior](https://github.com/Robot-Exp-Platform/robot_behavior) · `robot_behavior 0.6.1` | 类型化运动与控制，状态视图，阻塞和原生异步会话。 |
| 不连接设备，练习接口 | [roplat_exrobot](https://github.com/Robot-Exp-Platform/roplat_exrobot) · `0.2.0` | 打印指令和合成状态的参考实现，不是物理仿真。 |
| Franka | [franka-rust](https://github.com/Robot-Exp-Platform/franka-rust) · `franka_rust 0.2.0` | FCI 协议、设备模式，以及同步与原生异步控制。 |
| JAKA | [libjaka-rs](https://github.com/Robot-Exp-Platform/libjaka-rs) · `libjaka 0.2.0` | TCP 状态查询与 servo；选择正确的状态读取入口。 |
| Hans | [libhans-rs](https://github.com/Robot-Exp-Platform/libhans-rs) · `libhans 0.2.0` | 已实现的状态 / 运动能力，以及构造与未实现接口的边界。 |
| AUBO 类型与接口扩展 | [libaubo-rs](https://github.com/Robot-Exp-Platform/libaubo-rs) · `libaubo 0.2.0` | 当前是接口骨架，尚不能作为完整真机驱动使用。 |
| Booster K1 | [libk1](https://github.com/Robot-Exp-Platform/libk1) · `0.1.0` | 无 SDK 的离线入口、FastDDS 接入、状态与指令检查。 |
| Unitree Go2 | [unitree-go2-rs](https://github.com/Robot-Exp-Platform/unitree-go2-rs) · `libgo2` | Git 源码安装、Linux SDK 条件、低层状态与 Sport API；尚未发布 crates.io。 |
| 物理仿真 | [rsbullet](https://github.com/Robot-Exp-Platform/rsbullet) · `rsbullet 0.4.0` | DIRECT 最小场景、步进、模型资源与机器人适配。 |
| 把 URDF 几何显示到 Rerun | [rerun_urdf](https://github.com/Robot-Exp-Platform/rerun_urdf) · `0.1.1` | 注册几何、提供 link 位姿、保存录制或接入 Viewer。 |
| 在仿真循环中记录机器人状态 | [roplat_rerun](https://github.com/Robot-Exp-Platform/roplat_rerun) · `0.2.0` | RsBullet 与 Rerun 的接入流程，观察回调在物理步进前读取状态。 |

RsBullet 的内部层为 `rsbullet-core 0.4.0` 和 `rsbullet_sys 0.3.2`；Hans 的辅助宏包为 `libhans_derive 0.1.3`。通常从上层 crate 入手即可。已发布版本与本工作区记录的 Git 子模块提交是两种不同的依赖入口，不会自动互相更新。

## 最小使用案例：先跑通共同接口

当前行为库和依赖它的多款驱动使用 nightly Rust 特性，依赖构建需要 C++ 工具链（Windows 使用 MSVC Build Tools 和 Windows SDK）。下面通过参考机器人演示完整调用流程，不连接设备，也不打开图形界面。

```sh
rustup toolchain install nightly
cargo new robot-demo --edition 2024
cd robot-demo
```

在 `Cargo.toml` 中添加：

```toml
[dependencies]
robot_behavior = "0.6.1"
roplat_exrobot = "0.2.0"
```

将 `src/main.rs` 替换为：

```rust
use robot_behavior::{
    Control, JointPositionControl, JointSpace, Motion, Robot, RobotResult,
};
use roplat_exrobot::ExRobot;

fn main() -> RobotResult<()> {
    let mut arm = ExRobot::<6>::new();
    arm.init()?;
    arm.move_to::<JointSpace<6>>([0.1; 6])?;

    let mut cycles = 0;
    arm.control_with::<JointPositionControl<6>, _>(|_state, _dt| {
        cycles += 1;
        ([0.1; 6], cycles == 3)
    })?;

    println!("completed {cycles} control cycles");
    arm.shutdown()
}
```

执行 `cargo +nightly run`。程序打印目标和三轮指令，输出 `completed 3 control cycles` 后关闭参考机器人。`JointSpace<6>` 选择六关节目标空间，`JointPositionControl<6>` 选择周期位置通道，闭包返回的布尔值表示是否在发送本条指令后结束。

这个后端没有物理模型，不会因发送目标而产生真实位姿变化。下一步可阅读上表对应库的最小例子：仿真需要准备原生构建工具与场景；真机需要配置设备连接和运行模式；可视化需要知道模型与位姿来自哪里。

## 状态、控制与异步：先明确三个约定

**状态由后端提供。** 通用状态将测量、已接受指令和期望值分为 `meas`、`cmd`、`des`；缺失字段通常用 `Option` 表达。字段存在或值为零，不代表传感器已更新。读取方法、单位、坐标系与新鲜度以具体驱动文档为准。

**控制闭包在设备会话内执行。** `control_with` 的闭包返回 `(command, done)`：先发送指令，`done = true` 时结束。需要本周期无算法指令退出时，让 `control_with_flow` 的闭包返回 `ControlFlow::Break(())`，由驱动执行协议收尾；它不等同于急停。

**async 回调和异步会话不同。** `control_with_async` 是阻塞会话中的 async 回调；`control_native_async` 才返回整个会话的 Future，而且只适用于实现了该能力的后端。运行时选择与周期要求见驱动说明。

需要把这些操作组织成 Roplat 应用图时，再阅读[核心 README](https://github.com/Robot-Exp-Platform/roplat)与 [robot_behavior 的可选适配](https://github.com/Robot-Exp-Platform/robot_behavior#features-and-roplat-integration)。核心当前发布版本为 `0.3.0`；行为接口的默认构建不要求启用 Roplat。

## 开发整个工作区

单独使用发布包时可以跳过本节。参与跨库开发时，目录需要包含同级核心仓库：

```text
Robot-Exp-Platform/
├── roplat/
└── drives/
    ├── robot_behavior/
    ├── franka-rust/
    ├── rsbullet/
    └── ...
```

获取源码（下面使用 Bash；PowerShell 的子模块与 LFS 设置见 [SUBMODULES.md](SUBMODULES.md)）：

```sh
git clone https://github.com/Robot-Exp-Platform/roplat.git
git clone https://github.com/Robot-Exp-Platform/drives.git
cd drives
GIT_LFS_SKIP_SMUDGE=1 git submodule update --init --recursive
```

需要对相关 GitHub 仓库有访问权限。部分子模块和 Cargo 源依赖使用 SSH 地址，因此还需配置 Git 的 SSH 访问；能克隆外层仓库不代表所有依赖都已获取。

根 `Cargo.toml` 将 crates.io 和固定 Git 来源的部分依赖替换为本地子目录，并通过 `../roplat/roplat` 接入核心。这让修改本地公共接口能够直接用于集成验证。子模块固定到父仓记录的提交；更新某个子仓的 `main` 不会让其他检出自动跟随它。

先选择要开发的包进行编译，例如：

```sh
ROPLAT_SKIP_ASSET_EXPORT=1 BULLET_SKIP_ASSET_EXPORT=1   cargo +nightly check -p robot_behavior --lib
```

这两个环境变量关闭构建期间的模型资源导出。若需要模型文件，再按对应库 README 明确准备；构建成功不等于模型或设备已经配置好。RsBullet 还需要 C++、CMake 和平台图形开发库，即使应用使用 DIRECT 模式，当前原生构建仍包含图形支撑组件。K1 与 Go2 的厂商 SDK 条件各不相同，应按各自 feature 说明配置。

## 示例与进一步阅读

- [各驱动 README](#选择一个入口)：首先完成对应库的独立示例。
- [双 JAKA 集成示例](examples/jaka_dual)：需要实际设备与配置，适合在单机接入完成后阅读。
- [Rust / Python / C++ 集成示例仓库](https://github.com/yixing312/jaka_roplat_multilang)。
- [模型资源](asserts)：历史目录名为 `asserts`；读取模型时仍需要核对外部网格和资源路径。
- [子模块获取说明](SUBMODULES.md)。
- [历史设计与变更记录](docs/review)：用于理解实现演进；历史记录中的版本和发布状态以当前库说明为准。

本工作区的维护入口负责源码集成，各子仓独立维护自己的 API、示例与发布版本。报告问题时请附 crate 版本或 Git 提交、平台、启用的 features、最小复现和具体错误；涉及设备时补充型号与协议/固件条件。

## 许可

工作区自身采用 [Apache-2.0](LICENSE)。各子仓及所包含的厂商 SDK、上游组件按各自 LICENSE 使用；外层工作区的许可不替代它们的许可。
