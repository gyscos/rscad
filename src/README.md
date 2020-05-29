# rs-cad

Rust re-implementation of OpenSCAD.

## Raw Parser

Input text -> AST

## Parser

AST -> SCAD AST
No strings keys (no hashes)

## Interpreter

SCAD AST -> CSG TREE
No variables, no user modules
