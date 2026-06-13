=== CRQA 代码质量检查 ===
静态质量分析

--- C++ 代码检查 ---
文件: examples/cases/cpp_raw_namespace_bad.hpp
行数: 15
问题: 10

[ERROR] C++ | line  10 | assignment-in-condition
  -> 条件表达式中疑似误用赋值运算符 =，请确认是否应为 ==
[ERROR] C++ | line  10 | control-keyword-space
  -> if 语句后缺少空格，建议写成 `if (...)`
[ERROR] C++ | line  10 | empty-control-statement
  -> if 语句后不应出现空语句分号
[WARNING] C++ | line   3 | cpp-header-using-namespace
  -> 头文件中不应使用 using namespace，容易污染包含方命名空间
[WARNING] C++ | line   7 | cpp-raw-new-delete
  -> 检测到裸 new/delete，建议优先使用 RAII、智能指针或容器
[WARNING] C++ | line  10 | cpp-raw-new-delete
  -> 检测到裸 new/delete，建议优先使用 RAII、智能指针或容器
[WARNING] C++ | line  13 | cpp-raw-new-delete
  -> 检测到裸 new/delete，建议优先使用 RAII、智能指针或容器
[INFO] C++ | line   3 | cpp-using-namespace
  -> 检测到 using namespace，建议缩小作用域或使用显式命名空间
[INFO] C++ | line   8 | cpp-c-style-cast
  -> 检测到疑似 C 风格强转，建议使用 static_cast/reinterpret_cast/const_cast 表达意图
[INFO] C++ | line  12 | cpp-std-endl
  -> 检测到 endl，若不需要强制刷新缓冲区，建议使用 '\n'

小结: error 3 | warning 4 | info 3

=== 统计报告 ===
错误: 3 | 警告: 4 | 建议: 3
检查文件: 1 | 总行数: 15
代码质量评分: 32/100
质量等级: 风险较高
规则热点: cpp-raw-new-delete x3, assignment-in-condition x1, control-keyword-space x1
处理建议: 优先处理 error 级问题。
