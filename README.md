# notion-async

A notion sync tool, in `async` style.

## Usage

Just set the following environment variables (`.env` file is also supported):

- `NOTION_TOKEN`: Your notion integration token, can get from: [notion
  integrations](https://www.notion.so/my-integrations)
- `NOTION_ROOT_ID`: The root notion page/database ID which you want to sync.
  The tool will sync all children pages/databases/comments into a sqlite db
  file, *recursively*. You can get the ID from any notion page/database link.
  
Then run `cargo run sync`, everything under the `NOTION_ROOT_ID` will be
synchronized into `notion.db` (can be changed by command line argument).

You can also set the token & id in the command line arguments, please refer to
the help message.

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

## Roadmap

The features will be implemented one by one in order.

- [x] Basic notion async API (`notion-async-api` crate)
- [x] Sync into sqlite database
- [ ] Simple query
- [ ] Full-Text search
- [ ] Export as markdown files
