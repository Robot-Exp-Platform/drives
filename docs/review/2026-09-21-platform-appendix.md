> 阶段说明：本文分阶段保存静态发现、无设备验证和Go2小修。早期“源码未改/子仓clean/未运行编译”不覆盖第三阶段的结果；最终状态以[主报告](2026-09-21-core-alignment.md)为准。

# drives 平台、libk1 与 libgo2 只读审阅

审阅日期：2026-09-21。根目录 `/Users/yanjizhou/yixing/drives`；核心契约以 `../roplat` 的 `c595393df441056a25b2af8bedfcbd0f598b3fec` 为准。本文的相对 `file:line` 均相对 drives 根目录，`../roplat/...` 除外。

已读 drives/AGENTS.md、libk1/AGENTS.md、unitree-go2-rs/AGENTS.md。第一阶段未修改仓库、未构建 workspace、未连接设备，只运行了 `cargo metadata --no-deps --format-version 1 --offline --locked`，成功；这不证明 Rust 编译或 SDK 链接成功。metadata 报 robot_behavior 子包 profile 被根 workspace 忽略。后续经 root 指派完成的窄目标验证见第 6 节。

保护的原有状态：根 Cargo.lock 的 roplat_py dependencies 中增加一行 roplat；ref/libfranka、ref/libfranka-rs、utils/copp、utils/topp dirty。结束时状态与开始一致，libk1/libgo2 本身 clean。

## 1. 总体判断

- libk1：零默认第三方 Rust 依赖、默认不链接 SDK，协议/限幅/状态校验/看门狗可 mock 测试；`fastdds` 开官方 C++ 后端，`real-robot` 进一步开放低层发布。尚未实现 roplat Node/Rhythm/Lifecycle 桥接，不应误写为“新核心编译失败”。
- libgo2：SDK 消息、CRC、静态 DDS endpoint、Sport API 路由与 mock tests 比较完整；roplat 是可选 feature，但桥接仍是旧 trait，确定不能符合 c595393；公开 Robot 生命周期、状态有效性和停止接口仍有 scaffold 行为。
- drives：当前开发机上的 Cargo 成员发现可工作，但文档、CI、默认构建范围与 sibling roplat 依赖没有形成可复现的入口。先修语义和构建基线，再做正式控制桥接。

## 2. 明确可修：不需要重新选择项目架构

### A1. 根 AGENTS 的成员/排除表已过期

证据：`Cargo.toml:2`、`:17`、`:19`，`AGENTS.md:34`、`:41`、`:49`、`:194`。

metadata 得到 16 个 workspace/default members：jaka_dual、franka_rust、libjaka、roplat_rerun、rerun_urdf、rsbullet、rsbullet-core、rsbullet_sys、jaka_roplat_multilang、robot_behavior、libaubo、libhans、libhans_derive、libk1、libgo2、roplat_exrobot。

`exclude = ["libhans_derive", "rsbullet_sys"]` 写的是不对应实际目录的名字，不能排除 `libhans-rs/src/libhans_derive` 和 `rsbullet/rsbullet-sys`；metadata 证明二者仍是成员。robot_behavior 已明确在 members，topp/copp 则确实排除；franka_letters 已注释，不是当前默认编译前置条件。应先把文档改成事实，并说明即使排除 sys 成员，其上层依赖仍会构建它；不要承诺 exclude 可以去掉依赖 SDK 的构建。

### A2. CI 没有获取 sibling roplat，也没覆盖关键可选集成

证据：`.github/workflows/ffi.yml:12`、`:14`、`:18`、`:22`，`Cargo.toml:29`，`unitree-go2-rs/Cargo.toml:22`。

workflow 仅 checkout drives 及子仓，随后 cargo check --workspace；根 patch 和 Go2 path 依赖都指向 `../roplat/roplat`，干净 CI checkout 不会自动获得该目录。nightly 未固定日期；没有 --locked；唯一 Windows job 无 Linux SDK build，也没有 libgo2/roplat feature 检查和 mock test 执行。

可修部分：声明经过验证的 sibling checkout 布局、固定测试用 core commit/toolchain、把基础 mock tests 与 feature compile checks 加入 CI。核心版本选择是发布策略，不应随便绑定某个不断变化的 main。GitHub checkout 的 SSH 子仓 URL 是否能自动改写并访问需在干净 runner 验证，不仅凭 URL 判断必失败。

### A3. Go2 roplat 桥接确定落后于核心契约

证据：`unitree-go2-rs/src/roplat_rhythm.rs:76`、`:84`、`:87`、`:157`、`:165`，`examples/go2_roplat_bringup.rs:14`；核心 `../roplat/roplat/src/rhythm.rs:53`、`:72`，`../roplat/roplat/src/lifecycle.rs:27`。

旧实现把 `type Error` 放在 Rhythm 中，没有独立 Lifecycle；drive 缺 ExecutionContext 参数，周期闭包只有 (N,Yield)，返回原始 Feed/Output。新核心要求 `(Execution<Output>,N)` 和 `(Execution<Feed>,N)`，闭包三参。两种 Go2 Rhythm 和直调示例都必须迁移。该结论是源级契约对比，不冒充本轮编译结果。

纯签名迁移、示例更新、mock 契约测试可以明确实施；设备 stop 策略、Output 中既有业务 Result 是否保留、状态超时如何映射 execution failure，属于下方 B 类，不能靠把所有结果包两层来糊弄。

### A4. async drive 内阻塞 sleep

证据：`unitree-go2-rs/src/roplat_rhythm.rs:124`、`:205`。

两个 drive 都在 async loop 内 std::thread::sleep。在 roplat 同任务组合的多个节律场景，它会阻塞其他分支，停止信号也不能及时被同线程推进。应选择可唤醒的异步定时等待并检查 context；若另走实时专用线程，应明确该后端及 handoff，而不是在 async API 内暗含阻塞。不要简单改 Tokio timer 却不声明其 runtime/time feature 前提。

### A5. 部分公开“成功”值与实际能力不符，应显式标识

证据：`unitree-go2-rs/src/robot.rs:130`–`:151`；`src/streaming.rs:144`、`:164`；`src/robot.rs:19`。

Go2 init/shutdown/enable/disable/reset 全部空 Ok，is_moving 固定 false；TripleBuffer/RingBuffer 只是 PhantomData，没有存取行为。流式 start 已正确返回 unsupported，可作为公开能力表达的参考。应先完善能力表和 unsupported/unknown 表达，不能把“返回成功”说明成已启用、已关闭或已确认静止。是否将某个 init 定义为合法 no-op 要按 transport 在构造时已创建的事实解释，不能一概替换而破坏合理的 no-op。

### A6. SDK/model 构建资料可复现性

K1 较好：`libk1/references/official_sources.md:5` 有 SDK 和 assets commit，`scripts/fetch_official_references.sh:9` 对应固定 SHA；忽略本地 clones。默认 SDK 目录可以通过 BOOSTER_SDK_ROOT 覆盖，缺路径错误有操作提示（build.rs:39）。

Go2：`unitree-go2-rs/build.rs:28` 硬编码 vendored SDK；references README:18 声明快照保存，没有在所读清单中找到明确上游 commit/校验和。官方模型只给 git clone default branch（references/README.md:31），多设备可拿到不同模型。应补 SDK 来源版本清单和模型固定 revision；可增加 SDK root override、缺 SDK 的明确诊断。

两套 build.rs 都手工调用 CXX/AR；已有 rerun-if-changed 只覆盖自身桥文件，SDK 头文件、CXX/AR 变化没有完整重建跟踪（libk1/build.rs:5；Go2/build.rs:5）。这是构建缓存正确性缺口。不能由 target_arch 选库就宣称支持跨平台交叉编译：compiler/sysroot 仍需契约。

K1 README:112 / AGENTS:5 仍说未来才加入 drives；现在已是 gitlink 且 workspace member，应修正。根无 README，只有 AGENTS/SUBMODULES；建议有用户入口与支持平台/feature/验证级别表。

### A7. 已有测试隔离现状应准确记录

- K1：12 个 tests/safety.rs 测试，全部 MockStateSource/MockCommandSink，不调用 SDK；默认 features=[]。三个示例需要 fastdds，低层 publisher API 另需 real-robot。`k1_dry_run_hold` 不创建命令 publisher。
- Go2：33 个 tests/command_routing.rs 集成测试，使用 mock endpoint，覆盖路由、CRC、packet layout、sport response 配对等。`[lib] test=false, doctest=false` 不代表无测试，但意味着 rustdoc 例子和未来内部 unit tests 不会默认验证（Cargo.toml:17）。现有 `roplat_walk` 只创建 rhythm，根本没有 drive 行为验证（examples/roplat_walk.rs:12）。
- `cyclonedds` 只在 Linux 链接，macOS 为 unsupported stub；macOS check 通过不能当 SDK 链接验证。K1 fastdds 默认在非 Linux 明确失败，LIBK1_SKIP_NATIVE_LINK 仅适合类型检查，不是链接或设备功能验收。

## 3. 须先讨论：影响执行/硬件/接口边界

### B1. 统一“停止、最终关闭、设备模式切换”的驱动职责

Go2 `Robot::stop` 发默认 SportModeCmd（robot.rs:154），README 验证过的停止路径却是 `rt/api/sport/request` 的 stop_move（README:63、:131；sport_api_client.rs:79）。不能直接把两者当作同一效果。

需要明确：roplat ExecutionContext 的停止由控制节律在什么位置响应；收回 N 后发送何种已验证设备停止/保持动作；故障时 stop 失败怎么保留；最终 Lifecycle::on_shutdown 是否关闭 DDS，而不是把设备停机、transport drop、对象销毁混为一谈。Go2 还没有独立 Lifecycle；K1 用 Drop 删除 native handles，关闭错误在 C++ destructor 内吞掉（libk1/src/native/booster_k1_bridge.cpp:141、:161），没有可观测的显式 shutdown Result。

### B2. Go2 状态有效性与发布门槛

`unitree-go2-rs/src/robot_impl.rs:34` 初始化全零状态；`:58` 即便无任一新消息仍返回 Ok(latest_state)。首次未收到状态、长时间断流和正常采样无法区分，控制 loop 会继续向用户闭包提供状态并允许发布（robot.rs:197）。这不是要求消费每条旁路消息，而是最新值仍应携带可用性与新鲜度。

建议讨论是否返回带 valid/age/seq 的状态快照、何时因为断流使本 drive 执行失败、每种 command route 需要哪组状态。可先增加反例测试；阈值和硬件 fallback 不替用户决定。

K1 有确定的时龄计算缺口：C++ 在真正接收时保存 received_mono_ns（native/booster_k1_bridge.cpp:134），但是 `K1GuardedRobot::poll_state` 用 Rust 当前 poll 时间刷新 freshness（robot.rs:48），而不是样本接收时间。若 DDS 最后一帧已积压很久，首次 poll 仍会被当作新鲜。应修时龄传递；Rust Instant 与 C++ steady_clock 的可比较性要明确，可以由 native 返回 age，而不是直接假定两语言时钟 epoch 一致。

K1 native publisher 只验证命令范围（native.rs:213），新鲜状态约束只在 K1GuardedRobot 上；公开 raw publisher 可以绕过它。需要决定 raw transport 是显式低层逃生口，还是要求所有公开控制路径必须持有 guard。当前 AGENTS 的“Never publish ... without fresh state”比公开 API 实际保证更强。

### B3. Go2 调用失败后的物理收尾

`unitree-go2-rs/src/preplanned.rs:145` 的 move_velocity_timed 仅正常跳出 while 后执行 stop_move；中间 publish_request? 失败直接返回，跳过停止尝试。`examples/go2_safe_smoke.rs:99`–`:119` 也可能因读状态、发命令或写日志错误提前返回，跳过尾部 stop。应先决定可观测的 best-effort stop 与主/次错误保留策略，再改。网络断开时不能承诺 stop 送达。

### B4. K1 加入 roplat 的边界和线程模型

libk1 没有任何 roplat dependency、Node、Rhythm、Lifecycle；它是原生驱动而非已完成控制节律。是否新增可选 roplat adapter、是否接 robot_behavior 通用模型、yield/feed 含何种平台状态/command，需讨论。

NativeLowStateSubscriber/NativeLowCommandPublisher/NativeLocoClient 持有 NonNull（native.rs:142、:188、:238），当前没有 Send 实现；新 Rhythm 的 Future:Send 不能仅靠包一层结构自动满足。必须证明 SDK handle 跨线程移动的安全前提，或保留线程归属再桥接；不要为通过编译盲加 unsafe impl Send。

### B5. realtime/旁路手段不能由占位名字推断

Go2 的 TripleBuffer/RingBuffer 仅类型占位（robot.rs:19），实际 native subscriber 用 std::mutex（native/unitree_go2_bridge.cpp:273、:292），sport request 转 C++ std::string/std::vector（:232），K1 command 每次构造 vector（native/booster_k1_bridge.cpp:98）。目前应只声明结构、mock 或 bring-up 验证级别，不能声明 hot path 已 lock-free/zero-allocation。

是否采用 roplat 新 Latest/Queue、是否维持驱动内部与语言绑定共享底座、DDS callback 到节律的时序与容量策略，要按数据语义和性能实测选；不该为了名称统一就替换实现。

### B6. 默认 workspace、SDK 资产和设备运行门槛

根当前 default-members 等于所有 16 成员，没有轻量入口；是否指定一组默认基础 crate，另有 simulator/SDK/FFI profiles，需要约定，以免隐藏集成破损。即使增加 default-members，cargo check --workspace 仍会构建全部。

Go2 追踪 references 下 1,289 个文件、114,912,219 字节（约 109.6 MiB），含两架构 SDK .a、DDS .so、PDF；K1 只追踪两个来源文档。二者都可合理，但版本升级和发布分发策略不一致。是否改为固定 hash 下载脚本或可复用 artifact cache 需讨论；不能擅自删 vendor 内容或重写历史。

Go2 真实运动示例只要求 cyclonedds，未像 K1 另设 real-robot；go2_safe_smoke 默认 vx=0.05，实际会 stand/walk（examples/go2_safe_smoke.rs:29、:69）。默认 mock 测试没有真机副作用，但“启用 DDS feature”不等于“只读”。是否统一状态读取、模式切换、命令发布的 gates，建议明确。

## 4. 建议验证顺序（本轮未执行编译）

1. 固定 core commit、干净 workspace 布局；修正成员/feature 文档并加入 no-deps metadata 校验。
2. 默认离线 mock：`cargo test -p libk1 --locked --offline`、`cargo test -p libgo2 --test command_routing --locked --offline`。可用临时 target；先确认不会被 workspace resolution 拉进不必要更新。本轮只提出建议，未执行。
3. 先写 Go2 新 Rhythm mock 契约反例：pre-stopped 不读不发、周期 Err 归还节点且不发业务命令、等候时可停止、执行中 await 回收、再入 drive 使用新 context 且状态不刷新；再迁移代码。K1 时龄用可控时钟/模拟历史收帧测，不靠 1ms sleep 测设计本身。
4. Linux x86_64/aarch64 独立 SDK compile/link jobs；不运行运动示例。macOS 只验证可支持部分与平台错误。
5. 设备实测单独且显式：读取 → 模式 → 有限命令 → 停止/故障路径；保持与自动 CI 隔离。模型/SDK/hash 与测试结果一起记录。

## 5. 本次快照

- roplat `c595393df441056a25b2af8bedfcbd0f598b3fec`
- libk1 `de2bfc75a5af15f28a97a863a8b97006924f806d`
- unitree-go2-rs `f05d61ac2ccd8f32e1dc3547d6eef7d6c7f29bd2`

没有进行网络查询；厂商协议判断基于仓库内官方参考源与代码，不重新验证市场上最新 SDK。未逐行审计 1,289 个 vendor reference 文件；重点为本仓 public API、构建入口、native bridge、关键官方引用、tests、examples、文档与当前 core 契约。

## 6. 第二阶段：实际无设备窄目标验证

环境 `rustc 1.100.0-nightly (67eda617e 2026-09-10)`。下列命令统一设置 `CARGO_TARGET_DIR=/tmp/drives-platform-audit-target`，使用默认 features；未开启 fastdds/cyclonedds/real-robot，未运行整个 workspace/all-targets，未写源文件。

| 命令 | 结果 |
| --- | --- |
| `cargo test -p libk1 --test safety --locked --offline` | PASS，12 项 |
| `cargo test -p libgo2 --test command_routing --locked --offline` | PASS，32 项；源码第 33 项由 cyclonedds + 非 Linux cfg 控制，本次未编译 |
| `cargo clippy -p libk1 --test safety --locked --offline -- -D warnings` | PASS |
| `cargo clippy -p libgo2 --test command_routing --locked --offline -- -D warnings` | FAIL，types/messages.rs:652、:657 两处 `clippy::chunks_exact_to_as_chunks` |

两项 Clippy 问题属于风格/新 lint，未修复。Cargo 还输出 workspace profile ignored、manual_readme 和 non_kebab_case_bins manifest 警告；Clippy 的 PASS 不等于 Cargo 全部无警告。默认 Go2 测试不启用 roplat feature，因此没有覆盖前文确定的旧 Rhythm API 不兼容。

根 Cargo.lock 前后 SHA256 均为 `af981eeb6f1bc9e316ea7e109508264706014655707944ba9776a9815d9ab1db`。本次 K1/Go2 仍 clean。结束时其他并行任务涉及的 libhans-rs/src/robot.rs、libjaka-rs/src/types/robot_type.rs、rsbullet/rsbullet/src/rsbullet.rs 出现 dirty，本 agent 没有写它们，已告知 root。

归档材料：`/tmp/drives-platform-verification.json`、`/tmp/drives-platform-metadata.json`、`/tmp/drives-platform-metadata-summary.json`、`/tmp/drives-platform-metadata.stderr`、`/tmp/drives-platform-lock-before.sha256`、`/tmp/drives-platform-lock-after.sha256` 和 `/tmp/drives-platform-lib*-*.log`。metadata summary 含 16 个 members 的版本、features、依赖以及 core/drives SHA。

## 7. 第三阶段：经授权的 Go2 两项 Clippy 小修

root 另行授权无设计决策的小修后，在原本 clean 的 unitree-go2-rs 创建 `codex/core-alignment-audit` 分支；只修改 `src/types/messages.rs:652` 的 `write_u8_words`。两次 `chunks_exact(4)` 改为一次 `chunks(4)`，每块复制进零初始化 `[u8;4]` 再 `u32::from_le_bytes`。保持小端序、末块补零、无堆分配和公开接口；未使用需要另行提高稳定版本要求的 `as_chunks`，未加 allow。crate 没有 rust-version，edition 为 2024，所用 chunks/copy_from_slice 为既有稳定接口。

修后重复原命令：Go2 mock 32/32 PASS、同一窄 Clippy `-D warnings` PASS、`git diff --check` PASS。Cargo manifest/profile 警告仍与之前相同。根 Cargo.lock SHA 未变。未修改生命周期/停止/状态新鲜度语义，未开 SDK feature，未 commit/push。

保留修前失败日志 `/tmp/drives-platform-libgo2-clippy.log`；修后完整日志为 `/tmp/drives-platform-libgo2-test-after.log` 和 `/tmp/drives-platform-libgo2-clippy-after.log`，验证 JSON 增加 post_fix 字段。前述“未改源”结论仅对应第一、二阶段；最终本 agent 只有这个 Go2 文件的小修。
