# What?

This server will compile each article from Markdown to HTML and store it in memory, react to
file changes by recompiling the changed articles, and change the index page accordingly. Sorting
on the index page depends only on the file modification date.

Articles should be written in Markdown. You can use two Markdown extensions in your articles:
strikethrough and footnotes. The title of the article is determined by the first Markdown heading
that was found in the file (if the heading wasn't found, the file name without the extension is
used instead).

Oh, and also it re-colors the index server-side every time it is accessed (done using a vector of
all possible re-colors, so it is very cheap and happens server-side). Neat, isn't it?

# Why?

As a school project, to practice GitHub Actions and to remember the Rust programming language.

# How?

* Download the [last release](https://github.com/megahomyak/blog/releases/latest) of the program (let's assume that you renamed your executable to `blog` (or `blog.exe` on Windows))
* Execute `blog create_sample_config` (or `blog.exe create_sample_config` on Windows) to get your configuration sample (it won't work without the configuration)
* Edit the configuration you got from the step above
* Execute `blog run` (or `blog.exe run` on Windows) to run the web server
