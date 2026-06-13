=== CRQA 代码质量检查 ===
静态质量分析

--- C 代码检查 ---
文件: examples/cases/c_resource_macro_bad.c
行数: 23
问题: 6

[WARNING] C | line   5 | c-macro-safety
  -> 函数式宏包含语句块但未使用 do { ... } while (0) 包裹，调用方容易踩坑
[WARNING] C | line  12 | c-dangerous-api
  -> 检测到危险 C API `scanf`，建议使用带长度限制或可检查错误的替代方案
[WARNING] C | line  13 | c-dangerous-api
  -> 检测到危险 C API `strcpy`，建议使用带长度限制或可检查错误的替代方案
[WARNING] C | line  14 | c-alloc-without-free
  -> 文件中检测到 malloc/calloc/realloc 但未看到 free，建议检查所有权和释放路径
[WARNING] C | line  14 | c-unchecked-resource
  -> 疑似直接调用资源/内存 API 且未保存返回值，建议检查失败路径和释放路径
[WARNING] C | line  17 | c-goto-usage
  -> 检测到 goto，建议确认控制流是否可以用早返回、循环状态或小函数表达

小结: error 0 | warning 6 | info 0

=== 统计报告 ===
错误: 0 | 警告: 6 | 建议: 0
检查文件: 1 | 总行数: 23
代码质量评分: 57/100
质量等级: 需要治理
规则热点: c-dangerous-api x2, c-alloc-without-free x1, c-goto-usage x1
处理建议: 复查 warning，重点关注资源管理、unsafe 和裸 new/delete。
