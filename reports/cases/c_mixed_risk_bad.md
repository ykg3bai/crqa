=== CRQA 代码质量检查 ===
静态质量分析

--- C 代码检查 ---
文件: examples/cases/c_mixed_risk_bad.c
行数: 12
问题: 8

[ERROR] C | line   5 | function-brace-newline
  -> 函数定义的首个左花括号应另起一行
[ERROR] C | line   5 | void-main-signature
  -> 建议将 void main() 改为 int main(void)
[ERROR] C | line   8 | assignment-in-condition
  -> 条件表达式中疑似误用赋值运算符 =，请确认是否应为 ==
[ERROR] C | line   8 | control-keyword-space
  -> if 语句后缺少空格，建议写成 `if (...)`
[ERROR] C | line   8 | empty-control-statement
  -> if 语句后不应出现空语句分号
[WARNING] C | line   9 | c-dangerous-api
  -> 检测到危险 C API `strcpy`，建议使用带长度限制或可检查错误的替代方案
[WARNING] C | line  10 | c-alloc-without-free
  -> 文件中检测到 malloc/calloc/realloc 但未看到 free，建议检查所有权和释放路径
[WARNING] C | line  10 | c-unchecked-resource
  -> 疑似直接调用资源/内存 API 且未保存返回值，建议检查失败路径和释放路径

小结: error 5 | warning 3 | info 0

=== 统计报告 ===
错误: 5 | 警告: 3 | 建议: 0
检查文件: 1 | 总行数: 12
代码质量评分: 19/100
质量等级: 需要立即整理
规则热点: assignment-in-condition x1, c-alloc-without-free x1, c-dangerous-api x1
处理建议: 优先处理 error 级问题。
