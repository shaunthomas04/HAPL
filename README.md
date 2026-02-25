# HAPL

<!-- INSERT BANNER IMAGE HERE -->
<!-- e.g. ![HAPL Banner](./assets/banner.png) -->

> **H**TML **A**s a **P**rogramming **L**anguage — write programs in plain HTML.

HAPL is a programming language implemented in Rust where the source code *is* a valid HTML file. Instead of a custom syntax, HAPL uses standard HTML tags, classes, and IDs to express variables, arithmetic, conditionals, loops, and functions. Open it in a browser and it looks like a webpage. Run it through the HAPL interpreter and it executes as a program.

---

## Table of Contents

- [How It Works](#how-it-works)
- [Running a Program](#running-a-program)
- [Literals and Types](#literals-and-types)
- [Variables](#variables)
- [Printing](#printing)
- [Arithmetic](#arithmetic)
- [Comparison and Logic](#comparison-and-logic)
- [Conditionals](#conditionals)
- [While Loops](#while-loops)
- [For Loops](#for-loops)
- [Functions](#functions)

---

<a name="how-it-works"></a>
## How It Works

HAPL programs are `.html` files. The interpreter parses the HTML into a tag tree, then walks that tree through three stages:

```
HTML file → Tag Extraction → Lexer → Token stream → Parser → AST → Interpreter → Output
```

Standard HTML boilerplate (`<html>`, `<head>`, `<body>`, `<meta>` etc.) is ignored transparently, so your HAPL code can live inside a real, valid HTML file.

---

<a name="running-a-program"></a>
## Running a Program

```bash
cargo run -- path/to/your/program.html
```

---

<a name="literals-and-types"></a>
## Literals and Types

HAPL has four primitive types. Literals are written inside `<span>` tags with a class indicating the type.

| Type | Class | Example |
|------|-------|---------|
| Integer | `integer` | `<span class="integer">42</span>` |
| Double | `double` | `<span class="double">3.14</span>` |
| String | `string` | `<span class="string">"hello"</span>` |
| Boolean | `boolean` | `<span class="boolean">true</span>` |

> String literals must be wrapped in double quotes inside the tag content.

---

<a name="variables"></a>
## Variables

### Declaration

Declare a variable with a `<var>` tag. The `class` is the type, the `id` is the variable name, and the content is the initial value.

```html
<!-- integer x = 10 -->
<var class="integer" id="x">
    <span class="integer">10</span>
</var>

<!-- string name = "Shaun" -->
<var class="string" id="name">
    <span class="string">"Shaun"</span>
</var>

<!-- boolean flag = true -->
<var class="boolean" id="flag">
    <span class="boolean">true</span>
</var>

<!-- double pi = 3.14 -->
<var class="double" id="pi">
    <span class="double">3.14</span>
</var>
```

### Reference

Reference an existing variable with a `<var>` tag that has only a `class` (the variable name) and no `id` or children.

```html
<!-- reference variable x -->
<var class="x"></var>
```

### Assignment

Reassign a variable with a `<var>` tag that has a `class` (the variable name) and children (the new value), but no `id`.

```html
<!-- x = 99 -->
<var class="x">
    <span class="integer">99</span>
</var>
```

---

<a name="printing"></a>
## Printing

Wrap any expression in a `<p>` tag to print it.

```html
<!-- print a literal -->
<p>
    <span class="string">"Hello, World!"</span>
</p>

<!-- print a variable -->
<p>
    <var class="name"></var>
</p>
```

---

<a name="arithmetic"></a>
## Arithmetic

Arithmetic operations are written as `<div>` tags with a class of `+`, `-`, `*`, or `/`. Operands are nested inside and can be literals, variable references, or other operations.

```html
<!-- 3 + 4 -->
<div class="+">
    <span class="integer">3</span>
    <span class="integer">4</span>
</div>

<!-- (2 + 3) * 4 -->
<div class="*">
    <div class="+">
        <span class="integer">2</span>
        <span class="integer">3</span>
    </div>
    <span class="integer">4</span>
</div>

<!-- print x + 10 -->
<p>
    <div class="+">
        <var class="x"></var>
        <span class="integer">10</span>
    </div>
</p>
```

---

<a name="comparison-and-logic"></a>
## Comparison and Logic

### Comparison Operators

| Operator | Class |
|----------|-------|
| `==` | `equal` |
| `!=` | `not_equal` |
| `<` | `less` |
| `<=` | `less_equal` |
| `>` | `greater` |
| `>=` | `greater_equal` |

```html
<!-- x == 10 -->
<div class="equal">
    <var class="x"></var>
    <span class="integer">10</span>
</div>

<!-- x > 5 -->
<div class="greater">
    <var class="x"></var>
    <span class="integer">5</span>
</div>
```

### Logical Operators

| Operator | Class |
|----------|-------|
| `&&` | `&&` |
| `\|\|` | `\|\|` |
| `!` | `!` |

```html
<!-- x > 0 && x < 100 -->
<div class="&&">
    <div class="greater">
        <var class="x"></var>
        <span class="integer">0</span>
    </div>
    <div class="less">
        <var class="x"></var>
        <span class="integer">100</span>
    </div>
</div>

<!-- !flag -->
<div class="!">
    <var class="flag"></var>
</div>
```

---

<a name="conditionals"></a>
## Conditionals

Conditionals use a `<div class="conditional">` wrapper containing `<div class="if">`, optional `<div class="elif">`, and optional `<div class="else">` blocks.

The **first expression** inside an `if` or `elif` block is the condition. Everything after it is the body.

```html
<div class="conditional">

    <div class="if">
        <!-- condition: x > 10 -->
        <div class="greater">
            <var class="x"></var>
            <span class="integer">10</span>
        </div>

        <!-- body -->
        <p>
            <span class="string">"x is greater than 10"</span>
        </p>
    </div>

    <div class="elif">
        <!-- condition: x == 10 -->
        <div class="equal">
            <var class="x"></var>
            <span class="integer">10</span>
        </div>

        <!-- body -->
        <p>
            <span class="string">"x is exactly 10"</span>
        </p>
    </div>

    <div class="else">
        <p>
            <span class="string">"x is less than 10"</span>
        </p>
    </div>

</div>
```

---

<a name="while-loops"></a>
## While Loops

A while loop uses `<div class="while">` with a `<div class="condition">` and a `<div class="body">`.

```html
<!-- while x > 0: print x, then x = x - 1 -->
<div class="while">

    <div class="condition">
        <div class="greater">
            <var class="x"></var>
            <span class="integer">0</span>
        </div>
    </div>

    <div class="body">
        <p>
            <var class="x"></var>
        </p>

        <!-- x = x - 1 -->
        <var class="x">
            <div class="-">
                <var class="x"></var>
                <span class="integer">1</span>
            </div>
        </var>
    </div>

</div>
```

---

<a name="for-loops"></a>
## For Loops

A for loop uses `<div class="for">` with four sections: `iterator`, `condition`, `increment`, and `body`. The iterator must be an integer variable.

```html
<!-- for i = 0; i < 5; i + 1 -->
<div class="for">

    <div class="iterator">
        <var class="i"></var>
    </div>

    <div class="condition">
        <div class="less">
            <var class="i"></var>
            <span class="integer">5</span>
        </div>
    </div>

    <div class="increment">
        <span class="integer">1</span>
    </div>

    <div class="body">
        <p>
            <var class="i"></var>
        </p>
    </div>

</div>
```

---

<a name="functions"></a>
## Functions

### Declaration

Declare a function with `<div class="{returnType}-function" id="{functionName}">`. It contains a `<div class="params">` and a `<div class="body">`.

Return types: `integer`, `double`, `string`, `boolean`, `void`

```html
<div class="void-function" id="greet">

    <div class="params">
        <div class="string-param" id="name"></div>
        <div class="integer-param" id="age"></div>
    </div>

    <div class="body">
        <p>
            <var class="name"></var>
        </p>
        <p>
            <var class="age"></var>
        </p>
    </div>

</div>
```

### Return Values

Use `<div class="return">` inside the function body to return a value.

```html
<div class="integer-function" id="double">

    <div class="params">
        <div class="integer-param" id="n"></div>
    </div>

    <div class="body">
        <div class="return">
            <div class="*">
                <span class="integer">2</span>
                <var class="n"></var>
            </div>
        </div>
    </div>

</div>
```

### Calling a Function

Call a function with `<div class="{functionName}">`. Pass arguments inside a `<div class="args">`. Arguments can be literals or variable references.

```html
<!-- greet("Shaun", 25) -->
<div class="greet">
    <div class="args">
        <span class="string">"Shaun"</span>
        <span class="integer">25</span>
    </div>
</div>

<!-- call with variable references -->
<var class="string" id="myName">
    <span class="string">"Shaun"</span>
</var>

<var class="integer" id="myAge">
    <span class="integer">25</span>
</var>

<div class="greet">
    <div class="args">
        <var class="myName"></var>
        <var class="myAge"></var>
    </div>
</div>
```

### Storing a Return Value

Use a function call as the value expression inside a variable declaration to capture the return value.

```html
<!-- result = circumference(7) -->
<var class="integer" id="result">
    <div class="circumference">
        <div class="args">
            <span class="integer">7</span>
        </div>
    </div>
</var>

<p>
    <var class="result"></var>
</p>
```

### Full Example

```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>HAPL Demo</title>
</head>
<body>

    <!-- circumference(radius) = 2 * 3 * radius -->
    <div class="integer-function" id="circumference">
        <div class="params">
            <div class="integer-param" id="radius"></div>
        </div>
        <div class="body">
            <div class="return">
                <div class="*">
                    <span class="integer">2</span>
                    <div class="*">
                        <span class="integer">3</span>
                        <var class="radius"></var>
                    </div>
                </div>
            </div>
        </div>
    </div>

    <!-- myRadius = 7 -->
    <var class="integer" id="myRadius">
        <span class="integer">7</span>
    </var>

    <!-- result = circumference(myRadius) -->
    <var class="integer" id="result">
        <div class="circumference">
            <div class="args">
                <var class="myRadius"></var>
            </div>
        </div>
    </var>

    <!-- print result -->
    <p>
        <var class="result"></var>
    </p>

</body>
</html>
```

Output:
```
42
```

---
