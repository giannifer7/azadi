:stylesheet: dracula.css
:source-highlighter: pygments

= macros

=== define some structural macros for generating python modules

==== %%pypackage(package)
create a package directory under src and an empty +__init__+.py within

[,python]
----
%def(pypackage, package, %{
<[@file src/%(package)/__init__.py]>=
$$
%})
----

==== %%pymodule(package, module)
create a python module module.py in package package +
with slots for module doc, import and body +
and minimal boilerplate like a comment with the filemodule +
and from +__future__+ import annotations

[source,python]
----
%def(pymodule, package, module, %{
<[@file src/%(package)/%(module).py]>=
# src/%(package)/%(module).py
<[%(package)_%(module)_module_doc]>

from __future__ import annotations

<[%(package)_%(module)_import]>

<[%(package)_%(module)_body]>
$$
%})
----

==== %%here(tpl_pymodule, package, module)
template to create a module in a package with some example code in it

[source,python]
----
%def(tpl_pymodule, package, module, %{
%%pypackage(%(package))
%%pymodule(%(package), %(module))


<[%(package)_%(module)_module_doc]>=
"""%(module)"""
$$

<[%(package)_%(module)_import]>=
import os
from pathlib import Path
from dataclasses import dataclass, field

from %(package).xxx import yyy
$$
========================================================================== %(package)_%(module)
<[%(package)_%(module)_body]>=
def some_func(arg: str) -> str:
    return arg * 2

@dataclass
class SomeClass:
    file: str
    line: int
    parts: list[int] = field(default_factory=list)

    def __str__(self) -> str:
        return f"{self.file}:{self.line}"

$$
========================================================================== %(package)_%(module) END
%})
----
