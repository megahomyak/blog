# Blog

This server will compile each Markdown (`.md`) article to HTML and store it in memory, react to
file changes by recompiling the changed articles, and change the index page accordingly. Sorting
on the index page depends only on the file modification date. Each article should reside in the
`/articles` directory, and other files should go to the `/files` directory.

Articles should be written in Markdown to be cached and added to the index. You can use two
Markdown extensions in your articles: strikethrough and footnotes. The title of the article is
determined by the first Markdown heading that was found in the file (if the heading wasn't found,
the file name without the extension is used instead).

Oh, and also it re-colors the index server-side every time it is accessed (done using a vector of
all possible re-colors). Neat, isn't it?
