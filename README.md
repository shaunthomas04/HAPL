# HAPL — HTML As A Programming Language

HAPL is a programming language written in valid HTML. Every program is a real `.html` file. Tags are statements, classes are keywords, and ids are names. A HAPL program is also a webpage — your browser can open it, and HAPL can run it.

```html
<html>
<body>
  <p><span class="string">"Hello, world!"</span></p>
</body>
</html>
```

```
$ HAPL hello.html
Hello, world!
```

---

## Table of contents

- [Installation](#installation)
- [Running a program](#running-a-program)
- [Language reference](#language-reference)
  - [Types](#types)
  - [Variables](#variables)
  - [Printing](#printing)
  - [Operators](#operators)
  - [Conditionals](#conditionals)
  - [While loops](#while-loops)
  - [For loops](#for-loops)
  - [Functions](#functions)
  - [Lists](#lists)
  - [Maps](#maps)
  - [User input](#user-input)
  - [HTTP requests](#http-requests)
  - [HTTP server](#http-server)
- [Keyword remapping](#keyword-remapping)
- [Complete examples](#complete-examples)
- [Error messages](#error-messages)

---

## Installation

HAPL requires [Rust](https://rustup.rs) (stable, 1.70+).

```bash
git clone https://github.com/your-username/hapl
cd hapl
cargo build --release
```

The compiled binary lives at `target/release/HAPL`. You can copy it anywhere on your `PATH`.

---

## Running a program

```bash
cargo run -- path/to/program.html
```

Or with the release build:

```bash
./target/release/HAPL path/to/program.html
```

---

## Language reference

### Types

HAPL has six types. Every variable must be declared with one.

| Type      | HTML class  | Example value      |
|-----------|-------------|--------------------|
| `integer` | `integer`   | `42`, `-7`         |
| `double`  | `double`    | `3.14`, `-0.5`     |
| `string`  | `string`    | `"hello"`          |
| `boolean` | `boolean`   | `true`, `false`    |
| `map`     | `map`       | `{ "key": value }` |
| list      | `TYPE-list` | `[1, 2, 3]`        |

Strings must always be wrapped in double quotes inside the span content.

---

### Variables

#### Declaration

```html
<var class="TYPE" id="NAME">
  VALUE
</var>
```

`TYPE` is one of `integer`, `double`, `string`, `boolean`, or `map`. `NAME` is the variable name. `VALUE` is any expression — a literal, another variable, an operation, or a function call.

```html
<!-- integer -->
<var class="integer" id="age">
  <span class="integer">25</span>
</var>

<!-- double -->
<var class="double" id="pi">
  <span class="double">3.14159</span>
</var>

<!-- string -->
<var class="string" id="name">
  <span class="string">"Alice"</span>
</var>

<!-- boolean -->
<var class="boolean" id="active">
  <span class="boolean">true</span>
</var>
```

#### Reference

To read a variable, use a `<var>` with no `id` and no children:

```html
<!-- Print the value of 'age' -->
<p><var class="age"></var></p>
```

#### Assignment

To overwrite a variable, use a `<var>` with no `id` but with a child value:

```html
<var class="age">
  <span class="integer">30</span>
</var>
```

---

### Printing

Use the `<p>` tag. The single child is the value to print.

```html
<!-- Print a string literal -->
<p><span class="string">"Hello, world!"</span></p>

<!-- Print a variable -->
<p><var class="name"></var></p>

<!-- Print the result of an expression -->
<p>
  <div class="+">
    <span class="integer">10</span>
    <span class="integer">5</span>
  </div>
</p>
```

---

### Operators

All operators are `<div>` tags. The operands are the children.

#### Arithmetic

| Operation      | Class | Min operands |
|----------------|-------|--------------|
| Addition       | `+`   | 2+           |
| Subtraction    | `-`   | 2+           |
| Multiplication | `*`   | 2+           |
| Division       | `/`   | 2+           |
| Modulo         | `%`   | 2+           |

```html
<!-- 10 + 5 = 15 -->
<div class="+">
  <span class="integer">10</span>
  <span class="integer">5</span>
</div>

<!-- Operators nest freely: (2 * 3) + 1 = 7 -->
<div class="+">
  <div class="*">
    <span class="integer">2</span>
    <span class="integer">3</span>
  </div>
  <span class="integer">1</span>
</div>

<!-- String concatenation with + -->
<div class="+">
  <span class="string">"Hello, "</span>
  <var class="name"></var>
</div>
```

#### Comparison

All comparison operators require exactly 2 operands and return a `boolean`.

| Operation             | Class           |
|-----------------------|-----------------|
| Equal                 | `equal`         |
| Not equal             | `not_equal`     |
| Less than             | `less`          |
| Less than or equal    | `less_equal`    |
| Greater than          | `greater`       |
| Greater than or equal | `greater_equal` |

```html
<!-- age >= 18 -->
<div class="greater_equal">
  <var class="age"></var>
  <span class="integer">18</span>
</div>
```

#### Boolean

| Operation | Class | Notes                       |
|-----------|-------|-----------------------------|
| And       | `&&`  | 2+ operands, short-circuits |
| Or        | `\|\|`  | 2+ operands, short-circuits |
| Not       | `!`   | exactly 1 operand           |

`&&` and `||` short-circuit — if the result is determined by the first operand, the rest are never evaluated.

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

<!-- !active -->
<div class="!">
  <var class="active"></var>
</div>
```

---

### Conditionals

```html
<div class="conditional">
  <div class="if">
    CONDITION
    STATEMENTS...
  </div>
  <div class="elif">
    CONDITION
    STATEMENTS...
  </div>
  <div class="else">
    STATEMENTS...
  </div>
</div>
```

`elif` and `else` are optional. You can have as many `elif` blocks as you need.

```html
<var class="integer" id="score">
  <span class="integer">85</span>
</var>

<div class="conditional">
  <div class="if">
    <div class="greater_equal">
      <var class="score"></var>
      <span class="integer">90</span>
    </div>
    <p><span class="string">"Grade: A"</span></p>
  </div>
  <div class="elif">
    <div class="greater_equal">
      <var class="score"></var>
      <span class="integer">80</span>
    </div>
    <p><span class="string">"Grade: B"</span></p>
  </div>
  <div class="else">
    <p><span class="string">"Grade: C or lower"</span></p>
  </div>
</div>
```

---

### While loops

```html
<div class="while">
  <div class="condition">
    BOOLEAN_EXPRESSION
  </div>
  <div class="body">
    STATEMENTS...
  </div>
</div>
```

```html
<var class="integer" id="count">
  <span class="integer">0</span>
</var>

<div class="while">
  <div class="condition">
    <div class="less">
      <var class="count"></var>
      <span class="integer">5</span>
    </div>
  </div>
  <div class="body">
    <p><var class="count"></var></p>
    <var class="count">
      <div class="+">
        <var class="count"></var>
        <span class="integer">1</span>
      </div>
    </var>
  </div>
</div>
```

---

### For loops

```html
<div class="for">
  <div class="iterator">
    VARIABLE_REFERENCE
  </div>
  <div class="condition">
    BOOLEAN_EXPRESSION
  </div>
  <div class="increment">
    INTEGER_EXPRESSION
  </div>
  <div class="body">
    STATEMENTS...
  </div>
</div>
```

The iterator variable is automatically declared as `integer` and starts at `0`. The increment expression's value is added to it at the end of each iteration.

```html
<!-- Print 0, 1, 2, 3, 4 -->
<div class="for">
  <div class="iterator"><var class="i"></var></div>
  <div class="condition">
    <div class="less">
      <var class="i"></var>
      <span class="integer">5</span>
    </div>
  </div>
  <div class="increment"><span class="integer">1</span></div>
  <div class="body">
    <p><var class="i"></var></p>
  </div>
</div>
```

Any integer expression works as the increment, so you can step by any amount:

```html
<!-- Count by 2: 0, 2, 4, 6, 8 -->
<div class="increment">
  <span class="integer">2</span>
</div>
```

---

### Functions

#### Declaration

```html
<div class="RETURNTYPE-function" id="NAME">
  <div class="params">
    <div class="PARAMTYPE-param" id="PARAMNAME"></div>
    ...
  </div>
  <div class="body">
    STATEMENTS...
    <div class="return">RETURN_VALUE</div>
  </div>
</div>
```

Valid return types: `integer`, `double`, `string`, `boolean`, `map`, `void`.
Valid parameter types: `integer`, `double`, `string`, `boolean`, `map`.

```html
<!-- integer add(integer a, integer b) -->
<div class="integer-function" id="add">
  <div class="params">
    <div class="integer-param" id="a"></div>
    <div class="integer-param" id="b"></div>
  </div>
  <div class="body">
    <div class="return">
      <div class="+">
        <var class="a"></var>
        <var class="b"></var>
      </div>
    </div>
  </div>
</div>

<!-- void greet(string name) -->
<div class="void-function" id="greet">
  <div class="params">
    <div class="string-param" id="name"></div>
  </div>
  <div class="body">
    <p>
      <div class="+">
        <span class="string">"Hello, "</span>
        <var class="name"></var>
      </div>
    </p>
  </div>
</div>
```

#### Calling a function

```html
<div class="FUNCTIONNAME">
  <div class="args">
    ARG1
    ARG2
    ...
  </div>
</div>
```

```html
<!-- result = add(3, 4) -->
<var class="integer" id="result">
  <div class="add">
    <div class="args">
      <span class="integer">3</span>
      <span class="integer">4</span>
    </div>
  </div>
</var>

<p><var class="result"></var></p>

<!-- greet("Bob") — void, used as a statement -->
<div class="greet">
  <div class="args">
    <span class="string">"Bob"</span>
  </div>
</div>
```

Functions can be called before they are declared — HAPL pre-scans all function signatures before executing.

---

### Lists

#### Declaration

```html
<div class="TYPE-list" id="NAME">
  <span class="TYPE">VALUE</span>
  ...
</div>
```

Valid element types: `integer`, `double`, `string`, `boolean`.

```html
<div class="integer-list" id="scores">
  <span class="integer">10</span>
  <span class="integer">20</span>
  <span class="integer">30</span>
</div>
```

All elements must match the declared type. Pushing or assigning a value of the wrong type is a runtime error.

#### Access by index

```html
<!-- scores[1] — evaluates to 20 -->
<p>
  <div class="index">
    <var class="scores"></var>
    <span class="integer">1</span>
  </div>
</p>
```

#### Assign by index

```html
<!-- scores[0] = 99 -->
<div class="index-assign">
  <var class="scores"></var>
  <span class="integer">0</span>
  <span class="integer">99</span>
</div>
```

#### Push (append to end)

```html
<div class="push">
  <var class="scores"></var>
  <span class="integer">40</span>
</div>
```

#### Pop (remove and return last element)

```html
<var class="integer" id="last">
  <div class="pop">
    <var class="scores"></var>
  </div>
</var>
```

#### Length

Returns an `integer`. Also works on strings and maps.

```html
<p>
  <div class="length">
    <var class="scores"></var>
  </div>
</p>
```

---

### Maps

A map is a string-keyed dictionary. Values can be any type.

#### Declaration

The `class` attribute of each child div is used as the key.

```html
<div class="map" id="person">
  <div class="name"><span class="string">"Alice"</span></div>
  <div class="age"><span class="integer">30</span></div>
  <div class="active"><span class="boolean">true</span></div>
</div>
```

#### Get a value

```html
<!-- person["name"] -->
<p>
  <div class="map-get">
    <var class="person"></var>
    <div class="key"><span class="string">"name"</span></div>
  </div>
</p>
```

#### Set a value

```html
<div class="map-set">
  <var class="person"></var>
  <div class="key"><span class="string">"age"</span></div>
  <div class="value"><span class="integer">31</span></div>
</div>
```

#### Remove a key

```html
<div class="map-remove">
  <var class="person"></var>
  <div class="key"><span class="string">"active"</span></div>
</div>
```

#### Check if a key exists

Returns `boolean`. Always use this before `map-get` if the key might not be present.

```html
<div class="conditional">
  <div class="if">
    <div class="map-contains">
      <var class="person"></var>
      <div class="key"><span class="string">"email"</span></div>
    </div>
    <p>
      <div class="map-get">
        <var class="person"></var>
        <div class="key"><span class="string">"email"</span></div>
      </div>
    </p>
  </div>
</div>
```

---

### User input

Reads a line from stdin and returns it as a `string`. It takes no children.

```html
<p><span class="string">"Enter your name: "</span></p>

<var class="string" id="username">
  <div class="input"></div>
</var>

<p>
  <div class="+">
    <span class="string">"Hello, "</span>
    <var class="username"></var>
  </div>
</p>
```

---

### HTTP requests

#### GET

Sends a GET request and stores the JSON response as a `map` with the given `id`.

```html
<div class="http-get" id="RESPONSENAME">
  <div class="url">
    <span class="string">"https://example.com/api/endpoint"</span>
  </div>
</div>
```

```html
<div class="http-get" id="todo">
  <div class="url">
    <span class="string">"https://jsonplaceholder.typicode.com/todos/1"</span>
  </div>
</div>

<p>
  <div class="map-get">
    <var class="todo"></var>
    <div class="key"><span class="string">"title"</span></div>
  </div>
</p>
```

JSON objects become maps, and JSON arrays become typed lists. If the request fails, the response map contains an `"error"` key with a description.

#### POST

Sends a POST request with a map variable serialised as the JSON body.

```html
<div class="http-post" id="RESPONSENAME">
  <div class="url">
    <span class="string">"https://example.com/api/endpoint"</span>
  </div>
  <div class="body">
    <var class="MAPNAME"></var>
  </div>
</div>
```

```html
<div class="map" id="payload">
  <div class="title"><span class="string">"Buy milk"</span></div>
  <div class="done"><span class="boolean">false</span></div>
</div>

<div class="http-post" id="created">
  <div class="url">
    <span class="string">"https://jsonplaceholder.typicode.com/todos"</span>
  </div>
  <div class="body">
    <var class="payload"></var>
  </div>
</div>

<p>
  <div class="map-get">
    <var class="created"></var>
    <div class="key"><span class="string">"id"</span></div>
  </div>
</p>
```

---

### HTTP server

```html
<div class="server" id="PORT">
  <div class="endpoint-get" id="/PATH">
    <div class="handler">
      STATEMENTS...
      <div class="respond">RESPONSE_VALUE</div>
    </div>
  </div>
  <div class="endpoint-post" id="/PATH">
    <div class="handler">
      STATEMENTS...
      <div class="respond">RESPONSE_VALUE</div>
    </div>
  </div>
</div>
```

Inside every handler, two variables are automatically available:

- `params` — a `map` of query string key/value pairs
- `body` — a `map` of the parsed JSON request body (POST only)

The value passed to `<div class="respond">` becomes the HTTP response body. Maps are serialised to JSON automatically.

```html
<div class="server" id="8080">

  <div class="endpoint-get" id="/hello">
    <div class="handler">
      <div class="respond">
        <span class="string">"Hello from HAPL!"</span>
      </div>
    </div>
  </div>

  <div class="endpoint-post" id="/echo">
    <div class="handler">
      <div class="respond">
        <var class="body"></var>
      </div>
    </div>
  </div>

</div>
```

```
$ HAPL server.html
HAPL server running on http://localhost:8080

$ curl http://localhost:8080/hello
Hello from HAPL!

$ curl -X POST http://localhost:8080/echo \
    -H "Content-Type: application/json" \
    -d '{"message": "hi"}'
{"message":"hi"}
```

Query string parameters are available via the `params` map:

```html
<div class="endpoint-get" id="/greet">
  <div class="handler">
    <div class="respond">
      <div class="+">
        <span class="string">"Hello, "</span>
        <div class="map-get">
          <var class="params"></var>
          <div class="key"><span class="string">"name"</span></div>
        </div>
      </div>
    </div>
  </div>
</div>
```

```
$ curl "http://localhost:8080/greet?name=Alice"
Hello, Alice
```

---

## Keyword remapping

You can provide a JSON config file to give any HAPL keyword a custom name or alias. This is useful for shorthand, or writing HAPL programs in a different spoken language.

```bash
HAPL myprogram.html --keyword-configs remap.json
```

The config file maps your custom name to the real HAPL keyword:

```json
{
  "afficher": "p",
  "ajouter": "+",
  "egal": "equal"
}
```

With this loaded you can write `<div class="egal">` instead of `<div class="equal">`. Any keyword can be remapped — operators, type names, control flow, everything.

---

## Complete examples

### FizzBuzz

```html
<html>
<body>

<div class="for">
  <div class="iterator"><var class="i"></var></div>
  <div class="condition">
    <div class="less_equal">
      <var class="i"></var>
      <span class="integer">100</span>
    </div>
  </div>
  <div class="increment"><span class="integer">1</span></div>
  <div class="body">
    <div class="conditional">
      <div class="if">
        <div class="equal">
          <div class="%"><var class="i"></var><span class="integer">15</span></div>
          <span class="integer">0</span>
        </div>
        <p><span class="string">"FizzBuzz"</span></p>
      </div>
      <div class="elif">
        <div class="equal">
          <div class="%"><var class="i"></var><span class="integer">3</span></div>
          <span class="integer">0</span>
        </div>
        <p><span class="string">"Fizz"</span></p>
      </div>
      <div class="elif">
        <div class="equal">
          <div class="%"><var class="i"></var><span class="integer">5</span></div>
          <span class="integer">0</span>
        </div>
        <p><span class="string">"Buzz"</span></p>
      </div>
      <div class="else">
        <p><var class="i"></var></p>
      </div>
    </div>
  </div>
</div>

</body>
</html>
```

---

### Factorial (recursive)

```html
<html>
<body>

<div class="integer-function" id="factorial">
  <div class="params">
    <div class="integer-param" id="n"></div>
  </div>
  <div class="body">
    <div class="conditional">
      <div class="if">
        <div class="less_equal">
          <var class="n"></var>
          <span class="integer">1</span>
        </div>
        <div class="return"><span class="integer">1</span></div>
      </div>
    </div>
    <div class="return">
      <div class="*">
        <var class="n"></var>
        <div class="factorial">
          <div class="args">
            <div class="-">
              <var class="n"></var>
              <span class="integer">1</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</div>

<p>
  <div class="factorial">
    <div class="args"><span class="integer">10</span></div>
  </div>
</p>

</body>
</html>
```

---

### Sum a list

```html
<html>
<body>

<div class="integer-list" id="numbers">
  <span class="integer">10</span>
  <span class="integer">20</span>
  <span class="integer">30</span>
  <span class="integer">40</span>
  <span class="integer">50</span>
</div>

<var class="integer" id="total"><span class="integer">0</span></var>
<var class="integer" id="idx"><span class="integer">0</span></var>

<div class="while">
  <div class="condition">
    <div class="less">
      <var class="idx"></var>
      <div class="length"><var class="numbers"></var></div>
    </div>
  </div>
  <div class="body">
    <var class="total">
      <div class="+">
        <var class="total"></var>
        <div class="index">
          <var class="numbers"></var>
          <var class="idx"></var>
        </div>
      </div>
    </var>
    <var class="idx">
      <div class="+"><var class="idx"></var><span class="integer">1</span></div>
    </var>
  </div>
</div>

<p><var class="total"></var></p>

</body>
</html>
```

---

### Map as a record

```html
<html>
<body>

<div class="map" id="user">
  <div class="name"><span class="string">"Alice"</span></div>
  <div class="score"><span class="integer">0</span></div>
</div>

<!-- Increment the score -->
<div class="map-set">
  <var class="user"></var>
  <div class="key"><span class="string">"score"</span></div>
  <div class="value">
    <div class="+">
      <div class="map-get">
        <var class="user"></var>
        <div class="key"><span class="string">"score"</span></div>
      </div>
      <span class="integer">10</span>
    </div>
  </div>
</div>

<p>
  <div class="map-get">
    <var class="user"></var>
    <div class="key"><span class="string">"name"</span></div>
  </div>
</p>
<p>
  <div class="map-get">
    <var class="user"></var>
    <div class="key"><span class="string">"score"</span></div>
  </div>
</p>

</body>
</html>
```

---

### Fetch and display an API response

```html
<html>
<body>

<div class="http-get" id="post">
  <div class="url">
    <span class="string">"https://jsonplaceholder.typicode.com/posts/1"</span>
  </div>
</div>

<p>
  <div class="map-get">
    <var class="post"></var>
    <div class="key"><span class="string">"title"</span></div>
  </div>
</p>
<p>
  <div class="map-get">
    <var class="post"></var>
    <div class="key"><span class="string">"body"</span></div>
  </div>
</p>

</body>
</html>
```

---

### REST API server

```html
<html>
<body>

<div class="server" id="3000">

  <!-- GET /hello?name=Alice -->
  <div class="endpoint-get" id="/hello">
    <div class="handler">
      <div class="respond">
        <div class="+">
          <span class="string">"Hello, "</span>
          <div class="map-get">
            <var class="params"></var>
            <div class="key"><span class="string">"name"</span></div>
          </div>
        </div>
      </div>
    </div>
  </div>

  <!-- POST /echo — reflects the body back as JSON -->
  <div class="endpoint-post" id="/echo">
    <div class="handler">
      <div class="respond">
        <var class="body"></var>
      </div>
    </div>
  </div>

</div>

</body>
</html>
```

---

## Error messages

HAPL gives detailed errors that point to the exact problem:

```
error[E04]: unknown type 'integar' in variable declaration
  --> my_program.html
   |
   | <var class="integar" id="x">
   |            ^^^^^^^^^
   | 'integar' is not a valid type
   |
   = hint: valid types: integer, double, string, boolean, map
```

Every error includes a code, a plain-English description, the offending tag with the relevant attribute highlighted, and a hint with the correct syntax.
