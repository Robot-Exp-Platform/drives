# 2026-10-26 本地主线整合

本目录保留每个仓库原 main 与待合 PR 的 merge ancestry，未推送远端、发布 crate 或打 tag。
驱动使用 roplat_behavior 0.9 的四包契约；核心与行为仓为同级目录。

## 准备与 CI

- CI 从 `.github/integration-sources.json` 读取核心与行为仓的完整提交，分别 checkout 为同级 `roplat/`、`roplat_behavior/`。
- 仅初始化 Cargo 输入子模块；旧 skills、冻结文档站和无关参考仓不是编译前置。
- `ROPLAT_READ_TOKEN` 可提供跨私有仓的只读访问；未配置时回退到当前仓的 `GITHUB_TOKEN`。该回退不授予对私有核心、K1、Go2 等仓的权限，缺权限时准备步骤会失败。
- 本轮没有配置或上传任何凭据，也没有在远端运行 CI。新的本地整合提交推送前，固定的核心、行为和子模块提交均须在相应远端可取得。
- 默认依赖检查入口已改为同级四包行为仓，仍检查默认构建不引入 roplat 运行时、开启 `roplat` 后只解析一份核心身份。兼容的 0.5.0 / 0.5.1 范围可以共用同一实际版本。
- 独立消费者仍须等待未发布的 0.9 / 消息包可取得；本地 path 验证不是 crates.io 发布验证。

## 整合修复与验证

JAKA 的 0.9 迁移保留了已删除的 `utils::rad_to_deg` 导入，导致默认库编译失败。
两处数组转换改为等价的 `map(f64::to_degrees)`，没有改变控制指令的单位或设备协议。

Franka 原 TCP 写超时测试假设 16 MiB 一定超过 loopback 缓冲；Windows 上写入完成后
等待不回包的 peer，导致测试挂起。写入超时分支提取为等价 helper，以容量 1024 的
Tokio duplex 写入 2048 字节验证部分写后的 3ms 超时，并核对已传输的 1024 字节。
Unix 仍保留真实 TCP 集成测试并增加防挂看门狗；Windows 的该项背压检查是 helper 级验证。

Windows nightly 下，全 workspace 的 all-targets check（包含 Rerun、JAKA 多语言和
Bullet 默认 GUI 编译）通过；核心驱动的 Python/C++ FFI 组合 check 通过。
65 项有限驱动软件测试通过，原有独立 release 性能测试仍为 ignored。
只做软件编译与 loopback/mock 测试，不连接真机或启动 GUI。
最终组合验证和提交号由主线合并窗口的汇总记录保存。

## 历史实验

roplat-exp 的 main 仍包含旧 0.2.2 / robot_behavior 0.5.4 实验契约，不能把它直接
指向新行为仓当成迁移。实验仓提供独立 `legacy/` 恢复环境与明确版本清单；
已登记的 E1 交付及其较新依赖另存 `legacy-e1/`，
保留原结果、原 Cargo.lock 和旧源码；新生态检查不替代历史实验的复现。
