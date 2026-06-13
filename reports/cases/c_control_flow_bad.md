=== CRQA 代码质量检查 ===
静态质量分析

--- C 代码检查 ---
文件: examples/cases/c_control_flow_bad.c
行数: 24
问题: 10

[ERROR] C | line   4 | function-brace-newline
  -> 函数定义的首个左花括号应另起一行
[ERROR] C | line   4 | void-main-signature
  -> 建议将 void main() 改为 int main(void)
[ERROR] C | line   8 | assignment-in-condition
  -> 条件表达式中疑似误用赋值运算符 =，请确认是否应为 ==
[ERROR] C | line   8 | control-keyword-space
  -> if 语句后缺少空格，建议写成 `if (...)`
[ERROR] C | line   8 | empty-control-statement
  -> if 语句后不应出现空语句分号
[ERROR] C | line   9 | equality-as-statement
  -> 独立语句中出现 ==，疑似将赋值 = 误写为比较 ==
[WARNING] C | line   2 | unused-include
  -> 头文件 'math.h' 可能未被使用
[WARNING] C | line   5 | unused-variable
  -> 变量 'unused' 定义后未使用
[WARNING] C | line  11 | single-line-for-braces
  -> 单行 for 循环不应使用花括号
[WARNING] C | line  17 | deep-nesting
  -> 嵌套深度超过 4 层，建议重构

小结: error 6 | warning 4 | info 0

=== 统计报告 ===
错误: 6 | 警告: 4 | 建议: 0
检查文件: 1 | 总行数: 24
代码质量评分: 16/100
质量等级: 需要立即整理
规则热点: assignment-in-condition x1, control-keyword-space x1, deep-nesting x1
处理建议: 优先处理 error 级问题。
