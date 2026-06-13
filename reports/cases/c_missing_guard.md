=== CRQA 代码质量检查 ===
静态质量分析

--- C 代码检查 ---
文件: examples/cases/c_missing_guard.h
行数: 5
问题: 1

[WARNING] C | line   1 | c-header-guard
  -> 头文件缺少 #pragma once 或传统 include guard，容易被重复包含

小结: error 0 | warning 1 | info 0

=== 统计报告 ===
错误: 0 | 警告: 1 | 建议: 0
检查文件: 1 | 总行数: 5
代码质量评分: 96/100
质量等级: 优秀
规则热点: c-header-guard x1
处理建议: 复查 warning，重点关注资源管理、unsafe 和裸 new/delete。
