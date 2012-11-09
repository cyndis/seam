Seam carving rescale
====================

Seam carving / liquid rescale algorithm implemented in Rust.
Uses gradient magnitude for pixel energy calculation.

Requirements
------------

Tested on rust-incoming-git c8b4dea.

Usage
-----

$ ./seam input.ppm <pixels> output.ppm

Input must be in the format created by
    convert abc.png -compress none abc.ppm

Issues
------

Performance issues; e.g. recalculates energies for
the whole image after each carved seam.