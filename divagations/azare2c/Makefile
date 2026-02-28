all: azare2c

azare2c: lexer.cpp main.cpp
	g++ -o azare2c main.cpp lexer.cpp

lexer.cpp: lexer.re
	re2c -o lexer.cpp lexer.re

clean:
	rm -f azare2c lexer.cpp
