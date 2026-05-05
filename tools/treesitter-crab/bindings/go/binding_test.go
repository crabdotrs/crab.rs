package tree_sitter_crab_test

import (
	"testing"

	tree_sitter "github.com/tree-sitter/go-tree-sitter"
	tree_sitter_crab "github.com/crabdotrs/crab.rs/bindings/go"
)

func TestCanLoadGrammar(t *testing.T) {
	language := tree_sitter.NewLanguage(tree_sitter_crab.Language())
	if language == nil {
		t.Errorf("Error loading Crab grammar")
	}
}
