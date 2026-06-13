# CRQA

CRQA 是一个用 Rust 编写的 C / C++ / Rust 代码质量检查工具。它提供本地静态规则检查、文本报告、YAML 报告和可选的 DeepSeek 二次评分。

## 功能

- 支持文件检查：`-f FILE`
- 支持目录递归检查：`-d DIR`
- 支持语言：C、C++、Rust
- 可用 `--yaml` 输出结构化报告
- 可用 `-o FILE` 写入报告文件
- 可用 `--quiet` 只展示 error
- 可选 `--aireview` 调用 DeepSeek

目录扫描会尊重 `.gitignore`，并跳过 `.git/`、`build/`、`target/`。

## 安装

需要 Rust 工具链：

```bash
rustc --version
cargo --version
```

构建：

```bash
cargo build --release
```

直接运行：

```bash
./target/release/crqa --help
```

也可以把 release 二进制复制到项目根目录，方便本地体验：

```bash
cp target/release/crqa ./crqa
./crqa --help
```

安装到本机 PATH：

```bash
cargo install --path .
crqa --help
```

## 使用

检查单个文件：

```bash
cargo run -- -f examples/cases/c_mixed_risk_bad.c
```

检查多个文件：

```bash
cargo run -- -f examples/cases/c_clean.c -f examples/cases/rust_clean.rs
```

检查目录：

```bash
cargo run -- -d examples/cases
```

输出 YAML：

```bash
cargo run -- -d examples/cases --yaml
```

写入文件：

```bash
cargo run -- -d examples/cases -o reports/cases/all_cases.md
cargo run -- -d examples/cases --yaml -o reports/cases/all_cases.yaml
```

只展示 error：

```bash
cargo run -- -d examples/cases --quiet
```

`--quiet` 只影响文本展示，不影响统计、评分和退出码。

## 退出码

| 退出码 | 含义 |
| --- | --- |
| `0` | 分析完成，且没有 error |
| `1` | 分析完成，但存在 error |
| `2` | 参数错误、读取失败或 AI review 失败 |

## AI Review

AI review 固定使用 DeepSeek：

| 项 | 值 |
| --- | --- |
| model | `deepseek-v4-pro` |
| base URL | `https://api.deepseek.com` |
| endpoint | `/chat/completions` |

使用前设置 API key：

```bash
export DEEPSEEK_API_KEY="sk-..."
cargo run -- -f examples/cases/cpp_raw_namespace_bad.hpp --aireview
```

不要把真实 API key 写入源码、README、报告文件或提交历史。开源前可以检查：

```bash
rg "sk-[A-Za-z0-9]+" .
```

CRQA 的报告格式是文本或 YAML。AI review 内部使用 JSON 是 DeepSeek HTTP API 的协议要求，不是报告输出格式。

## 评分

本地评分满分 100，最低 0。每条问题按规则风险扣分。

高风险规则扣分较高，例如：

- `assignment-in-condition`
- `empty-control-statement`
- `rust-transmute`
- `c-alloc-without-free`
- `c-dangerous-api`
- `rust-unsafe`

提示类规则扣分较低，例如：

- `rust-dbg-macro`
- `rust-allow-attr`
- `cpp-std-endl`
- `cpp-null-literal`


## 规则概览

### C

| 规则 ID | 级别 | 说明 |
| --- | --- | --- |
| `function-brace-newline` | error | 函数定义首个左花括号应另起一行 |
| `single-line-for-braces` | warning | 单行 for 循环不应使用花括号 |
| `void-main-signature` | error | `void main()` 应改为 `int main(void)` |
| `unused-include` | warning | 可能未使用的头文件 |
| `unused-variable` | warning | 可能未使用的变量 |
| `assignment-in-condition` | error | 条件表达式中疑似误用赋值运算符 |
| `equality-as-statement` | error | 独立语句中疑似误用 `==` |
| `empty-control-statement` | error | if/for/while 后的空语句 |
| `long-function` | info | 函数超过 100 行 |
| `deep-nesting` | warning | 嵌套超过 4 层 |
| `control-keyword-space` | error | 控制关键字后缺少空格 |
| `c-dangerous-api` | warning | 高风险 C API |
| `c-macro-safety` | warning | 语句块宏缺少 `do { ... } while (0)` |
| `c-header-guard` | warning | 头文件缺少 include guard |
| `c-unchecked-resource` | warning | 资源 API 返回值未保存 |
| `c-goto-usage` | warning | 使用 `goto` |
| `c-alloc-without-free` | warning | 出现分配调用但未看到 `free` |

### C++

| 规则 ID | 级别 | 说明 |
| --- | --- | --- |
| `long-function` | info | 函数超过 100 行 |
| `deep-nesting` | warning | 嵌套超过 4 层 |
| `assignment-in-condition` | error | 条件表达式中疑似误用赋值运算符 |
| `empty-control-statement` | error | if/for/while 后的空语句 |
| `cpp-too-many-includes` | warning | include 数量过多 |
| `cpp-using-namespace` | info | 使用 `using namespace` |
| `cpp-header-using-namespace` | warning | 头文件中使用 `using namespace` |
| `cpp-raw-new-delete` | warning | 使用裸 `new/delete` |
| `cpp-c-style-cast` | info | 疑似 C 风格强制转换 |
| `cpp-catch-all` | warning | 使用 `catch (...)` |
| `cpp-std-endl` | info | 使用 `std::endl` 或输出流中的 `endl` |
| `cpp-null-literal` | info | 使用 `NULL` |
| `control-keyword-space` | error | 控制关键字后缺少空格 |

### Rust

| 规则 ID | 级别 | 说明 |
| --- | --- | --- |
| `long-function` | info | 函数超过 100 行 |
| `deep-nesting` | warning | 嵌套超过 4 层 |
| `rust-panic-unwrap` | warning | 使用 `unwrap/expect/panic/todo/unimplemented` |
| `rust-unsafe` | warning | 使用 `unsafe` |
| `rust-large-impl` | info | `impl` 块过大 |
| `rust-allow-attr` | info | 使用 `allow(...)` 属性 |
| `rust-dbg-macro` | info | 使用 `dbg!` |
| `rust-clone-noise` | info | 使用 `clone/to_owned` |
| `rust-complex-type` | info | 使用 `Rc<RefCell>` / `Arc<Mutex>` |
| `rust-transmute` | warning | 使用 `transmute` |
| `rust-as-cast` | info | 使用 `as` 转换 |
| `rust-unsafe-without-safety-comment` | info | unsafe 附近缺少 Safety 说明 |

## 样例报告

样例源码位于 `examples/cases/`，报告位于 `reports/cases/`。

重新生成样例报告：

```bash
chmod +x ./run.sh
./run.sh
```
