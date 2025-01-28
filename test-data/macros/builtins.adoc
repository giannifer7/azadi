:stylesheet: dracula.css
:source-highlighter: pygments

= builtins for azadi macros

=== define the module

[source,python]
----
%include(macros.adoc.py)

%pymodule(azadi_macros_pyeval, builtins)


<[azadi_macros_pyeval_builtins_module_doc]>=
"""Built-in macro implementations"""
$$


<[azadi_macros_pyeval_builtins_import]>=
from typing import NoReturn, TYPE_CHECKING


from azadi_macros_pyeval.types import (
    ASTNode,
    EvalError,
    IncludeError,
    AzaHereCall,
    NodeType as Nt,
)

from azadi_macros_pyeval.evaluator import BuiltinMacro
from azadi_macros_pyeval.context import MacroDefinition
from azadi_macros_pyeval.source_utils import modify_source

if TYPE_CHECKING:
    from azadi_macros_pyeval.evaluator import MacroEvaluator
$$


<[azadi_macros_pyeval_builtins_body]>=

class BuiltinBase:
    """Base class for builtin macros"""

    def __init__(self, evaluator: MacroEvaluator, name: bytes) -> None:
        self.evaluator = evaluator
        self.context = evaluator.context
        self.name = name

    def call(self, _node: ASTNode) -> bytes:
        return b""

    def raise_error(self, node: ASTNode, message: str) -> NoReturn:
        raise EvalError(
            self.context.get_location(node),
            f"{self.name.decode('utf-8')} {message}",
        )

<[builtins_impl]>


def create_builtins(evaluator: MacroEvaluator) -> dict[bytes, BuiltinMacro]:
    """Create all builtin macros with the given evaluator"""
    return {
        <[builtins_register]>
    }

$$
----

=== macro to define a builtin

[,azadi]
----
%def(builtin, name, base, member, %{

define the class

<[builtins_impl]>=

class %capitalize(%(name))Builtin(%if(%(base),%(base),BuiltinBase)):
    def __init__(self, evaluator: MacroEvaluator) -> None:
        super().__init__(evaluator, b"%(name)")

    <[%capitalize(%(name))Builtin]>
$$

<[%capitalize(%(name))Builtin]>=
%(member)
$$

register the builder

<[builtins_register]>=
b"%(name)": %capitalize(%(name))Builtin(evaluator),
$$

%})
----

=== example builtin

[,azadi]
----
%builtin(dummy,,%{
def call(self, node: ASTNode) -> bytes:
    if len(node.parts) != 1:
        self.raise_error(node, "requires exactly one argument")
    param = self.evaluator.evaluate(node[0]).decode("utf-8")
    if not param:
        return b""
    return param.encode("utf-8")
%})
----

