# 修改前 Franka 控制路径的性能基线

此目录复现本轮修改之前的生产实现，同时使用与当前版本相同口径的有限 TCP/UDP loopback 测量工具。它用于检查同步控制和旧阻塞 async 回调入口的前后代价；当前版本的 native 路径另行比较。脚本不连接设备，也不会自动编译或计时。

固定生产源码：

- `franka-rust`：`70e0e022f9d85ce02021d672501ff8d0bf309560`。
- `robot_behavior`：`38094b239f9d03a9b0977ed98e2d2969ed60808b`。

## 基线只改测试配置与可达性

`prepare.py` 从两个本地子仓的 Git 对象导出旧版到指定临时目录。所有原始 `src` 文件与 `build.rs` 均逐字节审计；只有下列测试改动在比对前明确移除：

1. `src/robot.rs` 增加 `#[cfg(test)] from_test_impl`，把已有 loopback 实现放入公开机器人类型。工厂调用在测量窗口之外。
2. `src/realtime/tests.rs` 挂接测试模块；新增本目录的 `native.rs` 和 `performance.rs`。
3. 旧 behavior manifest 的可选 roplat path 改为参数指定的核心路径；生产源码不动。
4. 旧 Franka manifest **仅追加 dev-dependencies**：`robot_behavior/roplat`、同一 roplat 核心、Tokio `io-util`。这些用于对齐当前 Franka 的测试构建，不改变旧版普通依赖声明。

当前 Franka 测试的 dev-dependencies 会启用 behavior 的可选 roplat 适配，并合并 Tokio 特征。最终基线保留这种测试特征对齐：Franka 为 `default`，behavior 为 `default,roplat`；Tokio 为 `bytes,default,io-util,libc,macros,mio,net,rt,rt-multi-thread,socket2,sync,time,tokio-macros`。首次未对齐的试验被替代，不用于最终前后指标。核心源码应与当前测试构建一致；本轮使用 `c595393df441056a25b2af8bedfcbd0f598b3fec`，依赖发布清理提交 `b47f7b6aa54a230417331e4b85cefcc8eff9e128` 的 Rust 源码相同。

这不声称两个二进制完全相同：生产实现正是比较对象。`features.py` 读取确切测试单元的依赖图，核验编译特征集合；版本与 checksum 则由 `verify.py` 独立检查。本轮最终旧/新构建各 149 个可达编译单元的特征集合相同，122 个解析到的 registry 包均在同一集成锁中。

## 测量内容

`performance.rs` 是当前 `franka-rust/src/realtime/tests/native/performance.rs` 的小型兼容快照：删除旧版不存在的 `Native` 分支/回调，只保留 `Std` 和 `BlockingAsync`，轮换次数由三个后端调整为两个，测试名增加 `baseline` 标识。TCP/UDP fixture、测量边界、指令校验、预热次数、分位数算法保持一致。仍在窗口外建立一个外部 current-thread runtime，与当前测试一致；旧阻塞入口内部创建 runtime 的成本仍在会话时长之内。

- 使用公开 `JointPositionControl<7>`，闭包计算完成即返回，没有人为等待或周期 pacing。
- 每个后端分别测量 1 周期和 `FRANKA_PERF_CYCLES` 周期会话；每组先预热 3 次，再轮换执行各后端。
- `session_ns` 覆盖公开控制方法调用到返回，包括协议启动和终止。
- `cycle_p50_ns` / `cycle_p99_ns` 来自对端线程发送状态到收到对应命令的 RTT，包含系统调度、协议处理、过滤与序列化开销；不是单向通讯延迟，也不是物理机器人周期期限。单周期组只有一个样本。
- 每次收到的 command ID 和最终 `done` 标志均校验。这里没有另加线程、Box 或节点复制改变旧生产路径；对端协议线程与当前 fixture 相同。

## 准备、锁审计、仅构建

需要 Python 3.11+、Git、本轮使用的 Rust nightly、完整同级 roplat 核心与已初始化的两个子仓。下面从 drives 根运行；`PYTHON` 可换成设备上的实际 Python 3.11+ 路径。所有源码副本、锁、metadata、日志与 target 都留在临时目录，不加入 Git。

```sh
PYTHON=python3
BASELINE_OUT=/tmp/franka-control-baseline
DRIVES_ROOT="$PWD"

"$PYTHON" scripts/native-control-baseline/prepare.py \
  --drives-root "$DRIVES_ROOT" \
  --roplat-root "$DRIVES_ROOT/../roplat" \
  --lock-seed "$DRIVES_ROOT/Cargo.lock" \
  --output "$BASELINE_OUT"

cd "$BASELINE_OUT/workspace"
export BULLET_SKIP_ASSET_EXPORT=1
export ROPLAT_SKIP_ASSET_EXPORT=1
export CARGO_TARGET_DIR="$BASELINE_OUT/target"
rustc -vV > "$BASELINE_OUT/toolchain.txt"
BASELINE_HOST=$(rustc -vV | sed -n 's/^host: //p')

# 先用匹配当前测量构建的完整集成锁播种，再离线裁剪到此 workspace。
# host 过滤避免为了无关平台尝试读取本机未缓存的包。
cargo metadata --format-version 1 --offline --filter-platform "$BASELINE_HOST" \
  > "$BASELINE_OUT/metadata.json"
"$PYTHON" "$DRIVES_ROOT/scripts/native-control-baseline/verify.py" \
  --output "$BASELINE_OUT"

# 只有版本/checksum 审计通过，才构建。不能通过时先补齐指定依赖缓存，
# 不删除锁、不退化为一次无锁解析后直接比较。
cargo test -p franka_rust --release --lib --no-run --locked --offline \
  > "$BASELINE_OUT/build.log" 2>&1
```

`--output` 必须尚不存在；`prepare.py` 拒绝在 drives 内生成基线。输出包括源码哈希、测试 graft patch、manifest 调整 patch 与原始锁种子。`verify.py` 检查 registry 包身份和 checksum 均来自种子，额外确认 Tokio `1.48.0`、nalgebra `0.34.1`。Cargo metadata 的 feature 并集不能替代具体编译单元审计。

读取 `build.log` 中的确切 executable，找到对应 `test-lib-franka_rust.json`。Cargo nightly 新布局位于 `target/release/build/franka_rust/<hash>/fingerprint/`；传统布局位于 `target/release/.fingerprint/franka_rust-<hash>/`。使用同一 nightly 构建当前版本，然后运行：

```sh
"$PYTHON" "$DRIVES_ROOT/scripts/native-control-baseline/features.py" \
  --baseline-target "$BASELINE_OUT/target" \
  --baseline-unit "$BASELINE_UNIT_JSON" \
  --current-target "$CURRENT_TARGET" \
  --current-unit "$CURRENT_UNIT_JSON" \
  --output "$BASELINE_OUT/compiled-features.json"
```

四个变量须指向本次两个确切二进制对应的 target/指纹文件，不能从混有历史构建的目录随意选取第一个文件。脚本只读 Cargo 指纹；布局或字段变化时会失败，应核对工具链，不能把未完成审计视为一致。

## 单独计时

构建完成后停止其他 Cargo/测试进程，再交错运行旧/新二进制，记录测试输出和工具链。用构建日志中的确切 executable 填写 `BASELINE_BIN` 和 `CURRENT_BIN`；两个测试的名字不同：

```sh
FRANKA_PERF_ROUNDS=11 FRANKA_PERF_CYCLES=1000 "$BASELINE_BIN" \
  --ignored --exact realtime::tests::native::performance::baseline_loopback_performance \
  --nocapture --test-threads=1

FRANKA_PERF_ROUNDS=11 FRANKA_PERF_CYCLES=1000 "$CURRENT_BIN" \
  --ignored --exact realtime::tests::native::performance::native_loopback_performance \
  --nocapture --test-threads=1
```

每次必须看到 **1 个测试通过**，默认参数下旧版为 **44 行**、当前版为 **66 行** CSV 数据。`--exact` 名字不匹配也可能退出成功但运行 0 项，不能接受这样的日志。保留多轮会话分布，避免只看一次最小值。有限 loopback 结果不代表真机、硬实时或跨平台验收。
