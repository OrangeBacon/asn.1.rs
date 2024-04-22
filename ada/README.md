# Ada Compiler

This compiler aims to comply with Ada 2022, as specified in the Ada 2022 Annotated Reference Manual.

# Implementation Defined Choices

## 1.1.2(37.a/2) Implementation Advice
Both implementation defined choices and implementation advice is listed here, even if the advice is not taken.

## 1.1.3(3.a) Capacity Limitations
The following restrictions are imposed by this implementation:
- All source files must contain less than `2^32-1` bytes.
- No more than `2^32-1` source files can be used in a single compilation.

## 1.1.3(6.a) Standard Variations
No variations from the Ada standard are currently made.

## 1.1.3(10.a) External Interactions
Which `code_statements` cause external interactions is not defined yet as I haven't implemented enough to know.

## 1.1.3(20.a.1/2) Unsupported Specialized Needs Annexes
`Program_Error` will be raised where possible.

## 1.1.3(21.a.1/2) Language Defined Library Extensions
None are implemented at this time, so this is not applicable.

## 1.1.5(12.a.1/2) Erroneous Execution
If a bounded error or erroneous execution is detected, `Program_Error` should be raised

## 2.1(4.a) Coded Representation of Characters
The coded representation of characters used is not the one from ISO/IEC 10646:2020, instead Unicode 15.1.0 shall be used.

## 2.1(4.c/5) NFC Source Text
Any source text not represented in NFC form shall be rejected.

## 2.2(2.a) Line Breaks
Line breaks shall be as specified in 2.1(16/3) "The character pair CARRIAGE RETURN/LINE FEED (code points 16#0D# 16#0A#) signifies a single end of line (see 2.2); every other occurrence of a format_effector other than the character whose code point position is 16#09# (CHARACTER TABULATION) also signifies a single end of line."

## 2.2(14.a) Lexical Lengths and Line Lengths
There are no restrictions to the length of a single line or lexical element, however the source file capacity limitations (2^32-1 bytes) must be observed.
