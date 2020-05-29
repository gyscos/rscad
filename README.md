Big-picture pipeline:

## (Done) Step 1: lalrpop
`&'input str` -> AST

## (In-progress) Step 2: re-parser? resolver?
AST -> refined AST
* Remove all string keys, only number (no more string hashes) (do we need this?)
* Escape all strings (Could be done in step1?)
* Outputs list (map?) of modules, variables, functions, items

## (Planned) Step 3: Interpreter
refined AST -> simplified CSG tree
* No more variables, only values
* No more user modules, only primitives and base CSG operations

## (Considered) Step 4: Render
CSG tree -> polygons/triangle mesh
* Rasterize curves with given precision (`$fn`)
* Actually use CGAL or other library to compute the final geometry

## (Considered) Step 5: Export
geometry -> STL/AMF/3MF file
