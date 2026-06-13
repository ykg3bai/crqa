=== CRQA 代码质量检查 ===
静态质量分析

--- Rust 代码检查 ---
文件: examples/cases/rust_panic_unsafe_bad.rs
行数: 31
问题: 10

[WARNING] Rust | line   3 | rust-unsafe
  -> 检测到 unsafe 使用，建议收敛边界并补充 Safety 说明
[WARNING] Rust | line   4 | rust-unsafe
  -> 检测到 unsafe 使用，建议收敛边界并补充 Safety 说明
[WARNING] Rust | line  10 | rust-panic-unwrap
  -> 检测到 unwrap/expect/panic/todo/unimplemented，建议显式处理错误或限制到测试代码
[WARNING] Rust | line  18 | deep-nesting
  -> 嵌套深度超过 4 层，建议重构
[WARNING] Rust | line  19 | rust-panic-unwrap
  -> 检测到 unwrap/expect/panic/todo/unimplemented，建议显式处理错误或限制到测试代码
[INFO] Rust | line   1 | rust-allow-attr
  -> 检测到 allow 属性，建议确认是否有明确原因和最小作用域
[INFO] Rust | line   3 | rust-unsafe-without-safety-comment
  -> unsafe 附近未看到 Safety 说明，建议写清前置条件和调用方责任
[INFO] Rust | line   4 | rust-unsafe-without-safety-comment
  -> unsafe 附近未看到 Safety 说明，建议写清前置条件和调用方责任
[INFO] Rust | line  11 | rust-as-cast
  -> 检测到 as 转换，建议确认截断、符号位和平台相关行为是否符合预期
[INFO] Rust | line  12 | rust-dbg-macro
  -> 检测到 dbg! 调试宏，提交前建议改成日志或移除

小结: error 0 | warning 5 | info 5

=== 统计报告 ===
错误: 0 | 警告: 5 | 建议: 5
检查文件: 1 | 总行数: 31
代码质量评分: 57/100
质量等级: 需要治理
规则热点: rust-panic-unwrap x2, rust-unsafe x2, rust-unsafe-without-safety-comment x2
处理建议: 复查 warning，重点关注资源管理、unsafe 和裸 new/delete。
