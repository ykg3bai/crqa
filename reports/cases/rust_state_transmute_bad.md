=== CRQA 代码质量检查 ===
静态质量分析

--- Rust 代码检查 ---
文件: examples/cases/rust_state_transmute_bad.rs
行数: 16
问题: 7

[WARNING] Rust | line   9 | rust-transmute
  -> 检测到 transmute，建议优先使用安全转换 API 或把 unsafe 边界压到最小
[WARNING] Rust | line   9 | rust-unsafe
  -> 检测到 unsafe 使用，建议收敛边界并补充 Safety 说明
[INFO] Rust | line   5 | rust-complex-type
  -> 检测到 Rc<RefCell> 或 Arc<Mutex> 组合，建议确认共享可变状态边界清晰
[INFO] Rust | line   7 | rust-clone-noise
  -> 检测到 clone/to_owned，建议确认是否真的需要分配或复制所有权
[INFO] Rust | line   8 | rust-clone-noise
  -> 检测到 clone/to_owned，建议确认是否真的需要分配或复制所有权
[INFO] Rust | line   9 | rust-unsafe-without-safety-comment
  -> unsafe 附近未看到 Safety 说明，建议写清前置条件和调用方责任
[INFO] Rust | line  11 | rust-dbg-macro
  -> 检测到 dbg! 调试宏，提交前建议改成日志或移除

小结: error 0 | warning 2 | info 5

=== 统计报告 ===
错误: 0 | 警告: 2 | 建议: 5
检查文件: 1 | 总行数: 16
代码质量评分: 74/100
质量等级: 可维护
规则热点: rust-clone-noise x2, rust-complex-type x1, rust-dbg-macro x1
处理建议: 复查 warning，重点关注资源管理、unsafe 和裸 new/delete。
