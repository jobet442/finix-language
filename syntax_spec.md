# Finix Language Specification

Finix is designed to hit the sweet spot between **Python's rapid readability** and **Java's robust, scalable structure**. It is a gradually-typed language, meaning you can write quick scripts without type annotations, and later harden them with strict types and interfaces as your codebase grows.

---

## 1. Naming Conventions

Consistency is key to a readable codebase. Finix follows a blend of Python and Java conventions:

- **Variables & Functions**: `snake_case` (e.g., `user_name`, `calculate_total()`)
- **Classes & Interfaces**: `PascalCase` (e.g., `HttpRequest`, `EventListener`)
- **Constants**: `UPPER_SNAKE_CASE` (e.g., `MAX_RETRIES`, `PI`)
- **File Names**: `snake_case.fx`

---

## 2. Style Guide

- **Indentation**: 4 spaces per indentation level. Tabs are discouraged.
- **Braces**: One True Brace Style (1TBS). Opening braces `{` go on the same line as the statement. Closing braces `}` go on their own line.
- **Semicolons**: Required at the end of statements to ensure unambiguous, fast parsing.
- **Type Annotations**: Optional, but highly recommended for function parameters and return types in public APIs.

---

## 3. Syntax Examples

### Hello World
```finix
import system.io as io;

func main() {
    io.println("Hello, World!");
}
```

### Gradual Typing & Variables
```finix
// Dynamic, inferred type
let greeting = "Welcome";

// Statically typed
let max_connections: int = 100;
let is_active: boolean = true;
```

### Functions
```finix
// Dynamically typed function
func add(a, b) {
    return a + b;
}

// Statically typed function
func multiply(a: int, b: int): int {
    return a * b;
}
```

### Control Flow
```finix
// Conditionals
if score >= 90 {
    print("A");
} else if score >= 80 {
    print("B");
} else {
    print("C");
}

// Loops
while is_running {
    process_task();
}

for item in items {
    print(item);
}
```

### Object-Oriented Structure (Classes & Interfaces)
```finix
interface Animal {
    func speak(): void;
}

class Dog implements Animal {
    let name: string;

    func init(name: string) {
        this.name = name;
    }

    func speak(): void {
        print(this.name + " says Woof!");
    }
}
```

---

## 4. Unique Finix Features

### Data Pipelines (`|>`)
Avoid deeply nested function calls using the pipeline operator. It passes the evaluated left-hand expression as the first argument to the right-hand function call.
```finix
let clean_data = raw_input
    |> trim()
    |> split(",")
    |> parse_ints();
```

### Error & Null Propagation (`?`)
Avoid deep `if` statements and `try/catch` boilerplate. Appending `?` to an expression will immediately return from the current function if the value evaluates to an Error or `null`.
```finix
func fetch_config(path: string): Config {
    let file = fs.open(path)?;       // Returns early if file not found
    let data = json.parse(file)?;    // Returns early on parse error
    return data;
}
```

---

## 5. Operator Precedence

Operators in Finix are evaluated based on their binding power. The table below lists operators from **highest to lowest** precedence.

| Level | Category | Operators | Associativity |
| :--- | :--- | :--- | :--- |
| 10 | Postfix Unary | `?` | Left-to-Right |
| 9 | Index / Property / Call | `a[b]`, `a.b`, `a(b)` | Left-to-Right |
| 8 | Prefix Unary | `-a`, `!a` | Right-to-Left |
| 7 | Multiplicative | `*`, `/`, `%` | Left-to-Right |
| 6 | Additive | `+`, `-` | Left-to-Right |
| 5 | Relational | `<`, `<=`, `>`, `>=` | Left-to-Right |
| 4 | Equality | `==`, `!=` | Left-to-Right |
| 3 | Logical AND | `&&` | Left-to-Right |
| 2 | Logical OR | `\|\|` | Left-to-Right |
| 1.5 | Pipeline | `|>` | Left-to-Right |
| 1 | Assignment | `=` | Right-to-Left |

---

## 6. Formal Grammar (EBNF)

The following represents the core grammar rules parsed by the Finix compiler.

### Programs and Blocks
```ebnf
program     ::= statement* EOF ;
block       ::= "{" statement* "}" ;
```

### Statements
```ebnf
statement   ::= expr_stmt
              | let_stmt
              | if_stmt
              | while_stmt
              | for_stmt
              | fun_stmt
              | class_stmt
              | interface_stmt
              | return_stmt
              | import_stmt
              | block ;

let_stmt       ::= "let" IDENTIFIER ( ":" type )? ( "=" expression )? ";" ;
return_stmt    ::= "return" expression? ";" ;
expr_stmt      ::= expression ";" ;

if_stmt        ::= "if" expression block ( "else" ( if_stmt | block ) )? ;
while_stmt     ::= "while" expression block ;
for_stmt       ::= "for" IDENTIFIER "in" expression block ;

import_stmt    ::= "import" IDENTIFIER ( "." IDENTIFIER )* ( "as" IDENTIFIER )? ";" ;
```

### Declarations
```ebnf
fun_stmt       ::= "func" IDENTIFIER "(" parameters? ")" ( ":" type )? block ;
parameters     ::= param ( "," param )* ;
param          ::= IDENTIFIER ( ":" type )? ;

class_stmt     ::= "class" IDENTIFIER ( "extends" IDENTIFIER )? 
                   ( "implements" IDENTIFIER ( "," IDENTIFIER )* )? 
                   "{" class_body* "}" ;
class_body     ::= let_stmt | fun_stmt ;

interface_stmt ::= "interface" IDENTIFIER "{" interface_body* "}" ;
interface_body ::= "func" IDENTIFIER "(" parameters? ")" ( ":" type )? ";" ;
```

### Expressions
```ebnf
expression  ::= assignment ;
assignment  ::= IDENTIFIER "=" assignment | logic_or ;
logic_or    ::= logic_and ( "||" logic_and )* ;
logic_and   ::= equality ( "&&" equality )* ;
equality    ::= relational ( ( "!=" | "==" ) relational )* ;
relational  ::= additive ( ( ">" | ">=" | "<" | "<=" ) additive )* ;
additive    ::= multiplicative ( ( "-" | "+" ) multiplicative )* ;
multiplicative ::= unary ( ( "/" | "*" | "%" ) unary )* ;
unary       ::= ( "!" | "-" ) unary | pipeline ;
pipeline    ::= call ( "|>" call )* ;
call        ::= postfix ( "(" arguments? ")" | "." IDENTIFIER | "[" expression "]" )* ;
postfix     ::= primary "?"* ;
arguments   ::= expression ( "," expression )* ;

primary     ::= "true" | "false" | "null" | "this" | "super"
              | NUMBER | STRING | IDENTIFIER 
              | "(" expression ")" ;

type        ::= "int" | "float" | "boolean" | "string" | "any" | "void" | IDENTIFIER "?"? ;
```