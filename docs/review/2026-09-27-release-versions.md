# 发行版本与依赖基线更新（2026-09-27）

本轮将待发布的库分配到未占用、能反映实际兼容性的版本，并同步直接依赖下限和固定 Git 来源。用户授权将修改提交、推送 GitHub。本轮不上传 crates.io，不把 GitHub 提交视为发行完成，也不宣告核心版本冻结。

此前[发布预检](2026-09-27-publish-preflight.md)中的 23 包验证、归档 SHA 和版本表是修改版本前的证据，保持原样。下文单独记录本轮新版本和最终清单的验证范围。

## 版本选择

| 包 | 原清单版本 | 本轮待发布版本 | 理由 |
| --- | --- | --- | --- |
| roplat_launch / roplat_system / roplat_macros / roplat / roplat_build / roplat_cpp / roplat_cpp_msg / roplat_py / cargo-roplat | 0.2.2 | 0.3.0 | 核心统一版本；当前 Execution、Lifecycle、Node、Rhythm 契约与已发布 0.2.2 不兼容 |
| robot_behavior | 0.6.0 | 0.6.0 | 尚未发布，保留已选换代版本 |
| franka_rust / libjaka | 0.2.0 | 0.2.0 | 尚未发布，保留已选换代版本 |
| libk1 / libgo2 | 0.1.0 | 0.1.0 | 首次发行版本保留 |
| libhans | 0.1.11 | 0.2.0 | 公共能力 trait、状态结构与 Python 接口已换代 |
| libaubo | 0.1.1 | 0.2.0 | 公共模块路径及能力 trait 已变化 |
| roplat_exrobot | 0.1.2 | 0.2.0 | 控制能力以及 C++/Python 公共接口存在替换和删除 |
| rsbullet-core | 0.3.4 | 0.4.0 | 公开 Entity 实现与 RobotException 转换改用 behavior 0.6 的类型身份 |
| rsbullet | 0.3.11 | 0.4.0 | 控制和状态接口换代；SimRhythm 改为显式 feature |
| rsbullet_sys | 0.3.1 | 0.3.2 | 许可证及 macOS framework 链接修复，FFI API 不变 |
| roplat_rerun | 0.1.2 | 0.2.0 | RobotFile 泛型约束改为 RobotDescription，URDF 成为 Option，并消费新行为/仿真类型 |
| rerun_urdf | 0.1.0 | 0.1.1 | 已有改动为等价 Clippy/格式调整，适合补丁版本 |
| libhans_derive | 0.1.2 | 0.1.3 | repository、许可证及等价循环调整，宏生成契约不变 |

2026-09-27 09:14:45–50 UTC 查询官方 crates.io API，以上 **23 个候选版本均未占用**。K1/Go2 包名查询返回 404；其他 21 个包存在，但没有候选版本。该观察不预留名称/版本，也不证明上传权限或服务器会接受包。[查询证据](release-version-results/registry.json)保留逐项 URL、时间与结果；[版本依据](release-version-results/version-rationale.md)保留官方旧归档的源码对比、VCS 提交和 SHA256。

## 依赖与独立构建

独立驱动继续保留完整依赖声明，不从父 drives workspace 继承。尚未实际发布的内部包采用 `git + rev + version`：Git 提交定位当前源代码，version 约束给出今后发布到 registry 时的真实最低版本。只改版本字符串却仍固定旧源码会产生不一致，二者同步更新。

- 所有当前核心依赖要求 roplat 0.3.0。
- robot_behavior 保持 0.6.0，但使用者统一固定到本轮提交。
- rsbullet-core 和 rsbullet 要求 rsbullet_sys 0.3.2；rsbullet 要求 rsbullet-core 0.4.0。
- roplat_rerun 要求 rsbullet 0.4.0、rerun_urdf 0.1.1，均固定 Git 提交；后者尚未发布，不能只提高 registry 版本后就声称独立 Git 构建可用。
- Hans 要求 libhans_derive 0.1.3。所有 Git URL 对应的集成根 patch 与锁文件随清单同步。

当前固定提交见[依赖来源](../dependency-sources.md)。按上游先提交和推送、消费者后固定新提交、最后更新父仓 gitlink 的顺序执行。第三方参考仓无新提交；没有变更的 K1 不创建空提交。各 Python 包未重复定义版本，继续由 Cargo/maturin 提供。

本轮使用 `codex/release-versions` 分支。核心和 11 个变动子仓已通过 `git ls-remote` 核对 GitHub 分支与本机 HEAD 一致，工作区干净；[提交核验记录](release-version-results/github-commits.json)列出完整提交。drives 父仓以这些可远程获取的 gitlink 组装本轮结果，没有合并主分支或创建稳定 tag。

原生控制基线准备脚本也读取指定核心清单的实际版本，为临时导出的可选 roplat 依赖和 Franka 测试依赖使用同一版本。历史生产源、实验参数和结果保持不变；准备过程的源文件审计单独记录，不替代 Cargo 编译或性能复测。

## 验证状态

本轮对最终版本清单重新验证，不借用前轮旧归档的成功结论。

| 验证 | 结果与范围 |
| --- | --- |
| 发行元数据静态审查 | 23 包版本和依赖下限、16 处 Git pin 一致；独立库无越界 path 或父 workspace 继承，锁文件身份与种子集合核对通过，见 [metadata-audit.json](release-version-results/metadata-audit.json) |
| 官方版本占用检查 | 23/23 候选版本未占用，见本轮 registry.json |
| 核心 `cargo check --workspace --all-targets --locked --offline` | 通过，包含多语言示例编译 |
| behavior 独立快照 `cargo check --lib --features roplat --offline` | 通过，真实获取固定 Git 核心，不借父 workspace patch；以 drives 锁文件播种第三方版本 |
| Franka / Go2 / RsBullet 独立快照依赖解析 | `cargo metadata --offline --filter-platform aarch64-apple-darwin` 通过；以 drives 锁文件播种，真实 Git 上游均保持单一 core/behavior 包身份，解析后的锁同步回各仓；此项只证明依赖解析，不等于完整编译或打包通过 |
| 旧控制基线准备脚本 | Python 生成检查通过；两处临时 roplat 依赖均为 0.3.0，39 个 Franka + 46 个 behavior 生产文件逐字节审计通过；未运行 Cargo 或性能计时 |
| 独立 `rsbullet_sys 0.3.2` 发布包 | `cargo package --offline` 通过，2330 个文件；真实解包后完成 CMake/Rust 编译 |
| 最终 23 包发布批次 | `cargo package --workspace --offline --features robot_behavior/roplat,rsbullet/roplat,libgo2/roplat` 通过，23/23 完成归档与解包编译，458 个 Rust 源文件构建后逐字节一致；每包版本、尺寸及 SHA256 见[打包验证](release-version-results/package-validation.json) |
| 独立 Git / 根 feature 边界 | 独立 behavior、Go2、ExRobot 共 5 种配置完成真实 Git 依赖的库编译；父工作区 behavior、Go2、RsBullet 共 6 种配置完成 `--locked` metadata/tree 校验。默认不含核心，显式 roplat feature 引入单一核心身份；见[依赖验证](release-version-results/dependency-validation.json) |

发布批次在临时目录运行：使用 `scripts/prepare-publish-rehearsal.py` 将 15 条第一方依赖临时绑定到同批次 `path + version`，不修改正式清单或引入 patch。Cargo 的最终 23 份发布清单均为 registry 依赖，无 Git、path 或 patch；归档通过 Cargo 临时 registry 的实际依赖验证。该方法不声称尚未上传的上游已经存在于 crates.io，复现时需要相同锁版本和本机编译环境。构建缓存与 `.crate` 文件保留在临时目录，仓库只提交精简结果和脚本。

本轮根集成锁中的 1039 个 registry 包，其名称、版本、来源与 checksum 集合保持不变；核心根锁的 160 个 registry 包也未变化。独立子仓锁文件同步到这一已验证的集成依赖集合，因此相对各自此前未同步的旧锁可能出现版本替换，不宣称所有子仓锁逐项不变。Cargo 重新解析时自动归一化了 8 个 Windows 相关依赖边，将部分 windows-sys 使用者改选到集成锁中原已存在的 0.60.2 / 0.61.2；本轮未验证 Windows 构建。使用锁文件播种的独立构建不代表全新无锁解析验收。

## 仍有的发布边界

Go2 仅同步可选 roplat 依赖的版本和来源，libgo2 保持 0.1.0；本轮不改 SDK、资源目录、打包布局或运行语义。本轮实际发布包为 29,930,754 字节（约 28.54 MiB），超过 crates.io 默认 10 MB 限制，仍是独立发行阻塞。作者已明确要求暂只记录此项。

实际 crates.io 上传需要按依赖顺序提供新核心和 behavior 等上游，并在上传时重新检查版本占用与权限。Linux 厂商 SDK、真机和完整多语言分发验证不因修改版本号自动完成。前次记录的 JAKA 显式资源安装 ZIP/解码器不匹配等问题仍在，本轮未扩大修改范围。

建议发行顺序：`roplat_launch → roplat_system → roplat_macros → roplat → robot_behavior → 原生驱动 / rsbullet-core → rsbullet → roplat_rerun`。roplat_build、roplat_cpp、roplat_cpp_msg、rsbullet_sys、libhans_derive、rerun_urdf 可先准备；roplat_py 与 cargo-roplat 在核心之后。Go2 必须先单独解决包体积问题，不能随批次宣称可上传。
