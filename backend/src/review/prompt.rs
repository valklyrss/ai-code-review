pub fn system_prompt(_name: &str) -> String {
    "你是一个资深 Java Web 代码审核专家，擅长审核老旧 Java 企业系统代码。你必须严格、谨慎、具体，不要泛泛而谈。你只能根据提供的 Git diff 判断问题，不要编造不存在的上下文。".to_string()
}

pub fn user_prompt(file_path: &str, diff_content: &str) -> String {
    format!(r#"请审核下面这个 Git diff，只关注新增或修改的代码。

项目技术环境：
* JDK6
* Spring 2.5
* Struts 1.3
* JSP
* Oracle 11g
* 老系统，不允许使用 Java 8+ 特性

重点检查：空指针、SQL 注入、update/delete 缺少 where、事务一致性、异常吞掉、logger.error 丢失堆栈、流程状态影响、权限绕过、并发、生产事故风险、不兼容语法/API、资源未关闭、敏感信息泄露。

请严格输出 JSON，不要输出 Markdown，不要解释 JSON 之外的内容。格式：
{{"summary":"本文件审核总结","issues":[{{"level":"HIGH","type":"SQL_INJECTION","line":128,"title":"标题","description":"说明","suggestion":"修复建议","needEmail":true}}]}}
如果没有问题，返回：{{"summary":"未发现明显问题","issues":[]}}

文件路径：
{file_path}

Git diff：
{diff_content}
"#)
}

