

在 Axum 中，您可以使用不同的方式来获取 GET 请求中的参数。以下是一些常见的方法：

Path 参数：
Path 参数，也称为路径参数，是直接从 URL 路径中提取的参数。
您可以将 URL 的一部分变成参数化，以便动态地处理不同的请求。
示例：GET /user/:id，其中 :id 是一个 Path 参数。
使用 axum::extract::Path 可以方便地获取 Path 参数。
URL 参数：
URL 参数是附加在 URL 后面的键值对，以 ? 开头，多个参数之间使用 & 分隔。
示例：GET /subject?page=1&keyword=axum.rs
使用 axum::extract::Query 可以获取 URL 参数。


使用 #[debug_handler] 可以查看axum的详细报错。