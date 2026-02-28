#include <iostream>
#include <fstream>
#include <string>

#include "antlr4-runtime.h"
#include "AzadiLexer.h"

using namespace antlr4;

int main(int argc, const char* argv[]) {
    std::ifstream stream;
    ANTLRInputStream input;

    if (argc > 1) {
        stream.open(argv[1]);
        if (!stream.is_open()) {
            std::cerr << "Error: Cannot open file " << argv[1] << std::endl;
            return 1;
        }
        input = ANTLRInputStream(stream);
    } else {
        // Read from stdin
        input = ANTLRInputStream(std::cin);
    }

    AzadiLexer lexer(&input);
    CommonTokenStream tokens(&lexer);

    tokens.fill();

    for (auto token : tokens.getTokens()) {
        std::cout << token->toString() << std::endl;
    }

    return 0;
}
