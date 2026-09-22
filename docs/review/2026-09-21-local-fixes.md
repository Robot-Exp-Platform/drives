> 本文记录本轮限定小修与验证边界。临时路径仅作日志补充，持久化摘要及复现命令见[主报告](2026-09-21-core-alignment.md)和[验证JSON](2026-09-21-local-verification.json)。

# drives 限定小修与验证记录

2026-09-21。本轮仅实施获准的局部修复，没有改核心 trait、ControlRhythm、停止策略、命令时序、设备协议或仿真语义。未连接硬件，未运行 GUI，未 cargo test 全工作区。

## 改动

三个子仓开始均干净，分别建立 `codex/core-alignment-audit` 分支，HEAD 未变；未暂存、提交、推送或更新父 gitlink：

- libjaka-rs：`src/types/robot_type.rs:858-859,889-890` 的 Request/Response JSON 解析将 unwrap 改为已有 DeserializeError。新增4项有限测试位于`:910,925,940,959`：分别覆盖请求/响应的空、畸形、截断及字段类型错误，合法报文和序列化往返。cmdName移除与合法报文协议保持。
- libhans-rs：`src/robot.rs:70` 的 `is_moving` 从查询结果unwrap改为`?`，保持RobotResult错误；状态查询失败不panic。
- rsbullet：`rsbullet/src/rsbullet.rs:183-186` 原普通test使用Mode::Gui并无限step，新增交互说明与带理由的ignore。原演示动作/时序完全保留，仅从默认自动测试中排除。

各子仓 git diff --check 通过。根已有脏Cargo.lock和其他子仓改动未编辑；上述三子仓仅各自预定源码文件改变。

## 实际验证

所有Cargo验证使用 `CARGO_TARGET_DIR=/tmp/drives-audit-target`。

1. `cargo test -p libjaka --lib types::robot_type::tests --locked --offline`：exit 101。robot_behavior的旧Node/Lifecycle/Rhythm接口产生13项错误（E0050/E0277/E0437），未到libjaka测试执行。日志 `/tmp/drives-jaka-parser-test.log`。
2. `cargo check -p libhans --lib --locked --offline`：exit 101，同一robot_behavior阻塞，不能宣布Hans整crate通过。日志 `/tmp/drives-hans-check.log`。
3. `cargo check -p rsbullet --lib --tests --locked --offline`：exit 101，同一robot_behavior阻塞；原生Bullet依赖后台编译结束后命令退出，未运行GUI。另有build.rs尝试更新用户Application Support/bullet被sandbox拒绝的警告，未为该路径申请写权限或重跑。日志 `/tmp/drives-rsbullet-check.log`。

三个真实crate命令全部保留失败，不因局部parser验证通过而掩盖上游接口不兼容。没有为跑通而机械改写ControlRhythm。

## 隔离 parser 回归

目录 `/tmp/drives-jaka-parser-harness`。生成器 `/tmp/prepare-jaka-parser-harness.py` 从实际生产文件逐字节提取CommandSerde、Command/Request/Response、用到的报文类型及完整泛型方法；直接path引用生产robot_behavior/src/exception.rs错误类型。当前新增的4项测试原文分别用于旧HEAD和修后方法。

- 旧HEAD `bda271ba7907d1f63ec2d67710b18b4486abc5de`：2合法报文测试通过，2反例测试因JSON unwrap panic失败，exit101。
- 修后生产方法：4通过、0失败、0忽略，exit0。
- 以`cargo generate-lockfile --offline`只在/tmp生成独立锁，然后`cargo test --locked --offline`执行；不修改工作区Cargo.lock。
- 来源、源码/锁摘要见`/tmp/drives-jaka-parser-harness/provenance.json`；日志分别为`/tmp/drives-jaka-parser-isolated-baseline.log`、`/tmp/drives-jaka-parser-isolated-fixed.log`。
- 独立harness产生原错误模块两项未使用函数warning，不声称strict Clippy通过。它证明本次JSON解析变化与合法报文行为，**不代表libjaka全crate、robot_behavior接口、网络协议会话或实机通过**。

本轮任务结束时没有残留由该子任务启动的验证进程；后续应先讨论并解决roplat控制适配，再重新执行真实crate矩阵。
