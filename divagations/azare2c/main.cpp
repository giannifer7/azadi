#include <iostream>
#include <fstream>
#include <sstream>
#include <vector>

// Include the generated lexer implementation
// Note: In a real build, you'd compile lexer.cpp separately and link.
// For simplicity here, we assume lexer.cpp is generated from lexer.re
// and we include it or compile it. 
// Since we can't easily include a .cpp file that might not exist yet, 
// let's assume the user will compile main.cpp and lexer.cpp together.
// So this file just needs the Lexer struct definition.
// BUT, the Lexer struct is defined IN lexer.re. 
// To make this self-contained for the user to "try", I will include the 
// necessary headers and forward declarations, but the cleanest way is 
// to have a header file. 

// Since I am generating the files, I will create a header file for the Lexer.

#include "lexer.h"

int main(int argc, char **argv) {
    std::string input;
    if (argc > 1) {
        std::ifstream t(argv[1]);
        std::stringstream buffer;
        buffer << t.rdbuf();
        input = buffer.str();
    } else {
        // Read from stdin
        std::istreambuf_iterator<char> begin(std::cin), end;
        input = std::string(begin, end);
    }

    // Append null terminator for safety if re2c expects it (our rules handle it)
    input.push_back('\0');

    Lexer lexer(input.c_str(), input.size());

    while (true) {
        Token token = lexer.next_token();
        if (token.kind == TOK_EOF) break;
        
        std::cout << "Token: " << token.kind << " | Text: '" << token.text << "'" << std::endl;
    }

    return 0;
}
