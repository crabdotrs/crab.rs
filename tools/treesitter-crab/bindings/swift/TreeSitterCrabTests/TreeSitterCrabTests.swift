import XCTest
import SwiftTreeSitter
import TreeSitterCrab

final class TreeSitterCrabTests: XCTestCase {
    func testCanLoadGrammar() throws {
        let parser = Parser()
        let language = Language(language: tree_sitter_crab())
        XCTAssertNoThrow(try parser.setLanguage(language),
                         "Error loading Crab grammar")
    }
}
