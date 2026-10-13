# 资源借用通道迁移与版本准备（2026-10-13）

本轮按作者确认，将 roplat 0.5 的资源借用契约迁入行为库、驱动集成和有限示例。代码分支统一为 `codex/resource-borrow-channel`，尚未合并 main、发布 registry 或打 tag。最终上游固定提交见 [依赖说明](../dependency-sources.md)；本仓 gitlink 记录各子仓的完整提交。

## 设计与迁移范围

- Node / Rhythm 的业务 Input 和 Output 与 Resources 分离。Resources 是本体类型元组，调用得到对应的可变引用元组；引用与一次 process / drive 的调用生命周期一致。
- `MotionNode<R, Target>` 声明 Resources `(R,)`、Input `Target`、Output `RobotResult<()>`。失败不需要输出设备本体。
- `ControlRhythm` / `AsyncControlRhythm` 声明 Resources `(R,)`，Input / Output `()`，删除持有/归还输入的旧状态。内部逻辑保留阻塞控制会话与原生异步会话的区别。
- System 仅用显式 `.resources.N` 端口绑定资源，删除旧归还边。创建层 Lifecycle、普通 Result 的业务语义、合作式 Execution / N 归还规则不变。允许替换本体，不要求身份不变。
- Go2 和 Bullet 已内置持有的设备或引擎仍保持内部所有权，Resources `()`。外语节点也声明 Resources `()`；未引入跨进程借用 ABI。
- 多资源可以按引用元组同时借入；同一资源按源码绑定顺序使用。复杂交叉资源 DAG 无法保持当前静态借用布局时明确拒绝，作者决定后续讨论，不擅自引入锁、克隆或串行回退。

## 版本与依赖

| 包 | 本轮版本 |
| --- | --- |
| 核心相关九包 | 0.5.0 |
| robot_behavior | 0.8.0 |
| franka_rust / libjaka / libhans / libaubo / roplat_exrobot / roplat_rerun | 0.4.0 |
| libgo2 | 0.3.0 |
| rsbullet / rsbullet-core | 0.6.0 |

`rsbullet_sys` 维持 0.3.2；derive、轨迹工具及应用示例不因本次迁移空升版本。独立驱动保留完整 manifest、精确 Git 来源及 40 位 rev，不借助父 workspace 的 dependency 继承。核心先推送，随后 behavior，再直接使用者、RsBullet、Rerun，最后父仓 gitlink。未发布包的版本号不能代替 Git 来源。

## 已执行的有限验证

核心统一入口 19 项检查全部通过：696 项目标测试、22 项文档测试（另 7 项既有忽略）、工作区严格 Clippy、Python buffer 检查、有限 Rust / C++ / Python 示例，以及 graph feature / DOT 导出。详细来源指纹及边界见核心仓 `docs/candidate/resource-borrow-validation.md`。

本仓迁移矩阵：

| 检查 | 结果 |
| --- | --- |
| robot_behavior 默认 / roplat | 10 / 34 项测试通过；all-features/all-targets 严格 Clippy 通过 |
| Go2 | 32 项路由 + 7 项节律测试通过；示例及 cyclonedds bring-up 编译通过 |
| Franka | 默认与 fci_v8 各 42 项通过，各 1 项既有性能测试忽略；包含 loopback 原生异步 System 测试 |
| RsBullet roplat | 库编译通过，含原生 Bullet 构建 |
| JAKA / Hans / ExRobot | 10 / 8 / 4 项有限测试通过 |
| 多语言规划示例 | 5 项通过，实际调用 Rust / Python / C++ |
| drives 工作区 | all-targets 编译通过 |

最终提交阶段另外执行独立来源验证：临时源快照的 manifest 与工作树逐字一致，离开 drives 父 manifest，依赖从 Cargo Git 缓存中的实际推送 revision 获取。behavior / Go2 分别检查默认与 roplat；behavior 34 项、Go2 39 项有限测试再次通过。Franka、JAKA、Hans、Aubo、ExRobot 检查默认库。独立依赖图确认默认 normal edges 无 roplat，启用 roplat 的图中核心类型只有一个来源身份。

behavior、Go2、Franka 与 RsBullet 的 tracked Cargo.lock 从独立快照更新，优先使用各子仓自己的原锁播种；其他独立快照使用父锁播种保留已验证 registry 版本，不给原本不跟踪锁文件的库新增锁。这是独立 Git 来源验证，不是全新无锁解析或清空缓存的安装测试。

RsBullet 独立检查使用其自身嵌套 workspace，唯一外接的是同一提交、干净的 Bullet vendor 目录，未引用 drives 的 Rust patch。默认 / roplat 库检查均通过；Rerun 使用最终 RsBullet pin 的独立库检查也通过，默认 normal dependency 不含 roplat。Bullet vendor 的固定提交为 `d1a4256b3a019117f2bb6cb8c63d6367aaf512e2`。

最终父锁只更新内部包版本，保留既有 registry 选择。`cargo check --workspace --all-targets --locked --offline` 通过；`check-dependency-boundaries.py --offline --skip-build` 的 behavior / Go2 / RsBullet 默认和 roplat 六组 source identity / feature 边界检查均通过。最终日志分别为 `/tmp/roplat-resource-drives-final-pins-check.log` 与 `/tmp/roplat-resource-drives-final-boundaries.log`。原始命令日志存于本次会话的 `/tmp/roplat-resource-*` 和 `/tmp/roplat-drivers-release-checks.log`，不把 target 或大体积日志提交到 Git。

## 保留范围

没有运行真机、仿真 GUI、长期服务或性能实验。macOS 上 Go2 cyclonedds 检查使用可用的平台桩，不能据此声称 SDK 或真机验收通过。没有修改设备运动算法、SDK 或统一停止策略。

父仓原有 libk1 gitlink 与本地 checkout 不一致，本轮未纳入提交；本机集成编译使用现有本地 checkout，不能把这一结果称为干净递归克隆测试。旧 robot_behavior_page 网站、弃用技能子模块、第三方 ref/copp/topp 保持不动。活跃技能在独立 roplat-skills 分支另行迁移和验证。
