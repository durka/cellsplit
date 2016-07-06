What?
=====

This program can split apart, and recombine (TODO), MATLAB scripts written using the Cell Mode feature.

Why?
====

Cell Mode is highly convenient for interactive development. A large task can be split into separate chunks which are repeatedly run independently, inspecting local variables and making iterative changes. I often use Cell Mode to document the steps of a machine learning pipeline. But Cell Mode can't be used when operating MATLAB from the comand line (such as on a remote server). You can only run the whole script as a unit. But after processing with `cellsplit`, each cell _is_ a whole script so the flexibility is regained.

How?
====

`cellsplit` contains a very rudimentary parser of MATLAB syntax, and it breaks out every cell into a new script. It also breaks out bodies of conditionals and loops, since cells can be placed inside those.

Status
======

Unsupported features of MATLAB:

    - `switch`
    - ...

Planned tasks for `cellsplit`:

    - Recombine a set of scripts into a Cell Mode script
    - Use the cell title comments to make better filenames
    - Print a TOC somehow
    - Tests and documentation
