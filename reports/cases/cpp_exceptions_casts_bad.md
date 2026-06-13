=== CRQA 代码质量检查 ===
静态质量分析

--- C++ 代码检查 ---
文件: examples/cases/cpp_exceptions_casts_bad.cpp
行数: 26
问题: 6

[WARNING] C++ | line  13 | deep-nesting
  -> 嵌套深度超过 4 层，建议重构
[WARNING] C++ | line  21 | cpp-catch-all
  -> 检测到 catch (...)，建议只捕获明确异常类型并保留错误上下文
[INFO] C++ | line   5 | cpp-null-literal
  -> 检测到 NULL，现代 C++ 建议使用 nullptr 表达空指针
[INFO] C++ | line   6 | cpp-c-style-cast
  -> 检测到疑似 C 风格强转，建议使用 static_cast/reinterpret_cast/const_cast 表达意图
[INFO] C++ | line  14 | cpp-std-endl
  -> 检测到 endl，若不需要强制刷新缓冲区，建议使用 '\n'
[INFO] C++ | line  22 | cpp-std-endl
  -> 检测到 endl，若不需要强制刷新缓冲区，建议使用 '\n'

小结: error 0 | warning 2 | info 4

=== 统计报告 ===
错误: 0 | 警告: 2 | 建议: 4
检查文件: 1 | 总行数: 26
代码质量评分: 84/100
质量等级: 良好
规则热点: cpp-std-endl x2, cpp-c-style-cast x1, cpp-catch-all x1
处理建议: 复查 warning，重点关注资源管理、unsafe 和裸 new/delete。
