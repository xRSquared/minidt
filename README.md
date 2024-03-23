<div align="center">
  <p><strong>MiniDT: a simple but powerful jinja compiler for SQL</strong></p>
</div>

MiniDT is a powerful tool that allows you to use Jinja2 templates to generate SQL queries.
It is designed to be a lightweight alternative to dbt, for cases where dbt is too much, and raw SQL is too little.
All without the need for a database connection, or a complex setup!

As simple as:

```bash
pipx install minidt
minidt init
```

Now you can use MiniDT to compile your SQL templates:

```bash
minidt compile my_template.sql -o my_query.sql
```

## Getting Started

### Installation

### Setup

Before using MiniDT, you need to create a `.minidt` file in the root of your project.
This is where you can define your configuration settings.

By Defualt, MiniDT will look for macros in the `macros` directory, and templates
in the `templates` directory. You can change these paths in the `.minidt` file:

## Design

Because we never make any assumptions about the data, we can't provide any data validation or type checking.

```
```
