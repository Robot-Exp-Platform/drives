# crates.io 发布预检（2026-09-27）

范围是 drives 的 14 个 Rust 库，以及作为上游依赖的 9 个 roplat 核心包。示例、第三方参考子仓和空的 `command_derive/Cargo.toml` 不作为独立发行包。本轮只做打包、解包编译和 dry-run，没有真正上传 crate、选择新版本号或修改核心/驱动执行语义。

## 判定标准

Git checkout / workspace check 通过不等于 crates.io 包可用。Cargo 打包时将有 version 的 Git/path 依赖转换为 registry 依赖，去掉 workspace 和 patch，并从归档解出的内容重新构建。`publish --dry-run` 不上传；`package` 覆盖实际上传前的打包与构建步骤。单独 `package --list`、`--no-verify`，以及带根 path patch 的 check 均不能替代这个验证。

dry-run 成功也不保证服务器接受：本轮 `roplat_launch 0.2.2` 解包编译通过，但 Cargo 同时警告版本已存在。版本可用性另外查询官方 registry。依据：[Cargo package](https://doc.rust-lang.org/cargo/commands/cargo-package.html)、[Cargo publish](https://doc.rust-lang.org/cargo/commands/cargo-publish.html)。

## 原仓直接预检

- `robot_behavior --features roplat` 的 publish dry-run **实际编译失败**：registry `roplat 0.2.2` 缺少本轮使用的 Execution / Lifecycle，Node / Rhythm 签名也不同，产生 16 个编译错误。默认 feature 的首次在线命令超时，不能用该超时推断源码失败。
- `franka_rust`、`roplat_exrobot` 的 publish dry-run **实际依赖解析失败**：registry 尚无 `robot_behavior 0.6.0`，最新为 0.5.4。
- `roplat_launch` 完整 dry-run 通过；9 个核心包的当前 0.2.2 均已在 registry 存在。不能以同名同版本重新上传本轮代码。
- `rsbullet_sys` 在独立临时目录执行完整 `cargo package --offline` 通过，从包内源码编译了 Bullet/CMake；不借父仓 patch，不重新下载 Bullet，不运行 GUI。这份仅补 LICENSE、尚未补 framework 的独立包有 2330 个文件，压缩 7,525,470 字节；后续链接修复使用另一份归档。
- 初次独立 `rerun_urdf` 在线索引刷新超时；离线缓存只有已撤回的 `lz4_flex 0.11.5`。官方 sparse index 的 `0.11.6` 未撤回且满足依赖要求，因此这是缓存缺口，不能误判为其依赖范围不可解析。

## 整批源码发布预演

为验证尚未发布的上游，另建一次性临时工作区，包含 9 个核心包和 14 个驱动/配套包。仅在快照清单中作 15 处依赖身份调整：14 处内部 Git 依赖，以及 `roplat_rerun → rerun_urdf` 这一处本批 registry 依赖，转为 `path + 原 version`，让 Cargo 识别同批成员。后一处避免把 registry 旧包与本地同版本新包的校验和混在一起。不使用 `[patch]`，正式仓库清单与版本不变。458 个 Rust 文件与来源逐字节核验一致。

Cargo 多包 package 将这些路径从最终清单去掉，再通过自身 `tmp-registry` 验证本批 `.crate`。这可以回答“正确的上游已按顺序发布后，本批包能否构建”，不能把临时路径转换描述为原仓直接发布已经通过，也不能覆盖已有版本。

本次最终预演复制 drives 现有 `Cargo.lock` 作为依赖版本种子，让 Cargo 重新处理快照中的包身份；没有修改原锁文件。种子 SHA256 为 `87d5227a8837aab4ece24256cc8960d1c6941b0e98f7621ef04eacbc1b5a4bde`。全新无锁解析的在线尝试长时间停在索引刷新，因此本次不声称覆盖全新依赖解析。实际 feature 组合为 `robot_behavior/roplat,rsbullet/roplat,libgo2/roplat`。

**整批基线结果：23/23 打包及解包编译通过，Cargo 退出码 0。** 这是下文 macOS framework 修复前的批次；修复后的 sys 和消费端单独验证，不混用归档指纹。 最终完整命令耗时 921.28 秒，复用了前一次验证缓存，不作为干净构建耗时指标；前一次在通过 20 项后达到 600 秒执行时限，记录保留。最终 23 个归档均没有 Git/path 依赖或 `[patch]`，构建后再次确认 Rust 源码指纹未变化。

完整包名、大小、SHA256、规范化依赖、源提交及验证范围见 [rehearsal.json](publish-preflight-results/rehearsal.json)；registry 版本查询见 [registry.json](publish-preflight-results/registry.json)。归档本体不入仓。下表仅代表上述暂存批次的库包验证，最终程序链接另见后文。

| 包 | 版本 | 归档大小（字节） | 解包编译 |
| --- | --- | ---: | --- |
| roplat_launch | 0.2.2 | 20,584 | 通过 |
| roplat_system | 0.2.2 | 103,355 | 通过 |
| roplat_macros | 0.2.2 | 34,708 | 通过 |
| roplat | 0.2.2 | 152,336 | 通过 |
| roplat_build | 0.2.2 | 31,422 | 通过 |
| roplat_cpp | 0.2.2 | 15,932 | 通过 |
| roplat_cpp_msg | 0.2.2 | 11,714 | 通过 |
| roplat_py | 0.2.2 | 22,403 | 通过 |
| cargo-roplat | 0.2.2 | 18,209 | 通过 |
| robot_behavior | 0.6.0 | 117,667 | 通过 |
| franka_rust | 0.2.0 | 3,161,788 | 通过 |
| libaubo | 0.1.1 | 22,296 | 通过 |
| libhans | 0.1.11 | 42,683 | 通过 |
| libhans_derive | 0.1.2 | 6,333 | 通过 |
| libjaka | 0.2.0 | 56,431 | 通过 |
| libk1 | 0.1.0 | 18,105 | 通过 |
| libgo2 | 0.1.0 | 29,930,784 | 通过 |
| roplat_exrobot | 0.1.2 | 25,702 | 通过 |
| roplat_rerun | 0.1.2 | 75,536 | 通过 |
| rsbullet | 0.3.11 | 505,835 | 通过 |
| rsbullet-core | 0.3.4 | 61,101 | 通过 |
| rsbullet_sys | 0.3.1 | 7,525,471 | 通过 |
| rerun_urdf | 0.1.0 | 77,300 | 通过 |

独立验证的 `rsbullet_sys` 归档为 7,525,470 字节；整批重打包后为 7,525,471 字节。打包锁文件/元数据不同导致 SHA 和压缩长度不同，源码已分别核对，不混用这两份归档的指纹。

这次依赖锁仍使用已撤回的 `lz4_flex 0.11.5`。Cargo 可继续使用已经锁定的撤回版本；正式发行前还应验证新解析选出的未撤回版本。第三方 `binrw 0.12.0`、`re_data_loader 0.26.2` 和 `re_video 0.26.2` 还产生未来 Rust 兼容性提示，本轮未改动第三方源码。


## 已确认的发行阻塞

1. **发行版本与内部依赖下限必须同步升级。** 当前内部 Git 核心与 registry 0.2.2 API 不同。即便以后发布新核心，驱动仍声明最低支持 0.2.2 也不准确；已有锁文件或最小版本解析仍可能选到旧 API。必须选择实际提供当前契约的发行版本，并更新下游的 version 要求和对应 Git rev。此原则适用于所有内部依赖边；例如 `rsbullet → rsbullet-core` 目前仍声明 `0.3.3`，也要在发行矩阵中核查实际支持范围。
2. **未发布上游需要按顺序提供。** `robot_behavior 0.6.0` 是多个驱动及 RsBullet 的发布先决条件。
3. **已有版本号不能重新使用。** 本轮查询见下表；没有为了解除冲突擅自修改版本。
4. **Go2 发布包超限。** 整批实际归档约 28.5 MiB，主要是内置 SDK/参考文件。Cargo 官方说明默认限制为 10 MB。作者明确选择“本轮只记录，暂不改 Go2”，因此本轮保留其源码和打包策略；没有简单排除 SDK 来换取体积通过，因为这会破坏 Linux `cyclonedds` 的构建路径。[Cargo 发布说明](https://doc.rust-lang.org/cargo/reference/publishing.html)

| 包 | 当前版本 | 该版本是否已存在 |
| --- | --- | --- |
| roplat 的 9 个包 | 0.2.2 | 全部已存在 |
| robot_behavior | 0.6.0 | 未发布 |
| franka_rust | 0.2.0 | 未发布；registry 最新 0.1.15 |
| libjaka | 0.2.0 | 未发布；registry 最新 0.1.16 |
| libhans | 0.1.11 | 已存在 |
| libhans_derive | 0.1.2 | 已存在；宏生成语义与现有 registry 版一致，可考虑继续复用 |
| libaubo | 0.1.1 | 已存在 |
| libk1 / libgo2 | 0.1.0 | 查询时包名均返回 404，不代表已保留名称或授权上传 |
| roplat_exrobot | 0.1.2 | 已存在 |
| rsbullet_sys | 0.3.1 | 已存在 |
| rsbullet-core | 0.3.4 | 已存在 |
| rsbullet | 0.3.11 | 已存在 |
| rerun_urdf | 0.1.0 | 已存在 |
| roplat_rerun | 0.1.2 | 已存在 |

建议发布依赖顺序：`roplat_launch → roplat_system → roplat_macros → roplat → robot_behavior → 原生驱动 / rsbullet-core → rsbullet → roplat_rerun`。`roplat_build`、`roplat_cpp`、`roplat_cpp_msg`、`rsbullet_sys`、`libhans_derive` 可独立先准备；`roplat_py` 和 `cargo-roplat` 在核心之后，`rerun_urdf` 在 roplat_rerun 之前。无需为复用内容完全兼容的既有包创建空发行。

## 本轮安全修正与其他发现

已修正 6 个文件：Aubo description 的品牌笔误；Hans 宏包 repository 和已有 Apache-2.0 许可证副本；三个 RsBullet 子 crate 的既有 MIT 许可证副本。不改变授权选择、版本、SDK 布局或执行代码。Go2 保持原状。

JAKA 当前从 Release 下载 `jaka.zip`，但 `download_jaka_assets()` 使用 gzip + tar 解码；真实 ZIP 魔数与解码器不匹配。它不阻止 Rust 包编译，但影响发布后的显式资源安装，需要单独修复。不能仅重命名后缀，也不应转成 build.rs 隐式联网下载。本轮记录，不扩大到资源安装逻辑重构。

Bullet 包有意排除了大部分模型数据，默认资源导出会提示找不到这些数据。本轮验证跳过资源导出，只确认编译所需的 C++ 内核与许可证包含在包内；不将编译通过等同于演示模型已经安装。

库的 `cargo package` 验证不一定执行最终应用链接。额外的独立消费端证实：普通 `cargo build --offline` 因 Cocoa/Objective-C 和 OpenGL 符号未定义而失败；只对最终 binary 增加 `-l framework=Cocoa -l framework=OpenGL` 后，原依赖归档即链接通过。这确认了 CMake 静态库依赖没有被构建脚本传递到 Rust 消费端的问题。

已在 `rsbullet_sys/build.rs` 中按 `CARGO_CFG_TARGET_OS` 为 macOS 补齐这两条系统 framework 声明，保持 CMake 构建目标与仿真 API 不变。除了上述 6 处清单/许可证修正，本轮另有这一处构建脚本修正。修正版 sys 的完整 `cargo package --offline` 和消费端普通 `cargo build --offline` **均通过**。最终二进制确实自动链接 Cocoa/OpenGL，没有 `RUSTFLAGS` 或手动 framework 参数。新归档为 **7,525,608 字节**，SHA256 为 `94959ef131aec7205d65be59465bdd5787582c5a56d6992bd3f2657fdcc4e0ff`；包内 2328 个文件与当前来源逐字节匹配。

修复前后的消费端 182 个依赖包名称和版本完全相同，只有 sys 归档 checksum 改变。其余 22 个包保持原有整批基线证据，没有在这次构建脚本修复后全部重新打包；本次针对变化的 sys 包及实际消费端复验。阶段证据见 [macos-link-fix.json](publish-preflight-results/macos-link-fix.json)，原批次归档指纹保留，不覆盖。

消费端使用第一方真实 `.crate` 归档与校验和核对后的官方缓存依赖构建一次性 local registry，无父 workspace、Git/path 依赖或 `[patch]`。只执行编译和链接，未运行 binary：

```rust
fn main() {
    let mut engine = rsbullet::RsBullet::new(rsbullet::Mode::Direct).unwrap();
    engine.client.step_simulation().unwrap();
}
```

## 复现与范围

使用本机 `aarch64-apple-darwin`、Rust/Cargo 1.100.0-nightly。设置 `ROPLAT_SKIP_ASSET_EXPORT=1`、`BULLET_SKIP_ASSET_EXPORT=1`，避免构建时改写用户资源。归档、target、逐文件来源指纹、registry 响应和原始日志保留在 `/tmp/*publish*-20260927`，不加入 Git。

脚本 [prepare-publish-rehearsal.py](../../scripts/prepare-publish-rehearsal.py) 需要 Python 3.11+，只准备快照，不自动运行 Cargo 或发布。需先提供包含 LICENSE、macOS framework 修复且通过完整 `cargo package` 验证的当前 `rsbullet_sys` 归档。脚本会逐文件比对归档与当前 C++/CMake/Rust/原始清单，拒绝使用旧归档。本轮已用修正版归档重新验证脚本准备过程：23 个包、15 处临时依赖替换、458 个 Rust 文件和 2328 个 sys 来源文件全部匹配。以下路径中的 `/tmp/sys-check/target/package/rsbullet_sys-0.3.1.crate` 需替换为实际文件：

```sh
python3 scripts/prepare-publish-rehearsal.py \
  --core-source ../roplat --drives-source . \
  --bullet-archive /tmp/sys-check/target/package/rsbullet_sys-0.3.1.crate \
  --lock-seed Cargo.lock \
  --output /tmp/roplat-publish-rehearsal-new

cd /tmp/roplat-publish-rehearsal-new
ROPLAT_SKIP_ASSET_EXPORT=1 BULLET_SKIP_ASSET_EXPORT=1 \
  PYO3_PYTHON=/path/to/python3 \
  CARGO_TARGET_DIR=/tmp/roplat-publish-rehearsal-target \
  cargo package --workspace --offline \
  --features robot_behavior/roplat,rsbullet/roplat,libgo2/roplat
```

`PYO3_PYTHON` 替换为本机真实 Python 路径，本次是 `/opt/miniconda3/bin/python`。离线模式要求对应依赖已缓存；去掉 `--offline` 可以联网补齐，去掉 `--lock-seed` 则改为另一次全新解析测试。准备脚本拒绝覆盖既有输出目录，不运行 Cargo，不更改源仓。

不要加 `--no-verify` 来声称通过。多语言扩展的互斥链接配置、Linux 厂商 SDK feature、服务器端权限、真机行为和硬实时性能不由本机这组发布测试保证。正式选择版本后，仍应对最终清单重新执行预检。
