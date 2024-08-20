# notion-async

A notion sync tool, in `async` style.

```
Usage: notion-async [OPTIONS] <COMMAND>

Commands:
  sync  Sync all pages/databases/comments into db, recursively
  help  Print this message or the help of the given subcommand(s)

Options:
      --token <TOKEN>  Notion integration token, can get from: https://www.notion.so/my-integrations. If it's not set, will read from env var NOTION_TOKEN
      --db <FILE>      Sqlite database file path [default: notion.db]
  -h, --help           Print help
  -V, --version        Print version
```
