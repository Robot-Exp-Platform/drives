# drives

机器人驱动、通用行为接口，以及 roplat 的设备、仿真和可视化适配工作区。核心编译器在同级 [roplat](https://github.com/Robot-Exp-Platform/roplat)，业务实验在 roplat-exp；当前为内部开发阶段。

本轮完成 ControlFlow 控制退出、可选 roplat 适配和当前 Lifecycle / Execution 接口对齐。整仓所有目标编译通过，152个有限测试实例通过（含协议配置重复覆盖），测试范围见[实现与验收记录](docs/review/2026-09-21-control-flow-alignment.md)。编译和 mock 通过不能代替真机、实时期限或平台 SDK 验收。

## 设备控制语义

`ControlWith<S>` 由驱动实现。`control_with_flow` 和 `control_with_flow_async` 使用 `ControlFlow<(), (Command, bool)>`：Continue 发送有效指令；done=true 时先发送最后指令再结束；Break 不发送本周期算法指令，执行设备协议收尾。旧 tuple 闭包接口仍可使用。

`control_with_async` 延续 0.6 的**阻塞设备会话 + async 周期闭包**，不是返回 Future 的异步会话。真正异步会话、状态有效性和仿真语义仍需单独讨论。Franka 内部 local Tokio runtime 与外部已进入的 Tokio runtime 还存在嵌套调用限制，不能据 mock 图测试宣称已可从任意 Tokio System 直接控制真机。

`robot_behavior` 默认构建不依赖 roplat；启用 `features = ["roplat"]` 才提供 `ControlRhythm` / 三类 Node。ControlRhythm 的 Feed 仍是有效指令与 done；框架失败/停止走 Execution。创建层负责 Lifecycle；重复 drive 不重置外部对象。所有合作退出归还 N，但 Input 中且不在 N 中的设备只在正常完成时作为 Output 返回。

## 仓库与能力边界

| 子仓 | 用途与本轮状态 | 仍有边界 |
|---|---|---|
| robot_behavior | 公共行为能力、可选 Node/ControlRhythm、完整 mock System 测试 | 阻塞会话、Input资源失败归还边界 |
| franka-rust | 六类通道 flow 同步/async回调、TCP分帧/编号、取消总等待期限 | Tokio嵌套限制；未做真机验证；正常周期超时未重设计 |
| libjaka-rs | 两类 servo 通道、有效最终指令、无指令退出、双错保留 | 设备协议与物理停止时间需验收 |
| libhans-rs | 状态/运动部分能力，状态读取失败改为返回 | 若干状态和停止接口未实现 |
| libaubo-rs | Aubo 类型与接口占位 | 不能当完整真机驱动使用 |
| libk1 | Booster K1 默认 mock 与guard测试 | FastDDS/真机feature另验；尚无roplat适配 |
| unitree-go2-rs | 两个可选节律对齐Execution；路由/退出mock测试 | 状态新鲜度、真机SDK/退出策略仍待验证 |
| roplat_exrobot | 17类ControlWith示例实现支持flow | 合成状态、立即循环，不能代表设备性能 |
| rsbullet | SimRhythm接口/合作退出对齐；构建保留已有资产 | 步进错误/计时语义仍待单独校订；GUI/模型另验 |
| roplat_rerun / utils/rerun_urdf | URDF可视化，纳入整仓check | 不是独立设备控制节律 |
| examples/jaka_roplat_multilang | 有限PlanningRhythm + 真正System DSL；Rust/Python/C++图测试 | 保持先规划再move_traj；不等于真机闭环验收 |
| robot_behavior_page / robot_behavior-skills | 中英文档与AI能力指导同步 | 配合核心roplat-skills使用 |

## 获取与验证

```text
yixing/
├── roplat/      # 本轮对齐 main c595393df441056a25b2af8bedfcbd0f598b3fec
└── drives/      # 本仓及受管子仓
```

获取子模块见 [SUBMODULES](SUBMODULES.md)。根 `[patch.crates-io]` 和部分直接 path 依赖要求同级 roplat；只克隆 drives 不足以构建。Cargo 实际解析 16 个成员，包括 robot_behavior、libhans_derive、rsbullet_sys；不能把不对应实际目录的 exclude 条目当成已排除成员。

有限验收入口（默认不开设备SDK，不运行GUI或真机示例）：

```sh
bash scripts/check-core-alignment.sh
# 已有依赖缓存时可加 CARGO_NET_OFFLINE=true
# 可单独验证/测量
cargo check -p robot_behavior --no-default-features --lib --locked
cargo test -p robot_behavior --features roplat --lib --tests --locked
CONTROL_BENCH_CYCLES=1000000 cargo bench -p robot_behavior --features roplat --bench control_flow --locked
```

脚本会设置 `ROPLAT_SKIP_ASSET_EXPORT=1` 与 `BULLET_SKIP_ASSET_EXPORT=1`，避免构建顺带写入用户模型目录。Franka/Jaka离线协议测试需要本机loopback监听权限；多语言运行测试还需配置Python动态库，见验收记录。不以无人值守 `cargo test --workspace` 起步；GUI、模型和真实设备单独授权。

应用组图用 `#[roplat::system]`，参考[核心语法](https://github.com/Robot-Exp-Platform/roplat/blob/c595393df441056a25b2af8bedfcbd0f598b3fec/docs/system-syntax.md)及 [roplat-skills](https://github.com/Robot-Exp-Platform/roplat-skills)。Node作者实现process是正常扩展；应用层手写整条process链不能替代System的生命周期、停止和对象归还。

本轮仅本地提交。受管子仓先提交，父仓再引用这些commit；发布同步时须先推子仓再推父仓。第三方参考仓保持上游提交；没有改动的子仓不创建空提交。
