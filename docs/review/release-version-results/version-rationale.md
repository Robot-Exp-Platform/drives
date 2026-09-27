# 待发布版本选择依据（2026-09-27）

本记录只用于选择本次发行元数据。核心/驱动源码契约不在本次变更；不进行实际 crates.io 上传。仓库中的前次验收证据保持原版本与原提交。以下第一方已发布归档在 2026-09-27 从 `https://static.crates.io/crates/<name>/<name>-<version>.crate` 只读下载至内存，与当前本机源码比较。

## 版本矩阵

| 包 | 原版本 | 候选版本 | 选择依据 |
| --- | --- | --- | --- |
| roplat_launch / roplat_system / roplat_macros / roplat / roplat_build / roplat_cpp / roplat_cpp_msg / roplat_py / cargo-roplat | 0.2.2 | 0.3.0 | 核心按统一版本发行；registry 0.2.2 不含当前 Execution/Lifecycle 契约，Node/Rhythm 公共签名不同。已发布兼容系列不能描述当前 API。 |
| robot_behavior | 0.6.0 | 0.6.0 | 已选但尚未发布的换代版本，保留。 |
| franka_rust / libjaka | 0.2.0 | 0.2.0 | 已选但尚未发布的换代版本，保留。 |
| libk1 / libgo2 | 0.1.0 | 0.1.0 | 初版尚未发布，保留。Go2 的 SDK/包装边界继续不变，体积阻塞仍在。 |
| libhans | 0.1.11 | 0.2.0 | 官方旧包依赖 behavior 0.5.1；ArmParam 改为 Joints/EndPoint，ArmState 结构、Python 宏/导出及 Robot 能力改动，不是兼容补丁。 |
| libaubo | 0.1.1 | 0.2.0 | 官方旧包依赖 behavior 0.5.1；公开 aubo_c/e/i/ih/is 模块改到 aubo，能力 trait 由 ArmParam/ArmDOF 改为新体系。 |
| roplat_exrobot | 0.1.2 | 0.2.0 | 官方旧包依赖 behavior 0.5.1；控制能力换代，Python ExStreamHandle 导出删除，C++ 公共函数/消息接口替换。 |
| rsbullet-core | 0.3.4 | 0.4.0 | 当前生产 Rust 源码与官方 0.3.4 相同，但公开 impl Entity 及 RobotException 双向 From 改用 behavior 0.6 类型，类型身份不再兼容原 behavior 0.5.4。 |
| rsbullet | 0.3.11 | 0.4.0 | 旧包的 RobotFile 改为 RobotDescription，控制/状态公共接口换代，SimRhythm 从默认导出改为显式 roplat feature。 |
| rsbullet_sys | 0.3.1 | 0.3.2 | 许可证补齐与 macOS Cocoa/OpenGL 链接修复，未变更 FFI API。 |
| roplat_rerun | 0.1.2 | 0.2.0 | 原公开 RobotFile 泛型约束改为 RobotDescription；URDF 从必有字符串改为 Option；同时消费新 behavior/RsBullet 类型。 |
| rerun_urdf | 0.1.0 | 0.1.1 | 官方归档之后仅 Clippy 等价变换和格式调整，没有公开签名变化；补丁版本足够。 |
| libhans_derive | 0.1.2 | 0.1.3 | 本轮 repository/许可证元数据补齐；宏主体只换行及等价 for 循环变化，生成契约不变。 |

## 归档证据

| 归档 | 官方归档内 VCS SHA | SHA256 |
| --- | --- | --- |
| libhans 0.1.11 | 333e6bd90654ec91741d012fda8ea3048e7b5ba6 | 0192881a5cff7e5cafb3b46667333001b75634349dc1a5bd23d3b6d45c9cbb3d |
| libaubo 0.1.1 | b214e115e2ee078ea8c7fbab2f2590c1d19961d8 | 05da57b77d508faab4a8e612360423b02f18479538a8d0095bb7ab8514751c40 |
| roplat_exrobot 0.1.2 | 179e7a8a0ec49e760f7bacef8532dbfb2f71e1fe | 018c6bdebeb531a735c6dcb520323ad9bf61cf04af64d6f35e8f541acbc7a166 |
| rsbullet-core 0.3.4 | 11390b7ffe38550102a5790a4b84db29f9f3abda | 7c8b9951843b6471aebc4f1b1e740cc5d616df2d6ebdd3fc5e099b7841fa9547 |
| rsbullet 0.3.11 | 11390b7ffe38550102a5790a4b84db29f9f3abda | c9d31c9042c44cc6c339eb99204ef251fec33ec8ab52db51c99a87a925fe566e |
| rerun_urdf 0.1.0 | ab42054405c6829e14d46d37444949dc5c8667c5 | 8c18629a1c4e0021e9199e0ba1ccb7b725a66e3f6b4658ec5840b4fbde9fe03e |
| roplat_rerun 0.1.2 | dce062b00ea7649117174fccefd90293d53c2743 | 16370701a86ee682c7fdba0dbfcf28a7db5d93a075fa255de34262c443cbb7fd |

宏包对比和核心旧 API 验证沿用前轮发布预检中的官方归档证据；本表不杜撰未另计算的 SHA。

## 活跃安装说明与版本引用

应定点更新的现行文件：

- `robot_behavior_page/docs/zh/guide/install.md`：第 19 行依赖示例固定旧 behavior Git rev；第 24 行解释当前发行状态。
- `robot_behavior_page/docs/en/guide/install.md`：第 18 行依赖示例固定旧 behavior Git rev；第 23 行解释当前发行状态。
- `docs/dependency-sources.md`：当前 core/behavior/RsBullet 源提交和版本要求；新增 rerun_urdf Git pin 时也同步说明。
- `AGENTS.md`：开头与“本地依赖与 feature”当前基线说明。保留“registry 旧 0.2.2 API 不同”这一历史事实，但不可继续称旧 Git rev 是当前源码。
- `rsbullet/README.md`：当前开发说明应包含本次 0.4.0；“已发布 0.3.11 不含可选节律 feature”可保留为事实。
- `scripts/native-control-baseline/prepare.py:139`：当前硬编码核心 0.2.2；改为读取调用者指定 core 的 Cargo 清单，保持历史源码实验可重放。
- Franka README 的 `franka_rust = "0.2"` / `robot_behavior = "0.6"` 范围仍正确，无需为更新而改动。

五份第一方 pyproject（robot_behavior、franka-rust、libjaka-rs、libhans-rs、roplat_exrobot）没有独立版本字段；由 Cargo/maturin 提供版本，无需新增重复版本来源。历史 docs/review 记录和旧实验数据不要批量替换。

## 依赖下限与来源

所有当前 roplat 使用处设为 0.3.0 并固定新的 core Git 提交；behavior 仍 0.6.0，但所有消费者的 Git rev 同步到新提交。rsbullet-core/rsbullet 对 sys 下限 0.3.2；rsbullet 对 core 下限 0.4.0；roplat_rerun 对 rsbullet 下限 0.4.0、对 rerun_urdf 下限 0.1.1；Hans 对 derive 下限 0.1.3。

未实际发布之前继续保留可独立获取的 Git 来源。尤其 roplat_rerun 如果只把 rerun_urdf registry 依赖提高到尚未发布的 0.1.1，将破坏独立 Git 构建；应同时设新 Git pin。驱动保持独立清单，不从父 drives workspace 继承依赖。
