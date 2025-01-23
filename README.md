# Azadi

Now users can:

Use - as input to read from stdin
When input is stdin, output also goes to stdout
When reading normal files, output goes to .ast files as before
Input directory is ignored for stdin

Example usages:

'''bash
# Read from stdin, write to stdout
cat input.txt | azadi - --dump-ast

# Read from file, write to file.ast
azadi input.txt --dump-ast

# Mix stdin and files
azadi - input.txt --dump-ast

# Pipe through tools
cat input.txt | azadi - --dump-ast | jq .
'''
