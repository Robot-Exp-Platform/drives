# 当前工作入口（2026-09-23）

先阅读 [README](README.md)、[原生异步控制记录](docs/review/2026-09-23-native-async-control.md) 与 [前轮接口对齐记录](docs/review/2026-09-21-control-flow-alignment.md)。此前[审阅快照](docs/review/2026-09-21-core-alignment.md)记录修改前状态，不能继续作为已修问题的当前结论。

- 对齐同级 roplat 核心 `c595393df441056a25b2af8bedfcbd0f598b3fec`；独立 Git 依赖固定到仅修复失效 gitlink 的 `b47f7b6aa54a230417331e4b85cefcc8eff9e128`，Rust 源码一致。应用用 `#[roplat::system]`，不要用手写 process 链绕过生命周期与合作退出。
- `robot_behavior` 默认不依赖 roplat，显式 feature `roplat` 启用 Node / ControlRhythm / AsyncControlRhythm。节点/节律由创建层启用与关闭；传入子域不重置状态。合作退出归还 N；设备 Input 不在 N 内时，失败不承诺返回该设备。
- `control_with_async` 延续 0.6 的**阻塞设备会话 + async 周期闭包**；不要擅自改为异步会话、spawn 或跨线程调度。规范入口 `control_with_flow` / `control_with_flow_async` 返回 `ControlFlow<(), (Command, bool)>`：Continue 发送有效命令，done 后正常结束；Break 不发送本周期算法命令，执行设备协议收尾。收尾不是对象 on_shutdown，更不是统一零命令或急停。
- 原生异步控制通过独立 AsyncControlWith / AsyncControlRhythm 接入，不改变同步 move_to 或旧 control_with_async。跨 runtime 限制仍适用于旧阻塞包装器；异步 System 使用原生路径。状态有效性/新鲜度与仿真语义仍有边界，参见本轮记录。不要因编译或 mock 通过就宣称真机验证通过。仿真器另行校订，以实物机器人为优先设计语义。
- 构建/测试设置 `BULLET_SKIP_ASSET_EXPORT=1` 和 `ROPLAT_SKIP_ASSET_EXPORT=1` 避免模型资源写入；不要无人值守运行 GUI 或连接设备的测试。先按包有限验证，再 check workspace；不要直接运行整个 workspace 的全部测试。
- 本轮用户授权实现、提交与 GitHub 推送。受管子仓按依赖顺序先提交并推送，再提交和推送父仓 gitlink。第三方 ref/copp/topp 维持可获取的上游提交；.DS_Store 规则放本地 Git exclude，不创建无法从上游下载的 gitlink。

---

# AGENTS.md — drives（机器人驱动 / Hardware Drivers）

> **TL;DR / 一句话**
> 真实机械臂 / 仿真器 / 可视化的 Rust 驱动 workspace；通过 `[patch.crates-io]` 把 `roplat` 重定向到本地源，使驱动可作为 roplat 节点直接被业务工程调用。
> Rust workspace of robot drivers (real arms / simulators / viz). Uses `[patch.crates-io]` to redirect `roplat` to a local path so drivers can plug in as roplat nodes.

---

## 1. 仓库定位 / Role

* **驱动+适配器层**：真实机器人 SDK 的 Rust 绑定 + 仿真器后端 + Rerun/URDF 可视化。
* **不在此仓库**：roplat 框架本身（在 [`roplat/`](../roplat)）、论文实验业务（在 [`roplat-exp/`](../roplat-exp)）。
* **构建系统**：单 Cargo workspace，`resolver = "3"`。
* **关键 patch**（[Cargo.toml](Cargo.toml)）：

  ```toml
  [patch.crates-io]
  roplat = { path = "../roplat/roplat" }
  robot_behavior = { path = "./robot_behavior" }
  ```

  这意味着 **修改 [`roplat/`](../roplat) 的公共 API 会立即影响本仓所有 crate**。

---

## 2. Workspace 成员 / Members

来自 [`Cargo.toml`](Cargo.toml)：

| Crate | 角色 | 说明 |
|---|---|---|
| `franka-rust/` | 真机驱动 | Franka Emika Panda 的 Rust SDK。含 `build.rs`、`pyproject.toml`、`*.pyi`、`*.hpp` —— 多语言友好。 |
| `libaubo-rs/` | 真机驱动 | 遨博机械臂 SDK 绑定 |
| `libhans-rs/` | 真机驱动 | Hans/翰森 机械臂 SDK 绑定（宏包 `libhans_derive` 也是实际 workspace 成员） |
| `libjaka-rs/` | 真机驱动 | JAKA 节卡机械臂 SDK 绑定 |
| `libk1/` | 真机驱动 | Booster K1 原生 Rust 驱动；Git submodule，crate 名为 `libk1`，通过官方 SDK FastDDS 直接接入，不依赖 ROS2 |
| `unitree-go2-rs/` | 真机驱动 | Unitree Go2 原生 Rust 驱动；Git submodule，crate 名为 `libgo2`，含 SDK2 DDS / Sport API 与可选 roplat bridge |
| `roplat_exrobot/` | 适配层 | 把上述驱动统一封装为 roplat 节点；含 Python `.pyi` |
| `rsbullet/rsbullet/` | 仿真器 | PyBullet 的 Rust 包装（高层 API） |
| `rsbullet/rsbullet-core/` | 仿真器 | 底层 FFI（核心） |
| `rsbullet/rsbullet-sys/` | 仿真器 | 原始 sys 绑定（在 `exclude`，因为体积大且 ABI 易变） |
| `utils/rerun_urdf/` | 可视化 | URDF 加载 + Rerun 推送 |
| `utils/topp/` | 轨迹工具 | 轨迹在线参数化工具 |
| `roplat_rerun/` | 可视化 | Rerun 与 roplat 节律对齐的发送适配 |
| `examples/jaka_dual/` | 示例 | 双 JAKA 协作 |
| `examples/cxx_exrobot/` | 示例 | C++ 调用 roplat_exrobot（被 workspace `exclude`，独立构建） |

实际成员以 `cargo metadata --no-deps` 为准（目前 16 个），包括 `robot_behavior`、`libhans_derive` 和 `rsbullet_sys`。根 manifest 的部分 exclude 名称没有指向实际子目录，不能据此宣称这些包被排除。`utils/topp` 和 `utils/copp` 是独立工具仓。

---

## 3. 关键资源 / Assets

[`asserts/`](asserts/)（拼写：应是 `assets`，但保留）：

* `sample.urdf` — 通用样例机器人模型
* `franka_panda/` — Franka URDF + 网格
* `jaka/` — JAKA URDF
* `profile/` — 标定/参数样例

> 这些是**测试/示例资源**，不是生产数据；改 URDF 时同步改 `utils/rerun_urdf/examples/`。

### 3.1 RsBullet 上游源码 / RsBullet Upstream Source

`rsbullet/rsbullet-sys/bullet3/` 是 RsBullet 使用的 Bullet3 子模块。新机器首次构建
RsBullet 前必须初始化它：

```bash
git -C rsbullet submodule update --init --recursive rsbullet-sys/bullet3
```

若要构建整个 `drives` workspace，应先在仓库根初始化全部受管 submodule，
包括 `unitree-go2-rs/` 与 `utils/topp/`（默认跳过大型 LFS 数据；参见 SUBMODULES）：

```bash
GIT_LFS_SKIP_SMUDGE=1 git submodule update --init --recursive
```

Ubuntu 主机还需准备 C++、CMake、OpenGL 和 X11 开发包：

```bash
sudo apt-get install build-essential cmake libgl1-mesa-dev libglu1-mesa-dev libx11-dev libxi-dev
```

即使业务只使用 Bullet DIRECT 模式，当前 `rsbullet-sys/build.rs` 仍会构建
Bullet OpenGL demo 支撑库，因此无头 Ubuntu 主机也需要上述开发包。

### 3.2 Go2 官方模型 / Go2 Official Model

`unitree-go2-rs/` 是受管 submodule。它内部的
`references/go2-rs/unitree_ros/` 是 Unitree 官方模型仓的本地参考 clone，
由 Go2 子仓 `.gitignore` 排除，不作为嵌套 gitlink 提交。新机器运行 Go2
Bullet 实验前初始化：

```bash
git clone https://github.com/unitreerobotics/unitree_ros.git \
  unitree-go2-rs/references/go2-rs/unitree_ros
```

### 3.3 Booster K1 官方 SDK 与模型 / Booster K1 Official SDK and Assets

`libk1/` 是受管 submodule。它内部通过脚本拉取固定 commit 的 Booster 官方
SDK 与模型仓，本地参考 clone 由 `libk1/.gitignore` 排除，不作为嵌套 gitlink
提交。首次构建 K1 FastDDS bridge 前初始化：

```bash
./libk1/scripts/fetch_official_references.sh
```

默认 `cargo test -p libk1` 不依赖厂商 SDK；Ubuntu 真机主机上按 staged
bring-up 顺序开启 `fastdds` 与 `real-robot` feature。

---

## 4. 与 [`roplat/`](../roplat) 的耦合 / Coupling

* `[patch.crates-io] roplat = { path = "../roplat/roplat" }` —— 任何对 `roplat::Node` / `roplat::Rhythm` / `#[roplat::system]` 的破坏性改动，会让本仓所有 crate 编译失败。
* 当 [`roplat/TODO.md`](../roplat/TODO.md) 标 ✓ 的功能（如 IPC、replay）想在本仓用：
  * 按当前核心文档的模块、feature 和能力边界接入；本地 patch 不会自动消除 API 或版本要求。
* 当上游 [`roplat/`](../roplat) 改了 `Node::process` 签名：
  1. 在 [`roplat/`](../roplat) 改完
  2. 在本仓跑 `cargo check --workspace` 看哪些驱动炸
  3. 同步改本仓的实现

---

## 5. 与 [`roplat-exp/`](../roplat-exp) 的关系 / Relation

* 实验仓也用同款 patch：

  ```toml
  # roplat-exp/Cargo.toml
  [patch.crates-io]
  roplat = { path = "../roplat/roplat" }
  robot_behavior = { path = "../drives/robot_behavior" }
  rsbullet = { path = "../drives/rsbullet/rsbullet" }
  franka_rust = { path = "../drives/franka-rust" }
  ```

* 因此本仓 ↔ roplat-exp 是**双向影响**：
  * 实验需要的新驱动能力 → 加在本仓
  * 实验仓不会修改本仓代码（应保持只读消费）

---

## 6. 多语言层 / Multi-Language

* **Python 绑定**：`franka-rust` / `roplat_exrobot` 都有 `pyproject.toml` + `*.pyi`，使用 PyO3。
  * 构建：`maturin develop`（在各 crate 目录下）
* **C++ 头**：`franka-rust/robot_behavior.hpp` —— 给 C++ 工程接入用。
* **C++ 节点生成**：示例的 build.rs 使用同级 `roplat_build`；生成目录不是独立的语义来源，实际手写节点与 System 图仍需一起验证。

---

## 7. 常用命令 / Commands

```powershell
# 全 workspace 构建（不含 exclude 的成员）
cargo build --workspace

# 仅构建一个驱动
cargo build -p franka_rust
cargo build -p rsbullet
cargo build -p libgo2
cargo build -p libk1

# 只编译真机示例；运行前必须有设备操作授权
cargo check -p jaka_dual

# Python 绑定本地安装
cd franka-rust ; maturin develop --release

# 修改主仓 roplat 后，验证下游不破
cargo check --workspace

# Clippy（注意 Cargo.lock 同级目录有个 clippt.out 是历史输出，可忽略）
cargo clippy --workspace --all-targets
```

---

## 8. 设计决策与注意事项 / Design Notes

### 8.1 本地依赖与 feature

独立子仓保留完整依赖声明，不继承 drives 的 workspace.dependencies。内部基线固定 Git 来源和提交；集成根通过对应 source patch 指向本地实现。已发 roplat 0.2.2 与内部同版本 API 有差异，不能用版本号代替提交和来源校验。集成示例仍可依赖 drives 目录布局；各驱动独立构建须单独验证。`robot_behavior` 的 roplat 适配是可选 feature；直接使用行为接口不应启用它。

### 8.2 Workspace 成员与 patch 是不同概念

`robot_behavior` 同时是显式 workspace 成员和 patch 目标，不会因此被编译两份。sys/derive 包也在实际成员列表中。默认检查含原生依赖；先选择有限包/feature，不用错误的 exclude 描述推断构建边界。

### 8.3 真机驱动的安全 / Safety

* 真机 crate（`franka-rust`、`libaubo-rs` 等）在 release 模式下应启用看门狗与速度限幅。
* **不要在本仓直接做轨迹规划/控制律** —— 那是 [`roplat-exp/`](../roplat-exp) 的事；驱动只暴露"读关节状态 / 写关节命令"的能力。

### 8.4 命名"包袱" / Spelling Quirks

* `asserts/` 实为 `assets/`（拼错保留）。
* `roplat_exrobot` = "roplat external robot adapter"。
* `Gopilot`（在 paper/讨论/）= 笔误的 Copilot，与本仓无关，但可能在跨仓搜索结果中出现，勿混淆。

### 8.5 依赖巨量 SDK 的取舍

* C 库（如 PyBullet、Franka FCI）通过 `*-sys` crate 桥接 → bindgen 自动生成 → 本仓 release 时不必带 C 头，CI 上自动构建。
* 真机 SDK 通常需要厂商 license + 网络 SDK 服务，CI 上跑不动 → 真机相关测试需打 `#[cfg(feature = "real-robot")]` 标签。

---

## 9. 给后来 Agent 的提示 / Notes for Future Agents

1. **改 `Node` / `Rhythm` 公共 API 时**：先在 [`roplat/`](../roplat) 跑测试，再 `cd ../drives ; cargo check --workspace` 验证下游不破。
2. **新增驱动**：放新文件夹（如 `libxxx-rs/`）→ 加到根 `Cargo.toml::members` → 在 `roplat_exrobot/` 加适配。
3. **不要 commit 大型二进制资产**（标定数据、轨迹日志）—— 走 `.gitignore` + 外部存储。
4. **`robot_behavior/` 是独立子仓**（Robot-Exp-Platform/robot_behavior）。先提交子仓，再更新父 gitlink；推送/PR 遵守当轮用户授权，不能把未推送引用声称为其他设备已可获取。
5. **PyBullet 版本敏感** —— `rsbullet-sys` 锁定特定 PyBullet 版本，升级要重新跑 `bindgen` 并测试 ABI。
6. **`visualShapeBench.json_0.json`**（仓库根有一份）是 PyBullet 的运行时副产物，可以删但不该 commit。
