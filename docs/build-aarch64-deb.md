如何为 aarch64 (arm64) 架构构建 tuwunel 的 Debian (.deb) 包

概述
 - 本文档说明如何在 x86_64 或任意 Linux 主机上，生成适用于 aarch64-unknown-linux-gnu 的 .deb 包。
 - 推荐使用 cross（基于 Docker + QEMU），可以避免在宿主机安装交叉编译工具链。

先决条件
 - Rust（建议使用 rustup 管理 toolchain）
 - Docker（如果使用 cross）
 - cross（可选，但推荐）：cargo install cross 或 apt 安装含包
 - cargo-deb：cargo install cargo-deb
 - 如果不使用 cross，需要安装目标与交叉链接器：
   - rustup target add aarch64-unknown-linux-gnu
   - 在 Debian/Ubuntu 上安装： gcc-aarch64-linux-gnu 或 aarch64-linux-gnu-gcc

使用脚本构建

仓库中提供了脚本： `scripts/build-deb-aarch64.sh`。

用法：

```
./scripts/build-deb-aarch64.sh
```

脚本逻辑
 - 优先使用 `cross build --target aarch64-unknown-linux-gnu --release`。
 - 若没有 cross，会尝试用 `cargo build --target aarch64-unknown-linux-gnu --release`（需本地交叉工具链）。
 - 将生成的二进制复制到 `target/release/tuwunel`，然后运行 `cargo deb --no-build` 生成 .deb。
 - 最终输出放在 `out/` 目录下。

常见问题
 - 找不到 cross：安装 `cargo install cross`，或在仓库根使用 Docker（需可用的 Docker）。
 - cargo-deb 未安装： `cargo install cargo-deb`。
 - 本地交叉构建失败：建议使用 cross；若必须本地构建，请确保交叉链接器已安装并在 PATH 中。

进阶：在 CI 中使用
 - 在 CI（例如 GitHub Actions）中，可使用官方 cross 镜像并运行脚本，或使用 QEMU + Docker runner。

其他说明
 - 打包元数据位于 `src/main/Cargo.toml` 的 `[package.metadata.deb]` 部分，包含了 systemd unit、配置文件、维护脚本路径等信息。
