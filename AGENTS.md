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
| `libhans-rs/` | 真机驱动 | Hans/翰森 机械臂 SDK 绑定（注意 `libhans_derive` 在 workspace 的 `exclude` 列表里） |
| `libjaka-rs/` | 真机驱动 | JAKA 节卡机械臂 SDK 绑定 |
| `roplat_exrobot/` | 适配层 | 把上述驱动统一封装为 roplat 节点；含 Python `.pyi` |
| `rsbullet/rsbullet/` | 仿真器 | PyBullet 的 Rust 包装（高层 API） |
| `rsbullet/rsbullet-core/` | 仿真器 | 底层 FFI（核心） |
| `rsbullet/rsbullet-sys/` | 仿真器 | 原始 sys 绑定（在 `exclude`，因为体积大且 ABI 易变） |
| `utils/rerun_urdf/` | 可视化 | URDF 加载 + Rerun 推送 |
| `roplat_rerun/` | 可视化 | Rerun 与 roplat 节律对齐的发送适配 |
| `examples/jaka_dual/` | 示例 | 双 JAKA 协作 |
| `examples/cxx_exrobot/` | 示例 | C++ 调用 roplat_exrobot（被 workspace `exclude`，独立构建） |

`exclude = ["libhans_derive", "rsbullet_sys", "./robot_behavior"]` —— 这些不参与默认 `cargo build --workspace`：

* `libhans_derive`：宏库，只在 `libhans-rs` 内部用
* `rsbullet_sys`：sys 绑定，体积大
* `robot_behavior/`：上游 crate（[github.com/...](robot_behavior/)），通过 patch 重定向用

---

## 3. 关键资源 / Assets

[`asserts/`](asserts/)（拼写：应是 `assets`，但保留）：

* `sample.urdf` — 通用样例机器人模型
* `franka_panda/` — Franka URDF + 网格
* `jaka/` — JAKA URDF
* `profile/` — 标定/参数样例

> 这些是**测试/示例资源**，不是生产数据；改 URDF 时同步改 `utils/rerun_urdf/examples/`。

---

## 4. 与 [`roplat/`](../roplat) 的耦合 / Coupling

* `[patch.crates-io] roplat = { path = "../roplat/roplat" }` —— 任何对 `roplat::Node` / `roplat::Rhythm` / `roplat::system!` 的破坏性改动，会让本仓所有 crate 编译失败。
* 当 [`roplat/TODO.md`](../roplat/TODO.md) 标 ✓ 的功能（如 IPC、replay）想在本仓用：
  * 直接 `use roplat::comm::ipc::*;` —— 因为 patch 是源码引入，无版本号问题。
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
* **Cmake 整合**：通过 [`roplat/cmake-gen/`](../roplat/cmake-gen/) 生成顶层 CMake；本仓不写 CMakeLists。

---

## 7. 常用命令 / Commands

```powershell
# 全 workspace 构建（不含 exclude 的成员）
cargo build --workspace

# 仅构建一个驱动
cargo build -p franka-rust
cargo build -p rsbullet

# 跑示例
cargo run -p jaka_dual

# Python 绑定本地安装
cd franka-rust ; maturin develop --release

# 修改主仓 roplat 后，验证下游不破
cargo check --workspace

# Clippy（注意 Cargo.lock 同级目录有个 clippt.out 是历史输出，可忽略）
cargo clippy --workspace --all-targets
```

---

## 8. 设计决策与注意事项 / Design Notes

### 8.1 为何用 `[patch.crates-io]` 而非 `path = "..."` 直接依赖

* 各驱动 crate 的 `Cargo.toml` 写 `roplat = "0.1"`（看似 crates.io 版本），实际 workspace 顶层 patch 把它替换为本地路径。
* 好处：单个 crate 可以**脱离 workspace 单独发布**到 crates.io，不需要改源；workspace 内通过 patch 自动转向本地。

### 8.2 `robot_behavior` 既在 `[patch]` 里又在 `exclude` 里

* `exclude` 排除的是把 `./robot_behavior` 当作 workspace 成员（避免它被双重编译）。
* `[patch]` 把 `crates.io` 上的 `robot_behavior` 替换为本地源。
* 这两件事**不冲突**：crate 仍参与编译，只是不作为 workspace member 被默认 build。

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
4. **`robot_behavior/` 是上游开源项目**（[github.com/StarrySky16/robot_behavior](robot_behavior/) 之类），通过 git submodule 或子树管理；改它要走上游 PR，不要本地改完忘了 push。
5. **PyBullet 版本敏感** —— `rsbullet-sys` 锁定特定 PyBullet 版本，升级要重新跑 `bindgen` 并测试 ABI。
6. **`visualShapeBench.json_0.json`**（仓库根有一份）是 PyBullet 的运行时副产物，可以删但不该 commit。
