# netlist-db

[![build](https://github.com/zao111222333/netlist-db/actions/workflows/build.yml/badge.svg)](https://github.com/zao111222333/netlist-db/actions/workflows/build.yml)
[![License](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![netlist-db](https://shields.io/crates/v/netlist-db.svg?style=flat-square&label=crates.io)](https://crates.io/crates/netlist-db)
[![Docs](https://docs.rs/netlist-db/badge.svg)](https://docs.rs/netlist-db)
[![codecov](https://codecov.io/github/zao111222333/netlist-db/graph/badge.svg)](https://codecov.io/github/zao111222333/netlist-db)

Concurrent/Parallel SPICE (HSPICE) parser, under building.

## Features
+ Concurrent/Parallel parse multi SPICE files (`.inc` and `.lib`) command
+ Use span to store string, avoid small allocations
+ Units system
+ Circular definition detection

## Example
Download the [releases/latest/examples.zip](https://github.com/zao111222333/netlist-db/releases/latest/download/examples.zip), then
``` shell
cd examples
./parser_x86_64-unknown-linux-musl tests/top.sp
```

Or you can compile & run this example by

``` shell
cargo run --example parser --release -- tests/top.sp
```
<details>
<summary>The output should be</summary>
<?xml version="1.0" encoding="UTF-8" ?>
<!DOCTYPE html PUBLIC "-//W3C//DTD XHTML 1.0 Strict//EN" "http://www.w3.org/TR/xhtml1/DTD/xhtml1-strict.dtd">
<!-- This file was created with the aha Ansi HTML Adapter. https://github.com/theZiz/aha -->
<html xmlns="http://www.w3.org/1999/xhtml">
<head>
<meta http-equiv="Content-Type" content="application/xml+xhtml; charset=UTF-8"/>
</head>
<body>
<pre style="color:white; background-color:black">
cargo run --example parser --release -- tests/top.sp | aha --black > output.html
2025-01-27T01:40:43.135Z DEBUG [netlist_db::file] load File &quot;tests/inc/inc0.sp&quot;
2025-01-27T01:40:43.135Z DEBUG [netlist_db::file] load File &quot;tests/inc/inc1.sp&quot;
2025-01-27T01:40:43.135Z DEBUG [netlist_db::file] load File &quot;tests/lib.sp&quot;, section tt
2025-01-27T01:40:43.135Z DEBUG [netlist_db::file] load File &quot;tests/units.sp&quot;
2025-01-27T01:40:43.135Z DEBUG [netlist_db::file] load File &quot;tests/inc/inc2.sp&quot;
2025-01-27T01:40:43.135Z DEBUG [netlist_db::file] load File &quot;tests/inc/../lib.sp&quot;, section pre_layout
2025-01-27T01:40:43.136Z DEBUG [netlist_db::file] load File &quot;tests/inc/../cycle_ref0.sp&quot;
2025-01-27T01:40:43.136Z DEBUG [netlist_db::file] load File &quot;tests/inc/../cycle_ref1.sp&quot;
2025-01-27T01:40:43.137Z ERROR [netlist_db::lexer] 
File <span style="filter: contrast(70%) brightness(190%);color:fuchsia;">&quot;tests/top.sp&quot;</span>, line <span style="filter: contrast(70%) brightness(190%);color:fuchsia;">5</span>
.inc 'inc/inc0.sp'
                  <span style="font-weight:bold;"></span><span style="font-weight:bold;filter: contrast(70%) brightness(190%);color:red;">&lt;-</span>
<span style="font-weight:bold;"></span><span style="font-weight:bold;filter: contrast(70%) brightness(190%);color:fuchsia;">Error</span>: <span style="filter: contrast(70%) brightness(190%);color:fuchsia;">No such file or directory (os error 2)</span>

2025-01-27T01:40:43.137Z ERROR [netlist_db::lexer] 
File <span style="filter: contrast(70%) brightness(190%);color:fuchsia;">&quot;tests/inc/../cycle_ref1.sp&quot;</span>, line <span style="filter: contrast(70%) brightness(190%);color:fuchsia;">2</span>
.inc 'cycle_ref0.sp'
                    <span style="font-weight:bold;"></span><span style="font-weight:bold;filter: contrast(70%) brightness(190%);color:red;">&lt;-</span>
<span style="font-weight:bold;"></span><span style="font-weight:bold;filter: contrast(70%) brightness(190%);color:fuchsia;">CircularDefinition</span>: <span style="filter: contrast(70%) brightness(190%);color:fuchsia;">Detect circular definition in File &quot;tests/inc/../cycle_ref0.sp&quot;, line 2</span>
   File &quot;tests/top.sp&quot;, line 6
     ↓
   File &quot;tests/inc/inc2.sp&quot;, line 2
     ↓
   File &quot;tests/inc/../lib.sp&quot;, line 3, section pre_layout
     ↓
<span style="font-weight:bold;"></span><span style="font-weight:bold;filter: contrast(70%) brightness(190%);color:red;"> * File &quot;tests/inc/../cycle_ref0.sp&quot;, line 2</span>
     ↓
   File &quot;tests/inc/../cycle_ref1.sp&quot;, line 2
     ↓
<span style="font-weight:bold;"></span><span style="font-weight:bold;filter: contrast(70%) brightness(190%);color:red;"> * File &quot;tests/inc/../cycle_ref0.sp&quot;, line 2</span>

2025-01-27T01:40:43.137Z ERROR [netlist_db::lexer] 
File <span style="filter: contrast(70%) brightness(190%);color:fuchsia;">&quot;tests/inc/../lib.sp&quot;</span>, line <span style="filter: contrast(70%) brightness(190%);color:fuchsia;">7</span>
.params flag=1
       <span style="font-weight:bold;"></span><span style="font-weight:bold;filter: contrast(70%) brightness(190%);color:red;">&lt;-</span>
<span style="font-weight:bold;"></span><span style="font-weight:bold;filter: contrast(70%) brightness(190%);color:fuchsia;">SyntaxError</span>: <span style="filter: contrast(70%) brightness(190%);color:fuchsia;">Unknwon command `params`</span>

2025-01-27T01:40:43.137Z ERROR [netlist_db::lexer] 
File <span style="filter: contrast(70%) brightness(190%);color:fuchsia;">&quot;tests/top.sp&quot;</span>, line <span style="filter: contrast(70%) brightness(190%);color:fuchsia;">8</span>
.lib 'lib.sp' tt
                <span style="font-weight:bold;"></span><span style="font-weight:bold;filter: contrast(70%) brightness(190%);color:red;">&lt;-</span>
<span style="font-weight:bold;"></span><span style="font-weight:bold;filter: contrast(70%) brightness(190%);color:fuchsia;">Error</span>: <span style="filter: contrast(70%) brightness(190%);color:fuchsia;">Can NOT find section `tt` in file &quot;tests/lib.sp&quot;</span>

2025-01-27T01:40:43.137Z ERROR [netlist_db::lexer] 
File <span style="filter: contrast(70%) brightness(190%);color:fuchsia;">&quot;tests/units.sp&quot;</span>, line <span style="filter: contrast(70%) brightness(190%);color:fuchsia;">7</span>
+ micr?o=1u
<span style="font-weight:bold;"></span><span style="font-weight:bold;filter: contrast(70%) brightness(190%);color:red;">&lt;-</span>
<span style="font-weight:bold;"></span><span style="font-weight:bold;filter: contrast(70%) brightness(190%);color:fuchsia;">ParserError</span>: <span style="filter: contrast(70%) brightness(190%);color:fuchsia;">TakeWhile1</span>

======= AST ===========

.subckt DEMO A1 A2 var1=1 var2=2.option 
+ gmindc=0.00000000000001 scale=0.9
.param 
+ prelayout=1 flag_cc=1
X0.CCC net8 net23 VSS VPW NHVT11LL_CKT W=0.000000135 L=0.00000004
.params flag=1
.ends DEMO
.subckt UNITS A
.ends UNITS
X0.BBB net8 net23 VSS VPW NHVT11LL_CKT W=0.000000135 L=0.00000004
X0.AAA net8 net23 VSS VPW NHVT11LL_CKT W=0.000000135 L=0.00000004
======= ERR ===========
true
======= stats =========
parse: 2.200291ms
build: 40.833µs
print: 11.334µs
=======================
</pre>
</body>
</html>
</details>


## TODO
+ Fully expression system
+ Support more commands